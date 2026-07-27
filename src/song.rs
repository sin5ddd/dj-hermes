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
    /// Full original file text (for live highlight display).
    pub source: String,
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

    // Walk by lines while tracking byte offsets in `text`.
    let mut offset = 0usize;
    for (lineno, raw) in text.split_inclusive('\n').enumerate() {
        let line_start = offset;
        offset += raw.len();
        // Strip trailing newline for parsing (keep line_start for absolute offsets).
        let line_no_nl = raw.trim_end_matches(['\r', '\n']);
        let line = line_no_nl.trim();
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

        let colon = line_no_nl
            .find(':')
            .ok_or_else(|| format!("line {}: expected 'name: code'", lineno + 1))?;
        // name/code on the raw line (not fully trimmed) so offsets stay correct.
        let name_part = line_no_nl[..colon].trim();
        if name_part.is_empty() {
            return Err(format!("line {}: empty track name", lineno + 1));
        }
        let code_raw = &line_no_nl[colon + 1..];
        // Absolute start of the code portion in the full source.
        let code_abs = line_start
            + line_no_nl[..colon + 1]
                .len();
        let code = parse_code(code_raw)
            .map_err(|e| format!("line {} ({}): {}", lineno + 1, name_part, e))?
            .with_source_base(code_abs);
        tracks.push(Track {
            name: name_part.to_string(),
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
        source: text.to_string(),
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
        assert_eq!(s.source, text);
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

    #[test]
    fn absolute_mini_span_matches_source() {
        let text = "bpm: 120\n---\nkick: s(\"bd*4\").gain(0.9)\n";
        let s = parse_song(text, "t").unwrap();
        let pc = &s.tracks[0].code;
        assert_eq!(pc.mini_src, "bd*4");
        let base = pc.mini_base;
        assert_eq!(&s.source[base..base + 4], "bd*4");

        // Atom span relative + base → absolute in file
        let ev = crate::mini::events(&pc.pattern, 0);
        let sp = ev[0].span.unwrap();
        let abs = sp.offset(base);
        assert_eq!(&s.source[abs.start..abs.end], "bd");
    }
}
