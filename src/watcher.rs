//! Watch `.strudel` files and hot-reload loaded decks at the next bar.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crossbeam::channel::Sender;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::engine::Command;
use crate::song::parse_song;

/// Which path is loaded on deck A / B (for reload targeting).
pub type DeckPaths = Arc<Mutex<[Option<PathBuf>; 2]>>;

pub fn new_deck_paths() -> DeckPaths {
    Arc::new(Mutex::new([None, None]))
}

/// Watch `dir` non-recursively. Only paths present in `deck_paths` are reloaded.
pub fn watch_songs(
    dir: &Path,
    tx: Sender<Command>,
    deck_paths: DeckPaths,
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
            match std::fs::read_to_string(&path) {
                Ok(text) => match parse_song(&text, &path.to_string_lossy()) {
                    Ok(song) => {
                        eprintln!("↻ {} → deck {} (次の小節から反映)", song.title, deck);
                        let _ = tx.send(Command::LoadSong { deck, song });
                    }
                    Err(e) => {
                        eprintln!("⚠ {}: parse error (現行の曲を継続): {e}", path.display());
                    }
                },
                Err(e) => eprintln!("⚠ read error {}: {e}", path.display()),
            }
        }
    })?;
    watcher.watch(dir, RecursiveMode::NonRecursive)?;
    Ok(watcher)
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
