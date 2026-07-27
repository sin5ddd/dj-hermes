//! Watch `.strudel` files and hot-reload loaded decks at the next bar.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crossbeam::channel::Sender;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::engine::Command;
use crate::song::parse_song;

/// Which path is loaded on deck A / B (for reload targeting).
pub type DeckPaths = Arc<Mutex<[Option<PathBuf>; 2]>>;

/// Live-UI change log buffer (avoids `eprintln` under the alternate screen).
/// When set, watcher messages go here; otherwise they print to stderr.
pub type UiLogBuffer = Arc<Mutex<VecDeque<String>>>;

/// Soft cap so a stuck watcher cannot grow memory without bound.
const UI_LOG_CAP: usize = 64;

pub fn new_deck_paths() -> DeckPaths {
    Arc::new(Mutex::new([None, None]))
}

/// Watch `dir` non-recursively. Only paths present in `deck_paths` are reloaded.
///
/// When `ui_log` is `Some`, status lines are queued for the live UI (3-line footer)
/// instead of writing to stderr (which would flood the alternate screen).
pub fn watch_songs(
    dir: &Path,
    tx: Sender<Command>,
    deck_paths: DeckPaths,
    ui_log: Option<UiLogBuffer>,
) -> notify::Result<RecommendedWatcher> {
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        let Ok(ev) = res else {
            return;
        };
        if !matches!(
            ev.kind,
            EventKind::Modify(_) | EventKind::Create(_) | EventKind::Any
        ) {
            return;
        }
        for path in ev.paths {
            if path.extension().and_then(|s| s.to_str()) != Some("strudel") {
                continue;
            }
            let deck = {
                let dp = match deck_paths.lock() {
                    Ok(g) => g,
                    Err(_) => return,
                };
                find_deck(&dp, &path)
            };
            let Some(deck) = deck else {
                continue; // not currently loaded
            };
            let deck_label = if deck == 0 { "A" } else { "B" };
            match std::fs::read_to_string(&path) {
                Ok(text) => match parse_song(&text, &path.to_string_lossy()) {
                    Ok(song) => {
                        push_log(
                            &ui_log,
                            format!("↻ {} → deck {deck_label} (次の小節から反映)", song.title),
                        );
                        let _ = tx.send(Command::LoadSong {
                            deck,
                            song: Box::new(song),
                        });
                    }
                    Err(e) => {
                        push_log(
                            &ui_log,
                            format!("⚠ {}: parse error (現行の曲を継続): {e}", path.display()),
                        );
                    }
                },
                Err(e) => {
                    push_log(&ui_log, format!("⚠ read error {}: {e}", path.display()));
                }
            }
        }
    })?;
    watcher.watch(dir, RecursiveMode::NonRecursive)?;
    Ok(watcher)
}

fn push_log(ui_log: &Option<UiLogBuffer>, msg: String) {
    if let Some(buf) = ui_log {
        if let Ok(mut q) = buf.lock() {
            q.push_back(msg);
            while q.len() > UI_LOG_CAP {
                q.pop_front();
            }
            return;
        }
    }
    eprintln!("{msg}");
}

fn find_deck(dp: &[Option<PathBuf>; 2], path: &Path) -> Option<usize> {
    for (i, slot) in dp.iter().enumerate() {
        let Some(p) = slot.as_ref() else {
            continue;
        };
        if paths_match(p, path) {
            return Some(i);
        }
    }
    None
}

fn paths_match(a: &Path, b: &Path) -> bool {
    if a == b {
        return true;
    }
    // Compare file names when one side is relative.
    if a.file_name() == b.file_name() {
        if let (Ok(ca), Ok(cb)) = (a.canonicalize(), b.canonicalize()) {
            return ca == cb;
        }
        // Fallback: equal after components normalize
        return a.components().eq(b.components());
    }
    if let (Ok(ca), Ok(cb)) = (a.canonicalize(), b.canonicalize()) {
        return ca == cb;
    }
    false
}
