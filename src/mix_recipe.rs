//! Bundled DJ mix recipes (`mixes/<slug>.strudel`).
//!
//! Mix-only meaning lives in comment tags (`@dsp`, `@bars`, `@grid`). `$:` tracks
//! play on the hidden mix lane. Control thread I/O only.

use std::fs;
use std::path::{Path, PathBuf};

use crate::mixer::{FillKind, MixGrid};
use crate::song::{parse_song, Song};

pub const DEFAULT_MIXES_DIR: &str = "mixes";
const MAX_LANE_TRACKS: usize = 4;

#[derive(Debug, Clone)]
pub struct MixRecipe {
    pub slug: String,
    pub dsp: FillKind,
    pub bars: u32,
    pub grid: MixGrid,
    pub song: Song,
}

#[derive(Debug, Clone)]
pub struct ResolvedMix {
    pub dsp: FillKind,
    pub bars: u32,
    pub grid: MixGrid,
    pub lane: Option<Box<Song>>,
}

/// Map CLI/MCP aliases onto a catalog slug.
pub fn canonical_mix_slug(kind: &str) -> String {
    let s = kind.trim().to_ascii_lowercase();
    match s.as_str() {
        "countin" => "count".into(),
        other => other.to_string(),
    }
}

pub fn valid_mix_slug(slug: &str) -> bool {
    let mut chars = slug.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if slug.len() > 32 {
        return false;
    }
    (first.is_ascii_lowercase() || first.is_ascii_digit())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn mixes_dirs() -> Vec<PathBuf> {
    vec![
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_MIXES_DIR),
        PathBuf::from(DEFAULT_MIXES_DIR),
    ]
}

fn find_mix_file(slug: &str) -> Option<PathBuf> {
    if !valid_mix_slug(slug) {
        return None;
    }
    for dir in mixes_dirs() {
        let p = dir.join(format!("{slug}.strudel"));
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

pub fn list_mix_slugs() -> Vec<String> {
    let mut out = Vec::new();
    for dir in mixes_dirs() {
        let Ok(rd) = fs::read_dir(&dir) else {
            continue;
        };
        for ent in rd.flatten() {
            let path = ent.path();
            if path.extension().and_then(|e| e.to_str()) != Some("strudel") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let stem = stem.to_ascii_lowercase();
            if valid_mix_slug(&stem) && !out.iter().any(|s| s == &stem) {
                out.push(stem);
            }
        }
    }
    out.sort();
    out
}

#[derive(Default)]
struct MixTags {
    mix: Option<String>,
    dsp: Option<String>,
    bars: Option<u32>,
    grid: Option<String>,
}

fn comment_body(line: &str) -> Option<&str> {
    let t = line.trim();
    if let Some(rest) = t.strip_prefix("//") {
        return Some(rest.trim());
    }
    if let Some(rest) = t.strip_prefix('#') {
        return Some(rest.trim());
    }
    None
}

fn extract_mix_tags(text: &str) -> MixTags {
    let mut tags = MixTags::default();
    for line in text.lines() {
        let Some(body) = comment_body(line) else {
            continue;
        };
        let mut toks = body.split_whitespace().peekable();
        while let Some(tok) = toks.next() {
            let key = tok.to_ascii_lowercase();
            match key.as_str() {
                "@mix" => {
                    if let Some(v) = toks.next() {
                        tags.mix = Some(v.to_ascii_lowercase());
                    }
                }
                "@dsp" => {
                    if let Some(v) = toks.next() {
                        tags.dsp = Some(v.to_ascii_lowercase());
                    }
                }
                "@bars" => {
                    if let Some(v) = toks.next() {
                        if let Ok(n) = v.parse::<u32>() {
                            tags.bars = Some(n);
                        }
                    }
                }
                "@grid" => {
                    if let Some(v) = toks.next() {
                        tags.grid = Some(v.to_ascii_lowercase());
                    }
                }
                _ => {
                    if let Some(v) = key.strip_prefix("@bars=") {
                        if let Ok(n) = v.parse::<u32>() {
                            tags.bars = Some(n);
                        }
                    } else if let Some(v) = key.strip_prefix("@dsp=") {
                        tags.dsp = Some(v.to_string());
                    } else if let Some(v) = key.strip_prefix("@grid=") {
                        tags.grid = Some(v.to_string());
                    } else if let Some(v) = key.strip_prefix("@mix=") {
                        tags.mix = Some(v.to_string());
                    }
                }
            }
        }
    }
    tags
}

fn has_strudel_tracks(text: &str) -> bool {
    text.lines().any(|line| {
        let t = line.trim();
        t.starts_with("$:")
    })
}

fn empty_song(text: &str, path: &str) -> Song {
    let title = Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("mix")
        .to_string();
    Song {
        title,
        meta: Default::default(),
        bpm: None,
        tracks: Vec::new(),
        path: path.to_string(),
        source: text.to_string(),
    }
}

pub fn parse_mix_recipe(text: &str, path: &str) -> Result<MixRecipe, String> {
    let song = if has_strudel_tracks(text) {
        parse_song(text, path)?
    } else {
        empty_song(text, path)
    };
    let tags = extract_mix_tags(text);
    if let Some(ref mv) = tags.mix {
        if mv != "fill" {
            return Err(format!("@mix must be fill (got {mv})"));
        }
    }
    if song.tracks.len() > MAX_LANE_TRACKS {
        return Err(format!(
            "mix lane allows at most {MAX_LANE_TRACKS} tracks, got {}",
            song.tracks.len()
        ));
    }
    let dsp = match tags.dsp.as_deref() {
        Some(s) => FillKind::parse(s)?,
        None if song.tracks.is_empty() => {
            return Err("mix recipe needs @dsp or at least one $:".into());
        }
        None => FillKind::Lane,
    };
    let bars = tags.bars.unwrap_or_else(|| dsp.default_bars()).clamp(1, 32);
    let grid = match tags.grid.as_deref() {
        None => MixGrid::Eighth,
        Some(s) => MixGrid::parse(s)?,
    };
    let slug = Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("mix")
        .to_ascii_lowercase();
    Ok(MixRecipe {
        slug,
        dsp,
        bars,
        grid,
        song,
    })
}

pub fn load_mix_recipe(slug: &str) -> Result<MixRecipe, String> {
    let slug = canonical_mix_slug(slug);
    if !valid_mix_slug(&slug) {
        return Err(format!("invalid mix slug: {slug}"));
    }
    let path = find_mix_file(&slug).ok_or_else(|| format!("mix not found: {slug}"))?;
    let text = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    parse_mix_recipe(&text, &path.to_string_lossy())
}

/// Resolve `kind` to a DSP fill plus optional lane song.
///
/// A bundled `mixes/<slug>.strudel` wins. If the file is missing, fall back to
/// [`FillKind::parse`]. A present but broken file is an error (no silent fallback).
pub fn resolve_mix_kind(kind: &str) -> Result<ResolvedMix, String> {
    let slug = canonical_mix_slug(kind);
    if valid_mix_slug(&slug) {
        if let Some(path) = find_mix_file(&slug) {
            let text =
                fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
            let r = parse_mix_recipe(&text, &path.to_string_lossy())?;
            let lane = if r.song.tracks.is_empty() {
                None
            } else {
                Some(Box::new(r.song))
            };
            return Ok(ResolvedMix {
                dsp: r.dsp,
                bars: r.bars,
                grid: r.grid,
                lane,
            });
        }
    }
    let dsp = FillKind::parse(&slug).map_err(|e| format!("{e}; or add mixes/{slug}.strudel"))?;
    Ok(ResolvedMix {
        dsp,
        bars: dsp.default_bars(),
        grid: MixGrid::Eighth,
        lane: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_dsp_only_delay() {
        let r = parse_mix_recipe(
            r#"// @title delay
// @mix fill
// @dsp delay
// @bars 1
"#,
            "mixes/delay.strudel",
        )
        .unwrap();
        assert_eq!(r.dsp, FillKind::Delay);
        assert_eq!(r.bars, 1);
        assert!(r.song.tracks.is_empty());
    }

    #[test]
    fn parse_lane_from_tracks_without_dsp() {
        let r = parse_mix_recipe(
            r#"// @title count
$: s("bd*4").gain(0.5)
"#,
            "mixes/count.strudel",
        )
        .unwrap();
        assert_eq!(r.dsp, FillKind::Lane);
        assert_eq!(r.song.tracks.len(), 1);
    }

    #[test]
    fn parse_count_pattern() {
        let r = parse_mix_recipe(
            r#"// @title count
// @mix fill
// @dsp lane
// @bars 2
$: s("<iv:iku [iv:ic iv:ni iv:sa iv:si]>").gain(0.5).cut(1)
"#,
            "mixes/count.strudel",
        )
        .unwrap();
        assert_eq!(r.dsp, FillKind::Lane);
        assert_eq!(r.bars, 2);
        assert_eq!(r.song.tracks.len(), 1);
    }

    #[test]
    fn rejects_too_many_tracks() {
        let text = r#"
$: s("bd")
$: s("sd")
$: s("hh")
$: s("oh")
$: s("cp")
"#;
        let err = parse_mix_recipe(text, "mixes/x.strudel").unwrap_err();
        assert!(err.contains("at most 4"), "{err}");
    }

    #[test]
    fn rejects_bad_mix_move() {
        let err = parse_mix_recipe("// @mix long\n// @dsp delay\n", "mixes/x.strudel").unwrap_err();
        assert!(err.contains("fill"), "{err}");
    }

    #[test]
    fn setcpm_does_not_fail_parse() {
        let r = parse_mix_recipe("setcpm(30)\n// @dsp delay\n", "mixes/delay.strudel").unwrap();
        assert_eq!(r.dsp, FillKind::Delay);
        // DSP-only recipes skip parse_song, so setcpm is not stored. Engine must not seed BPM from mix files.
        assert!(r.song.bpm.is_none());
    }

    #[test]
    fn invalid_slug_rejected() {
        assert!(!valid_mix_slug("../x"));
        assert!(!valid_mix_slug("a/b"));
        assert!(!valid_mix_slug(""));
        assert!(valid_mix_slug("count"));
        assert_eq!(canonical_mix_slug("countin"), "count");
    }

    #[test]
    fn load_bundled_count() {
        let r = load_mix_recipe("count").expect("bundled mixes/count.strudel");
        assert_eq!(r.dsp, FillKind::Lane);
        assert_eq!(r.bars, 2);
        assert_eq!(r.song.tracks.len(), 1);
    }

    #[test]
    fn load_bundled_riser() {
        let r = load_mix_recipe("riser").expect("bundled mixes/riser.strudel");
        assert_eq!(r.dsp, FillKind::Riser);
        assert_eq!(r.bars, 4);
        assert!(r.song.tracks.is_empty());
    }

    #[test]
    fn resolve_missing_falls_back_to_fillkind() {
        let r = resolve_mix_kind("delay").unwrap();
        assert_eq!(r.dsp, FillKind::Delay);
        assert!(r.lane.is_none() || r.dsp == FillKind::Delay);
    }

    #[test]
    fn resolve_count_has_lane() {
        let r = resolve_mix_kind("count").unwrap();
        assert_eq!(r.dsp, FillKind::Lane);
        assert!(r.lane.is_some());
        assert_eq!(r.bars, 2);
    }
}
