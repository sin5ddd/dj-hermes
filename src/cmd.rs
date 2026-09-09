//! Shared live/REPL command parser (colon-free, deck-first).
//!
//! Live TUI (with Hermes): prefix local commands with `/` (e.g. `/x 4`).
//! Bare natural language goes to Hermes. Text REPL (`--text`) still uses bare commands.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crossbeam::channel::Sender;

use crate::engine::{Command, Engine};
use crate::session::SessionKind;
use crate::song::{
    ensure_user_songs_dir, is_genre_slug, list_bundled_genres, list_bundled_slots,
    looks_like_slot_index, normalize_slot_index, parse_song, resolve_song_path,
    resolve_user_song_save_path, Song, MAX_SONG_CONTENT_BYTES,
};
use crate::transport::{RepeatDiv, TapeSpec};

/// Which path is loaded on deck A / B (for reload targeting).
pub type DeckPaths = Arc<Mutex<[Option<PathBuf>; 2]>>;

pub fn new_deck_paths() -> DeckPaths {
    Arc::new(Mutex::new([None, None]))
}

pub const HELP: &str = "\
# local commands (live TUI: prefix with / )
a|b <genre> <n>     load bundled slot (e.g. /a house 01 → songs/house/01.strudel)
a|b load <file>     load song (bare name, house/01, or house-01)
a|b reload          re-read last loaded file onto this deck (next bar)
a|b save [name]     write current deck source to user library (no playback change)
a|b mute <track>    mute track (next bar)
a|b unmute <track>
a|b gain <0..1>     fader (immediate)
a|b head <bar>      cue song bar (1-based; applies next bar). alias: cue
list [genre]        bundled genres, or slot numbers in one genre
x [bars]            xfade to the other deck (default 4)
a x [bars]          xfade to deck A
b x [bars]          xfade to deck B
mix long A|B [bars] long mix (EQ bass-swap + xfade, next phrase)
mix cut A|B         cut-in next bar (EQ reset)
mix fill <kind> A|B [8n|4n]  delay|lpf|flash|riser|switch|echo|hpf|roll|drop|vinyl then cut-in
mix hold            freeze xfade now
filter lpf <hz>|off post-mix LPF (held)
filter hpf <hz>|off post-mix HPF (held)
delay <0..1>        post-mix delay wet (held)
vinyl on|off        post-mix vinyl (held; worn BPF + pitch wow)
repeat 4n|8n|16n|32n|off  time-repeat (quarter/8th/16th/32nd notes, 1 bar)
tape [1n|2n|4n|8n] [reps]  tape-stop (immediate). default 1n. 4n 2 = two quarters. off cancels
bpm <n>             BPM from next bar
hush                stop all (immediate)  [operator]
status
help
quit / q            [operator]
viz [on|off]        toggle body punchcard / highlight (live TUI)
vfx [on|off]        toggle hit VFX overlay (live TUI; default on). aliases: dopa, flash
automix [on|off]    idle 5min → Hermes cron mix; off disables
↑↓ / Tab / Enter    suggest overlay: select / apply / apply+run (live TUI)
Esc                 dismiss suggest (or snapshot A/B + quit when prompt empty; --resume)

# live TUI + Hermes
bare text           send to Hermes (DJ assistant)
/…                  local command (e.g. /bpm 128, /a house 01, /viz)
";

/// Help for `dj-hermes play` (one song on deck A; no mix / xfade / B).
pub const HELP_PLAY: &str = "\
# local commands (live TUI: prefix with / )
<genre> <n>         load bundled slot (e.g. /house 01 → songs/house/01.strudel)
load <file>         load song (bare name, house/01, or house-01; alias of a load)
reload              re-read last loaded file (next bar)
save [name]         write current source to user library (no playback change)
mute <track>        mute track (next bar)
unmute <track>
gain <0..1>         fader (immediate)
head <bar>          cue song bar (1-based; applies next bar). alias: cue
list [genre]        bundled genres, or slot numbers in one genre
a load|save|…       same verbs with an explicit deck A prefix
filter lpf <hz>|off post-mix LPF (held)
filter hpf <hz>|off post-mix HPF (held)
delay <0..1>        post-mix delay wet (held)
vinyl on|off        post-mix vinyl (held; worn BPF + pitch wow)
repeat 4n|8n|16n|32n|off  time-repeat (quarter/8th/16th/32nd notes, 1 bar)
tape [1n|2n|4n|8n] [reps]  tape-stop (immediate). default 1n. 4n 2 = two quarters. off cancels
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
bare text           send to Hermes (play assistant)
/…                  local command (e.g. /bpm 128, /house 01, /load house/01, /viz)

play is one song (deck A). mix / xfade / deck B: use `dj-hermes dj`.
";

/// Help body for the current session (overlay / `help` command).
pub fn help_text(session: SessionKind) -> &'static str {
    match session {
        SessionKind::Play => HELP_PLAY,
        SessionKind::Dj => HELP,
    }
}

const PLAY_ALIAS_VERBS: &[&str] = &[
    "load", "save", "reload", "mute", "unmute", "gain", "head", "cue",
];

fn preprocess_play_line(line: &str) -> Result<String, String> {
    let args: Vec<&str> = line.split_whitespace().collect();
    if args.is_empty() {
        return Ok(line.to_string());
    }
    let head = args[0];
    if parse_deck(head) == Some(1) {
        return Err(
            "play は1曲（デッキ A）です。デッキ B は `dj-hermes dj` を使ってください。".into(),
        );
    }
    if matches!(head, "mix" | "x" | "xfade") {
        return Err("mix / xfade は dj 専用です。play は1曲です。".into());
    }
    if parse_deck(head) == Some(0) && args.get(1).is_some_and(|v| matches!(*v, "x" | "xfade")) {
        return Err("mix / xfade は dj 専用です。play は1曲です。".into());
    }
    if PLAY_ALIAS_VERBS.contains(&head) {
        return Ok(format!("a {line}"));
    }
    if args.len() >= 2 && is_genre_slug(head) && looks_like_slot_index(args[1]) {
        return Ok(format!("a {line}"));
    }
    Ok(line.to_string())
}

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
/// Dual-deck (DJ) session — same as [`exec_in`] with [`SessionKind::Dj`].
pub fn exec(
    line: &str,
    tx: &Sender<Command>,
    deck_paths: &DeckPaths,
    engine: Option<&Arc<Mutex<Engine>>>,
) -> ExecResult {
    exec_in(line, tx, deck_paths, engine, SessionKind::Dj)
}

/// Parse and run one command line in a DJ or play session.
pub fn exec_in(
    line: &str,
    tx: &Sender<Command>,
    deck_paths: &DeckPaths,
    engine: Option<&Arc<Mutex<Engine>>>,
    session: SessionKind,
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
    let rewritten;
    let line = if session == SessionKind::Play {
        match preprocess_play_line(line) {
            Ok(s) => {
                rewritten = s;
                rewritten.as_str()
            }
            Err(m) => return ExecResult::msg(m),
        }
    } else {
        line
    };
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
        "help" | "h" | "?" => return ExecResult::msg(help_text(session).trim_end()),
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
        "list" => return list_songs_cmd(&args[1..]),
        "mix" => return exec_mix(&args, tx, engine),
        "filter" => return exec_filter(&args, tx),
        "delay" => return exec_delay(&args, tx),
        "vinyl" => return exec_vinyl(&args, tx),
        "repeat" => return exec_repeat(&args, tx),
        "tape" => return exec_tape(&args, tx),
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
            return ExecResult::msg(
                "usage: a|b <genre> <n> | a|b <load|mute|unmute|gain|head|x|save|reload> …",
            );
        }
        let verb = args[1];
        match verb {
            "load" if args.len() >= 3 => {
                let path = load_path_from_args(&args[2..]);
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
                if is_genre_slug(other) {
                    if args.len() == 2 {
                        return list_songs_cmd(&[other]);
                    }
                    if args.len() >= 3 {
                        if let Some(slot) = slot_ref(other, args[2]) {
                            return load_song(deck, PathBuf::from(slot), tx, deck_paths);
                        }
                        return ExecResult::msg(format!(
                            "usage: a|b {other} <n>   (例: a {other} 01)"
                        ));
                    }
                }
                return ExecResult::msg(format!(
                    "unknown verb '{other}' (try: house 01, load, mute, unmute, gain, head, x, save, reload)"
                ));
            }
        }
    }

    ExecResult::msg(format!("unknown: {} (try help)", args[0]))
}

fn slot_ref(genre: &str, idx: &str) -> Option<String> {
    if !is_genre_slug(genre) {
        return None;
    }
    let nn = normalize_slot_index(idx)?;
    Some(format!("{genre}/{nn}"))
}

fn load_path_from_args(rest: &[&str]) -> PathBuf {
    if rest.len() >= 2 {
        if let Some(slot) = slot_ref(rest[0], rest[1]) {
            return PathBuf::from(slot);
        }
    }
    PathBuf::from(rest.join(" "))
}

fn list_songs_cmd(args: &[&str]) -> ExecResult {
    if args.is_empty() {
        let genres = list_bundled_genres();
        if genres.is_empty() {
            return ExecResult::msg(
                "no bundled genres (cwd songs/<genre>/<nn>.strudel). try /a load <file>",
            );
        }
        let mut lines: Vec<String> = vec!["genres:".into()];
        for g in genres {
            lines.push(format!("  {} ({})", g.name, g.count));
        }
        lines.push("load: /a <genre> <n>   (例: /a house 01)".into());
        return ExecResult::msgs(lines);
    }
    let genre = args[0];
    if !is_genre_slug(genre) {
        return ExecResult::msg(format!("bad genre name: {genre}"));
    }
    let slots = list_bundled_slots(genre);
    if slots.is_empty() {
        return ExecResult::msg(format!("no slots in songs/{genre}/"));
    }
    ExecResult::msg(format!(
        "{genre}: {}   load: /a {genre} {}",
        slots.join(" "),
        slots.first().map(|s| s.as_str()).unwrap_or("01")
    ))
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

fn exec_filter(args: &[&str], tx: &Sender<Command>) -> ExecResult {
    if args.len() < 3 {
        return ExecResult::msg("usage: filter lpf|hpf <hz>|off");
    }
    let which = args[1];
    let val = args[2];
    let hz = if val.eq_ignore_ascii_case("off") || val.eq_ignore_ascii_case("none") || val == "0" {
        None
    } else {
        match val.parse::<f32>() {
            Ok(n) if n.is_finite() && n > 0.0 => Some(n),
            _ => return ExecResult::msg(format!("filter Hz must be positive or off: {val}")),
        }
    };
    match which {
        "lpf" | "lp" => {
            let _ = tx.send(Command::SetMixerLpf(hz));
            ExecResult::msg(match hz {
                Some(n) => format!("filter lpf {n} Hz"),
                None => "filter lpf off".into(),
            })
        }
        "hpf" | "hp" => {
            let _ = tx.send(Command::SetMixerHpf(hz));
            ExecResult::msg(match hz {
                Some(n) => format!("filter hpf {n} Hz"),
                None => "filter hpf off".into(),
            })
        }
        other => ExecResult::msg(format!("usage: filter lpf|hpf <hz>|off (got {other})")),
    }
}

fn exec_repeat(args: &[&str], tx: &Sender<Command>) -> ExecResult {
    if args.len() < 2 {
        return ExecResult::msg("usage: repeat 4n|8n|16n|32n|off");
    }
    let token = args[1];
    if matches!(token, "off" | "0" | "none") {
        let _ = tx.send(Command::SetTimeRepeat(None));
        return ExecResult::msg("repeat off");
    }
    match RepeatDiv::parse(token) {
        Ok(d) => {
            let _ = tx.send(Command::SetTimeRepeat(Some(d)));
            ExecResult::msg(format!("repeat {}", d.as_str()))
        }
        Err(e) => ExecResult::msg(e),
    }
}

fn exec_tape(args: &[&str], tx: &Sender<Command>) -> ExecResult {
    let rest = &args[1..];
    if rest
        .first()
        .is_some_and(|t| matches!(*t, "off" | "0" | "none" | "false"))
    {
        let _ = tx.send(Command::SetTapeStop(None));
        return ExecResult::msg("tape off");
    }
    let spec = if rest.is_empty() || matches!(rest[0], "on" | "true") {
        let extra = if rest.len() > 1 { Some(rest[1]) } else { None };
        if let Some(tok) = extra {
            match TapeSpec::parse(tok, rest.get(2).copied()) {
                Ok(s) => s,
                Err(e) => return ExecResult::msg(e),
            }
        } else {
            TapeSpec::ONE_BAR
        }
    } else {
        match TapeSpec::parse(rest[0], rest.get(1).copied()) {
            Ok(s) => s,
            Err(e) => return ExecResult::msg(e),
        }
    };
    let _ = tx.send(Command::SetTapeStop(Some(spec)));
    ExecResult::msg(format!("tape {}", spec.as_status()))
}

fn exec_vinyl(args: &[&str], tx: &Sender<Command>) -> ExecResult {
    if args.len() < 2 {
        return ExecResult::msg("usage: vinyl on|off");
    }
    let on = match args[1].trim().to_ascii_lowercase().as_str() {
        "on" | "true" | "1" => true,
        "off" | "false" | "0" | "none" => false,
        other => return ExecResult::msg(format!("usage: vinyl on|off (got {other})")),
    };
    let _ = tx.send(Command::SetMixerVinyl(on));
    ExecResult::msg(if on { "vinyl on" } else { "vinyl off" })
}

fn exec_delay(args: &[&str], tx: &Sender<Command>) -> ExecResult {
    if args.len() < 2 {
        return ExecResult::msg("usage: delay <0..1>");
    }
    match args[1].parse::<f32>() {
        Ok(w) if w.is_finite() => {
            let wet = w.clamp(0.0, 1.0);
            let _ = tx.send(Command::SetMixerDelay {
                wet,
                feedback: None,
            });
            ExecResult::msg(format!("delay wet {wet}"))
        }
        _ => ExecResult::msg(format!("delay wet must be 0..1: {}", args[1])),
    }
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
            return ExecResult::msg(
                "usage: mix fill delay|lpf|flash|riser|switch|echo|hpf|roll|drop|vinyl A|B [8n|4n]",
            );
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
            let repeat = e
                .transport
                .repeat_div()
                .map(|d| d.as_str())
                .unwrap_or("off");
            let tape = e
                .transport
                .tape_spec()
                .map(|s| s.as_status())
                .unwrap_or_else(|| "off".into());
            messages.push(format!(
                "bpm={:.1} transport_bar={} repeat={repeat} tape={tape} gainA={:.2} gainB={:.2} songs={:?}/{:?}",
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
    fn parses_a_house_slot_as_path() {
        let (tx, _rx) = unbounded();
        let paths = new_deck_paths();
        let r = exec("a house 01", &tx, &paths, None);
        assert!(!r.quit);
        let m = &r.messages[0];
        assert!(
            m.contains("house/01") || m.contains("song not found") || m.contains("loaded"),
            "{m}"
        );
    }

    #[test]
    fn play_house_slot_rewrites_to_deck_a() {
        let (tx, _rx) = unbounded();
        let paths = new_deck_paths();
        let r = exec_in("house 01", &tx, &paths, None, SessionKind::Play);
        assert!(!r.quit);
        let m = &r.messages[0];
        assert!(
            m.contains("house/01") || m.contains("song not found") || m.contains("loaded"),
            "{m}"
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
    fn parses_filter_and_delay() {
        let (tx, rx) = unbounded();
        let paths = new_deck_paths();
        let r = exec("filter lpf 800", &tx, &paths, None);
        assert!(r.messages[0].contains("lpf"));
        match rx.try_recv().unwrap() {
            Command::SetMixerLpf(Some(hz)) => assert!((hz - 800.0).abs() < 1e-3),
            _ => panic!("expected SetMixerLpf"),
        }
        let r = exec("filter hpf off", &tx, &paths, None);
        assert!(r.messages[0].contains("off"));
        match rx.try_recv().unwrap() {
            Command::SetMixerHpf(None) => {}
            _ => panic!("expected SetMixerHpf(None)"),
        }
        let r = exec("delay 0.4", &tx, &paths, None);
        assert!(r.messages[0].contains("0.4"));
        match rx.try_recv().unwrap() {
            Command::SetMixerDelay { wet, feedback } => {
                assert!((wet - 0.4).abs() < 1e-5);
                assert!(feedback.is_none());
            }
            _ => panic!("expected SetMixerDelay"),
        }
        let r = exec("repeat 16", &tx, &paths, None);
        assert!(r.messages[0].contains("16n"), "{:?}", r.messages);
        match rx.try_recv().unwrap() {
            Command::SetTimeRepeat(Some(d)) => assert_eq!(d.as_str(), "16n"),
            _ => panic!("expected SetTimeRepeat 16n"),
        }
        let r = exec("repeat off", &tx, &paths, None);
        assert!(r.messages[0].contains("off"));
        match rx.try_recv().unwrap() {
            Command::SetTimeRepeat(None) => {}
            _ => panic!("expected SetTimeRepeat None"),
        }
        let r = exec("tape", &tx, &paths, None);
        assert!(r.messages[0].contains("1n"), "{:?}", r.messages);
        match rx.try_recv().unwrap() {
            Command::SetTapeStop(Some(s)) => assert_eq!(s, TapeSpec::ONE_BAR),
            _ => panic!("expected SetTapeStop 1n"),
        }
        let r = exec("tape 4n 2", &tx, &paths, None);
        assert!(r.messages[0].contains("4n*2"), "{:?}", r.messages);
        match rx.try_recv().unwrap() {
            Command::SetTapeStop(Some(s)) => {
                assert_eq!(s.as_status(), "4n*2");
            }
            _ => panic!("expected SetTapeStop 4n*2"),
        }
        let r = exec("tape off", &tx, &paths, None);
        assert!(r.messages[0].contains("off"));
        match rx.try_recv().unwrap() {
            Command::SetTapeStop(None) => {}
            _ => panic!("expected SetTapeStop None"),
        }
        let r = exec("vinyl on", &tx, &paths, None);
        assert!(r.messages[0].contains("on"));
        match rx.try_recv().unwrap() {
            Command::SetMixerVinyl(true) => {}
            _ => panic!("expected SetMixerVinyl true"),
        }
        let r = exec("vinyl off", &tx, &paths, None);
        assert!(r.messages[0].contains("off"));
        match rx.try_recv().unwrap() {
            Command::SetMixerVinyl(false) => {}
            _ => panic!("expected SetMixerVinyl false"),
        }
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
        let home = std::env::temp_dir().join(format!("dj_hermes_cmd_save_{}", std::process::id()));
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
            .join("dj-hermes")
            .join("songs")
            .join("visitor-mem.strudel");
        let written = std::fs::read_to_string(&expected).unwrap();
        assert!(written.contains("@title t"), "{written}");

        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn play_aliases_load_to_deck_a() {
        let (tx, rx) = unbounded();
        let paths = new_deck_paths();
        let r = exec_in("load no_such.strudel", &tx, &paths, None, SessionKind::Play);
        assert!(!r.quit);
        let m = &r.messages[0];
        assert!(
            m.contains("song not found") || m.contains("read error") || m.contains("parse"),
            "{m}"
        );
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn play_rejects_deck_b_and_mix() {
        let (tx, rx) = unbounded();
        let paths = new_deck_paths();
        let r = exec_in("b load house-01", &tx, &paths, None, SessionKind::Play);
        assert!(r.messages[0].contains("dj"), "{}", r.messages[0]);
        assert!(rx.try_recv().is_err());
        let r = exec_in("x 4", &tx, &paths, None, SessionKind::Play);
        assert!(r.messages[0].contains("dj"), "{}", r.messages[0]);
        assert!(rx.try_recv().is_err());
        let r = exec_in("mix hold", &tx, &paths, None, SessionKind::Play);
        assert!(r.messages[0].contains("dj"), "{}", r.messages[0]);
        assert!(rx.try_recv().is_err());
        let r = exec_in("a x 4", &tx, &paths, None, SessionKind::Play);
        assert!(r.messages[0].contains("dj"), "{}", r.messages[0]);
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn play_help_omits_xfade() {
        let (tx, _rx) = unbounded();
        let paths = new_deck_paths();
        let r = exec_in("help", &tx, &paths, None, SessionKind::Play);
        let h = r.messages.join("\n");
        assert!(h.contains("/load"), "{h}");
        assert!(!h.contains("xfade to the other deck"), "{h}");
        assert!(h.contains("dj-hermes dj"), "{h}");
    }
}
