//! dj-hermes — play .strudel songs (highlight TUI, headless, or dj live).

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
use dj_hermes::api::{self, AppState, DEFAULT_API_PORT};
use dj_hermes::bounce::{
    bounce_song, write_wav_i16_stereo, DEFAULT_MEASURE_BARS, DEFAULT_WARMUP_BARS, RENDER_SR,
};
use dj_hermes::cmd::{new_deck_paths, DeckPaths};
use dj_hermes::engine::{Command, Engine};
use dj_hermes::highlight::{
    active_spans, bar_index, bar_pos, format_header, render_ansi, HighlightModel,
};
use dj_hermes::live_ui;
use dj_hermes::mcp;
use dj_hermes::repl;
use dj_hermes::sample::SampleBank;
use dj_hermes::session::SessionKind;
use dj_hermes::song::{
    bundled_slot_path, is_genre_slug, list_bundled_slots, parse_song, pick_two_slots,
    resolve_song_path,
};

#[derive(Debug, PartialEq, Eq)]
enum CliAction {
    Help,
    Dj(Vec<String>),
    Play(Vec<String>),
    Mcp,
    Render(Vec<String>),
}

fn classify_args(args: &[String]) -> CliAction {
    let Some(first) = args.first().map(|s| s.as_str()) else {
        return CliAction::Dj(Vec::new());
    };
    match first {
        "help" | "-h" | "--help" => CliAction::Help,
        "play" if args.len() == 1 => CliAction::Help,
        "play" => CliAction::Play(args[1..].to_vec()),
        "dj" => CliAction::Dj(args[1..].to_vec()),
        "mcp" => CliAction::Mcp,
        "render" => CliAction::Render(args[1..].to_vec()),
        _ => CliAction::Dj(args.to_vec()),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match classify_args(&args) {
        CliAction::Help => print_usage(),
        CliAction::Dj(rest) => {
            if let Err(e) = cmd_dj(&rest) {
                eprintln!("dj-hermes: {e}");
                std::process::exit(1);
            }
        }
        CliAction::Play(rest) => {
            if let Err(e) = cmd_play(&rest) {
                eprintln!("dj-hermes play: {e}");
                std::process::exit(1);
            }
        }
        CliAction::Mcp => {
            if let Err(e) = mcp::run() {
                eprintln!("dj-hermes mcp: {e}");
                std::process::exit(1);
            }
        }
        CliAction::Render(rest) => {
            if let Err(e) = cmd_render(&rest) {
                eprintln!("dj-hermes render: {e}");
                std::process::exit(1);
            }
        }
    }
}

fn print_usage() {
    eprintln!(
        "\
dj-hermes — Strudel live CLI

Usage:
  dj-hermes [SONG_A] [SONG_B] [--port N] [--no-api] [--text] [--resume]
  dj-hermes dj [SONG_A] [SONG_B] [--port N] [--no-api] [--text] [--resume]
  dj-hermes GENRE [--port N] [--no-api] [--text]
  dj-hermes play [SONG] [--headless] [--seconds N] [--port N] [--no-api]
                 [--midi] [--midi-only] [--midi-port NAME|INDEX] [--midi-list]
  dj-hermes play --repl [SONG_A] [SONG_B]   (same 2-deck live UI as dj; compatibility)
  dj-hermes render SONG --out PATH [--bars N] [--warmup-bars N] [--samples-dir DIR]
  dj-hermes mcp

  (no args)     DJ, empty decks (same as: dj-hermes dj)
  play          with no extra args: this help (same as --help)
  SONG          song path or bare name (default dir: songs/; .strudel/.txt optional)
                play with a SONG: 1-deck; default file songs/house/01.strudel
  SONG_A/B      optional decks (A then B; omit both to start empty)
  GENRE         bundled songs/<genre>/ ; random A/B slots and automix on (not with --resume)
  --seconds N   stop after N seconds (play + --headless, or timed highlight without prompt)
  --headless    no TUI: meta log only (for scripts / non-TTY)
  --highlight   kept for compatibility (play default is live TUI which includes highlight)
  --repl        alias path into 2-deck live UI (prefer: dj-hermes dj …)
  --repl-text   text-only REPL (same as: dj --text)
  --text        with dj: rustyline text REPL instead of highlight live UI
  --resume      load last Esc-quit A/B snapshot (~/.config/dj-hermes/resume/)

  --midi        play only: notes + CC + Bank/PC to a MIDI port (built-in synth still sounds)
  --midi-only   like --midi, but skip the built-in synth (SEQTRAK exhibit)
  --midi-port   port index or name substring (USB SEQTRAK; Linux BLE if ALSA listed)
  --midi-list   print MIDI outputs and exit (same list as --midi-port)

  render        offline bounce to 16-bit stereo WAV (no device, no API). Default
                1 warmup bar + 16 measure bars at 48 kHz. For LUFS factory gate.

  --port N      HTTP API port (default {DEFAULT_API_PORT}; env DJ_HERMES_API_PORT)
  --no-api      do not start HTTP API
  -d, --debug   Hermes debug log to file (default: ./dj-hermes.debug.log)
  --debug-log P path for -d (env: DJ_HERMES_DEBUG_LOG; e.g. C:\\temp\\strudel-debug.log)

  dj-hermes mcp
                DEPRECATED debug stdio bridge → HTTP API.
                Hermes: set mcp_servers.dj-hermes.url to http://127.0.0.1:PORT/mcp

  Default play is a 1-deck live session (highlight + » prompt + Hermes).
  Quit: q / Esc. --headless loops until Ctrl+C (or --seconds).
  dj: left=A / right=B highlight, » prompt at bottom.
  Live TUI input:
    bare text     → Hermes (play: profile play-hermes; dj: dj-hermes; needs API + MCP)
    F12           → voice (Hermes STT → Hermes; optional DJ_HERMES_STT_BASE_URL)
    /cmd …        → local (play: /house 01  /bpm 128; dj: /a house 01  /x 4)
    --no-hermes   → bare text is local again (text REPL always local)
  Flags: --no-hermes  --no-voice  --hermes-bin PATH  --hermes-profile NAME  -d/--debug

Examples:
  cargo run --                             # DJ, empty decks
  cargo run -- songs/house/01.strudel songs/four-on-the-floor/01.strudel
  cargo run -- play                        # this help
  cargo run -- --help
  cargo run -- play songs/house/01.strudel
  cargo run -- play songs/house/01.strudel --headless --seconds 8
  cargo run -- play songs/house/01.strudel --midi
  cargo run -- play --midi-list
  cargo run -- play songs/house/01.strudel --midi-only --midi-port SEQTRAK
  cargo run -- play songs/house/01.strudel --headless --seconds 8 --midi-port \"TouchOSC Bridge\"
  cargo run -- dj songs/house/01.strudel songs/four-on-the-floor/01.strudel
  cargo run -- --resume                 # last dj Esc snapshot
  cargo run -- house                    # random house A/B + automix
  cargo run -- render songs/house/01.strudel --out out/house.wav
  cargo run -- dj                          # empty decks; load from »
  # then:  暗くして   or   /house 01   or   /a house 01   or   /x 4
  # API: curl http://127.0.0.1:{DEFAULT_API_PORT}/status

Samples: ./samples (or <song>/../samples). CC0 kit docs in samples/LICENSE.md.
"
    );
}

/// Shared flags for live session (`dj` / `play` / `play --repl`).
#[derive(Debug)]
struct LiveSessionOpts {
    session: SessionKind,
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
    /// Override debug log path (`--debug-log` / `DJ_HERMES_DEBUG_LOG`).
    debug_log: Option<PathBuf>,
    /// MIDI worker; keep alive for the session. `play --midi` only.
    midi_handle: Option<dj_hermes::midi::MidiHandle>,
    /// Skip synth/sample voices; MIDI still fires (`play --midi-only`).
    midi_only: bool,
    /// Load last Esc-quit A/B snapshot instead of song paths (`dj` only).
    resume: bool,
    /// Bare bundled genre (`dj-hermes house`). `cmd_dj` fills `song_a`/`song_b`.
    auto_genre: Option<String>,
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
    let mut resume = false;
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
            "--resume" => {
                resume = true;
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
    let mut auto_genre = None;
    if songs.len() == 1 {
        let name = songs[0].to_string_lossy();
        if is_genre_slug(name.as_ref()) && !list_bundled_slots(name.as_ref()).is_empty() {
            if resume {
                return Err("--resume does not take a genre".into());
            }
            auto_genre = Some(name.into_owned());
            songs.clear();
        }
    }
    if resume && !songs.is_empty() {
        return Err("--resume does not take song paths".into());
    }

    let mut it = songs.into_iter();
    Ok(Some(LiveSessionOpts {
        session: SessionKind::Dj,
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
        midi_handle: None,
        midi_only: false,
        resume,
        auto_genre,
    }))
}

fn print_midi_ports() -> Result<(), String> {
    let names = dj_hermes::midi::list_output_ports()?;
    if names.is_empty() {
        eprintln!("no MIDI output ports");
    } else {
        for (i, n) in names.iter().enumerate() {
            eprintln!("{i}\t{n}");
        }
    }
    Ok(())
}

fn attach_midi(engine: &mut Engine, handle: Option<&dj_hermes::midi::MidiHandle>) {
    if let Some(h) = handle {
        engine.set_midi(h.sender());
    }
}

/// Stop audio first, then drop the engine's MIDI sender clone so the worker can exit.
fn stop_audio_and_midi(engine: &Arc<Mutex<Engine>>, stream: cpal::Stream) {
    drop(stream);
    for _ in 0..50 {
        if let Ok(mut e) = engine.try_lock() {
            e.shutdown_midi();
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    eprintln!("warning: engine busy; MIDI worker may take a moment to stop");
}

fn cmd_dj(args: &[String]) -> Result<(), String> {
    let Some(mut opts) = parse_live_session_args(args)? else {
        return Ok(());
    };
    if opts.resume {
        let (a, b) = dj_hermes::resume::existing_paths()?;
        if a.is_none() && b.is_none() {
            return Err(
                "no resume snapshot (Esc-quit a dj session first, then dj-hermes --resume)".into(),
            );
        }
        opts.song_a = a;
        opts.song_b = b;
    }
    if let Some(ref g) = opts.auto_genre {
        let slots = list_bundled_slots(g);
        if slots.is_empty() {
            return Err(format!("no slots in songs/{g}/"));
        }
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(1);
        let (a, b) = pick_two_slots(&slots, seed);
        opts.song_a = a.map(|nn| bundled_slot_path(g, &nn));
        opts.song_b = b.map(|nn| bundled_slot_path(g, &nn));
    }
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
    let mut midi = false;
    let mut midi_only = false;
    let mut midi_port: Option<String> = None;
    let mut midi_list = false;
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
            "--midi" => {
                midi = true;
            }
            "--midi-only" => {
                midi = true;
                midi_only = true;
            }
            "--midi-list" => {
                midi_list = true;
            }
            "--midi-port" => {
                i += 1;
                let p = args
                    .get(i)
                    .ok_or_else(|| "--midi-port needs a name or index".to_string())?;
                midi_port = Some(p.clone());
                midi = true;
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

    if midi_list {
        return print_midi_ports();
    }
    if repl_mode && midi {
        return Err("MIDI output is play-only (not `play --repl` or `dj`)".into());
    }
    let midi_handle = if midi {
        match dj_hermes::midi::connect(midi_port.as_deref()) {
            Ok(h) => Some(h),
            Err(e) if midi_only => {
                return Err(format!("midi-only: {e}"));
            }
            Err(e) => {
                eprintln!("warning: midi: {e} (continuing with built-in synth)");
                None
            }
        }
    } else {
        None
    };

    if repl_mode {
        if song_paths.len() > 2 {
            return Err("play --repl accepts at most two song paths (A then B)".into());
        }
        let mut it = song_paths.into_iter();
        return cmd_live_session(LiveSessionOpts {
            session: SessionKind::Dj,
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
            midi_handle: None,
            midi_only: false,
            resume: false,
            auto_genre: None,
        });
    }

    if song_paths.len() > 1 {
        return Err("play accepts a single SONG (use `dj A B` for dual deck)".into());
    }

    // Interactive default: 1-deck live UI (prompt + Hermes). Timed / headless
    // keep the old playback loop (scripts, --seconds demos).
    if highlight && seconds.is_none() {
        let song_a = Some(
            song_paths
                .into_iter()
                .next()
                .unwrap_or_else(|| PathBuf::from("house-01")),
        );
        return cmd_live_session(LiveSessionOpts {
            session: SessionKind::Play,
            song_a,
            song_b: None,
            with_highlight: true,
            api_enabled,
            api_port,
            hermes_enabled,
            hermes_bin,
            hermes_profile,
            voice_enabled,
            debug,
            debug_log,
            midi_handle,
            midi_only,
            resume: false,
            auto_genre: None,
        });
    }
    let song_path = song_paths.into_iter().next();

    let song_path = match song_path {
        Some(p) => resolve_song_path(&p.to_string_lossy())?,
        None => resolve_song_path("house-01")
            .or_else(|_| resolve_song_path("house/01"))
            .or_else(|_| resolve_song_path("songs/house/01.strudel"))
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
    attach_midi(&mut engine, midi_handle.as_ref());
    if midi_only {
        engine.set_midi_only(true);
    }
    let playhead = engine.playhead_handle();
    engine.load_song_immediate(0, song);

    if !highlight {
        eprintln!("dj-hermes play");
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
    let deck_paths = new_deck_paths();
    if let Ok(mut dp) = deck_paths.lock() {
        dp[0] = Some(song_path.clone());
    }

    let _api = maybe_start_api(
        api_enabled,
        api_port,
        cmd_tx,
        Arc::clone(&engine),
        SessionKind::Play,
        deck_paths,
    );

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
    stop_audio_and_midi(&engine, stream);
    Ok(())
}

/// Live session used by `dj`, `play`, and `play --repl`.
fn cmd_live_session(opts: LiveSessionOpts) -> Result<(), String> {
    let LiveSessionOpts {
        session,
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
        midi_handle: _midi_handle,
        midi_only,
        resume: _,
        auto_genre,
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
    attach_midi(&mut engine, _midi_handle.as_ref());
    if midi_only {
        engine.set_midi_only(true);
    }
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

    let _api = maybe_start_api(
        api_enabled,
        api_port,
        cmd_tx.clone(),
        Arc::clone(&engine),
        session,
        deck_paths.clone(),
    );
    if with_highlight && hermes_enabled && !api_enabled {
        eprintln!(
            "warning: Hermes needs HTTP API for MCP tools; --no-api will make tool calls fail"
        );
    }

    let hermes_handle = if with_highlight && hermes_enabled {
        start_hermes_for_tui(hermes_bin, hermes_profile, debug, debug_log, session)
    } else {
        None
    };

    let voice_handle = match (with_highlight, voice_enabled, hermes_handle.as_ref()) {
        (true, true, Some(h)) => start_voice_for_tui(h.config()),
        (true, false, Some(_)) => {
            eprintln!("voice: disabled (--no-voice)");
            None
        }
        _ => None,
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
            session,
            auto_genre.is_some() && with_highlight,
        )?;
    } else {
        eprintln!(
            "dj-hermes dj --text  |  {sample_rate} Hz, {channels} ch, {sample_format:?}  |  samples {}",
            samples_dir.display()
        );
        repl::run(cmd_tx, deck_paths, Some(Arc::clone(&engine)));
    }
    stop_audio_and_midi(&engine, stream);
    eprintln!("bye.");
    Ok(())
}

/// Start voice capture worker (Hermes STT by default; HTTP if URL is set).
fn start_voice_for_tui(
    hermes: &dj_hermes::hermes::HermesConfig,
) -> Option<dj_hermes::voice_input::VoiceHandle> {
    let mut cfg = dj_hermes::voice_input::SttConfig::from_env()?;

    if matches!(
        cfg.backend,
        dj_hermes::voice_input::SttBackend::Hermes { .. }
    ) {
        let Some(python) = dj_hermes::voice_input::resolve_hermes_python(&hermes.bin) else {
            eprintln!(
                "voice: disabled (Hermes Python が見つからない。DJ_HERMES_HERMES_PYTHON か DJ_HERMES_STT_BASE_URL を設定)"
            );
            return None;
        };
        let Some(agent_root) = dj_hermes::voice_input::hermes_agent_root_from_python(&python)
        else {
            eprintln!(
                "voice: disabled (Hermes Python が見つからない。DJ_HERMES_HERMES_PYTHON か DJ_HERMES_STT_BASE_URL を設定)"
            );
            return None;
        };
        cfg.backend = dj_hermes::voice_input::SttBackend::Hermes {
            python,
            agent_root,
            profile: hermes.profile.clone(),
        };
    }

    let mode = cfg.mode;
    let hermes_stt = matches!(
        cfg.backend,
        dj_hermes::voice_input::SttBackend::Hermes { .. }
    );
    match dj_hermes::voice_input::VoiceHandle::start(cfg) {
        Ok(h) => {
            match (hermes_stt, mode) {
                (true, dj_hermes::voice_input::VoiceMode::Vad) => {
                    eprintln!("voice: VAD listen → Hermes STT → Hermes (F12 pauses)");
                }
                (true, dj_hermes::voice_input::VoiceMode::Push) => {
                    eprintln!("voice: F12 push-to-talk → Hermes STT → Hermes");
                }
                (false, dj_hermes::voice_input::VoiceMode::Vad) => {
                    eprintln!("voice: VAD listen → local STT → Hermes (F12 pauses)");
                }
                (false, dj_hermes::voice_input::VoiceMode::Push) => {
                    eprintln!("voice: F12 push-to-talk → local STT → Hermes");
                }
            }
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
    session: SessionKind,
) -> Option<dj_hermes::hermes::HermesHandle> {
    let mut cfg = dj_hermes::hermes::HermesConfig::from_env();
    cfg.session = session;
    if let Some(bin) = hermes_bin {
        cfg.bin = bin;
    }
    if let Some(profile) = hermes_profile {
        cfg.profile = profile;
    } else if session == SessionKind::Play {
        let env_set = std::env::var("DJ_HERMES_HERMES_PROFILE")
            .ok()
            .is_some_and(|s| !s.trim().is_empty());
        if !env_set {
            cfg.profile = dj_hermes::hermes::PLAY_PROFILE.to_string();
        }
    }
    if debug {
        cfg.debug = true;
    }
    if let Some(p) = debug_log {
        cfg.debug = true;
        cfg.debug_log_path = p;
    }
    if !dj_hermes::hermes::hermes_bin_available(&cfg.bin) {
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
        "        timeout={}s  (turns: profile agent.max_turns)",
        cfg.timeout.as_secs()
    );
    if cfg.debug {
        match dj_hermes::hermes::init_debug_log(&cfg) {
            Ok(abs) => {
                cfg.debug_log_path = abs.clone();
                eprintln!("        debug ON → file {}", abs.display());
                dj_hermes::hermes::debug_log(
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
    Some(dj_hermes::hermes::HermesHandle::start(cfg))
}

/// Start HTTP API in a background thread when enabled. Keeps `Sender` alive via clone.
fn maybe_start_api(
    enabled: bool,
    port: u16,
    tx: Sender<Command>,
    engine: Arc<Mutex<Engine>>,
    session: SessionKind,
    deck_paths: DeckPaths,
) -> Option<std::thread::JoinHandle<()>> {
    if !enabled {
        return None;
    }
    let state = AppState {
        tx,
        engine,
        session,
        deck_paths,
    };
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

#[derive(Debug)]
struct RenderOpts {
    song: String,
    out: PathBuf,
    bars: usize,
    warmup_bars: usize,
    samples_dir: Option<PathBuf>,
}

fn parse_render_args(args: &[String]) -> Result<RenderOpts, String> {
    let mut song: Option<String> = None;
    let mut out: Option<PathBuf> = None;
    let mut bars = DEFAULT_MEASURE_BARS;
    let mut warmup_bars = DEFAULT_WARMUP_BARS;
    let mut samples_dir: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--out" => {
                i += 1;
                let p = args
                    .get(i)
                    .ok_or_else(|| "--out needs a path".to_string())?;
                out = Some(PathBuf::from(p));
            }
            "--bars" => {
                i += 1;
                let s = args
                    .get(i)
                    .ok_or_else(|| "--bars needs a number".to_string())?;
                let n: usize = s.parse().map_err(|_| format!("bad --bars value: {s}"))?;
                if n == 0 {
                    return Err("--bars must be >= 1".into());
                }
                bars = n;
            }
            "--warmup-bars" => {
                i += 1;
                let s = args
                    .get(i)
                    .ok_or_else(|| "--warmup-bars needs a number".to_string())?;
                let n: usize = s
                    .parse()
                    .map_err(|_| format!("bad --warmup-bars value: {s}"))?;
                warmup_bars = n;
            }
            "--samples-dir" => {
                i += 1;
                let p = args
                    .get(i)
                    .ok_or_else(|| "--samples-dir needs a path".to_string())?;
                samples_dir = Some(PathBuf::from(p));
            }
            "-h" | "--help" => {
                return Err(
                    "usage: dj-hermes render SONG --out PATH [--bars N] [--warmup-bars N] [--samples-dir DIR]"
                        .into(),
                );
            }
            flag if flag.starts_with('-') => {
                return Err(format!("unknown render flag: {flag}"));
            }
            other => {
                if song.is_some() {
                    return Err(format!("unexpected extra argument: {other}"));
                }
                song = Some(other.to_string());
            }
        }
        i += 1;
    }
    let song = song.ok_or_else(|| "render needs a song path".to_string())?;
    let out = out.ok_or_else(|| "render needs --out PATH".to_string())?;
    Ok(RenderOpts {
        song,
        out,
        bars,
        warmup_bars,
        samples_dir,
    })
}

fn cmd_render(args: &[String]) -> Result<(), String> {
    if args.iter().any(|a| a == "-h" || a == "--help") {
        eprintln!(
            "usage: dj-hermes render SONG --out PATH [--bars N] [--warmup-bars N] [--samples-dir DIR]"
        );
        return Ok(());
    }
    let opts = parse_render_args(args)?;
    let song_path = resolve_song_path(&opts.song)?;
    let text = std::fs::read_to_string(&song_path)
        .map_err(|e| format!("read {}: {e}", song_path.display()))?;
    let song = parse_song(&text, song_path.to_string_lossy().as_ref())?;
    let samples_dir = match &opts.samples_dir {
        Some(p) => {
            if !p.is_dir() {
                return Err(format!("samples dir not found: {}", p.display()));
            }
            p.clone()
        }
        None => resolve_samples_dir(Some(&song_path))?,
    };
    let bank = SampleBank::load_dir(&samples_dir, RENDER_SR);
    let bounce = bounce_song(song, &bank, RENDER_SR, opts.warmup_bars, opts.bars)?;
    write_wav_i16_stereo(&opts.out, bounce.sample_rate, &bounce.samples)?;
    eprintln!(
        "wrote {}  ({:.1} BPM, {} bars after {} warmup, peak={:.3}, {} Hz)",
        opts.out.display(),
        bounce.bpm,
        bounce.bars,
        bounce.warmup_bars,
        bounce.peak_abs(),
        bounce.sample_rate
    );
    Ok(())
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
    fn classify_args_empty_is_dj() {
        assert_eq!(classify_args(&s(&[])), CliAction::Dj(vec![]));
    }

    #[test]
    fn classify_args_play_alone_is_help() {
        assert_eq!(classify_args(&s(&["play"])), CliAction::Help);
    }

    #[test]
    fn classify_args_long_help_is_help() {
        assert_eq!(classify_args(&s(&["--help"])), CliAction::Help);
    }

    #[test]
    fn classify_args_short_help_is_help() {
        assert_eq!(classify_args(&s(&["-h"])), CliAction::Help);
    }

    #[test]
    fn classify_args_help_word_is_help() {
        assert_eq!(classify_args(&s(&["help"])), CliAction::Help);
    }

    #[test]
    fn classify_args_play_with_song_is_play() {
        assert_eq!(
            classify_args(&s(&["play", "songs/house/01.strudel"])),
            CliAction::Play(s(&["songs/house/01.strudel"]))
        );
    }

    #[test]
    fn classify_args_dj_prefix_strips_cmd() {
        assert_eq!(
            classify_args(&s(&["dj", "a.strudel"])),
            CliAction::Dj(s(&["a.strudel"]))
        );
    }

    #[test]
    fn classify_args_two_song_paths_are_dj() {
        assert_eq!(
            classify_args(&s(&[
                "songs/house/01.strudel",
                "songs/four-on-the-floor/01.strudel"
            ])),
            CliAction::Dj(s(&[
                "songs/house/01.strudel",
                "songs/four-on-the-floor/01.strudel"
            ]))
        );
    }

    #[test]
    fn classify_args_text_flag_is_dj() {
        assert_eq!(
            classify_args(&s(&["--text"])),
            CliAction::Dj(s(&["--text"]))
        );
    }

    #[test]
    fn classify_args_mcp_is_mcp() {
        assert_eq!(classify_args(&s(&["mcp"])), CliAction::Mcp);
    }

    #[test]
    fn classify_args_render_keeps_rest() {
        let rest = s(&["s.strudel", "--out", "o.wav"]);
        assert_eq!(
            classify_args(&s(&["render", "s.strudel", "--out", "o.wav"])),
            CliAction::Render(rest.clone())
        );
        assert!(!rest.iter().any(|a| a == "play"));
        assert_eq!(rest.len(), 3);
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
            "songs/house/01.strudel",
            "songs/four-on-the-floor/01.strudel",
            "--no-api",
            "--text",
        ]))
        .unwrap()
        .unwrap();
        assert_eq!(
            opts.song_a.as_deref(),
            Some(Path::new("songs/house/01.strudel"))
        );
        assert_eq!(
            opts.song_b.as_deref(),
            Some(Path::new("songs/four-on-the-floor/01.strudel"))
        );
        assert!(!opts.with_highlight);
        assert!(!opts.api_enabled);
        assert!(!opts.resume);
        assert!(opts.auto_genre.is_none());
    }

    #[test]
    fn dj_args_rejects_three_songs() {
        let err =
            parse_live_session_args(&s(&["a.strudel", "b.strudel", "c.strudel"])).unwrap_err();
        assert!(err.contains("too many"));
    }

    #[test]
    fn dj_args_resume_flag() {
        let opts = parse_live_session_args(&s(&["--resume"])).unwrap().unwrap();
        assert!(opts.resume);
        assert!(opts.song_a.is_none());
        assert!(opts.song_b.is_none());
        assert!(opts.auto_genre.is_none());
    }

    #[test]
    fn dj_args_resume_rejects_song_paths() {
        let err = parse_live_session_args(&s(&["--resume", "songs/house/01.strudel"])).unwrap_err();
        assert!(err.contains("--resume does not take song paths"), "{err}");
    }

    #[test]
    fn dj_args_genre_house() {
        let opts = parse_live_session_args(&s(&["house"])).unwrap().unwrap();
        assert_eq!(opts.auto_genre.as_deref(), Some("house"));
        assert!(opts.song_a.is_none());
        assert!(opts.song_b.is_none());
        assert!(!opts.resume);
    }

    #[test]
    fn dj_args_resume_rejects_genre() {
        let err = parse_live_session_args(&s(&["house", "--resume"])).unwrap_err();
        assert!(err.contains("--resume does not take a genre"), "{err}");
        let err = parse_live_session_args(&s(&["--resume", "house"])).unwrap_err();
        assert!(err.contains("--resume does not take a genre"), "{err}");
    }

    #[test]
    fn dj_args_slot_and_ext_are_song_paths() {
        let opts = parse_live_session_args(&s(&["house/01"])).unwrap().unwrap();
        assert!(opts.auto_genre.is_none());
        assert_eq!(opts.song_a.as_deref(), Some(Path::new("house/01")));
        let opts = parse_live_session_args(&s(&["house.strudel"]))
            .unwrap()
            .unwrap();
        assert!(opts.auto_genre.is_none());
        assert_eq!(opts.song_a.as_deref(), Some(Path::new("house.strudel")));
    }

    #[test]
    fn render_args_need_song_and_out() {
        let err = parse_render_args(&s(&[])).unwrap_err();
        assert!(err.contains("song"), "{err}");
        let err = parse_render_args(&s(&["songs/house/01.strudel"])).unwrap_err();
        assert!(err.contains("--out"), "{err}");
    }

    #[test]
    fn render_args_ok() {
        let opts = parse_render_args(&s(&[
            "songs/house/01.strudel",
            "--out",
            "out/house.wav",
            "--bars",
            "4",
            "--warmup-bars",
            "1",
        ]))
        .unwrap();
        assert_eq!(opts.song, "songs/house/01.strudel");
        assert_eq!(opts.out, PathBuf::from("out/house.wav"));
        assert_eq!(opts.bars, 4);
        assert_eq!(opts.warmup_bars, 1);
    }
}
