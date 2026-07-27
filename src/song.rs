//! Song file format: header (bpm/title) + named tracks.

use crate::code::{parse_code, PatternCode};

#[derive(Debug, Clone)]
pub struct Track {
    pub name: String,
    pub code: PatternCode,
    pub muted: bool,
}

#[derive(Debug, Clone)]
pub struct Song {
    pub title: String,
    pub bpm: Option<f64>,
    pub tracks: Vec<Track>,
    pub path: String,
}

/// Format:
/// ```text
/// // comment
/// bpm: 126
/// title: My Song
/// ---
/// kick: s("bd*4").gain(0.9)
/// bass: note("c2 eb2 g2").s("sawtooth").lpf(400)
/// ```
pub fn parse_song(text: &str, path: &str) -> Result<Song, String> {
    let mut bpm = None;
    let mut title: Option<String> = None;
    let mut tracks = Vec::new();
    let mut in_header = true;

    for (lineno, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
            continue;
        }
        if in_header {
            if line == "---" {
                in_header = false;
                continue;
            }
            if let Some(v) = line.strip_prefix("bpm:") {
                bpm = Some(
                    v.trim()
                        .parse()
                        .map_err(|_| format!("line {}: bad bpm", lineno + 1))?,
                );
                continue;
            }
            if let Some(v) = line.strip_prefix("title:") {
                title = Some(v.trim().to_string());
                continue;
            }
            // non-header line → start tracks (--- optional)
            in_header = false;
        }

        let (name, code_str) = line
            .split_once(':')
            .ok_or_else(|| format!("line {}: expected 'name: code'", lineno + 1))?;
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(format!("line {}: empty track name", lineno + 1));
        }
        // Avoid treating `bpm:` / `title:` as tracks if they appear after tracks started
        // (already handled in header). Proceed with pattern parse.
        let code = parse_code(code_str.trim())
            .map_err(|e| format!("line {} ({}): {}", lineno + 1, name, e))?;
        tracks.push(Track {
            name,
            code,
            muted: false,
        });
    }

    if tracks.is_empty() {
        return Err("no tracks".into());
    }

    let title = title.unwrap_or_else(|| {
        PathLike::file_name(path)
            .map(|s| s.to_string())
            .unwrap_or_else(|| path.to_string())
    });

    Ok(Song {
        title,
        bpm,
        tracks,
        path: path.to_string(),
    })
}

/// Minimal path helper without pulling in Path for display titles.
struct PathLike;
impl PathLike {
    fn file_name(path: &str) -> Option<&str> {
        path.rsplit(['/', '\\']).next().filter(|s| !s.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_song() {
        let text = r#"
// demo
bpm: 126
title: Demo
---
kick: s("bd*4").gain(0.9)
bass: note("c2 eb2 g2 bb2").s("sawtooth").lpf(400).gain(0.7)
"#;
        let s = parse_song(text, "songs/demo.strudel").unwrap();
        assert_eq!(s.title, "Demo");
        assert_eq!(s.bpm, Some(126.0));
        assert_eq!(s.tracks.len(), 2);
        assert_eq!(s.tracks[0].name, "kick");
        assert!(!s.tracks[0].code.is_note);
        assert!(s.tracks[1].code.is_note);
    }

    #[test]
    fn reports_error_line() {
        let err = parse_song("---\nbad: nope(\"x\")", "t").unwrap_err();
        assert!(err.contains("line 2"), "{err}");
    }

    #[test]
    fn title_from_path_when_missing() {
        let s = parse_song(
            r#"---
kick: s("bd")
"#,
            "songs/foo.strudel",
        )
        .unwrap();
        assert_eq!(s.title, "foo.strudel");
    }
}
