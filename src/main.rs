//! strudel-rs — play .strudel songs through cpal (Tasks 1–12 + play wiring).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use strudel_rs::engine::Engine;
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
                // Device / path errors: non-zero so scripts notice; still fine for interactive use.
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
  strudel-rs play [SONG] [--seconds N]

  SONG       path to .strudel (default: songs/smoke.strudel)
  --seconds  play duration (default: 30; use 0 for 600s)

Examples:
  cargo run -- play songs/smoke.strudel
  cargo run -- play songs/smoke.strudel --seconds 15

Samples: ./samples (or <song>/../samples). CC0 kit docs in samples/LICENSE.md.
"
    );
}

fn cmd_play(args: &[String]) -> Result<(), String> {
    let mut song_path = PathBuf::from("songs/smoke.strudel");
    let mut seconds: u64 = 30;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--seconds" => {
                i += 1;
                let s = args
                    .get(i)
                    .ok_or_else(|| "--seconds needs a number".to_string())?;
                seconds = s.parse().map_err(|_| format!("bad --seconds value: {s}"))?;
                if seconds == 0 {
                    seconds = 600;
                }
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

    // cpal 0.18: sample_rate() is u32 on SupportedStreamConfig
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
    let mut engine = Engine::new(sample_rate, bpm);
    engine.load_song_immediate(0, song);

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
    eprintln!("  duration:{seconds}s");
    eprintln!("playing…");

    let engine = Arc::new(Mutex::new(engine));
    let bank = Arc::new(bank);
    // mono scratch reused inside callback via thread-local-ish Vec on the closure
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
    std::thread::sleep(Duration::from_secs(seconds));
    eprintln!("done.");
    Ok(())
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
            // normalize for display
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
