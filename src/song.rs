//! Song file format: Strudel-oriented patterns (+ legacy named tracks).

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

/// Preferred Strudel-like format (copy-paste friendly):
/// ```text
/// // title: smoke
/// setcpm(30)
/// // kick
/// $: s("bd*4").gain(0.9)
/// // hat
/// $: s("hh*8").gain(0.3)
/// ```
///
/// `setcpm(N)` is cycles-per-minute (Strudel). With 1 cycle = 1 bar of 4 beats,
/// engine BPM is `N * 4`. `setcpm(120/4)` is accepted. `setcps(x)` is also accepted
/// (`bpm = x * 240`).
///
/// Legacy format still works:
/// ```text
/// bpm: 126
/// title: My Song
/// ---
/// kick: s("bd*4").gain(0.9)
/// ```
pub fn parse_song(text: &str, path: &str) -> Result<Song, String> {
    let mut bpm = None;
    let mut title: Option<String> = None;
    let mut tracks = Vec::new();
    let mut pending_label: Option<String> = None;
    let mut anon_idx = 0usize;

    // Walk by lines while tracking byte offsets in `text`.
    let mut offset = 0usize;
    for (lineno, raw) in text.split_inclusive('\n').enumerate() {
        let line_start = offset;
        offset += raw.len();
        // Strip trailing newline for parsing (keep line_start for absolute offsets).
        let line_no_nl = raw.trim_end_matches(['\r', '\n']);
        let line = line_no_nl.trim();
        if line.is_empty() {
            continue;
        }

        // Comments: // title: …, # title: …, or label for the next `$:` track.
        if let Some(comment) = strip_comment(line) {
            if let Some(t) = comment.strip_prefix("title:") {
                let t = t.trim();
                if !t.is_empty() {
                    title = Some(t.to_string());
                }
                pending_label = None;
            } else if !comment.is_empty() {
                // Immediate previous non-title comment becomes `$:` track name.
                pending_label = Some(comment.to_string());
            }
            continue;
        }

        if line == "---" {
            continue;
        }

        // Tempo: setcpm / setcps (Strudel) or bpm: / title: (legacy header).
        if let Some(cpm) = parse_setcpm(line) {
            bpm = Some(cpm * 4.0);
            continue;
        }
        if let Some(cps) = parse_setcps(line) {
            bpm = Some(cps * 240.0);
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

        // Track: `$:` (anonymous / comment-labeled) or `name: code`.
        let colon = line_no_nl
            .find(':')
            .ok_or_else(|| format!("line {}: expected 'name: code' or '$: code'", lineno + 1))?;
        let name_part = line_no_nl[..colon].trim();
        if name_part.is_empty() {
            return Err(format!("line {}: empty track name", lineno + 1));
        }
        let name = if name_part == "$" {
            if let Some(label) = pending_label.take() {
                label
            } else {
                let n = format!("${anon_idx}");
                anon_idx += 1;
                n
            }
        } else {
            pending_label = None;
            name_part.to_string()
        };

        let code_raw = &line_no_nl[colon + 1..];
        // Absolute start of the code portion in the full source.
        let code_abs = line_start + line_no_nl[..colon + 1].len();
        let code = parse_code(code_raw)
            .map_err(|e| format!("line {} ({}): {}", lineno + 1, name, e))?
            .with_source_base(code_abs);
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
        source: text.to_string(),
    })
}

fn strip_comment(line: &str) -> Option<&str> {
    if let Some(rest) = line.strip_prefix("//") {
        return Some(rest.trim());
    }
    if let Some(rest) = line.strip_prefix('#') {
        return Some(rest.trim());
    }
    None
}

/// Parse `setcpm(30)`, `setcpm(120/4)`, optional trailing `;`.
fn parse_setcpm(line: &str) -> Option<f64> {
    parse_call_number(line, "setcpm")
}

/// Parse `setcps(0.5)`, `setcps(1/2)`, optional trailing `;`.
fn parse_setcps(line: &str) -> Option<f64> {
    parse_call_number(line, "setcps")
}

fn parse_call_number(line: &str, name: &str) -> Option<f64> {
    let line = line.trim().trim_end_matches(';').trim();
    let rest = line.strip_prefix(name)?.trim_start();
    let inner = rest.strip_prefix('(')?.strip_suffix(')')?.trim();
    parse_number_or_div(inner)
}

/// `90`, `30.5`, or `120/4` (no nested expressions).
fn parse_number_or_div(s: &str) -> Option<f64> {
    let s = s.trim();
    if let Some((a, b)) = s.split_once('/') {
        let num: f64 = a.trim().parse().ok()?;
        let den: f64 = b.trim().parse().ok()?;
        if den == 0.0 {
            return None;
        }
        Some(num / den)
    } else {
        s.parse().ok()
    }
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
    fn parses_strudel_style() {
        let text = r#"
// title: smoke
setcpm(30)
// kick
$: s("bd*4").gain(0.9)
// hat
$: s("hh*8").gain(0.3)
// bass
$: note("c2 eb2 g2 bb2").s("sawtooth").lpf(400).gain(0.55)
"#;
        let s = parse_song(text, "songs/smoke.strudel").unwrap();
        assert_eq!(s.title, "smoke");
        // setcpm(30) → 1 cycle/bar of 4 beats → BPM 120
        assert_eq!(s.bpm, Some(120.0));
        assert_eq!(s.tracks.len(), 3);
        assert_eq!(s.tracks[0].name, "kick");
        assert_eq!(s.tracks[1].name, "hat");
        assert_eq!(s.tracks[2].name, "bass");
        assert!(s.tracks[2].code.is_note);
        assert_eq!(s.tracks[0].code.mini_src, "bd*4");
    }

    #[test]
    fn setcpm_division_and_setcps() {
        let a = parse_song("setcpm(120/4)\n$: s(\"bd\")\n", "t").unwrap();
        assert_eq!(a.bpm, Some(120.0));
        let b = parse_song("setcps(0.5)\n$: s(\"bd\")\n", "t").unwrap();
        // 0.5 cps = 30 cpm → BPM 120
        assert_eq!(b.bpm, Some(120.0));
    }

    #[test]
    fn dollar_without_label_gets_index() {
        let s = parse_song("$: s(\"bd\")\n$: s(\"hh\")\n", "t").unwrap();
        assert_eq!(s.tracks[0].name, "$0");
        assert_eq!(s.tracks[1].name, "$1");
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

    #[test]
    fn dollar_mini_span_matches_source() {
        let text = "// kick\n$: s(\"bd*4\").gain(0.9)\n";
        let s = parse_song(text, "t").unwrap();
        let pc = &s.tracks[0].code;
        let base = pc.mini_base;
        assert_eq!(&s.source[base..base + 4], "bd*4");
    }

    #[test]
    fn bundled_songs_parse() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/songs");
        let mut found = 0usize;
        for ent in std::fs::read_dir(dir).expect("songs/") {
            let ent = ent.unwrap();
            let path = ent.path();
            if path.extension().and_then(|s| s.to_str()) != Some("strudel") {
                continue;
            }
            found += 1;
            let text = std::fs::read_to_string(&path).unwrap();
            let s = parse_song(&text, path.to_str().unwrap()).unwrap_or_else(|e| {
                panic!("parse {}: {e}", path.display());
            });
            assert!(!s.tracks.is_empty(), "{}", path.display());
            assert!(s.bpm.unwrap_or(0.0) > 0.0, "{}", path.display());
        }
        assert!(found >= 5, "expected demo songs, found {found}");
    }
}
