//! Shared live/REPL command parser (colon-free, deck-first).
//!
//! Live TUI (with Hermes): prefix local commands with `/` (e.g. `/x 4`).
//! Bare natural language goes to Hermes. Text REPL (`--text`) still uses bare commands.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crossbeam::channel::Sender;

use crate::engine::{Command, Engine};
use crate::song::{
    ensure_user_songs_dir, parse_song, resolve_song_path, resolve_user_song_save_path, Song,
    MAX_SONG_CONTENT_BYTES,
};

/// Which path is loaded on deck A / B (for reload targeting).
pub type DeckPaths = Arc<Mutex<[Option<PathBuf>; 2]>>;

pub fn new_deck_paths() -> DeckPaths {
    Arc::new(Mutex::new([None, None]))
}

pub const HELP: &str = "\
# local commands (live TUI: prefix with / )
a|b load <file>     load song (bare name → songs/; .strudel/.txt optional)
a|b reload          re-read last loaded file onto this deck (next bar)
a|b save [name]     write current deck source to user library (no playback change)
a|b mute <track>    mute track (next bar)
a|b unmute <track>
a|b gain <0..1>     fader (immediate)
a|b head <bar>      cue song bar (1-based; applies next bar). alias: cue
x [bars]            xfade to the other deck (default 4)
a x [bars]          xfade to deck A
b x [bars]          xfade to deck B
mix long A|B [bars] long mix (EQ bass-swap + xfade, next phrase)
mix cut A|B         cut-in next bar (EQ reset)
mix fill <kind> A|B [8n|4n]  delay|lpf|flash|riser|switch then cut-in
mix hold            freeze xfade now
bpm <n>             BPM from next bar
hush                stop all (immediate)  [operator]
status
help
quit / q            [operator]
viz [on|off]        toggle body punchcard / highlight (live TUI)
vfx [on|off]        toggle hit VFX overlay (live TUI; default on). aliases: dopa, flash
↑↓ / Tab / Enter    suggest overlay: select / apply / apply+run (live TUI)
Esc                 dismiss suggest (or quit when prompt empty)

# live TUI + Hermes
bare text           send to Hermes (DJ assistant)
/…                  local command (e.g. /bpm 128, /a load house-01, /viz)
";

/// How live TUI should route a prompt line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiveInput {
    Empty,
    /// Local command body (no leading `/`).
    LocalCommand(String),
    /// Natural-language prompt for Hermes.
    HermesPrompt(String),
}

/// Classify a live TUI input line.
///
/// When `hermes_enabled`, only lines starting with `/` (after optional `:`) are
/// local commands; everything else is a Hermes prompt.
/// When Hermes is off, bare lines are local (optional leading `/` still stripped).
pub fn classify_live_input(line: &str, hermes_enabled: bool) -> LiveInput {
    let line = line.trim();
    if line.is_empty() {
        return LiveInput::Empty;
    }
    // Legacy colon prefix (muscle memory).
    let line = line.strip_prefix(':').unwrap_or(line).trim_start();
    if line.is_empty() {
        return LiveInput::Empty;
    }
    if hermes_enabled {
        if let Some(rest) = line.strip_prefix('/') {
            return LiveInput::LocalCommand(rest.trim_start().to_string());
        }
        return LiveInput::HermesPrompt(line.to_string());
    }
    // Offline / --no-hermes: bare commands; optional `/` still accepted.
    let body = line.strip_prefix('/').unwrap_or(line).trim_start();
    if body.is_empty() {
        return LiveInput::Empty;
    }
    LiveInput::LocalCommand(body.to_string())
}

/// True if the local command body is help.
pub fn is_help_body(body: &str) -> bool {
    matches!(body.trim(), "help" | "h" | "?")
}

pub struct ExecResult {
    pub quit: bool,
    pub messages: Vec<String>,
    /// Song just loaded (for highlight model update in live UI).
    pub loaded: Option<(usize, Song)>,
}

impl ExecResult {
    fn msg(s: impl Into<String>) -> Self {
        Self {
            quit: false,
            messages: vec![s.into()],
            loaded: None,
        }
    }

    fn msgs(messages: Vec<String>) -> Self {
        Self {
            quit: false,
            messages,
            loaded: None,
        }
    }

    fn quit() -> Self {
        Self {
            quit: true,
            messages: Vec::new(),
            loaded: None,
        }
    }
}

/// Parse and run one command line. `engine` is optional (status / default xfade target).
pub fn exec(
    line: &str,
    tx: &Sender<Command>,
    deck_paths: &DeckPaths,
    engine: Option<&Arc<Mutex<Engine>>>,
) -> ExecResult {
    let line = line.trim();
    if line.is_empty() {
        return ExecResult {
            quit: false,
            messages: Vec::new(),
            loaded: None,
        };
    }
    // Strip a leading ':' so old muscle memory still works a bit.
    let line = line.strip_prefix(':').unwrap_or(line);
    let args: Vec<&str> = line.split_whitespace().collect();
    if args.is_empty() {
        return ExecResult {
            quit: false,
            messages: Vec::new(),
            loaded: None,
        };
    }

    // Global commands (no deck prefix)
    match args[0] {
        "quit" | "q" => {
            let _ = tx.send(Command::Hush);
            return ExecResult::quit();
        }
        "help" | "h" | "?" => return ExecResult::msg(HELP.trim_end()),
        "hush" => {
            let _ = tx.send(Command::Hush);
            return ExecResult::msg("(hushed)");
        }
        "bpm" if args.len() == 2 => {
            return match args[1].parse::<f64>() {
                Ok(b) if b > 0.0 && b.is_finite() => {
                    let _ = tx.send(Command::SetBpm(b));
                    ExecResult::msg(format!("BPM → {b} (次の小節から)"))
                }
                _ => ExecResult::msg(format!("bad bpm: {}", args[1])),
            };
        }
        "status" => return status(deck_paths, engine),
        "mix" => return exec_mix(&args, tx, engine),
        "x" | "xfade" => {
            let bars = args
                .get(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(4u32)
                .max(1);
            let to = other_deck(engine);
            let _ = tx.send(Command::XFade { to_deck: to, bars });
            let label = if to == 0 { "A" } else { "B" };
            return ExecResult::msg(format!("xfade → {label} ({bars} bars, 次の小節から)"));
        }
        _ => {}
    }

    // Deck-prefixed: `a load path`, `b mute kick`, `a x 4`, …
    if let Some(deck) = parse_deck(args[0]) {
        if args.len() < 2 {
            return ExecResult::msg("usage: a|b <load|mute|unmute|gain|head|x|save|reload> …");
        }
        let verb = args[1];
        match verb {
            "load" if args.len() >= 3 => {
                // Allow paths with spaces: join rest.
                let path = PathBuf::from(args[2..].join(" "));
                return load_song(deck, path, tx, deck_paths);
            }
            "reload" => return reload_song(deck, tx, deck_paths),
            "save" => {
                let name = if args.len() >= 3 {
                    Some(args[2..].join(" "))
                } else {
                    None
                };
                return save_deck_song(deck, name, deck_paths, engine);
            }
            "mute" | "unmute" if args.len() >= 3 => {
                let muted = verb == "mute";
                let track = args[2..].join(" ");
                let _ = tx.send(Command::SetTrackMute {
                    deck,
                    track: track.clone(),
                    muted,
                });
                return ExecResult::msg(format!(
                    "{} {track} deck {} (次の小節から)",
                    if muted { "mute" } else { "unmute" },
                    if deck == 0 { "A" } else { "B" }
                ));
            }
            "gain" if args.len() == 3 => {
                return match args[2].parse::<f32>() {
                    Ok(g) => {
                        let _ = tx.send(Command::SetDeckGain { deck, gain: g });
                        ExecResult::msg(format!("gain {} → {g}", if deck == 0 { "A" } else { "B" }))
                    }
                    Err(_) => ExecResult::msg(format!("bad gain: {}", args[2])),
                };
            }
            "head" | "cue" if args.len() == 3 => {
                return match args[2].parse::<u64>() {
                    Ok(bar) if bar >= 1 => {
                        let _ = tx.send(Command::Head { deck, bar });
                        ExecResult::msg(format!(
                            "head {} → bar {bar} (次の小節から同期)",
                            if deck == 0 { "A" } else { "B" }
                        ))
                    }
                    Ok(_) => ExecResult::msg("head bar must be >= 1 (1 = first bar)"),
                    Err(_) => ExecResult::msg(format!("bad head bar: {}", args[2])),
                };
            }
            "x" | "xfade" => {
                let bars = args
                    .get(2)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(4u32)
                    .max(1);
                let _ = tx.send(Command::XFade {
                    to_deck: deck,
                    bars,
                });
                return ExecResult::msg(format!(
                    "xfade → {} ({bars} bars, 次の小節から)",
                    if deck == 0 { "A" } else { "B" }
                ));
            }
            other => {
                return ExecResult::msg(format!(
                    "unknown verb '{other}' (try: load mute unmute gain head x save reload)"
                ));
            }
        }
    }

    ExecResult::msg(format!("unknown: {} (try help)", args[0]))
}

fn load_song(
    deck: usize,
    path: PathBuf,
    tx: &Sender<Command>,
    deck_paths: &DeckPaths,
) -> ExecResult {
    let path = match resolve_song_path(&path.to_string_lossy()) {
        Ok(p) => p,
        Err(e) => return ExecResult::msg(e),
    };
    match std::fs::read_to_string(&path) {
        Ok(text) => match parse_song(&text, &path.to_string_lossy()) {
            Ok(song) => {
                if let Ok(mut dp) = deck_paths.lock() {
                    dp[deck] = Some(path.clone());
                }
                let title = song.title.clone();
                let _ = tx.send(Command::LoadSong {
                    deck,
                    song: Box::new(song.clone()),
                });
                ExecResult {
                    quit: false,
                    messages: vec![format!(
                        "loaded {title} ({}) → deck {} (次の小節から)",
                        path.display(),
                        if deck == 0 { "A" } else { "B" }
                    )],
                    loaded: Some((deck, song)),
                }
            }
            Err(e) => ExecResult::msg(format!("parse error: {e}")),
        },
        Err(e) => ExecResult::msg(format!("read error {}: {e}", path.display())),
    }
}

fn deck_label(deck: usize) -> &'static str {
    if deck == 0 {
        "A"
    } else {
        "B"
    }
}

fn reload_song(deck: usize, tx: &Sender<Command>, deck_paths: &DeckPaths) -> ExecResult {
    let path = match deck_paths.lock() {
        Ok(dp) => dp[deck].clone(),
        Err(_) => None,
    };
    let Some(path) = path else {
        return ExecResult::msg(format!(
            "no file associated with deck {} (load a song first)",
            deck_label(deck)
        ));
    };
    load_song(deck, path, tx, deck_paths)
}

fn save_name_hint(deck: usize, deck_paths: &DeckPaths, song: &Song) -> Option<String> {
    if let Ok(dp) = deck_paths.lock() {
        if let Some(p) = &dp[deck] {
            if let Some(n) = p
                .file_stem()
                .and_then(|s| s.to_str())
                .filter(|n| !n.is_empty())
            {
                return Some(n.to_string());
            }
        }
    }
    PathBuf::from(&song.path)
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|n| !n.is_empty())
        .map(|s| s.to_string())
}

fn save_deck_song(
    deck: usize,
    name: Option<String>,
    deck_paths: &DeckPaths,
    engine: Option<&Arc<Mutex<Engine>>>,
) -> ExecResult {
    let label = deck_label(deck);
    let Some(engine) = engine else {
        return ExecResult::msg(format!("deck {label} has no song loaded"));
    };
    let mut e = match engine.lock() {
        Ok(g) => g,
        Err(err) => return ExecResult::msg(format!("engine lock: {err}")),
    };
    let Some(mut song) = e.decks[deck].song_ref().cloned() else {
        return ExecResult::msg(format!("deck {label} has no song loaded"));
    };
    let name = match name.filter(|n| !n.is_empty()) {
        Some(n) => n,
        None => match save_name_hint(deck, deck_paths, &song) {
            Some(n) => n,
            None => return ExecResult::msg("usage: a|b save <name>"),
        },
    };
    if song.source.len() > MAX_SONG_CONTENT_BYTES {
        return ExecResult::msg(format!(
            "content too large ({} bytes, max {MAX_SONG_CONTENT_BYTES})",
            song.source.len()
        ));
    }
    let dest = match resolve_user_song_save_path(&name) {
        Ok(p) => p,
        Err(e) => return ExecResult::msg(e),
    };
    let dest_str = dest.to_string_lossy().into_owned();
    if let Err(e) = parse_song(&song.source, &dest_str) {
        return ExecResult::msg(format!("parse error: {e}"));
    }
    if let Err(e) = ensure_user_songs_dir() {
        return ExecResult::msg(e);
    }
    if let Err(e) = std::fs::write(&dest, song.source.as_bytes()) {
        return ExecResult::msg(format!("write: {e}"));
    }
    if let Ok(mut dp) = deck_paths.lock() {
        dp[deck] = Some(dest.clone());
    }
    song.path = dest_str;
    e.decks[deck].set_song_data(song);
    ExecResult::msg(format!("saved {} → {}", label, dest.display()))
}

fn exec_mix(
    args: &[&str],
    tx: &Sender<Command>,
    engine: Option<&Arc<Mutex<Engine>>>,
) -> ExecResult {
    use crate::mixer::{FillKind, MixAction, MixCommand, MixGrid};
    if args.len() < 2 {
        return ExecResult::msg("usage: mix long|cut|fill|hold …");
    }
    let mv = args[1];
    if mv.eq_ignore_ascii_case("hold") {
        let _ = tx.send(Command::HoldXFade);
        return ExecResult::msg("xfade hold (now)");
    }
    let action = match MixAction::parse(mv) {
        Ok(a) => a,
        Err(e) => return ExecResult::msg(e),
    };
    if action == MixAction::Fill {
        if args.len() < 4 {
            return ExecResult::msg("usage: mix fill delay|lpf|flash|riser|switch A|B [8n|4n]");
        }
        let kind = match FillKind::parse(args[2]) {
            Ok(k) => k,
            Err(e) => return ExecResult::msg(e),
        };
        let to = match parse_deck(args[3]) {
            Some(d) => d,
            None => return ExecResult::msg(format!("deck must be A or B: {}", args[3])),
        };
        let grid = match args.get(4) {
            None => MixGrid::Eighth,
            Some(s) => match MixGrid::parse(s) {
                Ok(g) => g,
                Err(e) => return ExecResult::msg(e),
            },
        };
        if let Some(msg) = mix_deck_guard(engine, action, Some(kind), to) {
            return ExecResult::msg(msg);
        }
        let _ = tx.send(Command::Mix(MixCommand {
            action,
            to_deck: to,
            bars: kind.default_bars(),
            eq: true,
            reset_eq: true,
            fill: Some(kind),
            grid,
            mute_track: None,
            phrase: 1,
        }));
        return ExecResult::msg(format!(
            "mix fill {} → {} (次の小節から)",
            kind.as_str(),
            if to == 0 { "A" } else { "B" }
        ));
    }
    if args.len() < 3 {
        return ExecResult::msg("usage: mix long|cut A|B [bars]");
    }
    let to = match parse_deck(args[2]) {
        Some(d) => d,
        None => return ExecResult::msg(format!("deck must be A or B: {}", args[2])),
    };
    let bars = args
        .get(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| action.default_bars(None))
        .clamp(1, 32);
    if let Some(msg) = mix_deck_guard(engine, action, None, to) {
        return ExecResult::msg(msg);
    }
    let _ = tx.send(Command::Mix(MixCommand {
        action,
        to_deck: to,
        bars,
        eq: true,
        reset_eq: true,
        fill: None,
        grid: MixGrid::Eighth,
        mute_track: None,
        phrase: 1,
    }));
    let label = if to == 0 { "A" } else { "B" };
    match action {
        MixAction::Long => {
            ExecResult::msg(format!("mix long → {label} ({bars} bars, 次の小節から)"))
        }
        MixAction::Cut => ExecResult::msg(format!("mix cut → {label} (次の小節から)")),
        MixAction::Fill => ExecResult::msg("fill needs kind"),
    }
}

fn mix_deck_guard(
    engine: Option<&Arc<Mutex<Engine>>>,
    action: crate::mixer::MixAction,
    fill: Option<crate::mixer::FillKind>,
    to: usize,
) -> Option<String> {
    let eng = engine?;
    let Ok(e) = eng.try_lock() else {
        return None;
    };
    let a = e.decks[0].song_title().is_some();
    let b = e.decks[1].song_title().is_some();
    let both =
        action == crate::mixer::MixAction::Long || fill == Some(crate::mixer::FillKind::Switch);
    if both && !(a && b) {
        return Some("both decks need a song loaded for long mix / switch".into());
    }
    let loaded = if to == 1 { b } else { a };
    if !loaded {
        return Some("target deck has no song loaded".into());
    }
    None
}

fn status(deck_paths: &DeckPaths, engine: Option<&Arc<Mutex<Engine>>>) -> ExecResult {
    let mut messages = Vec::new();
    if let Ok(dp) = deck_paths.lock() {
        messages.push(format!(
            "A={:?} B={:?}",
            dp[0].as_ref().map(|p| p.display().to_string()),
            dp[1].as_ref().map(|p| p.display().to_string())
        ));
    }
    if let Some(eng) = engine {
        if let Ok(e) = eng.try_lock() {
            let gbar = e.transport.bar_index();
            messages.push(format!(
                "bpm={:.1} transport_bar={} gainA={:.2} gainB={:.2} songs={:?}/{:?}",
                e.transport.bpm,
                gbar,
                e.mixer.gain_a,
                e.mixer.gain_b,
                e.decks[0].song_title(),
                e.decks[1].song_title()
            ));
            messages.push(format!(
                "song_bar A={} B={} (1-based; head/cue offset)",
                e.decks[0].song_bar_1based(gbar),
                e.decks[1].song_bar_1based(gbar)
            ));
        } else {
            messages.push("(engine busy)".into());
        }
    }
    ExecResult::msgs(messages)
}

/// Pick the quieter deck (or B if equal) as xfade target when `x` has no deck prefix.
fn other_deck(engine: Option<&Arc<Mutex<Engine>>>) -> usize {
    let Some(eng) = engine else {
        return 1;
    };
    let Ok(e) = eng.try_lock() else {
        return 1;
    };
    if e.mixer.gain_a > e.mixer.gain_b + 0.05 {
        return 1; // A is up → go to B
    }
    if e.mixer.gain_b > e.mixer.gain_a + 0.05 {
        return 0; // B is up → go to A
    }
    // Equal-ish: prefer a loaded deck that isn't the only empty one.
    match (
        e.decks[0].song_title().is_some(),
        e.decks[1].song_title().is_some(),
    ) {
        (false, true) => 1,
        (true, false) => 0,
        _ => 1,
    }
}

fn parse_deck(s: &str) -> Option<usize> {
    match s {
        "A" | "a" | "0" => Some(0),
        "B" | "b" | "1" => Some(1),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam::channel::unbounded;

    #[test]
    fn classify_slash_vs_hermes() {
        assert_eq!(classify_live_input("", true), LiveInput::Empty);
        assert_eq!(
            classify_live_input("/bpm 128", true),
            LiveInput::LocalCommand("bpm 128".into())
        );
        assert_eq!(
            classify_live_input(":/x 4", true),
            LiveInput::LocalCommand("x 4".into())
        );
        assert_eq!(
            classify_live_input("暗くして", true),
            LiveInput::HermesPrompt("暗くして".into())
        );
        assert_eq!(
            classify_live_input("bpm 128", true),
            LiveInput::HermesPrompt("bpm 128".into())
        );
        // Hermes off: bare is local
        assert_eq!(
            classify_live_input("bpm 128", false),
            LiveInput::LocalCommand("bpm 128".into())
        );
        assert_eq!(
            classify_live_input("/status", false),
            LiveInput::LocalCommand("status".into())
        );
    }

    #[test]
    fn parses_a_load_and_x() {
        let (tx, rx) = unbounded();
        let paths = new_deck_paths();
        // load missing file → message, no panic
        let r = exec("a load no_such.strudel", &tx, &paths, None);
        assert!(!r.quit);
        let m = &r.messages[0];
        assert!(
            m.contains("song not found") || m.contains("read error") || m.contains("parse"),
            "{m}"
        );

        let r = exec("x 8", &tx, &paths, None);
        assert!(!r.quit);
        match rx.try_recv().unwrap() {
            Command::XFade { to_deck, bars } => {
                assert_eq!(to_deck, 1);
                assert_eq!(bars, 8);
            }
            _ => panic!("expected XFade"),
        }
        assert!(r.messages[0].contains("xfade"));
    }

    #[test]
    fn b_x_targets_b() {
        let (tx, rx) = unbounded();
        let paths = new_deck_paths();
        let _ = exec("b x 2", &tx, &paths, None);
        match rx.try_recv().unwrap() {
            Command::XFade { to_deck, bars } => {
                assert_eq!(to_deck, 1);
                assert_eq!(bars, 2);
            }
            _ => panic!("expected XFade"),
        }
    }

    #[test]
    fn mix_hold_and_fill_switch() {
        let (tx, rx) = unbounded();
        let paths = new_deck_paths();
        let r = exec("mix hold", &tx, &paths, None);
        assert!(r.messages[0].contains("hold"));
        assert!(matches!(rx.try_recv().unwrap(), Command::HoldXFade));
        let r = exec("mix fill switch A 8n", &tx, &paths, None);
        // no engine → no deck guard
        assert!(r.messages[0].contains("switch"));
        match rx.try_recv().unwrap() {
            Command::Mix(m) => {
                assert_eq!(m.to_deck, 0);
                assert_eq!(m.fill, Some(crate::mixer::FillKind::Switch));
            }
            _ => panic!("expected Mix"),
        }
    }

    #[test]
    fn b_head_and_cue_alias() {
        let (tx, rx) = unbounded();
        let paths = new_deck_paths();
        let r = exec("b head 33", &tx, &paths, None);
        assert!(r.messages[0].contains("33"));
        match rx.try_recv().unwrap() {
            Command::Head { deck, bar } => {
                assert_eq!(deck, 1);
                assert_eq!(bar, 33);
            }
            _ => panic!("expected Head"),
        }
        let r = exec("a cue 1", &tx, &paths, None);
        assert!(r.messages[0].contains("bar 1"));
        match rx.try_recv().unwrap() {
            Command::Head { deck, bar } => {
                assert_eq!(deck, 0);
                assert_eq!(bar, 1);
            }
            _ => panic!("expected Head"),
        }
        let r = exec("b head 0", &tx, &paths, None);
        assert!(r.messages[0].contains(">= 1"));
        assert!(rx.try_recv().is_err());
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn save_without_song_and_reload_without_path() {
        let (tx, rx) = unbounded();
        let paths = new_deck_paths();
        let r = exec("a save", &tx, &paths, None);
        assert!(
            r.messages[0].contains("no song loaded"),
            "{}",
            r.messages[0]
        );
        assert!(rx.try_recv().is_err());
        let r = exec("a reload", &tx, &paths, None);
        assert!(
            r.messages[0].contains("no file associated"),
            "{}",
            r.messages[0]
        );
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn save_writes_user_library_without_load() {
        let _home_guard = crate::song::lock_test_home();
        let home = std::env::temp_dir().join(format!("strudel_cmd_save_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(&home).unwrap();
        std::env::set_var("HOME", &home);

        let (tx, rx) = unbounded();
        let paths = new_deck_paths();
        let mut engine = Engine::new(44100, 120.0);
        let song = parse_song("// @title t\nsetcpm(30)\n$: s(\"bd*4\")\n", "").unwrap();
        engine.decks[0].set_song_data(song);
        let engine = Arc::new(Mutex::new(engine));
        let r = exec("a save visitor-mem", &tx, &paths, Some(&engine));
        assert!(r.messages[0].contains("saved"), "{}", r.messages[0]);
        assert!(rx.try_recv().is_err(), "TUI save must not LoadSong");
        let expected = home
            .join(".config")
            .join("strudel-rs")
            .join("songs")
            .join("visitor-mem.strudel");
        let written = std::fs::read_to_string(&expected).unwrap();
        assert!(written.contains("@title t"), "{written}");

        let _ = std::fs::remove_dir_all(&home);
    }
}
