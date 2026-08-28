//! strudel-rs — play .strudel songs (highlight TUI, headless, or dj live).

use std::io::{stdout, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig};
use crossbeam::channel::{unbounded, Receiver, Sender};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, execute, terminal, QueueableCommand};
use strudel_rs::api::{self, AppState, DEFAULT_API_PORT};
use strudel_rs::cmd::{new_deck_paths, DeckPaths};
use strudel_rs::engine::{Command, Engine};
use strudel_rs::highlight::{
    active_spans, bar_index, bar_pos, format_header, render_ansi, HighlightModel,
};
use strudel_rs::live_ui;
use strudel_rs::mcp;
use strudel_rs::repl;
use strudel_rs::sample::SampleBank;
use strudel_rs::song::{parse_song, resolve_song_path};

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
        "dj" => {
            if let Err(e) = cmd_dj(&args) {
                eprintln!("strudel-rs dj: {e}");
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
  strudel-rs dj [SONG_A] [SONG_B] [--port N] [--no-api] [--text]
  strudel-rs play --repl [SONG_A] [SONG_B]   (same live UI as dj; kept for compatibility)
  strudel-rs mcp

  SONG          song path or bare name (default dir: songs/; .strudel/.txt optional)
                play default: songs/smoke.strudel
  SONG_A/B      optional decks for dj (A then B; omit both to start empty)
  --seconds N   stop after N seconds (play only; omit to loop until quit)
  --headless    no TUI: meta log only (for scripts / non-TTY)
  --highlight   explicit highlight TUI (default; also: --hl)
  --repl        alias path into live UI (prefer: strudel-rs dj …)
  --repl-text   text-only REPL (same as: dj --text)
  --text        with dj: rustyline text REPL instead of highlight live UI

  --port N      HTTP API port (default {DEFAULT_API_PORT}; env STRUDEL_API_PORT)
  --no-api      do not start HTTP API
  -d, --debug   Hermes debug log to file (default: ./strudel-rs.debug.log)
  --debug-log P path for -d (env: STRUDEL_DEBUG_LOG; e.g. C:\\temp\\strudel-debug.log)

  strudel-rs mcp
                DEPRECATED debug stdio bridge → HTTP API.
                Hermes: set mcp_servers.strudel.url to http://127.0.0.1:PORT/mcp

  Default play loops forever (TUI: q / Esc; headless: Ctrl+C).
  dj: left=A / right=B highlight, » prompt at bottom.
  Live TUI input:
    bare text     → Hermes (profile dj-hermes; needs API + MCP)
    F12           → voice (xAI STT → Hermes; needs XAI_API_KEY)
    /cmd …        → local (e.g. /a load smoke  /x 4  /bpm 128  /viz  /help)
    --no-hermes   → bare text is local again (text REPL always local)
  Flags: --no-hermes  --no-voice  --hermes-bin PATH  --hermes-profile NAME  -d/--debug

Examples:
  cargo run -- play songs/smoke.strudel
  cargo run -- dj songs/techno1.strudel songs/ambient1.strudel
  cargo run -- dj                          # empty decks; load from »
  # then:  暗くして   or   /x 4
  # API: curl http://127.0.0.1:{DEFAULT_API_PORT}/status

Samples: ./samples (or <song>/../samples). CC0 kit docs in samples/LICENSE.md.
Exhibit Hermes setup: docs/exhibit/README.md
"
    );
}

/// Shared flags for live dual-deck session (`dj` / `play --repl`).
#[derive(Debug)]
struct LiveSessionOpts {
    song_a: Option<PathBuf>,
    song_b: Option<PathBuf>,
    with_highlight: bool,
    api_enabled: bool,
    api_port: u16,
    /// When false, live TUI treats bare input as local commands (no Hermes).
    hermes_enabled: bool,
    hermes_bin: Option<PathBuf>,
    hermes_profile: Option<String>,
    /// When false, skip F12 voice capture even if STT keys are set.
    voice_enabled: bool,
    /// Hermes spawn/wait I/O tracing (`-d` / `--debug`).
    debug: bool,
    /// Override debug log path (`--debug-log` / `STRUDEL_DEBUG_LOG`).
    debug_log: Option<PathBuf>,
}

/// Parse `dj` / live-session CLI. Returns `Ok(None)` when `--help` was printed.
fn parse_live_session_args(args: &[String]) -> Result<Option<LiveSessionOpts>, String> {
    let mut songs: Vec<PathBuf> = Vec::new();
    let mut with_highlight = true;
    let mut api_enabled = true;
    let mut hermes_enabled = true;
    let mut hermes_bin: Option<PathBuf> = None;
    let mut hermes_profile: Option<String> = None;
    let mut voice_enabled = true;
    let mut debug = false;
    let mut debug_log: Option<PathBuf> = None;
    let mut cli_port: Option<u16> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--port" => {
                i += 1;
                let s = args
                    .get(i)
                    .ok_or_else(|| "--port needs a number".to_string())?;
                let p: u16 = s.parse().map_err(|_| format!("bad --port value: {s}"))?;
                if p == 0 {
                    return Err("--port must be 1..=65535".into());
                }
                cli_port = Some(p);
            }
            "--no-api" => {
                api_enabled = false;
            }
            "--no-hermes" => {
                hermes_enabled = false;
            }
            "--no-voice" => {
                voice_enabled = false;
            }
            "--hermes-bin" => {
                i += 1;
                let p = args
                    .get(i)
                    .ok_or_else(|| "--hermes-bin needs a path".to_string())?;
                hermes_bin = Some(PathBuf::from(p));
            }
            "--hermes-profile" => {
                i += 1;
                let p = args
                    .get(i)
                    .ok_or_else(|| "--hermes-profile needs a name".to_string())?;
                hermes_profile = Some(p.clone());
            }
            "-d" | "--debug" => {
                debug = true;
            }
            "--debug-log" => {
                i += 1;
                let p = args
                    .get(i)
                    .ok_or_else(|| "--debug-log needs a path".to_string())?;
                debug_log = Some(PathBuf::from(p));
                debug = true; // imply -d
            }
            "--text" | "--repl-text" => {
                with_highlight = false;
            }
            "--help" | "-h" => {
                print_usage();
                return Ok(None);
            }
            flag if flag.starts_with('-') => {
                return Err(format!("unknown flag: {flag}"));
            }
            path => {
                songs.push(PathBuf::from(path));
            }
        }
        i += 1;
    }

    if songs.len() > 2 {
        return Err("too many song paths (expected at most SONG_A SONG_B)".into());
    }

    let mut it = songs.into_iter();
    Ok(Some(LiveSessionOpts {
        song_a: it.next(),
        song_b: it.next(),
        with_highlight,
        api_enabled,
        api_port: api::resolve_port(cli_port),
        hermes_enabled,
        hermes_bin,
        hermes_profile,
        voice_enabled,
        debug,
        debug_log,
    }))
}

fn cmd_dj(args: &[String]) -> Result<(), String> {
    let Some(opts) = parse_live_session_args(args)? else {
        return Ok(());
    };
    cmd_live_session(opts)
}

fn cmd_play(args: &[String]) -> Result<(), String> {
    let mut song_paths: Vec<PathBuf> = Vec::new();
    let mut seconds: Option<u64> = None;
    let mut highlight = true;
    let mut repl_mode = false;
    let mut repl_text = false;
    let mut api_enabled = true;
    let mut hermes_enabled = true;
    let mut hermes_bin: Option<PathBuf> = None;
    let mut hermes_profile: Option<String> = None;
    let mut voice_enabled = true;
    let mut debug = false;
    let mut debug_log: Option<PathBuf> = None;
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
            "--port" => {
                i += 1;
                let s = args
                    .get(i)
                    .ok_or_else(|| "--port needs a number".to_string())?;
                let p: u16 = s.parse().map_err(|_| format!("bad --port value: {s}"))?;
                if p == 0 {
                    return Err("--port must be 1..=65535".into());
                }
                cli_port = Some(p);
            }
            "--no-api" => {
                api_enabled = false;
            }
            "--no-hermes" => {
                hermes_enabled = false;
            }
            "--no-voice" => {
                voice_enabled = false;
            }
            "--hermes-bin" => {
                i += 1;
                let p = args
                    .get(i)
                    .ok_or_else(|| "--hermes-bin needs a path".to_string())?;
                hermes_bin = Some(PathBuf::from(p));
            }
            "--hermes-profile" => {
                i += 1;
                let p = args
                    .get(i)
                    .ok_or_else(|| "--hermes-profile needs a name".to_string())?;
                hermes_profile = Some(p.clone());
            }
            "-d" | "--debug" => {
                debug = true;
            }
            "--debug-log" => {
                i += 1;
                let p = args
                    .get(i)
                    .ok_or_else(|| "--debug-log needs a path".to_string())?;
                debug_log = Some(PathBuf::from(p));
                debug = true;
            }
            "--help" | "-h" => {
                print_usage();
                return Ok(());
            }
            flag if flag.starts_with('-') => {
                return Err(format!("unknown flag: {flag}"));
            }
            path => {
                song_paths.push(PathBuf::from(path));
            }
        }
        i += 1;
    }

    let api_port = api::resolve_port(cli_port);

    if repl_mode {
        if song_paths.len() > 2 {
            return Err("play --repl accepts at most two song paths (A then B)".into());
        }
        let mut it = song_paths.into_iter();
        return cmd_live_session(LiveSessionOpts {
            song_a: it.next(),
            song_b: it.next(),
            with_highlight: !repl_text,
            api_enabled,
            api_port,
            hermes_enabled,
            hermes_bin,
            hermes_profile,
            voice_enabled,
            debug,
            debug_log,
        });
    }

    if song_paths.len() > 1 {
        return Err("play accepts a single SONG (use `dj A B` for dual deck)".into());
    }
    let song_path = song_paths.into_iter().next();

    let song_path = match song_path {
        Some(p) => resolve_song_path(&p.to_string_lossy())?,
        None => resolve_song_path("smoke")
            .or_else(|_| resolve_song_path("songs/smoke.strudel"))
            .map_err(|e| format!("default song: {e}"))?,
    };
    let text = std::fs::read_to_string(&song_path)
        .map_err(|e| format!("read {}: {e}", song_path.display()))?;
    let path_str = song_path.to_string_lossy().into_owned();
    let song = parse_song(&text, &path_str)?;

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| "no default output device".to_string())?;
    let (stream_config, sample_rate, channels, sample_format) = pick_output_config(&device)?;

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
        eprintln!("  device:  {sample_rate} Hz, {channels} ch, {sample_format:?}");
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

/// Dual-deck live session used by `dj` and `play --repl`.
fn cmd_live_session(opts: LiveSessionOpts) -> Result<(), String> {
    let LiveSessionOpts {
        song_a,
        song_b,
        with_highlight,
        api_enabled,
        api_port,
        hermes_enabled,
        hermes_bin,
        hermes_profile,
        voice_enabled,
        debug,
        debug_log,
    } = opts;

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| "no default output device".to_string())?;
    let (stream_config, sample_rate, channels, sample_format) = pick_output_config(&device)?;

    let samples_dir = resolve_samples_dir(song_a.as_deref().or(song_b.as_deref()))?;
    let bank = SampleBank::load_dir(&samples_dir, sample_rate);
    if bank.names().is_empty() {
        eprintln!(
            "warning: no WAV samples from {} (synth-only still works)",
            samples_dir.display()
        );
    }

    let mut engine = Engine::new(sample_rate, 120.0);
    let deck_paths: DeckPaths = new_deck_paths();
    let mut initial_a: Option<HighlightModel> = None;
    let mut initial_b: Option<HighlightModel> = None;

    if let Some(ref path_in) = song_a {
        let path = resolve_song_path(&path_in.to_string_lossy())?;
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        let song = parse_song(&text, &path.to_string_lossy())?;
        let bpm = song.bpm.unwrap_or(120.0);
        engine.transport.set_bpm(bpm);
        if with_highlight {
            initial_a = Some(HighlightModel::from_song(&song, sample_rate));
        }
        engine.load_song_immediate(0, song);
        if let Ok(mut dp) = deck_paths.lock() {
            dp[0] = Some(path.clone());
        }
        if !with_highlight {
            eprintln!("loaded deck A: {}", path.display());
        }
    }

    if let Some(ref path_in) = song_b {
        let path = resolve_song_path(&path_in.to_string_lossy())?;
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        let song = parse_song(&text, &path.to_string_lossy())?;
        // Keep master BPM from A when both set; otherwise take B's tempo.
        if song_a.is_none() {
            let bpm = song.bpm.unwrap_or(120.0);
            engine.transport.set_bpm(bpm);
        }
        if with_highlight {
            initial_b = Some(HighlightModel::from_song(&song, sample_rate));
        }
        engine.load_song_immediate(1, song);
        if let Ok(mut dp) = deck_paths.lock() {
            dp[1] = Some(path.clone());
        }
        if !with_highlight {
            eprintln!("loaded deck B: {}", path.display());
        }
    }

    let (cmd_tx, cmd_rx) = unbounded::<Command>();
    let playhead = engine.playhead_handle();
    let engine = Arc::new(Mutex::new(engine));
    let bank = Arc::new(bank);

    let _api = maybe_start_api(api_enabled, api_port, cmd_tx.clone(), Arc::clone(&engine));
    if with_highlight && hermes_enabled && !api_enabled {
        eprintln!(
            "warning: Hermes needs HTTP API for MCP tools; --no-api will make tool calls fail"
        );
    }

    let hermes_handle = if with_highlight && hermes_enabled {
        start_hermes_for_tui(hermes_bin, hermes_profile, debug, debug_log)
    } else {
        None
    };

    let voice_handle = if with_highlight && hermes_handle.is_some() && voice_enabled {
        start_voice_for_tui()
    } else {
        if with_highlight && hermes_handle.is_some() && !voice_enabled {
            eprintln!("voice: disabled (--no-voice)");
        }
        None
    };

    let stream = build_stream(
        &device,
        stream_config,
        channels,
        Arc::clone(&engine),
        Arc::clone(&bank),
        Some(cmd_rx),
    )?;
    stream.play().map_err(|e| format!("play stream: {e}"))?;

    if with_highlight {
        live_ui::run(
            cmd_tx,
            deck_paths,
            Arc::clone(&engine),
            playhead,
            sample_rate,
            initial_a,
            initial_b,
            hermes_handle,
            voice_handle,
        )?;
    } else {
        eprintln!(
            "strudel-rs dj --text  |  {sample_rate} Hz, {channels} ch, {sample_format:?}  |  samples {}",
            samples_dir.display()
        );
        repl::run(cmd_tx, deck_paths, Some(Arc::clone(&engine)));
    }
    eprintln!("bye.");
    Ok(())
}

/// Start voice capture worker when xAI STT credentials are present.
fn start_voice_for_tui() -> Option<strudel_rs::voice_input::VoiceHandle> {
    let Some(cfg) = strudel_rs::voice_input::SttConfig::from_env() else {
        eprintln!("voice: disabled (set XAI_API_KEY or STRUDEL_STT_API_KEY for F12 STT → Hermes)");
        return None;
    };
    match strudel_rs::voice_input::VoiceHandle::start(cfg) {
        Ok(h) => {
            eprintln!("voice: F12 push-to-talk (xAI STT → Hermes)");
            Some(h)
        }
        Err(e) => {
            eprintln!("voice: disabled ({e})");
            None
        }
    }
}

/// Start Hermes worker for live TUI, or `None` if binary missing (bare → local fallback).
fn start_hermes_for_tui(
    hermes_bin: Option<PathBuf>,
    hermes_profile: Option<String>,
    debug: bool,
    debug_log: Option<PathBuf>,
) -> Option<strudel_rs::hermes::HermesHandle> {
    let mut cfg = strudel_rs::hermes::HermesConfig::from_env();
    if let Some(bin) = hermes_bin {
        cfg.bin = bin;
    }
    if let Some(profile) = hermes_profile {
        cfg.profile = profile;
    }
    if debug {
        cfg.debug = true;
    }
    if let Some(p) = debug_log {
        cfg.debug = true;
        cfg.debug_log_path = p;
    }
    if !strudel_rs::hermes::hermes_bin_available(&cfg.bin) {
        eprintln!(
            "warning: hermes binary not found ({}); bare input falls back to local commands",
            cfg.bin.display()
        );
        eprintln!("         install Hermes or pass --hermes-bin / --no-hermes");
        return None;
    }
    eprintln!(
        "hermes: profile={} (exhibit-isolated)  bin={}",
        cfg.profile,
        cfg.bin.display()
    );
    eprintln!(
        "        timeout={}s  (turns: profile agent.max_turns)  docs: docs/exhibit/README.md",
        cfg.timeout.as_secs()
    );
    if cfg.debug {
        match strudel_rs::hermes::init_debug_log(&cfg) {
            Ok(abs) => {
                cfg.debug_log_path = abs.clone();
                eprintln!("        debug ON → file {}", abs.display());
                strudel_rs::hermes::debug_log(
                    &cfg,
                    format!(
                        "session start profile={} bin={} timeout={}s log={}",
                        cfg.profile,
                        cfg.bin.display(),
                        cfg.timeout.as_secs(),
                        abs.display()
                    ),
                );
            }
            Err(e) => {
                eprintln!("        debug log init FAILED: {e}");
                eprintln!("        (tried path: {})", cfg.debug_log_path.display());
            }
        }
    }
    Some(strudel_rs::hermes::HermesHandle::start(cfg))
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

/// Pick an output config that works in **WASAPI shared mode** (Windows).
///
/// Shared mode only accepts the device mix format (`default_output_config`).
/// Scanning `supported_output_configs` often yields exclusive-only rates/formats
/// and fails with "Stream configuration is not supported in shared mode".
fn pick_output_config(
    device: &cpal::Device,
) -> Result<(StreamConfig, u32, usize, SampleFormat), String> {
    let default = device
        .default_output_config()
        .map_err(|e| format!("output config: {e}"))?;
    let sample_format = default.sample_format();
    if sample_format != SampleFormat::F32 {
        return Err(format!(
            "output device sample format {sample_format:?} is not F32; \
             shared-mode stream uses the default mix format only (engine needs F32)"
        ));
    }
    let sample_rate = default.sample_rate();
    let channels = default.channels() as usize;
    // Keep buffer_size from SupportedStreamConfig → StreamConfig (Default).
    let stream_config: StreamConfig = default.into();
    Ok((stream_config, sample_rate, channels, sample_format))
}

fn build_stream(
    device: &cpal::Device,
    stream_config: StreamConfig,
    channels: usize,
    engine: Arc<Mutex<Engine>>,
    bank: Arc<SampleBank>,
    cmd_rx: Option<Receiver<Command>>,
) -> Result<cpal::Stream, String> {
    let mut stereo = Vec::<f32>::new();
    let stream = device
        .build_output_stream(
            stream_config,
            move |data: &mut [f32], _| {
                let frames = data.len() / channels.max(1);
                let need = frames * 2;
                if stereo.len() < need {
                    stereo.resize(need, 0.0);
                }
                let stereo_buf = &mut stereo[..need];
                stereo_buf.fill(0.0);

                if let Ok(mut eng) = engine.try_lock() {
                    if let Some(rx) = &cmd_rx {
                        while let Ok(cmd) = rx.try_recv() {
                            eng.push_command(cmd);
                        }
                    }
                    eng.process(stereo_buf, &bank);
                }

                if channels <= 1 {
                    // Downmix mid for rare mono devices.
                    for i in 0..frames {
                        data[i] = 0.5 * (stereo_buf[i * 2] + stereo_buf[i * 2 + 1]);
                    }
                } else {
                    // ch0 = L, ch1 = R; extra channels silent (avoids multichannel bleed).
                    for frame_i in 0..frames {
                        let base = frame_i * channels;
                        data[base] = stereo_buf[frame_i * 2];
                        data[base + 1] = stereo_buf[frame_i * 2 + 1];
                        for c in 2..channels {
                            data[base + c] = 0.0;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn s(args: &[&str]) -> Vec<String> {
        args.iter().map(|a| (*a).to_string()).collect()
    }

    #[test]
    fn dj_args_empty_ok() {
        let opts = parse_live_session_args(&s(&[])).unwrap().unwrap();
        assert!(opts.song_a.is_none());
        assert!(opts.song_b.is_none());
        assert!(opts.with_highlight);
        assert!(opts.api_enabled);
    }

    #[test]
    fn dj_args_two_songs_and_flags() {
        let opts = parse_live_session_args(&s(&[
            "songs/techno1.strudel",
            "songs/ambient1.strudel",
            "--no-api",
            "--text",
        ]))
        .unwrap()
        .unwrap();
        assert_eq!(
            opts.song_a.as_deref(),
            Some(Path::new("songs/techno1.strudel"))
        );
        assert_eq!(
            opts.song_b.as_deref(),
            Some(Path::new("songs/ambient1.strudel"))
        );
        assert!(!opts.with_highlight);
        assert!(!opts.api_enabled);
    }

    #[test]
    fn dj_args_rejects_three_songs() {
        let err =
            parse_live_session_args(&s(&["a.strudel", "b.strudel", "c.strudel"])).unwrap_err();
        assert!(err.contains("too many"));
    }
}
