//! Shared live/REPL command parser (colon-free, deck-first).

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crossbeam::channel::Sender;

use crate::engine::{Command, Engine};
use crate::song::{parse_song, Song};
use crate::watcher::DeckPaths;

pub const HELP: &str = "\
a|b load <file>     load song on deck (next bar)
a|b mute <track>    mute track (next bar)
a|b unmute <track>
a|b gain <0..1>     fader (immediate)
x [bars]            xfade to the other deck (default 4)
a x [bars]          xfade to deck A
b x [bars]          xfade to deck B
bpm <n>             BPM from next bar
hush                stop all (immediate)
status
help
quit / q
";

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
            return ExecResult::msg("usage: a|b <load|mute|unmute|gain|x> …");
        }
        let verb = args[1];
        match verb {
            "load" if args.len() >= 3 => {
                // Allow paths with spaces: join rest.
                let path = PathBuf::from(args[2..].join(" "));
                return load_song(deck, path, tx, deck_paths);
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
                    "unknown verb '{other}' (try: load mute unmute gain x)"
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
                        "loaded {title} → deck {} (次の小節から)",
                        if deck == 0 { "A" } else { "B" }
                    )],
                    loaded: Some((deck, song)),
                }
            }
            Err(e) => ExecResult::msg(format!("parse error: {e}")),
        },
        Err(e) => ExecResult::msg(format!("read error: {e}")),
    }
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
            messages.push(format!(
                "bpm={:.1} bar={} gainA={:.2} gainB={:.2} songs={:?}/{:?}",
                e.transport.bpm,
                e.transport.bar_index(),
                e.mixer.gain_a,
                e.mixer.gain_b,
                e.decks[0].song_title(),
                e.decks[1].song_title()
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
    fn parses_a_load_and_x() {
        let (tx, rx) = unbounded();
        let paths = crate::watcher::new_deck_paths();
        // load missing file → message, no panic
        let r = exec("a load no_such.strudel", &tx, &paths, None);
        assert!(!r.quit);
        assert!(r.messages[0].contains("read error") || r.messages[0].contains("parse"));

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
        let paths = crate::watcher::new_deck_paths();
        let _ = exec("b x 2", &tx, &paths, None);
        match rx.try_recv().unwrap() {
            Command::XFade { to_deck, bars } => {
                assert_eq!(to_deck, 1);
                assert_eq!(bars, 2);
            }
            _ => panic!("expected XFade"),
        }
    }
}
