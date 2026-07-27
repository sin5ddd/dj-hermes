//! strudel-rs — play .strudel songs (highlight TUI, headless, or --repl live).

use std::io::{stdout, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam::channel::{unbounded, Receiver, Sender};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, execute, terminal, QueueableCommand};
use strudel_rs::api::{self, AppState, DEFAULT_API_PORT};
use strudel_rs::engine::{Command, Engine};
use strudel_rs::highlight::{
    active_spans, bar_index, bar_pos, format_header, render_ansi, HighlightModel,
};
use strudel_rs::live_ui;
use strudel_rs::mcp;
use strudel_rs::repl;
use strudel_rs::sample::SampleBank;
use strudel_rs::song::parse_song;
use strudel_rs::watcher::{self, DeckPaths};

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
        "mcp" => {
            if let Err(e) = mcp::run() {
                eprintln!("strudel-rs mcp: {e}");
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
  strudel-rs play [SONG] [--seconds N] [--headless] [--port N] [--no-api]
  strudel-rs play --repl [SONG] [--songs-dir DIR] [--port N] [--no-api]
  strudel-rs mcp

  SONG          path to .strudel (default without --repl: songs/smoke.strudel)
  --seconds N   stop after N seconds (omit to loop until quit; not for --repl)
  --headless    no TUI: meta log only (for scripts / non-TTY)
  --highlight   explicit highlight TUI (default; also: --hl)
  --repl        live UI: mini-notation highlight + command line + watcher
  --repl-text   text-only REPL (no highlight; rustyline) + watcher
  --songs-dir   directory to watch for .strudel saves (default: songs/)
  --port N      HTTP API port (default {DEFAULT_API_PORT}; env STRUDEL_API_PORT)
  --no-api      do not start HTTP API

  strudel-rs mcp
                MCP stdio bridge → HTTP API (play process must be running)

  Default play loops forever (TUI: q / Esc; headless: Ctrl+C).
  --repl: left=A / right=B highlight, » prompt at bottom.
  Commands (no colon):  a load <file>  |  x 4  |  b mute kick  |  bpm 128  |  quit

Examples:
  cargo run -- play songs/smoke.strudel
  cargo run -- play --repl songs/techno16.strudel
  # then:  b load songs/house16.strudel
  #        x 4
  # API: curl http://127.0.0.1:{DEFAULT_API_PORT}/status

Samples: ./samples (or <song>/../samples). CC0 kit docs in samples/LICENSE.md.
"
    );
}

fn cmd_play(args: &[String]) -> Result<(), String> {
    let mut song_path: Option<PathBuf> = None;
    let mut seconds: Option<u64> = None;
    let mut highlight = true;
    let mut repl_mode = false;
    let mut repl_text = false;
    let mut songs_dir = PathBuf::from("songs");
    let mut api_enabled = true;
    let mut cli_port: Option<u16> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--seconds" => {
                i += 1;
                let s = args
                    .get(i)
                    .ok_or_else(|| "--seconds needs a number".to_string())?;
                let n: u64 = s.parse().map_err(|_| format!("bad --seconds value: {s}"))?;
                seconds = if n == 0 { None } else { Some(n) };
            }
            "--headless" => {
                highlight = false;
            }
            "--highlight" | "--hl" => {
                highlight = true;
            }
            "--repl" => {
                repl_mode = true;
            }
            "--repl-text" => {
                repl_mode = true;
                repl_text = true;
            }
            "--songs-dir" => {
                i += 1;
                let d = args
                    .get(i)
                    .ok_or_else(|| "--songs-dir needs a path".to_string())?;
                songs_dir = PathBuf::from(d);
            }
            "--port" => {
                i += 1;
                let s = args
                    .get(i)
                    .ok_or_else(|| "--port needs a number".to_string())?;
                let p: u16 = s
                    .parse()
                    .map_err(|_| format!("bad --port value: {s}"))?;
                if p == 0 {
                    return Err("--port must be 1..=65535".into());
                }
                cli_port = Some(p);
            }
            "--no-api" => {
                api_enabled = false;
            }
            "--help" | "-h" => {
                print_usage();
                return Ok(());
            }
            flag if flag.starts_with('-') => {
                return Err(format!("unknown flag: {flag}"));
            }
            path => {
                song_path = Some(PathBuf::from(path));
            }
        }
        i += 1;
    }

    let api_port = api::resolve_port(cli_port);

    if repl_mode {
        return cmd_play_repl(song_path, songs_dir, !repl_text, api_enabled, api_port);
    }

    let song_path = song_path.unwrap_or_else(|| PathBuf::from("songs/smoke.strudel"));
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

    let samples_dir = resolve_samples_dir(Some(&song_path))?;
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

    let (cmd_tx, cmd_rx) = unbounded::<Command>();
    let engine = Arc::new(Mutex::new(engine));
    let bank = Arc::new(bank);

    let _api = maybe_start_api(api_enabled, api_port, cmd_tx, Arc::clone(&engine));

    let stream = build_stream(
        &device,
        stream_config,
        channels,
        Arc::clone(&engine),
        Arc::clone(&bank),
        Some(cmd_rx),
    )?;
    stream.play().map_err(|e| format!("play stream: {e}"))?;

    if let Some(model) = highlight_model {
        run_highlight_loop(&model, &playhead, seconds)?;
    } else {
        match seconds {
            Some(n) => {
                std::thread::sleep(Duration::from_secs(n));
                eprintln!("done.");
            }
            None => loop {
                std::thread::sleep(Duration::from_secs(3600));
            },
        }
    }
    Ok(())
}

fn cmd_play_repl(
    song_path: Option<PathBuf>,
    songs_dir: PathBuf,
    with_highlight: bool,
    api_enabled: bool,
    api_port: u16,
) -> Result<(), String> {
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

    let samples_dir = resolve_samples_dir(song_path.as_deref())?;
    let bank = SampleBank::load_dir(&samples_dir, sample_rate);
    if bank.names().is_empty() {
        eprintln!(
            "warning: no WAV samples from {} (synth-only still works)",
            samples_dir.display()
        );
    }

    let mut bpm = 120.0;
    let mut engine = Engine::new(sample_rate, bpm);
    let deck_paths: DeckPaths = watcher::new_deck_paths();
    let mut initial_hl: Option<(usize, HighlightModel)> = None;

    if let Some(ref path) = song_path {
        let text =
            std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
        let song = parse_song(&text, &path.to_string_lossy())?;
        bpm = song.bpm.unwrap_or(120.0);
        engine.transport.set_bpm(bpm);
        if with_highlight {
            initial_hl = Some((0, HighlightModel::from_song(&song, sample_rate)));
        }
        engine.load_song_immediate(0, song);
        if let Ok(mut dp) = deck_paths.lock() {
            dp[0] = Some(path.clone());
        }
        if !with_highlight {
            eprintln!("loaded deck A: {}", path.display());
        }
    }

    let (cmd_tx, cmd_rx) = unbounded::<Command>();
    let playhead = engine.playhead_handle();
    let engine = Arc::new(Mutex::new(engine));
    let bank = Arc::new(bank);

    let _api = maybe_start_api(api_enabled, api_port, cmd_tx.clone(), Arc::clone(&engine));

    let stream = build_stream(
        &device,
        stream_config,
        channels,
        Arc::clone(&engine),
        Arc::clone(&bank),
        Some(cmd_rx),
    )?;
    stream.play().map_err(|e| format!("play stream: {e}"))?;

    // Keep watcher alive for the REPL session.
    let _watcher = if songs_dir.is_dir() {
        match watcher::watch_songs(&songs_dir, cmd_tx.clone(), Arc::clone(&deck_paths)) {
            Ok(w) => {
                if !with_highlight {
                    eprintln!("watching {} for .strudel saves", songs_dir.display());
                }
                Some(w)
            }
            Err(e) => {
                eprintln!("warning: watcher not started: {e}");
                None
            }
        }
    } else {
        eprintln!(
            "warning: songs dir {} missing — watcher disabled",
            songs_dir.display()
        );
        None
    };

    if with_highlight {
        live_ui::run(
            cmd_tx,
            deck_paths,
            Arc::clone(&engine),
            playhead,
            sample_rate,
            initial_hl,
        )?;
    } else {
        eprintln!(
            "strudel-rs play --repl-text  |  {sample_rate} Hz, {channels} ch  |  samples {}",
            samples_dir.display()
        );
        repl::run(cmd_tx, deck_paths, Some(Arc::clone(&engine)));
    }
    eprintln!("bye.");
    Ok(())
}

/// Start HTTP API in a background thread when enabled. Keeps `Sender` alive via clone.
fn maybe_start_api(
    enabled: bool,
    port: u16,
    tx: Sender<Command>,
    engine: Arc<Mutex<Engine>>,
) -> Option<std::thread::JoinHandle<()>> {
    if !enabled {
        return None;
    }
    let state = AppState { tx, engine };
    Some(api::spawn_server(state, port))
}

fn build_stream(
    device: &cpal::Device,
    stream_config: cpal::StreamConfig,
    channels: usize,
    engine: Arc<Mutex<Engine>>,
    bank: Arc<SampleBank>,
    cmd_rx: Option<Receiver<Command>>,
) -> Result<cpal::Stream, String> {
    let mut mono = Vec::<f32>::new();
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

                if let Ok(mut eng) = engine.try_lock() {
                    if let Some(rx) = &cmd_rx {
                        while let Ok(cmd) = rx.try_recv() {
                            eng.push_command(cmd);
                        }
                    }
                    eng.process(mono_buf, &bank);
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
    Ok(stream)
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

fn resolve_samples_dir(song_path: Option<&Path>) -> Result<PathBuf, String> {
    let mut candidates = vec![PathBuf::from("samples")];
    if let Some(song_path) = song_path {
        candidates.push(
            song_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join("..")
                .join("samples"),
        );
        candidates.push(
            song_path
                .parent()
                .and_then(|p| p.parent())
                .unwrap_or_else(|| Path::new("."))
                .join("samples"),
        );
    }
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
