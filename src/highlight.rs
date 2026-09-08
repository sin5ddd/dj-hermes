//! Live mini-notation highlight: re-evaluate patterns on the UI thread and paint spans.

use crate::mini::{self, Span};
use crate::song::Song;

#[derive(Debug, Clone)]
pub struct HighlightTrack {
    pub name: String,
    pub muted: bool,
    pub pattern: mini::Node,
    pub mini_base: usize,
    /// Pattern speed (`.fast` / `.slow` method chain). Matches Deck scheduling.
    pub speed: f64,
    pub is_note: bool,
}

#[derive(Debug, Clone)]
pub struct HighlightModel {
    pub source: String,
    pub title: String,
    pub bpm: f64,
    pub sample_rate: u32,
    pub tracks: Vec<HighlightTrack>,
}

impl HighlightModel {
    pub fn from_song(song: &Song, sample_rate: u32) -> Self {
        let tracks = song
            .tracks
            .iter()
            .map(|t| HighlightTrack {
                name: t.name.clone(),
                muted: t.muted,
                pattern: t.code.pattern.clone(),
                mini_base: t.code.mini_base,
                speed: t.code.speed.max(1e-6),
                is_note: t.code.is_note,
            })
            .collect();
        Self {
            source: song.source.clone(),
            title: song.title.clone(),
            bpm: song.bpm.unwrap_or(120.0),
            sample_rate,
            tracks,
        }
    }
}

/// Transport helpers mirroring `Transport` without locking the engine.
pub fn bar_index(global_sample: u64, sample_rate: u32, bpm: f64) -> u64 {
    let spb = samples_per_bar(sample_rate, bpm);
    if spb <= 0.0 {
        return 0;
    }
    (global_sample as f64 / spb) as u64
}

pub fn bar_pos(global_sample: u64, sample_rate: u32, bpm: f64) -> f64 {
    let spb = samples_per_bar(sample_rate, bpm);
    if spb <= 0.0 {
        return 0.0;
    }
    (global_sample as f64 % spb) / spb
}

fn samples_per_bar(sample_rate: u32, bpm: f64) -> f64 {
    sample_rate as f64 * 60.0 / bpm * 4.0
}

/// Currently sounding mini atom (label + optional source span).
#[derive(Debug, Clone)]
pub struct ActiveAtom {
    pub span: Option<Span>,
    pub value: String,
    pub track_idx: usize,
    pub is_note: bool,
    /// Absolute cycle (`bar + start`) for VFX attack keys.
    pub start_cycle: f64,
}

/// Atoms active at `bar_pos` in `bar`.
pub fn active_atoms(model: &HighlightModel, bar: u64, bar_pos: f64) -> Vec<ActiveAtom> {
    let mut out = Vec::new();
    for (track_idx, track) in model.tracks.iter().enumerate() {
        if track.muted {
            continue;
        }
        // Deck: at = (ev.start * spb) / speed → event occupies [start/speed, (start+dur)/speed)
        // in bar units when speed stretches time. Match that mapping for UI.
        let speed = track.speed;
        for ev in mini::events(&track.pattern, bar) {
            if ev.value == "~" || ev.value.is_empty() {
                continue;
            }
            let start = ev.start / speed;
            let end = (ev.start + ev.dur) / speed;
            // Events past the bar (speed < 1 stretch) still show while inside range.
            if bar_pos + f64::EPSILON >= start && bar_pos < end {
                out.push(ActiveAtom {
                    span: ev.span.map(|rel| rel.offset(track.mini_base)),
                    value: ev.value.clone(),
                    track_idx,
                    is_note: track.is_note,
                    start_cycle: bar as f64 + start,
                });
            }
        }
    }
    out
}

/// Absolute file spans of atoms active at `bar_pos` in `bar`.
pub fn active_spans(model: &HighlightModel, bar: u64, bar_pos: f64) -> Vec<Span> {
    let mut out: Vec<Span> = active_atoms(model, bar, bar_pos)
        .into_iter()
        .filter_map(|a| a.span)
        .collect();
    // Sort + merge overlaps for stable rendering.
    out.sort_by_key(|s| (s.start, s.end));
    out
}

/// Build an ANSI frame: header + source with reverse-video active spans.
/// When `with_quit_footer` is true, append the standalone-play quit hint.
pub fn render_ansi(model: &HighlightModel, active: &[Span], header: &str) -> String {
    render_ansi_ex(model, active, header, true)
}

/// Same as [`render_ansi`], with optional quit footer (omit in live UI that draws its own).
pub fn render_ansi_ex(
    model: &HighlightModel,
    active: &[Span],
    header: &str,
    with_quit_footer: bool,
) -> String {
    let src = &model.source;
    let mut marks = vec![false; src.len()];
    for sp in active {
        let start = sp.start.min(src.len());
        let end = sp.end.min(src.len());
        for b in &mut marks[start..end] {
            *b = true;
        }
    }

    let mut out = String::with_capacity(src.len() + header.len() + 64 + active.len() * 16);
    out.push_str(header);
    if !header.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("────────────────────────────────────────\n");

    // Paint byte-by-byte so multi-byte UTF-8 stays intact (we only reverse ASCII mini atoms).
    let bytes = src.as_bytes();
    let mut i = 0;
    let mut on = false;
    while i < bytes.len() {
        let want = marks[i];
        if want != on {
            if want {
                out.push_str("\x1b[7m"); // reverse
            } else {
                out.push_str("\x1b[27m"); // reverse off
            }
            on = want;
        }
        // Copy one UTF-8 char
        let ch_len = utf8_char_len(bytes[i]);
        let end = (i + ch_len).min(bytes.len());
        // If any byte of the char is marked, keep style; marks are per-byte already set.
        out.push_str(std::str::from_utf8(&bytes[i..end]).unwrap_or("?"));
        i = end;
    }
    if on {
        out.push_str("\x1b[27m");
    }
    if !src.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("────────────────────────────────────────\n");
    if with_quit_footer {
        out.push_str("[q] quit  ·  mini-notation highlight\n");
    }
    out
}

fn utf8_char_len(first: u8) -> usize {
    if first < 0x80 {
        1
    } else if first < 0xE0 {
        2
    } else if first < 0xF0 {
        3
    } else {
        4
    }
}

/// Format status header line.
pub fn format_header(model: &HighlightModel, global_sample: u64, bar: u64, pos: f64) -> String {
    let secs = global_sample as f64 / model.sample_rate.max(1) as f64;
    format!(
        "dj-hermes  {}  ·  {:.0} BPM  ·  bar {}  ·  pos {:.2}  ·  t={:.1}s",
        model.title, model.bpm, bar, pos, secs
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::song::parse_song;

    #[test]
    fn kick_active_at_bar_start() {
        let text = "bpm: 120\n---\nkick: s(\"bd*4\").gain(0.9)\n";
        let song = parse_song(text, "t").unwrap();
        let model = HighlightModel::from_song(&song, 48_000);
        let spans = active_spans(&model, 0, 0.0);
        assert!(!spans.is_empty());
        let sp = spans[0];
        assert_eq!(&model.source[sp.start..sp.end], "bd");
    }

    #[test]
    fn snare_active_on_offbeats() {
        let text = "---\nsnare: s(\"~ sd ~ sd\")\n";
        let song = parse_song(text, "t").unwrap();
        let model = HighlightModel::from_song(&song, 48_000);
        // "~ sd ~ sd" → four slots; sd at 0.25 and 0.75
        let mid_first_sd = active_spans(&model, 0, 0.30);
        assert!(
            mid_first_sd
                .iter()
                .any(|s| &model.source[s.start..s.end] == "sd"),
            "expected sd at pos 0.30, got {mid_first_sd:?}"
        );
        let on_rest = active_spans(&model, 0, 0.05);
        assert!(
            on_rest
                .iter()
                .all(|s| &model.source[s.start..s.end] != "sd"),
            "sd should be inactive on first rest"
        );
    }

    #[test]
    fn render_contains_reverse_for_active() {
        let text = "---\nkick: s(\"bd\")\n";
        let song = parse_song(text, "t").unwrap();
        let model = HighlightModel::from_song(&song, 48_000);
        let spans = active_spans(&model, 0, 0.0);
        let frame = render_ansi(&model, &spans, "header");
        assert!(frame.contains("\x1b[7m"));
        assert!(frame.contains("bd"));
    }
}
