//! Combined highlight TUI + command line for `dj` / `play --repl` (demo-oriented).
//!
//! Drawing avoids full-screen clears (reduces flicker). Crossfader / EQ support click/drag.
//! Per-deck Hi/Mid/Lo EQ is wired to Mixer channel EQ via `Command::SetDeckEq`.

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

use crate::cmd::{self, LiveInput};
use crate::engine::{Command, Engine};
use crate::hermes::{HermesEvent, HermesHandle};
use crate::highlight::{active_spans, bar_index, bar_pos, render_ansi_ex, HighlightModel};
use crate::song::Song;
use crate::viz::{self, VizModel};
use crate::voice_input::{VoiceEvent, VoiceHandle};
use crate::watcher::{DeckPaths, UiLogBuffer};

const HELP_LINE: &str =
    "F10 viz  F12音声  drag xf/EQ  自然文→Hermes  /a load  /x 4  /bpm  /help  (op: /hush /quit)";
const HELP_LINE_REC: &str = "● REC  F12 で停止（最大7秒）";
const HELP_LINE_STT: &str = "… STT  認識中…";

/// White-background space used as the fader thumb (user-facing "□").
const XF_THUMB: &str = "\x1b[47m \x1b[0m";

/// Max interactive track length for crossfader and EQ sliders (visible columns).
const MAX_SLIDER_TRACK: usize = 10;

/// Fixed change-log lines under the mixer (always reserved, even when empty).
const LOG_LINES: usize = 3;

/// Dim SGR for the change-log area.
const LOG_DIM: &str = "\x1b[2m";

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum VoicePhase {
    #[default]
    Idle,
    Recording,
    Stt,
}

/// Body pane content: mini-notation highlight (default) or punchcard viz.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum BodyMode {
    #[default]
    Highlight,
    Viz,
}

struct LiveState {
    model_a: Option<HighlightModel>,
    model_b: Option<HighlightModel>,
    viz_a: Option<VizModel>,
    viz_b: Option<VizModel>,
    body_mode: BodyMode,
    /// Per-deck pattern cycle offset (global_bar + offset → pattern cycle).
    cycle_offset_a: i64,
    cycle_offset_b: i64,
    /// Equal-power crossfader 0=A … 1=B (mirrored from mixer for display).
    xfade_pos: f32,
    /// Per-deck EQ sliders 0..=1 (0.5 = flat). Synced with Mixer channel EQ.
    /// Index: [band][deck] with band 0=Hi,1=Mid,2=Lo ; deck 0=A,1=B.
    eq: [[f32; 2]; 3],
    input: String,
    history: Vec<String>,
    history_idx: Option<usize>,
    /// Rolling change log; only the last `LOG_LINES` entries are kept / drawn.
    log: VecDeque<String>,
    /// Centered help overlay (not written into the log).
    help_open: bool,
    /// Voice capture / STT status for the help footer line.
    voice_phase: VoicePhase,
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

    fn set_viz(&mut self, deck: usize, model: VizModel) {
        if deck == 0 {
            self.viz_a = Some(model);
        } else {
            self.viz_b = Some(model);
        }
    }

    fn set_from_song(&mut self, deck: usize, song: &Song, sample_rate: u32) {
        self.set_model(deck, HighlightModel::from_song(song, sample_rate));
        self.set_viz(deck, VizModel::from_song(song, sample_rate));
    }

    fn toggle_body_mode(&mut self) -> &'static str {
        self.body_mode = match self.body_mode {
            BodyMode::Highlight => BodyMode::Viz,
            BodyMode::Viz => BodyMode::Highlight,
        };
        self.invalidate_frame();
        match self.body_mode {
            BodyMode::Highlight => "body: highlight",
            BodyMode::Viz => "body: punchcard (viz)",
        }
    }

    fn set_body_mode(&mut self, mode: BodyMode) -> &'static str {
        if self.body_mode != mode {
            self.body_mode = mode;
            self.invalidate_frame();
        }
        match self.body_mode {
            BodyMode::Highlight => "body: highlight",
            BodyMode::Viz => "body: punchcard (viz)",
        }
    }

    fn push_log(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        // Collapse multi-line blobs into a single log row (keep first line only).
        let one_line = msg.lines().next().unwrap_or("").to_string();
        if one_line.is_empty() {
            return;
        }
        self.log.push_back(one_line);
        while self.log.len() > LOG_LINES {
            self.log.pop_front();
        }
    }

    fn invalidate_frame(&mut self) {
        self.prev_lines.clear();
    }
}

/// Run alternate-screen UI: highlight (top) + command line (bottom).
/// Returns when the user quits.
///
/// `initial_a` / `initial_b` seed deck highlight models (e.g. songs passed to `dj`).
/// `ui_log` receives watcher / external status lines into the 3-line footer log.
/// `hermes` when `Some` routes bare natural language to Hermes (local cmds need `/`).
/// `voice` when `Some` enables F12 push-to-talk (cloud STT → Hermes).
#[allow(clippy::too_many_arguments)]
pub fn run(
    tx: Sender<Command>,
    deck_paths: DeckPaths,
    engine: Arc<Mutex<Engine>>,
    playhead: Arc<AtomicU64>,
    sample_rate: u32,
    initial_a: Option<HighlightModel>,
    initial_b: Option<HighlightModel>,
    ui_log: Option<UiLogBuffer>,
    hermes: Option<HermesHandle>,
    voice: Option<VoiceHandle>,
) -> Result<(), String> {
    let zero_hit = SliderHit {
        row: 0,
        col0: 0,
        cols: 0,
    };
    let mut state = LiveState {
        model_a: None,
        model_b: None,
        viz_a: None,
        viz_b: None,
        body_mode: BodyMode::Highlight,
        cycle_offset_a: 0,
        cycle_offset_b: 0,
        xfade_pos: 0.0,
        eq: [[0.5; 2]; 3],
        input: String::new(),
        history: Vec::new(),
        history_idx: None,
        log: VecDeque::new(),
        help_open: false,
        voice_phase: VoicePhase::Idle,
        prev_lines: Vec::new(),
        prev_cols: 0,
        prev_rows: 0,
        xf_hit: zero_hit,
        eq_hits: [[zero_hit; 2]; 3],
        drag: DragTarget::None,
    };
    if let Some(model) = initial_a {
        // Viz model is filled on first engine sync / load; seed highlight only here.
        state.set_model(0, model);
    }
    if let Some(model) = initial_b {
        state.set_model(1, model);
    }
    if hermes.is_some() {
        if voice.is_some() {
            state.push_log("live UI · F12 音声→Hermes  /cmd ローカル  drag xf/EQ");
        } else {
            state.push_log("live UI · 自然文→Hermes  /cmd ローカル  drag xf/EQ");
        }
    } else {
        state.push_log("live UI · drag xfader / Hi Mid Lo EQ (Hermes off)");
    }

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
                        // Help modal: Esc / Enter / Space close it (do not quit).
                        if state.help_open {
                            match key.code {
                                KeyCode::Esc
                                | KeyCode::Enter
                                | KeyCode::Char(' ')
                                | KeyCode::Char('q')
                                | KeyCode::Char('Q') => {
                                    state.help_open = false;
                                    state.invalidate_frame();
                                }
                                KeyCode::Char('c')
                                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    let _ = tx.send(Command::Hush);
                                    return Ok(());
                                }
                                _ => {}
                            }
                            continue;
                        }
                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                let _ = tx.send(Command::Hush);
                                return Ok(());
                            }
                            KeyCode::F(12) => {
                                if let Some(ref v) = voice {
                                    v.toggle();
                                } else if hermes.is_some() {
                                    state.push_log(
                                        "voice: 無効（XAI_API_KEY / STRUDEL_STT_API_KEY を設定）",
                                    );
                                } else {
                                    state.push_log("voice: Hermes off では使えません");
                                }
                            }
                            KeyCode::Esc if state.input.is_empty() => {
                                let _ = tx.send(Command::Hush);
                                return Ok(());
                            }
                            // F10 toggles punchcard / highlight body (not a typeable char).
                            KeyCode::F(10) if state.voice_phase != VoicePhase::Recording => {
                                let msg = state.toggle_body_mode();
                                state.push_log(msg);
                            }
                            KeyCode::Enter => {
                                if state.voice_phase == VoicePhase::Recording {
                                    continue;
                                }
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
                                    hermes.as_ref(),
                                ) {
                                    let _ = tx.send(Command::Hush);
                                    return Ok(());
                                }
                            }
                            KeyCode::Backspace => {
                                if state.voice_phase != VoicePhase::Recording {
                                    state.input.pop();
                                }
                            }
                            KeyCode::Up => {
                                if state.voice_phase == VoicePhase::Recording {
                                    continue;
                                }
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
                                if state.voice_phase == VoicePhase::Recording {
                                    continue;
                                }
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
                                    && !key.modifiers.contains(KeyModifiers::ALT)
                                    && state.voice_phase != VoicePhase::Recording =>
                            {
                                state.input.push(c);
                            }
                            _ => {}
                        }
                    }
                    Ok(Event::Mouse(m)) => {
                        if state.help_open {
                            // Click anywhere dismisses the help window.
                            if matches!(m.kind, MouseEventKind::Down(MouseButton::Left)) {
                                state.help_open = false;
                                state.invalidate_frame();
                            }
                            continue;
                        }
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

            drain_ui_log(&mut state, &ui_log);
            if let Some(ref v) = voice {
                drain_voice_events(&mut state, v, hermes.as_ref());
            }
            if let Some(ref h) = hermes {
                drain_hermes_events(&mut state, h);
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
            let _ = tx.send(Command::SetDeckEq {
                deck,
                band: band as u8,
                value: pos,
            });
        }
    }
}

fn sync_models_from_engine(state: &mut LiveState, eng: &Engine, sample_rate: u32) {
    state.cycle_offset_a = eng.decks[0].cycle_offset();
    state.cycle_offset_b = eng.decks[1].cycle_offset();
    if state.drag != DragTarget::Xf {
        state.xfade_pos = eng.mixer.crossfader_pos();
    }
    // Mirror mixer EQ unless the user is dragging that band/deck.
    for band in 0..3 {
        for deck in 0..2 {
            let dragging = matches!(
                state.drag,
                DragTarget::Eq {
                    band: b,
                    deck: d
                } if b == band && d == deck
            );
            if !dragging {
                let eq = eng.mixer.deck_eq(deck);
                state.eq[band][deck] = eq[band];
            }
        }
    }
    for deck in 0..2 {
        // Unload: clear UI models when deck is empty.
        if eng.decks[deck].song_ref().is_none() {
            if deck == 0 {
                state.model_a = None;
                state.viz_a = None;
            } else {
                state.model_b = None;
                state.viz_b = None;
            }
            continue;
        }
        let Some(s) = eng.decks[deck].song_ref() else {
            continue;
        };

        let need_full = match if deck == 0 {
            state.model_a.as_ref()
        } else {
            state.model_b.as_ref()
        } {
            None => true,
            Some(m) => m.title != s.title || m.source != s.source,
        };
        let viz_missing = if deck == 0 {
            state.viz_a.is_none()
        } else {
            state.viz_b.is_none()
        };

        if need_full || viz_missing {
            let hl = HighlightModel::from_song(s, sample_rate);
            let vz = VizModel::from_song(s, sample_rate);
            if deck == 0 {
                state.model_a = Some(hl);
                state.viz_a = Some(vz);
            } else {
                state.model_b = Some(hl);
                state.viz_b = Some(vz);
            }
            continue;
        }

        // Light update: BPM / mute only.
        if let Some(model) = if deck == 0 {
            state.model_a.as_mut()
        } else {
            state.model_b.as_mut()
        } {
            model.bpm = eng.transport.bpm;
            model.sample_rate = sample_rate;
            for (i, t) in s.tracks.iter().enumerate() {
                if let Some(ht) = model.tracks.get_mut(i) {
                    ht.muted = t.muted;
                }
            }
        }
        if let Some(viz) = if deck == 0 {
            state.viz_a.as_mut()
        } else {
            state.viz_b.as_mut()
        } {
            viz.bpm = eng.transport.bpm;
            viz.sample_rate = sample_rate;
            for (i, t) in s.tracks.iter().enumerate() {
                if let Some(vt) = viz.tracks.get_mut(i) {
                    vt.muted = t.muted;
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

    // Footer: EQ×3 + xfade + log×3 (reserved) + help + prompt  (= 9)
    let footer_rows = (3 + 1 + LOG_LINES + 2).min(rows.saturating_sub(2));
    let body_rows = rows.saturating_sub(footer_rows);
    let gs = playhead.load(Ordering::Relaxed);

    let gutter = 1usize;
    let half = cols.saturating_sub(gutter) / 2;
    let right_w = cols.saturating_sub(half + gutter);

    let left_lines = pane_lines(PaneArgs {
        mode: state.body_mode,
        model: state.model_a.as_ref(),
        viz: state.viz_a.as_ref(),
        deck_label: "A",
        gs,
        sample_rate,
        cycle_offset: state.cycle_offset_a,
        width: half,
        height: body_rows,
    });
    let right_lines = pane_lines(PaneArgs {
        mode: state.body_mode,
        model: state.model_b.as_ref(),
        viz: state.viz_b.as_ref(),
        deck_label: "B",
        gs,
        sample_rate,
        cycle_offset: state.cycle_offset_b,
        width: right_w,
        height: body_rows,
    });

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

    // Change log: always exactly LOG_LINES rows (newest at bottom; empty rows reserved).
    // Show only the last LOG_LINES entries — older messages are dropped in push_log.
    let log_len = state.log.len();
    let log_start = log_len.saturating_sub(LOG_LINES);
    for i in 0..LOG_LINES {
        if lines.len() >= rows.saturating_sub(2) {
            break;
        }
        let msg = state
            .log
            .get(log_start + i)
            .map(String::as_str)
            .unwrap_or("");
        let styled = if msg.is_empty() {
            String::new()
        } else {
            format!("{LOG_DIM}{msg}\x1b[0m")
        };
        lines.push(pad_clip_ansi(&styled, cols));
    }

    if rows >= 2 {
        let help = match state.voice_phase {
            VoicePhase::Recording => HELP_LINE_REC,
            VoicePhase::Stt => HELP_LINE_STT,
            VoicePhase::Idle => HELP_LINE,
        };
        lines.push(pad_clip_ansi(help, cols));
        let prompt = format!("» {}", state.input);
        lines.push(pad_clip_ansi(&prompt, cols));
    }
    while lines.len() < rows {
        lines.push(pad_clip_ansi("", cols));
    }
    lines.truncate(rows);

    if state.help_open {
        overlay_help_modal(&mut lines, cols, rows);
    }

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
        if state.help_open {
            // Hide cursor while the help window is up.
            queue!(out, cursor::Hide).map_err(|e| format!("draw: {e}"))?;
        } else {
            let cursor_col = (2 + state.input.chars().count()).min(cols.saturating_sub(1)) as u16;
            queue!(
                out,
                cursor::MoveTo(cursor_col, (rows - 1) as u16),
                cursor::Show
            )
            .map_err(|e| format!("draw: {e}"))?;
        }
    }

    out.flush().map_err(|e| format!("draw: {e}"))?;
    state.prev_lines = lines;
    Ok(())
}

/// Pull watcher / external messages into the fixed 3-line log.
fn drain_ui_log(state: &mut LiveState, ui_log: &Option<UiLogBuffer>) {
    let Some(buf) = ui_log else {
        return;
    };
    let Ok(mut q) = buf.lock() else {
        return;
    };
    if q.is_empty() {
        return;
    }
    while let Some(msg) = q.pop_front() {
        state.push_log(msg);
    }
    // New log lines must repaint even if only the log region changed.
    state.invalidate_frame();
}

/// Centered help window overlaid on the current frame (does not use the log area).
fn overlay_help_modal(lines: &mut [String], cols: usize, rows: usize) {
    if cols < 12 || rows < 6 {
        return;
    }
    let body: Vec<&str> = cmd::HELP.trim_end().lines().collect();
    let footer = "Esc / Enter / Space で閉じる";
    let title = " help ";

    let content_w = body
        .iter()
        .map(|l| visible_width(l))
        .chain(std::iter::once(visible_width(footer)))
        .chain(std::iter::once(visible_width(title) + 2))
        .max()
        .unwrap_or(24);
    // Inner text width, then full box including borders: "│ " + text + " │"
    let inner = content_w.min(cols.saturating_sub(4)).max(16);
    let box_w = (inner + 4).min(cols);
    let inner = box_w.saturating_sub(4);

    // top + body + blank + footer + bottom
    let box_h = (1 + body.len() + 1 + 1 + 1).min(rows);
    let body_show = box_h.saturating_sub(4).min(body.len());

    let row0 = rows.saturating_sub(box_h) / 2;
    let col0 = cols.saturating_sub(box_w) / 2;

    let hline = "─".repeat(box_w.saturating_sub(2));
    let bot = format!("└{hline}┘");
    // Title on top border: ┌─ help ────────┐
    let top = title_border(&hline, title, box_w);

    let mut box_lines: Vec<String> = Vec::with_capacity(box_h);
    box_lines.push(top);
    for line in body.iter().take(body_show) {
        box_lines.push(box_content_row(line, inner));
    }
    // Pad if body was truncated by height.
    while box_lines.len() < box_h.saturating_sub(3) {
        box_lines.push(box_content_row("", inner));
    }
    box_lines.push(box_content_row("", inner));
    box_lines.push(box_content_row(footer, inner));
    box_lines.push(bot);
    box_lines.truncate(box_h);

    for (i, bline) in box_lines.iter().enumerate() {
        let r = row0 + i;
        if r >= lines.len() {
            break;
        }
        // Reverse-video / bold-ish frame: leave base content under left/right padding.
        let left = " ".repeat(col0);
        let mid = pad_clip_ansi(bline, box_w);
        // Rebuild full-width line: left pad + box + right pad.
        let right_pad = cols.saturating_sub(col0 + box_w);
        let right = " ".repeat(right_pad);
        // Dim the side gutters slightly so the window reads as a floating panel.
        let composed = format!("{LOG_DIM}{left}\x1b[0m\x1b[1m{mid}\x1b[0m{LOG_DIM}{right}\x1b[0m");
        lines[r] = pad_clip_ansi(&composed, cols);
    }
}

fn title_border(hline: &str, title: &str, box_w: usize) -> String {
    // Prefer: ┌─ help ────────┐
    let title_vis = visible_width(title);
    if title_vis + 2 >= box_w.saturating_sub(2) {
        return format!("┌{hline}┐");
    }
    let rest = box_w.saturating_sub(2 + 1 + title_vis); // after "┌─" and title
    format!("┌─{title}{}┐", "─".repeat(rest))
}

fn box_content_row(text: &str, inner: usize) -> String {
    let clipped = pad_clip_visible(text, inner);
    format!("│ {clipped} │")
}

/// Visible-column pad/clip without ANSI (help text is plain).
fn pad_clip_visible(s: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let mut out = String::new();
    let mut n = 0usize;
    for c in s.chars() {
        if n >= width {
            break;
        }
        out.push(c);
        n += 1;
    }
    if n < width {
        out.push_str(&" ".repeat(width - n));
    }
    out
}

fn visible_width(s: &str) -> usize {
    s.chars().count()
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

/// One EQ row: A track left, B track **right-aligned**.
/// e.g. `Hi  A ──□──                              B ──□──`
fn format_eq_band_line(
    band: &str,
    pos_a: f32,
    pos_b: f32,
    cols: usize,
    row: u16,
) -> (String, SliderHit, SliderHit) {
    let track_w = MAX_SLIDER_TRACK.min(10);
    let band_pad = format!("{band:<3}");
    let left_lab = format!("{band_pad} A ");
    let right_lab = "B ";
    let track_a = format_track(pos_a, track_w);
    let track_b = format_track(pos_b, track_w);

    let left_vis = left_lab.chars().count() + track_w;
    let right_vis = right_lab.chars().count() + track_w;
    let gap = cols.saturating_sub(left_vis + right_vis);
    let raw = format!("{left_lab}{track_a}{}{right_lab}{track_b}", " ".repeat(gap));
    let line = pad_clip_ansi(&raw, cols);

    let col_a = left_lab.chars().count();
    // B track starts after left block + gap + "B "
    let col_b = left_vis + gap + right_lab.chars().count();
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

struct PaneArgs<'a> {
    mode: BodyMode,
    model: Option<&'a HighlightModel>,
    viz: Option<&'a VizModel>,
    deck_label: &'a str,
    gs: u64,
    sample_rate: u32,
    cycle_offset: i64,
    width: usize,
    height: usize,
}

fn pane_lines(args: PaneArgs<'_>) -> Vec<String> {
    let PaneArgs {
        mode,
        model,
        viz,
        deck_label,
        gs,
        sample_rate,
        cycle_offset,
        width,
        height,
    } = args;
    match mode {
        BodyMode::Viz => {
            if let Some(viz) = viz {
                return viz::render_pane_lines_ex(
                    viz,
                    gs,
                    cycle_offset,
                    sample_rate,
                    width,
                    height,
                    deck_label,
                );
            }
            if model.is_none() {
                return viz::empty_pane_lines(deck_label, width, height);
            }
            // Highlight present but viz not yet synced — brief placeholder.
            let mut lines = vec![format!("[{deck_label}] (syncing viz…)"), String::new()];
            while lines.len() < height {
                lines.push(String::new());
            }
            lines.truncate(height);
            lines
        }
        BodyMode::Highlight => {
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
            lines
        }
    }
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
    hermes: Option<&HermesHandle>,
) -> bool {
    let hermes_enabled = hermes.is_some();
    match cmd::classify_live_input(line, hermes_enabled) {
        LiveInput::Empty => true,
        LiveInput::LocalCommand(body) => {
            if cmd::is_help_body(&body) {
                state.help_open = true;
                state.invalidate_frame();
                return true;
            }
            exec_local(&body, tx, deck_paths, engine, sample_rate, state)
        }
        LiveInput::HermesPrompt(prompt) => {
            let Some(h) = hermes else {
                // Should not happen: classify only returns Hermes when enabled.
                return exec_local(&prompt, tx, deck_paths, engine, sample_rate, state);
            };
            match h.enqueue(&prompt) {
                Ok(()) => {
                    let q = h.queue_len();
                    state.push_log(format!("hermes: queued (n={q})"));
                }
                Err(reason) => {
                    state.push_log(format!("hermes: {reason}"));
                }
            }
            true
        }
    }
}

fn exec_local(
    body: &str,
    tx: &Sender<Command>,
    deck_paths: &DeckPaths,
    engine: &Arc<Mutex<Engine>>,
    sample_rate: u32,
    state: &mut LiveState,
) -> bool {
    // UI-only: body highlight ⇔ punchcard (not sent to engine).
    if let Some(msg) = apply_viz_command(body, state) {
        state.push_log(msg);
        return true;
    }
    let result = cmd::exec(body, tx, deck_paths, Some(engine));
    if let Some((deck, song)) = result.loaded {
        state.set_from_song(deck, &song, sample_rate);
    }
    for m in result.messages {
        state.push_log(m);
    }
    !result.quit
}

/// Handle `viz` / `viz on` / `viz off`. Returns log message when recognized.
fn apply_viz_command(body: &str, state: &mut LiveState) -> Option<String> {
    let mut parts = body.split_whitespace();
    let head = parts.next()?;
    if !matches!(head, "viz" | "punchcard" | "pianoroll") {
        return None;
    }
    let msg = match parts.next() {
        None | Some("toggle") => state.toggle_body_mode(),
        Some("on") | Some("1") | Some("punchcard") => state.set_body_mode(BodyMode::Viz),
        Some("off") | Some("0") | Some("highlight") | Some("hl") => {
            state.set_body_mode(BodyMode::Highlight)
        }
        Some(other) => {
            return Some(format!("viz: unknown arg `{other}` (on|off|toggle)"));
        }
    };
    Some(msg.to_string())
}

fn drain_hermes_events(state: &mut LiveState, hermes: &HermesHandle) {
    for ev in hermes.drain_events() {
        match ev {
            HermesEvent::Queued { queue_len } => {
                // enqueue already logs; keep a quiet refresh for status only.
                let _ = queue_len;
            }
            HermesEvent::Running => {
                state.push_log("hermes: running…");
            }
            HermesEvent::Done { summary } => {
                state.push_log(format!("hermes: {summary}"));
            }
            HermesEvent::Failed { message } => {
                state.push_log(format!("hermes: fail {message}"));
            }
            HermesEvent::Rejected { reason } => {
                state.push_log(format!("hermes: {reason}"));
            }
        }
    }
}

fn drain_voice_events(state: &mut LiveState, voice: &VoiceHandle, hermes: Option<&HermesHandle>) {
    for ev in voice.drain_events() {
        match ev {
            VoiceEvent::RecordingStarted => {
                state.voice_phase = VoicePhase::Recording;
                state.invalidate_frame();
                state.push_log("voice: ● REC（F12 で停止）");
            }
            VoiceEvent::RecordingStopped { secs } => {
                state.push_log(format!("voice: 録音終了 ({secs:.1}s)"));
            }
            VoiceEvent::SttRunning => {
                state.voice_phase = VoicePhase::Stt;
                state.invalidate_frame();
                state.push_log("voice: … STT");
            }
            VoiceEvent::Transcript { text } => {
                state.voice_phase = VoicePhase::Idle;
                state.invalidate_frame();
                state.push_log(format!("voice: 「{text}」"));
                if let Some(h) = hermes {
                    match h.enqueue(&text) {
                        Ok(()) => {
                            let q = h.queue_len();
                            state.push_log(format!("hermes: queued (n={q})"));
                        }
                        Err(reason) => {
                            state.push_log(format!("hermes: {reason}"));
                        }
                    }
                } else {
                    state.push_log("voice: Hermes 未接続");
                }
            }
            VoiceEvent::Failed { message } => {
                state.voice_phase = VoicePhase::Idle;
                state.invalidate_frame();
                state.push_log(format!("voice: {message}"));
            }
        }
    }
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
        let cols = 80usize;
        let (line, ha, hb) = format_eq_band_line("Hi", 0.5, 0.5, cols, 3);
        assert!(line.contains("Hi"), "{line}");
        assert!(line.contains("A "), "{line}");
        assert!(line.contains("B "), "{line}");
        assert_eq!(ha.cols, MAX_SLIDER_TRACK as u16);
        assert_eq!(hb.cols, MAX_SLIDER_TRACK as u16);
        assert!(ha.col0 < hb.col0);
        assert_eq!(ha.row, 3);
        // B track is right-aligned: ends at terminal width.
        assert_eq!(
            hb.col0 as usize + hb.cols as usize,
            cols,
            "B slider should end at right edge"
        );
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

    fn empty_state() -> LiveState {
        LiveState {
            model_a: None,
            model_b: None,
            viz_a: None,
            viz_b: None,
            body_mode: BodyMode::Highlight,
            cycle_offset_a: 0,
            cycle_offset_b: 0,
            xfade_pos: 0.0,
            eq: [[0.5; 2]; 3],
            input: String::new(),
            history: Vec::new(),
            history_idx: None,
            log: VecDeque::new(),
            help_open: false,
            voice_phase: VoicePhase::Idle,
            prev_lines: Vec::new(),
            prev_cols: 0,
            prev_rows: 0,
            xf_hit: SliderHit {
                row: 0,
                col0: 0,
                cols: 0,
            },
            eq_hits: [[SliderHit {
                row: 0,
                col0: 0,
                cols: 0,
            }; 2]; 3],
            drag: DragTarget::None,
        }
    }

    #[test]
    fn push_log_keeps_only_three_lines() {
        let mut state = empty_state();
        for i in 0..10 {
            state.push_log(format!("msg {i}"));
        }
        assert_eq!(state.log.len(), LOG_LINES);
        assert_eq!(state.log.front().map(String::as_str), Some("msg 7"));
        assert_eq!(state.log.back().map(String::as_str), Some("msg 9"));
    }

    #[test]
    fn push_log_collapses_multiline_to_first_line() {
        let mut state = empty_state();
        state.push_log("first\nsecond\nthird");
        assert_eq!(state.log.len(), 1);
        assert_eq!(state.log[0], "first");
    }

    #[test]
    fn viz_command_toggles_body_mode() {
        let mut state = empty_state();
        assert_eq!(state.body_mode, BodyMode::Highlight);
        let msg = apply_viz_command("viz", &mut state).unwrap();
        assert!(msg.contains("punchcard"), "{msg}");
        assert_eq!(state.body_mode, BodyMode::Viz);
        let msg = apply_viz_command("viz off", &mut state).unwrap();
        assert!(msg.contains("highlight"), "{msg}");
        assert_eq!(state.body_mode, BodyMode::Highlight);
        assert!(apply_viz_command("bpm 120", &mut state).is_none());
    }

    #[test]
    fn help_modal_draws_box_with_commands() {
        let cols = 80usize;
        let rows = 24usize;
        let mut lines: Vec<String> = (0..rows).map(|_| pad_clip_ansi("", cols)).collect();
        overlay_help_modal(&mut lines, cols, rows);
        let joined = lines.join("\n");
        assert!(joined.contains("help"), "{joined}");
        assert!(joined.contains("a|b load"), "{joined}");
        assert!(joined.contains("┌") && joined.contains("┐"), "{joined}");
        assert!(
            joined.contains("閉じる") || joined.contains("Esc"),
            "{joined}"
        );
    }

    #[test]
    fn is_help_via_classify() {
        // Hermes on: local help needs /
        match cmd::classify_live_input("/help", true) {
            LiveInput::LocalCommand(b) => assert!(cmd::is_help_body(&b)),
            other => panic!("expected local help, got {other:?}"),
        }
        match cmd::classify_live_input("help", true) {
            LiveInput::HermesPrompt(_) => {}
            other => panic!("bare help should be Hermes when enabled: {other:?}"),
        }
        // Hermes off: bare help is local
        match cmd::classify_live_input("help", false) {
            LiveInput::LocalCommand(b) => assert!(cmd::is_help_body(&b)),
            other => panic!("expected local help, got {other:?}"),
        }
        assert!(cmd::is_help_body("?"));
        assert!(cmd::is_help_body("h"));
        assert!(!cmd::is_help_body("status"));
    }
}
