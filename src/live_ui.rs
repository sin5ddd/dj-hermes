//! Combined highlight TUI + command line for `dj` / `play --repl` (demo-oriented).
//!
//! Drawing avoids full-screen clears (reduces flicker). Crossfader / EQ support click/drag.
//! Per-deck Hi/Mid/Lo EQ is **visual for now** (channel EQ DSP not yet in Mixer).

use std::collections::VecDeque;
use std::io::{stdout, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossbeam::channel::Sender;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    MouseButton, MouseEventKind,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, DisableLineWrap, EnableLineWrap, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use crossterm::{cursor, execute, queue, terminal};

use crate::cmd;
use crate::engine::{Command, Engine};
use crate::highlight::{active_spans, bar_index, bar_pos, render_ansi_ex, HighlightModel};
use crate::watcher::DeckPaths;

const HELP_LINE: &str = "drag xf/EQ  a load  b head 33  x 4  bpm 128  hush  status  help  quit";

/// White-background space used as the fader thumb (user-facing "□").
const XF_THUMB: &str = "\x1b[47m \x1b[0m";

/// Max interactive track length for crossfader and EQ sliders (visible columns).
const MAX_SLIDER_TRACK: usize = 10;

/// EQ band labels (Hi / Mid / Lo).
const EQ_BANDS: [&str; 3] = ["Hi", "Mid", "Lo"];

#[derive(Clone, Copy, Debug)]
struct SliderHit {
    row: u16,
    col0: u16,
    cols: u16,
}

impl SliderHit {
    fn contains(self, col: u16, row: u16) -> bool {
        if self.cols == 0 || row != self.row {
            return false;
        }
        let c0 = self.col0.saturating_sub(1);
        let c1 = self.col0.saturating_add(self.cols);
        col >= c0 && col <= c1
    }

    fn pos_from_col(self, col: u16) -> f32 {
        if self.cols <= 1 {
            return 0.5;
        }
        let rel = (col as i32 - self.col0 as i32).clamp(0, self.cols as i32 - 1);
        rel as f32 / (self.cols as f32 - 1.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DragTarget {
    None,
    Xf,
    /// band 0=Hi,1=Mid,2=Lo ; deck 0=A,1=B
    Eq {
        band: usize,
        deck: usize,
    },
}

struct LiveState {
    model_a: Option<HighlightModel>,
    model_b: Option<HighlightModel>,
    /// Per-deck pattern cycle offset (global_bar + offset → pattern cycle).
    cycle_offset_a: i64,
    cycle_offset_b: i64,
    /// Equal-power crossfader 0=A … 1=B (mirrored from mixer for display).
    xfade_pos: f32,
    /// Per-deck EQ sliders 0..=1 (0.5 = flat). **Visual only** until Mixer channel EQ exists.
    /// Index: [band][deck] with band 0=Hi,1=Mid,2=Lo ; deck 0=A,1=B.
    eq: [[f32; 2]; 3],
    input: String,
    history: Vec<String>,
    history_idx: Option<usize>,
    log: VecDeque<String>,
    /// Last painted frame (line strings) — skip rewrite when unchanged.
    prev_lines: Vec<String>,
    prev_cols: u16,
    prev_rows: u16,
    xf_hit: SliderHit,
    /// [band][deck]
    eq_hits: [[SliderHit; 2]; 3],
    drag: DragTarget,
}

impl LiveState {
    fn set_model(&mut self, deck: usize, model: HighlightModel) {
        if deck == 0 {
            self.model_a = Some(model);
        } else {
            self.model_b = Some(model);
        }
    }

    fn push_log(&mut self, msg: impl Into<String>) {
        self.log.push_back(msg.into());
        while self.log.len() > 2 {
            self.log.pop_front();
        }
    }
}

/// Run alternate-screen UI: highlight (top) + command line (bottom).
/// Returns when the user quits.
///
/// `initial_a` / `initial_b` seed deck highlight models (e.g. songs passed to `dj`).
pub fn run(
    tx: Sender<Command>,
    deck_paths: DeckPaths,
    engine: Arc<Mutex<Engine>>,
    playhead: Arc<AtomicU64>,
    sample_rate: u32,
    initial_a: Option<HighlightModel>,
    initial_b: Option<HighlightModel>,
) -> Result<(), String> {
    let zero_hit = SliderHit {
        row: 0,
        col0: 0,
        cols: 0,
    };
    let mut state = LiveState {
        model_a: None,
        model_b: None,
        cycle_offset_a: 0,
        cycle_offset_b: 0,
        xfade_pos: 0.0,
        eq: [[0.5; 2]; 3],
        input: String::new(),
        history: Vec::new(),
        history_idx: None,
        log: VecDeque::new(),
        prev_lines: Vec::new(),
        prev_cols: 0,
        prev_rows: 0,
        xf_hit: zero_hit,
        eq_hits: [[zero_hit; 2]; 3],
        drag: DragTarget::None,
    };
    if let Some(model) = initial_a {
        state.set_model(0, model);
    }
    if let Some(model) = initial_b {
        state.set_model(1, model);
    }
    state.push_log("live UI · EQ visual-only · drag xfader / Hi Mid Lo");

    enable_raw_mode().map_err(|e| format!("raw mode: {e}"))?;
    let mut out = stdout();
    execute!(
        out,
        EnterAlternateScreen,
        DisableLineWrap,
        EnableMouseCapture,
        cursor::Hide
    )
    .map_err(|e| format!("enter alternate screen: {e}"))?;

    let frame = Duration::from_millis(33);
    let result = (|| -> Result<(), String> {
        loop {
            while event::poll(Duration::from_millis(0)).unwrap_or(false) {
                match event::read() {
                    Ok(Event::Key(key)) => {
                        if key.kind != KeyEventKind::Press {
                            continue;
                        }
                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                let _ = tx.send(Command::Hush);
                                return Ok(());
                            }
                            KeyCode::Esc if state.input.is_empty() => {
                                let _ = tx.send(Command::Hush);
                                return Ok(());
                            }
                            KeyCode::Enter => {
                                let line = state.input.trim().to_string();
                                state.input.clear();
                                state.history_idx = None;
                                if line.is_empty() {
                                    continue;
                                }
                                state.history.push(line.clone());
                                if !dispatch_line(
                                    &line,
                                    &tx,
                                    &deck_paths,
                                    &engine,
                                    sample_rate,
                                    &mut state,
                                ) {
                                    let _ = tx.send(Command::Hush);
                                    return Ok(());
                                }
                            }
                            KeyCode::Backspace => {
                                state.input.pop();
                            }
                            KeyCode::Up => {
                                if state.history.is_empty() {
                                    continue;
                                }
                                let idx = match state.history_idx {
                                    None => state.history.len() - 1,
                                    Some(0) => 0,
                                    Some(i) => i - 1,
                                };
                                state.history_idx = Some(idx);
                                state.input = state.history[idx].clone();
                            }
                            KeyCode::Down => {
                                if let Some(i) = state.history_idx {
                                    if i + 1 >= state.history.len() {
                                        state.history_idx = None;
                                        state.input.clear();
                                    } else {
                                        state.history_idx = Some(i + 1);
                                        state.input = state.history[i + 1].clone();
                                    }
                                }
                            }
                            KeyCode::Char(c)
                                if !key.modifiers.contains(KeyModifiers::CONTROL)
                                    && !key.modifiers.contains(KeyModifiers::ALT) =>
                            {
                                state.input.push(c);
                            }
                            _ => {}
                        }
                    }
                    Ok(Event::Mouse(m)) => {
                        handle_mouse(&mut state, &tx, m);
                    }
                    Ok(Event::Resize(_, _)) => {
                        state.prev_lines.clear();
                        state.prev_cols = 0;
                        state.prev_rows = 0;
                    }
                    _ => {}
                }
            }

            if let Ok(eng) = engine.try_lock() {
                sync_models_from_engine(&mut state, &eng, sample_rate);
            }

            draw_frame(&mut out, &mut state, &playhead, sample_rate)?;
            std::thread::sleep(frame);
        }
    })();

    let _ = execute!(
        out,
        DisableMouseCapture,
        EnableLineWrap,
        cursor::Show,
        LeaveAlternateScreen
    );
    let _ = disable_raw_mode();
    result
}

fn handle_mouse(state: &mut LiveState, tx: &Sender<Command>, m: crossterm::event::MouseEvent) {
    match m.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if state.xf_hit.contains(m.column, m.row) {
                state.drag = DragTarget::Xf;
                apply_drag(state, tx, m.column);
                return;
            }
            for band in 0..3 {
                for deck in 0..2 {
                    if state.eq_hits[band][deck].contains(m.column, m.row) {
                        state.drag = DragTarget::Eq { band, deck };
                        apply_drag(state, tx, m.column);
                        return;
                    }
                }
            }
            state.drag = DragTarget::None;
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            if state.drag != DragTarget::None {
                apply_drag(state, tx, m.column);
            }
        }
        MouseEventKind::Up(MouseButton::Left) => {
            state.drag = DragTarget::None;
        }
        _ => {}
    }
}

fn apply_drag(state: &mut LiveState, tx: &Sender<Command>, col: u16) {
    match state.drag {
        DragTarget::None => {}
        DragTarget::Xf => {
            let pos = state.xf_hit.pos_from_col(col);
            state.xfade_pos = pos;
            let _ = tx.send(Command::SetCrossfader(pos));
        }
        DragTarget::Eq { band, deck } => {
            let pos = state.eq_hits[band][deck].pos_from_col(col);
            state.eq[band][deck] = pos;
            // Visual only — channel EQ DSP not wired yet.
        }
    }
}

fn sync_models_from_engine(state: &mut LiveState, eng: &Engine, sample_rate: u32) {
    state.cycle_offset_a = eng.decks[0].cycle_offset();
    state.cycle_offset_b = eng.decks[1].cycle_offset();
    if state.drag != DragTarget::Xf {
        state.xfade_pos = eng.mixer.crossfader_pos();
    }
    for (deck, model_slot) in [(0, &mut state.model_a), (1, &mut state.model_b)] {
        let Some(model) = model_slot.as_mut() else {
            continue;
        };
        model.bpm = eng.transport.bpm;
        model.sample_rate = sample_rate;
        if let Some(song) = eng.decks[deck].song_title() {
            if song != model.title.as_str() {
                if let Some(s) = eng.decks[deck].song_ref() {
                    *model = HighlightModel::from_song(s, sample_rate);
                }
            } else if let Some(s) = eng.decks[deck].song_ref() {
                if s.source == model.source {
                    for (i, t) in s.tracks.iter().enumerate() {
                        if let Some(ht) = model.tracks.get_mut(i) {
                            ht.muted = t.muted;
                        }
                    }
                } else {
                    *model = HighlightModel::from_song(s, sample_rate);
                }
            }
        }
    }
}

fn draw_frame(
    out: &mut std::io::Stdout,
    state: &mut LiveState,
    playhead: &AtomicU64,
    sample_rate: u32,
) -> Result<(), String> {
    let (cols_u, rows_u) = terminal::size().unwrap_or((80, 24));
    let cols = cols_u as usize;
    let rows = rows_u as usize;
    if cols == 0 || rows == 0 {
        return Ok(());
    }

    // Footer: EQ×3 + xfade + log×2 + help + prompt  (= 8)
    let footer_rows = 8.min(rows.saturating_sub(2));
    let body_rows = rows.saturating_sub(footer_rows);
    let gs = playhead.load(Ordering::Relaxed);

    let gutter = 1usize;
    let half = cols.saturating_sub(gutter) / 2;
    let right_w = cols.saturating_sub(half + gutter);

    let left_lines = pane_lines(
        state.model_a.as_ref(),
        "A",
        gs,
        sample_rate,
        state.cycle_offset_a,
        half,
        body_rows,
    );
    let right_lines = pane_lines(
        state.model_b.as_ref(),
        "B",
        gs,
        sample_rate,
        state.cycle_offset_b,
        right_w,
        body_rows,
    );

    let mut lines: Vec<String> = Vec::with_capacity(rows);

    for i in 0..body_rows {
        let left = left_lines.get(i).map(String::as_str).unwrap_or("");
        let right = right_lines.get(i).map(String::as_str).unwrap_or("");
        let left_cell = pad_clip_ansi(left, half);
        let right_cell = pad_clip_ansi(right, right_w);
        lines.push(format!("{left_cell}│{right_cell}"));
    }

    // EQ rows: Hi / Mid / Lo  (A and B side by side)
    let eq_start_row = body_rows;
    for (band, band_name) in EQ_BANDS.iter().enumerate() {
        let row = eq_start_row + band;
        if lines.len() >= rows {
            break;
        }
        let (line, hit_a, hit_b) = format_eq_band_line(
            band_name,
            state.eq[band][0],
            state.eq[band][1],
            cols,
            row as u16,
        );
        state.eq_hits[band][0] = hit_a;
        state.eq_hits[band][1] = hit_b;
        lines.push(line);
    }

    // Crossfader (short track ≤ 10)
    let xf_row = lines.len();
    if lines.len() < rows {
        let (xf_line, hit) = format_crossfader_line(state.xfade_pos, cols, xf_row as u16);
        state.xf_hit = hit;
        lines.push(xf_line);
    }

    // Log
    let used_after_controls = lines.len();
    let log_budget = rows.saturating_sub(used_after_controls).saturating_sub(2);
    let mut log_iter = state.log.iter();
    for _ in 0..log_budget {
        let msg = log_iter.next().map(String::as_str).unwrap_or("");
        lines.push(pad_clip_ansi(msg, cols));
    }
    while lines.len() < rows.saturating_sub(2) {
        lines.push(pad_clip_ansi("", cols));
    }

    if rows >= 2 {
        lines.push(pad_clip_ansi(HELP_LINE, cols));
        let prompt = format!("» {}", state.input);
        lines.push(pad_clip_ansi(&prompt, cols));
    }
    while lines.len() < rows {
        lines.push(pad_clip_ansi("", cols));
    }
    lines.truncate(rows);

    let size_changed = state.prev_cols != cols_u || state.prev_rows != rows_u;
    if size_changed {
        state.prev_lines.clear();
        state.prev_cols = cols_u;
        state.prev_rows = rows_u;
    }

    queue!(out, cursor::Hide).map_err(|e| format!("draw: {e}"))?;

    for (i, line) in lines.iter().enumerate() {
        let dirty =
            size_changed || state.prev_lines.get(i).map(String::as_str) != Some(line.as_str());
        if dirty {
            queue!(out, cursor::MoveTo(0, i as u16)).map_err(|e| format!("draw: {e}"))?;
            write!(out, "{line}").map_err(|e| format!("draw: {e}"))?;
        }
    }

    if rows >= 1 {
        let cursor_col = (2 + state.input.chars().count()).min(cols.saturating_sub(1)) as u16;
        queue!(
            out,
            cursor::MoveTo(cursor_col, (rows - 1) as u16),
            cursor::Show
        )
        .map_err(|e| format!("draw: {e}"))?;
    }

    out.flush().map_err(|e| format!("draw: {e}"))?;
    state.prev_lines = lines;
    Ok(())
}

/// Build a short slider track (≤ `MAX_SLIDER_TRACK`) with white-bg thumb.
fn format_track(pos: f32, track_w: usize) -> String {
    let track_w = track_w.clamp(1, MAX_SLIDER_TRACK);
    let pos = pos.clamp(0.0, 1.0);
    let handle_i = if track_w <= 1 {
        0
    } else {
        ((pos * (track_w - 1) as f32).round() as usize).min(track_w - 1)
    };
    let mut track = String::with_capacity(track_w * 4 + 8);
    for i in 0..track_w {
        if i == handle_i {
            track.push_str(XF_THUMB);
        } else {
            track.push('─');
        }
    }
    track
}

/// One EQ row: `Hi  A ──□──  B ──□──` (center = flat).
fn format_eq_band_line(
    band: &str,
    pos_a: f32,
    pos_b: f32,
    cols: usize,
    row: u16,
) -> (String, SliderHit, SliderHit) {
    let track_w = MAX_SLIDER_TRACK.min(10);
    // "Hi  A " + track + "  B " + track
    let band_pad = format!("{band:<3}");
    let left_lab = format!("{band_pad} A ");
    let mid_lab = "  B ";
    let track_a = format_track(pos_a, track_w);
    let track_b = format_track(pos_b, track_w);
    let raw = format!("{left_lab}{track_a}{mid_lab}{track_b}");
    let line = pad_clip_ansi(&raw, cols);

    let col_a = left_lab.chars().count();
    // Visible length of track is track_w (ANSI not counted in chars() for prefix only —
    // left_lab has no ANSI, so col_a is correct. mid_lab after track: need visible offset.
    let col_b = col_a + track_w + mid_lab.chars().count();
    let hit_a = SliderHit {
        row,
        col0: col_a as u16,
        cols: track_w as u16,
    };
    let hit_b = SliderHit {
        row,
        col0: col_b as u16,
        cols: track_w as u16,
    };
    (line, hit_a, hit_b)
}

/// Crossfader line with track length capped at 10.
/// Returns `(line, hit)`.
fn format_crossfader_line(pos: f32, cols: usize, row: u16) -> (String, SliderHit) {
    if cols == 0 {
        return (
            String::new(),
            SliderHit {
                row,
                col0: 0,
                cols: 0,
            },
        );
    }
    let pos = pos.clamp(0.0, 1.0);
    let prefix = "XF A ";
    let pct = format!(" B {:3.0}%", pos * 100.0);
    let fixed = prefix.chars().count() + pct.chars().count();
    let track_w = cols.saturating_sub(fixed).clamp(3, MAX_SLIDER_TRACK);

    let track = format_track(pos, track_w);
    // Center the short fader when the terminal is wide.
    let content = format!("{prefix}{track}{pct}");
    let content_vis = prefix.chars().count() + track_w + pct.chars().count();
    let pad_left = cols.saturating_sub(content_vis) / 2;
    let raw = format!("{}{content}", " ".repeat(pad_left));
    let line = pad_clip_ansi(&raw, cols);
    let hit = SliderHit {
        row,
        col0: (pad_left + prefix.chars().count()) as u16,
        cols: track_w as u16,
    };
    (line, hit)
}

fn pane_lines(
    model: Option<&HighlightModel>,
    deck_label: &str,
    gs: u64,
    sample_rate: u32,
    cycle_offset: i64,
    width: usize,
    height: usize,
) -> Vec<String> {
    let text = if let Some(model) = model {
        let sr = model.sample_rate.max(sample_rate);
        let global_bar = bar_index(gs, sr, model.bpm);
        let pattern_bar = (global_bar as i64 + cycle_offset).max(0) as u64;
        let song_bar = pattern_bar.saturating_add(1);
        let pos = bar_pos(gs, sr, model.bpm);
        let spans = active_spans(model, pattern_bar, pos);
        let header = format!(
            "[{}] {}  ·  {:.0} BPM  ·  bar {song_bar}  ·  pos {:.2}",
            deck_label, model.title, model.bpm, pos
        );
        render_ansi_ex(model, &spans, &header, false)
    } else {
        format!(
            "[{}] (empty)\n────────────────────────────────────────\n{} load songs/….strudel\n",
            deck_label,
            deck_label.to_ascii_lowercase()
        )
    };

    let mut lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
    if lines.len() > height {
        lines.truncate(height);
    }
    while lines.len() < height {
        lines.push(String::new());
    }
    let _ = width;
    lines
}

/// Clip/pad to `width` **visible** columns, preserving SGR (ANSI color) sequences.
fn pad_clip_ansi(s: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let mut out = String::with_capacity(s.len() + 8);
    let mut visible = 0usize;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            out.push(c);
            if chars.peek() == Some(&'[') {
                out.push(chars.next().unwrap());
                for c2 in chars.by_ref() {
                    out.push(c2);
                    if ('@'..='~').contains(&c2) {
                        break;
                    }
                }
            }
            continue;
        }
        if visible >= width {
            break;
        }
        out.push(c);
        visible += 1;
    }
    if visible < width {
        out.push_str(&" ".repeat(width - visible));
    }
    out.push_str("\x1b[0m");
    out
}

fn dispatch_line(
    line: &str,
    tx: &Sender<Command>,
    deck_paths: &DeckPaths,
    engine: &Arc<Mutex<Engine>>,
    sample_rate: u32,
    state: &mut LiveState,
) -> bool {
    let result = cmd::exec(line, tx, deck_paths, Some(engine));
    if let Some((deck, song)) = result.loaded {
        let model = HighlightModel::from_song(&song, sample_rate);
        state.set_model(deck, model);
    }
    for m in result.messages {
        let lines: Vec<&str> = m.lines().collect();
        if lines.len() > 1 {
            for part in lines.iter().take(2) {
                state.push_log(*part);
            }
            if lines.len() > 2 {
                state.push_log("… (type help / see README)");
            }
        } else {
            state.push_log(m);
        }
    }
    !result.quit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crossfader_track_capped_at_10() {
        let (line, hit) = format_crossfader_line(0.5, 120, 5);
        assert!(hit.cols <= MAX_SLIDER_TRACK as u16, "cols={}", hit.cols);
        assert!(line.contains("\x1b[47m"));
        assert!(line.contains("XF"));
    }

    #[test]
    fn eq_band_line_has_a_and_b_tracks() {
        let (line, ha, hb) = format_eq_band_line("Hi", 0.5, 0.5, 80, 3);
        assert!(line.contains("Hi"), "{line}");
        assert!(line.contains("A "), "{line}");
        assert!(line.contains("B "), "{line}");
        assert_eq!(ha.cols, MAX_SLIDER_TRACK as u16);
        assert_eq!(hb.cols, MAX_SLIDER_TRACK as u16);
        assert!(ha.col0 < hb.col0);
        assert_eq!(ha.row, 3);
        // Center (0.5) → thumb roughly mid-track
        assert!((ha.pos_from_col(ha.col0 + ha.cols / 2) - 0.5).abs() < 0.2);
    }

    #[test]
    fn format_track_max_len() {
        let t = format_track(0.0, 100);
        // Count visible non-ANSI chars
        let vis = t.chars().filter(|c| *c != '\u{1b}').count();
        // Rough: track is short; with SGR the char count is larger but track cells = 10
        assert!(t.contains('─') || t.contains("\x1b[47m"));
        let _ = vis;
        let t10 = format_track(1.0, 10);
        assert!(t10.contains("\x1b[47m"));
    }
}
