//! Terminal punchcard / pianoroll-style note visualization (Task 25).
//!
//! Pure UI-side evaluation: re-walk mini-notation events (same idea as highlight).
//! Audio thread is not involved beyond the shared playhead sample counter.

use crate::code::note_to_midi;
use crate::highlight::{bar_index, bar_pos};
use crate::mini;
use crate::song::Song;

#[derive(Debug, Clone)]
pub struct VizTrack {
    pub muted: bool,
    pub is_note: bool,
    pub pattern: mini::Node,
    /// Pattern speed (`.fast` / `.slow`). Matches Deck / highlight mapping.
    pub speed: f64,
}

#[derive(Debug, Clone)]
pub struct VizModel {
    pub title: String,
    pub bpm: f64,
    pub sample_rate: u32,
    pub tracks: Vec<VizTrack>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VizNote {
    /// Absolute cycle position (bar + start/speed).
    pub start_cycle: f64,
    pub end_cycle: f64,
    pub midi: u8,
}

impl VizModel {
    pub fn from_song(song: &Song, sample_rate: u32) -> Self {
        let tracks = song
            .tracks
            .iter()
            .map(|t| VizTrack {
                muted: t.muted,
                is_note: t.code.is_note,
                pattern: t.code.pattern.clone(),
                speed: t.code.speed.max(1e-6),
            })
            .collect();
        Self {
            title: song.title.clone(),
            bpm: song.bpm.unwrap_or(120.0),
            sample_rate,
            tracks,
        }
    }

    pub fn has_note_tracks(&self) -> bool {
        self.tracks.iter().any(|t| t.is_note && !t.muted)
    }
}

/// Continuous cycle = integer bar index + fractional bar position.
pub fn play_cycle(global_sample: u64, sample_rate: u32, bpm: f64) -> f64 {
    bar_index(global_sample, sample_rate, bpm) as f64 + bar_pos(global_sample, sample_rate, bpm)
}

/// Collect note events whose interval overlaps `[cycle_lo, cycle_hi)`.
pub fn collect_notes(model: &VizModel, cycle_lo: f64, cycle_hi: f64) -> Vec<VizNote> {
    if cycle_hi <= cycle_lo {
        return Vec::new();
    }
    let bar_start = cycle_lo.floor().max(0.0) as u64;
    let bar_end = cycle_hi.ceil().max(0.0) as u64;
    let mut out = Vec::new();
    for track in &model.tracks {
        if track.muted || !track.is_note {
            continue;
        }
        let speed = track.speed;
        for bar in bar_start..=bar_end {
            for ev in mini::events(&track.pattern, bar) {
                if ev.value == "~" || ev.value.is_empty() {
                    continue;
                }
                let Ok(midi) = note_to_midi(&ev.value) else {
                    continue;
                };
                // Same mapping as highlight / deck: time in bar units = start/speed.
                let start = bar as f64 + ev.start / speed;
                let end = bar as f64 + (ev.start + ev.dur) / speed;
                if end <= cycle_lo || start >= cycle_hi {
                    continue;
                }
                out.push(VizNote {
                    start_cycle: start,
                    end_cycle: end,
                    midi,
                });
            }
        }
    }
    out
}

/// MIDI min/max over notes; defaults to C3..C5 when empty.
pub fn autorange(notes: &[VizNote]) -> (u8, u8) {
    if notes.is_empty() {
        return (48, 72);
    }
    let mut lo = u8::MAX;
    let mut hi = u8::MIN;
    for n in notes {
        lo = lo.min(n.midi);
        hi = hi.max(n.midi);
    }
    if lo == hi {
        lo = lo.saturating_sub(2);
        hi = hi.saturating_add(2).min(127);
    }
    (lo, hi)
}

/// Map MIDI to row index (0 = top = high pitch).
fn row_for_midi(midi: u8, min_midi: u8, max_midi: u8, n_rows: usize) -> usize {
    if n_rows == 0 {
        return 0;
    }
    if max_midi <= min_midi || n_rows == 1 {
        return 0;
    }
    let t = (midi as f64 - min_midi as f64) / (max_midi as f64 - min_midi as f64);
    let r = ((1.0 - t) * (n_rows - 1) as f64).round() as usize;
    r.min(n_rows - 1)
}

fn midi_label(midi: u8) -> String {
    const NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let pc = (midi % 12) as usize;
    let oct = (midi as i32 / 12) - 1;
    format!("{}{}", NAMES[pc], oct)
}

/// Visible cycle window width from pane columns (roughly 2–4 cycles).
fn window_cycles(grid_cols: usize) -> f64 {
    if grid_cols < 16 {
        2.0
    } else if grid_cols < 32 {
        3.0
    } else {
        4.0
    }
}

/// Render one deck pane as lines (no trailing quit footer).
///
/// `cycle_offset` matches live UI head/cue: pattern cycle shifts with head.
/// `deck_label` is typically `A` or `B`.
pub fn render_pane_lines_ex(
    model: &VizModel,
    global_sample: u64,
    cycle_offset: i64,
    sample_rate: u32,
    cols: usize,
    rows: usize,
    deck_label: &str,
) -> Vec<String> {
    if rows == 0 {
        return Vec::new();
    }
    let sr = model.sample_rate.max(sample_rate);
    let gs_cycle = play_cycle(global_sample, sr, model.bpm);
    let play = gs_cycle + cycle_offset as f64;
    let song_bar = (play.floor().max(0.0) as u64).saturating_add(1);

    let mut lines: Vec<String> = Vec::with_capacity(rows);
    let header = format!(
        "[{}] {}  ·  {:.0} BPM  ·  bar {song_bar}  ·  punchcard",
        deck_label, model.title, model.bpm
    );
    lines.push(header);
    if rows == 1 {
        return pad_lines(lines, rows);
    }
    lines.push("─".repeat(cols.clamp(8, 40)));

    if !model.has_note_tracks() {
        if lines.len() < rows {
            lines.push("(no note tracks — sample-only)".into());
        }
        return pad_lines(lines, rows);
    }

    let body_rows = rows.saturating_sub(lines.len()).saturating_sub(1); // leave 1 for axis
    let label_w = 4usize; // e.g. "C#4"
    let grid_cols = cols.saturating_sub(label_w + 1).max(4);
    let win = window_cycles(grid_cols);
    // Playhead ~25% from left so upcoming notes are visible.
    let cycle_lo = play - win * 0.25;
    let cycle_hi = cycle_lo + win;

    let notes = collect_notes(model, cycle_lo, cycle_hi);
    if notes.is_empty() {
        if lines.len() < rows {
            lines.push("(no notes in window)".into());
        }
        return pad_lines(lines, rows);
    }

    let (min_m, max_m) = autorange(&notes);
    let n_pitch_rows = body_rows.max(1);

    // Grid of chars
    let mut grid: Vec<Vec<char>> = vec![vec![' '; grid_cols]; n_pitch_rows];
    let span = (cycle_hi - cycle_lo).max(1e-9);

    for n in &notes {
        let row = row_for_midi(n.midi, min_m, max_m, n_pitch_rows);
        let c0 = ((n.start_cycle - cycle_lo) / span * grid_cols as f64).floor() as i64;
        let c1 = ((n.end_cycle - cycle_lo) / span * grid_cols as f64).ceil() as i64;
        let start_c = c0.clamp(0, grid_cols as i64 - 1) as usize;
        let end_c = c1.clamp(1, grid_cols as i64) as usize;
        let end = end_c.max(start_c + 1).min(grid_cols);
        for cell in &mut grid[row][start_c..end] {
            *cell = '█';
        }
    }

    // Playhead column
    let ph = ((play - cycle_lo) / span * grid_cols as f64).floor() as i64;
    let ph_col = ph.clamp(0, grid_cols as i64 - 1) as usize;
    for row in grid.iter_mut() {
        if row[ph_col] == ' ' {
            row[ph_col] = '│';
        } else {
            // Active cell under playhead: reverse-video style via marker; keep block
            row[ph_col] = '▌';
        }
    }

    // Which rows get MIDI labels: top, bottom, and any row with a note if few rows
    for (r, cells) in grid.iter().enumerate() {
        let midi_for_row = if n_pitch_rows <= 1 {
            max_m
        } else {
            let t = 1.0 - (r as f64) / (n_pitch_rows - 1) as f64;
            (min_m as f64 + t * (max_m - min_m) as f64).round() as u8
        };
        let show_label =
            r == 0 || r + 1 == n_pitch_rows || cells.iter().any(|c| *c == '█' || *c == '▌');
        let label = if show_label {
            pad_label(&midi_label(midi_for_row), label_w)
        } else {
            " ".repeat(label_w)
        };
        let grid_s: String = cells.iter().collect();
        lines.push(format!("{label} {grid_s}"));
        if lines.len() >= rows.saturating_sub(1) {
            break;
        }
    }

    // Axis footer
    if lines.len() < rows {
        let axis = format!(
            "{} └{} play={:.2}",
            " ".repeat(label_w),
            "─".repeat(grid_cols.saturating_sub(2).min(20)),
            play
        );
        lines.push(axis);
    }

    pad_lines(lines, rows)
}

fn pad_label(s: &str, w: usize) -> String {
    let mut out: String = s.chars().take(w).collect();
    while out.chars().count() < w {
        out.push(' ');
    }
    out
}

fn pad_lines(mut lines: Vec<String>, rows: usize) -> Vec<String> {
    if lines.len() > rows {
        lines.truncate(rows);
    }
    while lines.len() < rows {
        lines.push(String::new());
    }
    lines
}

/// Empty deck placeholder for viz mode.
pub fn empty_pane_lines(deck_label: &str, cols: usize, rows: usize) -> Vec<String> {
    let lines = vec![
        format!("[{deck_label}] (empty)  ·  punchcard"),
        "─".repeat(cols.clamp(8, 40)),
        format!("{} load songs/….strudel", deck_label.to_ascii_lowercase()),
    ];
    pad_lines(lines, rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::song::parse_song;

    fn stair_song() -> Song {
        let text = r#"
// @title stairs
setcpm(30)
$: note("c3 e3 g3 c4").s("sine").gain(0.5)
"#;
        parse_song(text, "stairs").unwrap()
    }

    #[test]
    fn collect_four_stair_notes() {
        let song = stair_song();
        let model = VizModel::from_song(&song, 48_000);
        let notes = collect_notes(&model, 0.0, 1.0);
        assert_eq!(notes.len(), 4, "{notes:?}");
        let midis: Vec<u8> = notes.iter().map(|n| n.midi).collect();
        assert_eq!(midis, vec![48, 52, 55, 60]);
        // starts should be increasing
        for w in notes.windows(2) {
            assert!(w[0].start_cycle < w[1].start_cycle);
        }
    }

    #[test]
    fn muted_note_track_excluded() {
        let text = r#"
// @title m
setcpm(30)
// bass
$: note("c3 e3").s("sine")
"#;
        let mut song = parse_song(text, "m").unwrap();
        song.tracks[0].muted = true;
        let model = VizModel::from_song(&song, 48_000);
        let notes = collect_notes(&model, 0.0, 1.0);
        assert!(notes.is_empty());
    }

    #[test]
    fn sample_only_has_no_note_tracks() {
        let text = r#"
// @title drums
setcpm(30)
$: s("bd*4")
"#;
        let song = parse_song(text, "drums").unwrap();
        let model = VizModel::from_song(&song, 48_000);
        assert!(!model.has_note_tracks());
        let lines = render_pane_lines_ex(&model, 0, 0, 48_000, 40, 8, "A");
        let joined = lines.join("\n");
        assert!(joined.contains("no note"), "{joined}");
    }

    #[test]
    fn render_contains_blocks_and_playhead() {
        let song = stair_song();
        let model = VizModel::from_song(&song, 48_000);
        let lines = render_pane_lines_ex(&model, 0, 0, 48_000, 48, 12, "A");
        let joined = lines.join("\n");
        assert!(joined.contains('█') || joined.contains('▌'), "{joined}");
        assert!(joined.contains("punchcard"), "{joined}");
        assert!(joined.contains("[A]"), "{joined}");
    }

    #[test]
    fn cycle_offset_shifts_content() {
        let song = stair_song();
        let model = VizModel::from_song(&song, 48_000);
        // Without offset, bar 0 content is in window around 0.
        let n0 = collect_notes(&model, 0.0, 1.0);
        assert!(!n0.is_empty());
        // Offset +2 means we look at pattern bars starting later when play is 0
        // render uses play = gs_cycle + offset; collect still absolute pattern cycles.
        let n2 = collect_notes(&model, 2.0, 3.0);
        // Same pattern every bar for this song (no stack), so same count.
        assert_eq!(n0.len(), n2.len());
    }

    #[test]
    fn autorange_expands_single_pitch() {
        let notes = vec![VizNote {
            start_cycle: 0.0,
            end_cycle: 0.25,
            midi: 60,
        }];
        let (lo, hi) = autorange(&notes);
        assert!(lo < 60 && hi > 60);
    }
}
