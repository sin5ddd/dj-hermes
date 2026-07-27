//! strudel-rs — play .strudel songs through cpal (Tasks 1–12 + highlight TUI).

use std::io::{stdout, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, execute, terminal, QueueableCommand};
use strudel_rs::engine::Engine;
use strudel_rs::highlight::{
    active_spans, bar_index, bar_pos, format_header, render_ansi, HighlightModel,
};
use strudel_rs::sample::SampleBank;
use strudel_rs::song::parse_song;

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        print_usage();
        std::process::exit(0);
    }

    let cmd = args.remove(0);
    match cmd.as_str() {
        "play" => {
            if let Err(e) = cmd_play(&args) {
                eprintln!("strudel-rs play: {e}");
                std::process::exit(1);
            }
        }
        "help" | "-h" | "--help" => print_usage(),
        other => {
            eprintln!("unknown command: {other}");
            print_usage();
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!(
        "\
strudel-rs — Strudel live CLI

Usage:
  strudel-rs play [SONG] [--seconds N] [--headless]

  SONG          path to .strudel (default: songs/smoke.strudel)
  --seconds N   stop after N seconds (omit to loop until quit)
  --headless    no TUI: meta log only (for scripts / non-TTY)
  --highlight   explicit highlight TUI (default; also: --hl)

  Default play loops forever (TUI: q / Esc to quit; headless: Ctrl+C).

Examples:
  cargo run -- play songs/smoke.strudel
  cargo run -- play songs/smoke.strudel --seconds 15
  cargo run -- play songs/smoke.strudel --headless

Samples: ./samples (or <song>/../samples). CC0 kit docs in samples/LICENSE.md.
"
    );
}

fn cmd_play(args: &[String]) -> Result<(), String> {
    let mut song_path = PathBuf::from("songs/smoke.strudel");
    // None = loop until quit; Some(n) = stop after n seconds.
    let mut seconds: Option<u64> = None;
    // Default: live mini-notation highlight TUI. Opt out with --headless.
    let mut highlight = true;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--seconds" => {
                i += 1;
                let s = args
                    .get(i)
                    .ok_or_else(|| "--seconds needs a number".to_string())?;
                let n: u64 = s.parse().map_err(|_| format!("bad --seconds value: {s}"))?;
                if n == 0 {
                    // 0 still means "no time limit" (same as omitting the flag).
                    seconds = None;
                } else {
                    seconds = Some(n);
                }
            }
            "--headless" => {
                highlight = false;
            }
            "--highlight" | "--hl" => {
                highlight = true;
            }
            "--help" | "-h" => {
                print_usage();
                return Ok(());
            }
            flag if flag.starts_with('-') => {
                return Err(format!("unknown flag: {flag}"));
            }
            path => {
                song_path = PathBuf::from(path);
            }
        }
        i += 1;
    }

    let text = std::fs::read_to_string(&song_path)
        .map_err(|e| format!("read {}: {e}", song_path.display()))?;
    let path_str = song_path.to_string_lossy().into_owned();
    let song = parse_song(&text, &path_str)?;

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| "no default output device".to_string())?;
    let supported = device
        .default_output_config()
        .map_err(|e| format!("output config: {e}"))?;

    let sample_rate = supported.sample_rate() as u32;
    let channels = supported.channels() as usize;
    let stream_config: cpal::StreamConfig = supported.into();

    let samples_dir = resolve_samples_dir(&song_path)?;
    let bank = SampleBank::load_dir(&samples_dir, sample_rate);
    let names = bank.names();
    if names.is_empty() {
        eprintln!(
            "warning: no WAV samples loaded from {} (synth-only tracks still play)",
            samples_dir.display()
        );
    }

    let bpm = song.bpm.unwrap_or(120.0);
    let title = song.title.clone();
    let highlight_model = if highlight {
        Some(HighlightModel::from_song(&song, sample_rate))
    } else {
        None
    };

    let mut engine = Engine::new(sample_rate, bpm);
    let playhead = engine.playhead_handle();
    engine.load_song_immediate(0, song);

    if !highlight {
        eprintln!("strudel-rs play");
        eprintln!("  song:    {}", song_path.display());
        eprintln!("  title:   {title}");
        eprintln!("  bpm:     {bpm}");
        eprintln!(
            "  samples: {} ({} sounds)",
            samples_dir.display(),
            names.len()
        );
        if !names.is_empty() {
            eprintln!("            {}", names.join(", "));
        }
        eprintln!("  device:  {sample_rate} Hz, {channels} ch");
        match seconds {
            Some(n) => eprintln!("  duration:{n}s"),
            None => eprintln!("  duration:loop (Ctrl+C to stop)"),
        }
        eprintln!("playing…");
    }

    let engine = Arc::new(Mutex::new(engine));
    let bank = Arc::new(bank);
    let mut mono = Vec::<f32>::new();

    let engine_cb = Arc::clone(&engine);
    let bank_cb = Arc::clone(&bank);
    let stream = device
        .build_output_stream(
            stream_config,
            move |data: &mut [f32], _| {
                let frames = data.len() / channels.max(1);
                if mono.len() < frames {
                    mono.resize(frames, 0.0);
                }
                let mono_buf = &mut mono[..frames];
                mono_buf.fill(0.0);

                if let Ok(mut eng) = engine_cb.try_lock() {
                    eng.process(mono_buf, &bank_cb);
                }

                if channels <= 1 {
                    data[..frames].copy_from_slice(mono_buf);
                } else {
                    for (frame_i, &s) in mono_buf.iter().enumerate() {
                        let base = frame_i * channels;
                        for c in 0..channels {
                            data[base + c] = s;
                        }
                    }
                }
            },
            |e| eprintln!("stream error: {e}"),
            None,
        )
        .map_err(|e| format!("build stream: {e}"))?;

    stream.play().map_err(|e| format!("play stream: {e}"))?;

    if let Some(model) = highlight_model {
        run_highlight_loop(&model, &playhead, seconds)?;
    } else {
        match seconds {
            Some(n) => {
                std::thread::sleep(Duration::from_secs(n));
                eprintln!("done.");
            }
            None => {
                // Block until the process is interrupted (Ctrl+C).
                loop {
                    std::thread::sleep(Duration::from_secs(3600));
                }
            }
        }
    }
    Ok(())
}

fn run_highlight_loop(
    model: &HighlightModel,
    playhead: &std::sync::atomic::AtomicU64,
    seconds: Option<u64>,
) -> Result<(), String> {
    enable_raw_mode().map_err(|e| format!("raw mode: {e}"))?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, cursor::Hide)
        .map_err(|e| format!("enter alternate screen: {e}"))?;

    let started = Instant::now();
    let limit = seconds.map(Duration::from_secs);
    let frame = Duration::from_millis(33); // ~30 fps
    let result = (|| -> Result<(), String> {
        loop {
            if let Some(lim) = limit {
                if started.elapsed() >= lim {
                    break;
                }
            }
            // Drain key events (non-blocking)
            while event::poll(Duration::from_millis(0)).unwrap_or(false) {
                if let Ok(Event::Key(key)) = event::read() {
                    if key.kind == KeyEventKind::Press
                        && (key.code == KeyCode::Char('q')
                            || key.code == KeyCode::Char('Q')
                            || key.code == KeyCode::Esc)
                    {
                        return Ok(());
                    }
                }
            }

            let gs = playhead.load(Ordering::Relaxed);
            let bar = bar_index(gs, model.sample_rate, model.bpm);
            let pos = bar_pos(gs, model.sample_rate, model.bpm);
            let spans = active_spans(model, bar, pos);
            let header = format_header(model, gs, bar, pos);
            let frame_text = render_ansi(model, &spans, &header);

            out.queue(cursor::MoveTo(0, 0))
                .map_err(|e| format!("draw: {e}"))?;
            out.queue(terminal::Clear(terminal::ClearType::FromCursorDown))
                .map_err(|e| format!("draw: {e}"))?;
            out.write_all(frame_text.as_bytes())
                .map_err(|e| format!("draw: {e}"))?;
            out.flush().map_err(|e| format!("draw: {e}"))?;

            std::thread::sleep(frame);
        }
        Ok(())
    })();

    let _ = execute!(out, cursor::Show, LeaveAlternateScreen);
    let _ = disable_raw_mode();
    if result.is_ok() {
        eprintln!("done.");
    }
    result
}

fn resolve_samples_dir(song_path: &Path) -> Result<PathBuf, String> {
    let candidates = [
        PathBuf::from("samples"),
        song_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("..")
            .join("samples"),
        song_path
            .parent()
            .and_then(|p| p.parent())
            .unwrap_or_else(|| Path::new("."))
            .join("samples"),
    ];
    for c in &candidates {
        if c.is_dir() {
            return Ok(c.components().collect());
        }
    }
    Err(format!(
        "samples/ not found (tried {} ). Run from repo root or place WAV under samples/",
        candidates
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    ))
}
