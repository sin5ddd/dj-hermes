//! Esc-quit snapshot for DJ live sessions (`~/.config/dj-hermes/resume/`).
//!
//! Play / `--text` do not write here. Mixer state is not stored.

use std::path::PathBuf;
use std::sync::Mutex;

use crate::engine::Engine;
use crate::song::home_dir;

pub const RESUME_REL: &str = ".config/dj-hermes/resume";
pub const FILE_A: &str = "a.strudel";
pub const FILE_B: &str = "b.strudel";

pub fn resume_dir() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(RESUME_REL))
}

pub fn path_for(deck: usize) -> Result<PathBuf, String> {
    let name = match deck {
        0 => FILE_A,
        1 => FILE_B,
        _ => return Err("resume: deck must be 0 or 1".into()),
    };
    Ok(resume_dir()?.join(name))
}

pub fn save_from_engine(engine: &Mutex<Engine>) -> Result<(), String> {
    let sources = {
        let e = engine.lock().map_err(|e| format!("engine lock: {e}"))?;
        [
            e.decks[0].song_ref().map(|s| s.source.clone()),
            e.decks[1].song_ref().map(|s| s.source.clone()),
        ]
    };
    std::fs::create_dir_all(resume_dir()?).map_err(|e| format!("create resume dir: {e}"))?;
    for (deck, src) in sources.into_iter().enumerate() {
        let path = path_for(deck)?;
        match src {
            Some(src) => {
                std::fs::write(&path, src.as_bytes())
                    .map_err(|e| format!("write {}: {e}", path.display()))?;
            }
            None => match std::fs::remove_file(&path) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => {
                    return Err(format!("remove {}: {e}", path.display()));
                }
            },
        }
    }
    Ok(())
}

pub fn existing_paths() -> Result<(Option<PathBuf>, Option<PathBuf>), String> {
    let a = path_for(0)?;
    let b = path_for(1)?;
    Ok((a.is_file().then_some(a), b.is_file().then_some(b)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::song::parse_song;

    fn test_home(tag: &str) -> (std::sync::MutexGuard<'static, ()>, std::path::PathBuf) {
        let guard = crate::song::lock_test_home();
        let home =
            std::env::temp_dir().join(format!("dj_hermes_resume_{}_{}", std::process::id(), tag));
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(&home).unwrap();
        std::env::set_var("HOME", &home);
        (guard, home)
    }

    fn song(src: &str) -> crate::song::Song {
        parse_song(src, "").unwrap()
    }

    fn resume_file(home: &std::path::Path, name: &str) -> std::path::PathBuf {
        home.join(".config")
            .join("dj-hermes")
            .join("resume")
            .join(name)
    }

    #[test]
    fn save_a_only_writes_a_not_b() {
        let (_g, home) = test_home("a_only");
        let mut engine = Engine::new(44_100, 120.0);
        let src_a = "// @title a\nsetcpm(30)\n$: s(\"bd*4\")\n";
        engine.load_song_immediate(0, song(src_a));
        save_from_engine(&Mutex::new(engine)).unwrap();

        let a = resume_file(&home, FILE_A);
        let b = resume_file(&home, FILE_B);
        assert_eq!(std::fs::read_to_string(&a).unwrap(), src_a);
        assert!(!b.exists());
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn save_both_then_unload_b_removes_b() {
        let (_g, home) = test_home("both");
        let mut engine = Engine::new(44_100, 120.0);
        let src_a = "// @title a\nsetcpm(30)\n$: s(\"bd*4\")\n";
        let src_b = "// @title b\nsetcpm(30)\n$: s(\"sd*4\")\n";
        engine.load_song_immediate(0, song(src_a));
        engine.load_song_immediate(1, song(src_b));
        let engine = Mutex::new(engine);
        save_from_engine(&engine).unwrap();

        let a = resume_file(&home, FILE_A);
        let b = resume_file(&home, FILE_B);
        assert_eq!(std::fs::read_to_string(&a).unwrap(), src_a);
        assert_eq!(std::fs::read_to_string(&b).unwrap(), src_b);

        engine.lock().unwrap().decks[1].unload();
        save_from_engine(&engine).unwrap();
        assert_eq!(std::fs::read_to_string(&a).unwrap(), src_a);
        assert!(!b.exists());
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn existing_paths_absolute_or_none() {
        let (_g, home) = test_home("paths");
        let (a, b) = existing_paths().unwrap();
        assert!(a.is_none() && b.is_none());

        let mut engine = Engine::new(44_100, 120.0);
        engine.load_song_immediate(0, song("// @title a\nsetcpm(30)\n$: s(\"bd*4\")\n"));
        save_from_engine(&Mutex::new(engine)).unwrap();

        let (a, b) = existing_paths().unwrap();
        let a = a.expect("deck A snapshot");
        assert!(b.is_none());
        assert!(a.is_absolute(), "{}", a.display());
        assert_eq!(a, resume_file(&home, FILE_A));
        let resolved =
            crate::song::resolve_song_path(&a.to_string_lossy()).expect("abs resume path");
        assert_eq!(resolved, a);
        let _ = std::fs::remove_dir_all(&home);
    }
}
