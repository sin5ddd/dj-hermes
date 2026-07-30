//! Terminal punchcard visualization (Task 25).
//!
//! Layout (top → bottom):
//! - **Drums / samples**: one lane per `$:` sample track, **shared single color**
//! - **Notes**: piano-roll (MIDI rows), **color per note instrument** (`$:` note track)
//!
//! UI re-evaluates patterns; audio only provides the playhead sample counter.

use crate::code::note_to_midi;
use crate::highlight::{bar_index, bar_pos};
use crate::mini;
use crate::song::Song;

/// Shared color for all drum / sample lanes (ANSI 256).
const DRUM_COLOR: u8 = 250;
/// Palette for note instruments only (cycled by note-track order).
const NOTE_PALETTE: [u8; 8] = [196, 220, 46, 51, 207, 39, 208, 141];
const SGR_RESET: &str = "\x1b[0m";
/// Visual length of a drum hit on the punchcard: one 16th note (1 bar = 16 sixteenths).
const DRUM_VIS_DUR_CYCLES: f64 = 1.0 / 16.0;
/// Lower-half block so stacked lanes do not look like one solid column (`█` fills the cell).
const HIT_GLYPH: char = '▄';
/// Hit under the playhead (still half-height).
const HIT_PLAYHEAD_GLYPH: char = '▅';
/// Playhead on empty cells (broken bar — less of a continuous vertical bar).
const PLAYHEAD_GLYPH: char = '┊';

#[derive(Debug, Clone)]
pub struct VizTrack {
    pub name: String,
    pub muted: bool,
    pub is_note: bool,
    pub pattern: mini::Node,
    /// Pattern speed (`.fast` / `.slow`). Matches Deck / highlight mapping.
    pub speed: f64,
    /// For note tracks: index into [`NOTE_PALETTE`]. Unused for samples.
    pub color_idx: u8,
}

#[derive(Debug, Clone)]
pub struct VizModel {
    pub title: String,
    pub bpm: f64,
    pub sample_rate: u32,
    pub tracks: Vec<VizTrack>,
}

/// One timed hit (note or sample).
#[derive(Debug, Clone, PartialEq)]
pub struct VizHit {
    pub start_cycle: f64,
    pub end_cycle: f64,
    pub track_idx: usize,
    /// Atom token (`bd`, `c3`, …).
    pub label: String,
    /// MIDI for note hits; `None` for sample hits.
    pub midi: Option<u8>,
}

impl VizModel {
    pub fn from_song(song: &Song, sample_rate: u32) -> Self {
        // Color index only advances among note tracks so instruments get distinct hues.
        let mut note_color: u8 = 0;
        let tracks = song
            .tracks
            .iter()
            .map(|t| {
                let color_idx = if t.code.is_note {
                    let c = note_color % NOTE_PALETTE.len() as u8;
                    note_color = note_color.wrapping_add(1);
                    c
                } else {
                    0
                };
                VizTrack {
                    name: t.name.clone(),
                    muted: t.muted,
                    is_note: t.code.is_note,
                    pattern: t.code.pattern.clone(),
                    speed: t.code.speed.max(1e-6),
                    color_idx,
                }
            })
            .collect();
        Self {
            title: song.title.clone(),
            bpm: song.bpm.unwrap_or(120.0),
            sample_rate,
            tracks,
        }
    }

    pub fn has_drawable_tracks(&self) -> bool {
        self.tracks.iter().any(|t| !t.muted)
    }
}

/// SGR for a note instrument color index.
pub fn note_sgr(color_idx: u8) -> String {
    let code = NOTE_PALETTE[color_idx as usize % NOTE_PALETTE.len()];
    format!("\x1b[38;5;{code}m")
}

/// SGR for drum / sample lanes (single shared color).
pub fn drum_sgr() -> String {
    format!("\x1b[38;5;{DRUM_COLOR}m")
}

fn sgr_open(code: u8) -> String {
    format!("\x1b[38;5;{code}m")
}

/// Continuous cycle = integer bar index + fractional bar position.
pub fn play_cycle(global_sample: u64, sample_rate: u32, bpm: f64) -> f64 {
    bar_index(global_sample, sample_rate, bpm) as f64 + bar_pos(global_sample, sample_rate, bpm)
}

/// Collect note + sample hits overlapping `[cycle_lo, cycle_hi)`.
pub fn collect_hits(model: &VizModel, cycle_lo: f64, cycle_hi: f64) -> Vec<VizHit> {
    if cycle_hi <= cycle_lo {
        return Vec::new();
    }
    let bar_start = cycle_lo.floor().max(0.0) as u64;
    let bar_end = cycle_hi.ceil().max(0.0) as u64;
    let mut out = Vec::new();
    for (track_idx, track) in model.tracks.iter().enumerate() {
        if track.muted {
            continue;
        }
        let speed = track.speed;
        for bar in bar_start..=bar_end {
            for ev in mini::events(&track.pattern, bar) {
                if ev.value == "~" || ev.value.is_empty() {
                    continue;
                }
                let midi = if track.is_note {
                    match note_to_midi(&ev.value) {
                        Ok(m) => Some(m),
                        Err(_) => continue,
                    }
                } else {
                    None
                };
                let start = bar as f64 + ev.start / speed;
                // Notes keep pattern duration; drums draw as a short 16th-note tick.
                let end = if track.is_note {
                    bar as f64 + (ev.start + ev.dur) / speed
                } else {
                    start + DRUM_VIS_DUR_CYCLES
                };
                if end <= cycle_lo || start >= cycle_hi {
                    continue;
                }
                out.push(VizHit {
                    start_cycle: start,
                    end_cycle: end,
                    track_idx,
                    label: ev.value.clone(),
                    midi,
                });
            }
        }
    }
    out
}

fn window_cycles(grid_cols: usize) -> f64 {
    if grid_cols < 16 {
        2.0
    } else if grid_cols < 32 {
        3.0
    } else {
        4.0
    }
}

fn autorange_midi(midis: impl Iterator<Item = u8>) -> (u8, u8) {
    let mut lo = u8::MAX;
    let mut hi = u8::MIN;
    let mut any = false;
    for m in midis {
        any = true;
        lo = lo.min(m);
        hi = hi.max(m);
    }
    if !any {
        return (48, 72);
    }
    if lo == hi {
        lo = lo.saturating_sub(2);
        hi = hi.saturating_add(2).min(127);
    }
    (lo, hi)
}

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

/// Map cycle range → column span on a grid.
fn col_span(start: f64, end: f64, cycle_lo: f64, span: f64, grid_cols: usize) -> (usize, usize) {
    let c0 = ((start - cycle_lo) / span * grid_cols as f64).floor() as i64;
    let c1 = ((end - cycle_lo) / span * grid_cols as f64).ceil() as i64;
    let start_c = c0.clamp(0, grid_cols as i64 - 1) as usize;
    let end_c = c1.clamp(1, grid_cols as i64) as usize;
    let end = end_c.max(start_c + 1).min(grid_cols);
    (start_c, end)
}

/// Render one deck pane as lines (no trailing quit footer).
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
    lines.push(format!(
        "[{}] {}  ·  {:.0} BPM  ·  bar {song_bar}  ·  punchcard",
        deck_label, model.title, model.bpm
    ));
    if rows == 1 {
        return pad_lines(lines, rows);
    }
    lines.push("─".repeat(cols.clamp(8, 40)));

    if !model.has_drawable_tracks() {
        if lines.len() < rows {
            lines.push("(no tracks)".into());
        }
        return pad_lines(lines, rows);
    }

    let body_budget = rows.saturating_sub(lines.len()).saturating_sub(1);
    let label_w = 6usize;
    let grid_cols = cols.saturating_sub(label_w + 1).max(4);
    let win = window_cycles(grid_cols);
    let cycle_lo = play - win * 0.25;
    let cycle_hi = cycle_lo + win;
    let span = (cycle_hi - cycle_lo).max(1e-9);
    let hits = collect_hits(model, cycle_lo, cycle_hi);
    let ph = ((play - cycle_lo) / span * grid_cols as f64).floor() as i64;
    let ph_col = ph.clamp(0, grid_cols as i64 - 1) as usize;

    let drum_lanes: Vec<(usize, &VizTrack)> = model
        .tracks
        .iter()
        .enumerate()
        .filter(|(_, t)| !t.muted && !t.is_note)
        .collect();
    let has_notes = model.tracks.iter().any(|t| !t.muted && t.is_note);

    // Allocate rows: prefer at least 1 piano row when notes exist; drums get one row each first.
    let (drum_show, piano_rows) = allocate_rows(body_budget, drum_lanes.len(), has_notes);
    let drum_omitted = drum_lanes.len().saturating_sub(drum_show);
    if drum_omitted > 0 {
        lines[0] = format!("{}  +{drum_omitted} drm", lines[0]);
    }

    // --- Drum lanes (shared color, separate lanes) ---
    for (track_idx, track) in drum_lanes.iter().take(drum_show) {
        let mut on = vec![false; grid_cols];
        for h in hits.iter().filter(|h| h.track_idx == *track_idx) {
            let (a, b) = col_span(h.start_cycle, h.end_cycle, cycle_lo, span, grid_cols);
            for cell in &mut on[a..b] {
                *cell = true;
            }
        }
        let label = pad_label(&track.name, label_w);
        let grid_s = paint_mono_lane(&on, ph_col, DRUM_COLOR);
        lines.push(format!("{label} {grid_s}"));
    }

    // Separator when both sections present and room for piano after it.
    if drum_show > 0 && piano_rows >= 2 && lines.len() < rows.saturating_sub(1) {
        lines.push(format!(
            "{} {}",
            " ".repeat(label_w),
            "·".repeat(grid_cols.min(cols))
        ));
    }

    // --- Piano roll (note instruments, per-track colors) ---
    if piano_rows > 0 && has_notes {
        // Recompute remaining rows after drums (+ optional sep already pushed).
        let remaining = rows.saturating_sub(lines.len()).saturating_sub(1);
        let n_pitch = remaining.min(piano_rows).max(1);

        let note_hits: Vec<&VizHit> = hits.iter().filter(|h| h.midi.is_some()).collect();
        let (min_m, max_m) = autorange_midi(note_hits.iter().filter_map(|h| h.midi));

        // grid[row][col] = optional color code of last writer (note instrument color)
        let mut grid: Vec<Vec<Option<u8>>> = vec![vec![None; grid_cols]; n_pitch];
        for h in &note_hits {
            let Some(midi) = h.midi else { continue };
            let Some(track) = model.tracks.get(h.track_idx) else {
                continue;
            };
            let row = row_for_midi(midi, min_m, max_m, n_pitch);
            let (a, b) = col_span(h.start_cycle, h.end_cycle, cycle_lo, span, grid_cols);
            let code = NOTE_PALETTE[track.color_idx as usize % NOTE_PALETTE.len()];
            for cell in &mut grid[row][a..b] {
                *cell = Some(code);
            }
        }

        for (r, row_cells) in grid.iter().enumerate() {
            let midi_for_row = if n_pitch <= 1 {
                max_m
            } else {
                let t = 1.0 - (r as f64) / (n_pitch - 1) as f64;
                (min_m as f64 + t * (max_m - min_m) as f64).round() as u8
            };
            let show_label = r == 0 || r + 1 == n_pitch || row_cells.iter().any(|c| c.is_some());
            let label = if show_label {
                pad_label(&midi_label(midi_for_row), label_w)
            } else {
                " ".repeat(label_w)
            };
            let grid_s = paint_color_row(row_cells, ph_col);
            lines.push(format!("{label} {grid_s}"));
            if lines.len() >= rows.saturating_sub(1) {
                break;
            }
        }
    } else if body_budget > 0 && drum_show == 0 && !has_notes && lines.len() < rows {
        lines.push("(no hits in window)".into());
    }

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

/// How many drum lanes vs piano rows to show given body height.
fn allocate_rows(body: usize, n_drums: usize, has_notes: bool) -> (usize, usize) {
    if body == 0 {
        return (0, 0);
    }
    if !has_notes {
        return (n_drums.min(body), 0);
    }
    if n_drums == 0 {
        return (0, body);
    }
    // Keep at least 1 row for piano; give drums up to half (or all if body is small).
    let min_piano = 1usize;
    let max_drums = body.saturating_sub(min_piano);
    let drum_show = n_drums.min(max_drums);
    // Optional separator costs 1 row when both sections and enough space.
    let sep = if drum_show > 0 && body > drum_show + min_piano {
        1
    } else {
        0
    };
    let piano = body.saturating_sub(drum_show).saturating_sub(sep);
    (
        drum_show,
        piano.max(min_piano).min(body.saturating_sub(drum_show)),
    )
}

fn paint_mono_lane(on: &[bool], ph_col: usize, color: u8) -> String {
    let sgr = sgr_open(color);
    let mut out = String::with_capacity(on.len() * 12);
    for (c, &hit) in on.iter().enumerate() {
        if c == ph_col {
            if hit {
                out.push_str(&sgr);
                out.push(HIT_PLAYHEAD_GLYPH);
                out.push_str(SGR_RESET);
            } else {
                out.push(PLAYHEAD_GLYPH);
            }
        } else if hit {
            out.push_str(&sgr);
            out.push(HIT_GLYPH);
            out.push_str(SGR_RESET);
        } else {
            out.push(' ');
        }
    }
    out
}

fn paint_color_row(cells: &[Option<u8>], ph_col: usize) -> String {
    let mut out = String::with_capacity(cells.len() * 12);
    for (c, cell) in cells.iter().enumerate() {
        if c == ph_col {
            if let Some(code) = cell {
                out.push_str(&sgr_open(*code));
                out.push(HIT_PLAYHEAD_GLYPH);
                out.push_str(SGR_RESET);
            } else {
                out.push(PLAYHEAD_GLYPH);
            }
        } else if let Some(code) = cell {
            out.push_str(&sgr_open(*code));
            out.push(HIT_GLYPH);
            out.push_str(SGR_RESET);
        } else {
            out.push(' ');
        }
    }
    out
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

    fn smoke_like() -> Song {
        let text = r#"
// @title smoke
setcpm(30)
// kick
$: s("bd*4").gain(0.9)
// hat
$: s("hh*8").gain(0.3)
// snare
$: s("~ sd ~ sd").gain(0.8)
// bass
$: note("c2 eb2 g2 bb2").s("sawtooth").gain(0.55)
"#;
        parse_song(text, "smoke").unwrap()
    }

    fn two_note_instruments() -> Song {
        let text = r#"
// @title duo
setcpm(30)
// bass
$: note("c2 g2").s("sawtooth")
// lead
$: note("c4 e4").s("sine")
"#;
        parse_song(text, "duo").unwrap()
    }

    #[test]
    fn collect_four_stair_notes_as_hits() {
        let song = stair_song();
        let model = VizModel::from_song(&song, 48_000);
        let hits = collect_hits(&model, 0.0, 1.0);
        assert_eq!(hits.len(), 4, "{hits:?}");
        assert!(hits.iter().all(|h| h.track_idx == 0 && h.midi.is_some()));
        for w in hits.windows(2) {
            assert!(w[0].start_cycle < w[1].start_cycle);
        }
    }

    #[test]
    fn muted_track_excluded() {
        let text = r#"
// @title m
setcpm(30)
// bass
$: note("c3 e3").s("sine")
"#;
        let mut song = parse_song(text, "m").unwrap();
        song.tracks[0].muted = true;
        let model = VizModel::from_song(&song, 48_000);
        assert!(collect_hits(&model, 0.0, 1.0).is_empty());
    }

    #[test]
    fn sample_only_renders_drum_lane() {
        let text = r#"
// @title drums
setcpm(30)
// kick
$: s("bd*4")
"#;
        let song = parse_song(text, "drums").unwrap();
        let model = VizModel::from_song(&song, 48_000);
        let hits = collect_hits(&model, 0.0, 1.0);
        assert_eq!(hits.len(), 4);
        assert!(hits.iter().all(|h| h.midi.is_none()));
        let joined = render_pane_lines_ex(&model, 0, 0, 48_000, 40, 8, "A").join("\n");
        assert!(joined.contains("kick"), "{joined}");
        assert!(
            joined.contains(&drum_sgr()) || joined.contains("\x1b[38;5;250m"),
            "{joined}"
        );
    }

    #[test]
    fn drum_hits_display_as_sixteenth_notes() {
        // bd*4 would be 1/4-bar long if we used pattern dur; viz forces 1/16.
        let text = r#"
// @title drums
setcpm(30)
// kick
$: s("bd*4")
"#;
        let song = parse_song(text, "drums").unwrap();
        let model = VizModel::from_song(&song, 48_000);
        let hits = collect_hits(&model, 0.0, 1.0);
        assert_eq!(hits.len(), 4);
        for h in &hits {
            let dur = h.end_cycle - h.start_cycle;
            assert!(
                (dur - DRUM_VIS_DUR_CYCLES).abs() < 1e-9,
                "expected 1/16 bar, got {dur} for {h:?}"
            );
        }
        // Notes keep longer pattern duration (not forced to 16th).
        let note_model = VizModel::from_song(&stair_song(), 48_000);
        let notes = collect_hits(&note_model, 0.0, 1.0);
        assert!(notes
            .iter()
            .any(|h| (h.end_cycle - h.start_cycle) > DRUM_VIS_DUR_CYCLES + 1e-6));
    }

    #[test]
    fn drums_share_one_color_across_lanes() {
        let song = smoke_like();
        let model = VizModel::from_song(&song, 48_000);
        let joined = render_pane_lines_ex(&model, 0, 0, 48_000, 48, 14, "A").join("\n");
        assert!(joined.contains("kick") && joined.contains("hat") && joined.contains("snare"));
        // Drum SGR appears; all drums use DRUM_COLOR only (250).
        let drum_tag = format!("\x1b[38;5;{DRUM_COLOR}m");
        assert!(joined.contains(&drum_tag), "{joined}");
        // Note instrument color from palette also appears for bass.
        assert!(
            joined.contains("\x1b[38;5;196m") || joined.contains(&note_sgr(0)),
            "{joined}"
        );
    }

    #[test]
    fn note_instruments_get_distinct_colors() {
        let song = two_note_instruments();
        let model = VizModel::from_song(&song, 48_000);
        assert_eq!(model.tracks[0].color_idx, 0);
        assert_eq!(model.tracks[1].color_idx, 1);
        assert_ne!(note_sgr(0), note_sgr(1));
        let joined = render_pane_lines_ex(&model, 0, 0, 48_000, 48, 12, "A").join("\n");
        // Piano-roll style MIDI labels, not track-name lanes only.
        assert!(
            joined.contains('C') || joined.contains('G') || joined.contains('E'),
            "{joined}"
        );
        assert!(joined.contains(&note_sgr(0)), "{joined}");
        assert!(joined.contains(&note_sgr(1)), "{joined}");
    }

    #[test]
    fn smoke_collects_sample_and_note_hits() {
        let song = smoke_like();
        let model = VizModel::from_song(&song, 48_000);
        let hits = collect_hits(&model, 0.0, 1.0);
        assert!(hits.iter().any(|h| h.label == "bd" && h.midi.is_none()));
        assert!(hits.iter().any(|h| h.midi.is_some()));
    }

    #[test]
    fn render_contains_blocks_and_playhead() {
        let song = stair_song();
        let model = VizModel::from_song(&song, 48_000);
        let joined = render_pane_lines_ex(&model, 0, 0, 48_000, 48, 12, "A").join("\n");
        assert!(
            joined.contains(HIT_GLYPH) || joined.contains(HIT_PLAYHEAD_GLYPH),
            "{joined}"
        );
        assert!(joined.contains("punchcard"));
        assert!(joined.contains("[A]"));
    }

    #[test]
    fn cycle_offset_window_still_has_hits() {
        let song = stair_song();
        let model = VizModel::from_song(&song, 48_000);
        assert_eq!(
            collect_hits(&model, 0.0, 1.0).len(),
            collect_hits(&model, 2.0, 3.0).len()
        );
    }
}
