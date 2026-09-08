//! Combined highlight TUI + command line for `dj` and `play` live sessions.
//!
//! Drawing avoids full-screen clears (reduces flicker). Crossfader / EQ support click/drag.
//! Per-deck Hi/Mid/Lo EQ is wired to Mixer channel EQ via `Command::SetDeckEq`.

use std::collections::VecDeque;
use std::io::{stdout, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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

use crate::automix::{AutomixEvent, AutomixHandle, AutomixMode, GATEWAY_DOWN_MSG};
use crate::cmd::{self, DeckPaths, LiveInput};
use crate::code::note_to_midi;
use crate::complete::{self, CompleteCtx, CompleteResult};
use crate::engine::{Command, Engine};
use crate::hermes::{HermesEvent, HermesHandle};
use crate::highlight::{
    active_atoms, active_spans, bar_index, bar_pos, render_ansi_ex, HighlightModel,
};
use crate::live_fx::{self, FxState};
use crate::session::SessionKind;
use crate::song::Song;
use crate::viz::{self, VizModel};
use crate::voice_input::{VoiceEvent, VoiceHandle, VoiceMode};

const HELP_LINE: &str = "F9 vfx  F10 viz  F12音声  mute/FLT  drag xf/EQ  /help";
const HELP_LINE_PLAY: &str = "F9 vfx  F10 viz  F12音声  mute/FLT  drag EQ  /help";
/// Max candidate rows inside the suggest overlay (scroll window).
const SUGGEST_MAX_ROWS: usize = 10;
const HELP_LINE_LISTEN: &str = "VAD 待ち  話してね  F12 で一時停止";
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
    Flt,
    Dly,
}

#[derive(Clone, Debug)]
struct MuteHit {
    hit: SliderHit,
    deck: usize,
    track: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum VoicePhase {
    #[default]
    Idle,
    Listening,
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
    /// In-progress mix macro, shown dim on the XF row.
    mix_label: Option<String>,
    /// Per-deck EQ sliders 0..=1 (1.0 = 0 dB). Synced with Mixer channel EQ.
    /// Index: [band][deck] with band 0=Hi,1=Mid,2=Lo ; deck 0=A,1=B.
    eq: [[f32; 2]; 3],
    /// Post-mix filter knob 0..=1 (0.5 = bypass, left = HPF, right = LPF).
    flt_pos: f32,
    /// Post-mix delay wet 0..=1.
    delay_wet: f32,
    input: String,
    history: Vec<String>,
    history_idx: Option<usize>,
    /// Index into the current local-command suggestion list (↑↓).
    suggest_idx: usize,
    /// Esc closed the suggest overlay; cleared when the input changes.
    suggest_dismissed: bool,
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
    flt_hit: SliderHit,
    dly_hit: SliderHit,
    mute_hits: Vec<MuteHit>,
    drag: DragTarget,
    /// Issue #42 overlay. Default on; independent of `/viz`.
    vfx_on: bool,
    fx: FxState,
    vfx_hit: SliderHit,
    last_frame: Option<Instant>,
    session: SessionKind,
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

    fn toggle_vfx(&mut self) -> &'static str {
        self.vfx_on = !self.vfx_on;
        if !self.vfx_on {
            self.fx.clear();
        }
        self.invalidate_frame();
        if self.vfx_on {
            "vfx: on"
        } else {
            "vfx: off"
        }
    }

    fn set_vfx(&mut self, on: bool) -> &'static str {
        if self.vfx_on != on {
            self.vfx_on = on;
            if !on {
                self.fx.clear();
            }
            self.invalidate_frame();
        }
        if self.vfx_on {
            "vfx: on"
        } else {
            "vfx: off"
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

fn snapshot_dj_then_hush(
    session: SessionKind,
    engine: &Arc<Mutex<Engine>>,
    tx: &Sender<Command>,
    snap_err: &mut Option<String>,
) {
    if !session.single_deck() {
        if let Err(e) = crate::resume::save_from_engine(engine) {
            *snap_err = Some(e);
        }
    }
    let _ = tx.send(Command::Hush);
}

/// Run alternate-screen UI: highlight (top) + command line (bottom).
/// Returns when the user quits.
///
/// `initial_a` / `initial_b` seed deck highlight models (e.g. songs passed to `dj`).
/// `hermes` when `Some` routes bare natural language to Hermes (local cmds need `/`).
/// `voice` when `Some` enables F12 / VAD (Hermes STT → Hermes（URL があれば HTTP STT）).
#[allow(clippy::too_many_arguments)]
pub fn run(
    tx: Sender<Command>,
    deck_paths: DeckPaths,
    engine: Arc<Mutex<Engine>>,
    playhead: Arc<AtomicU64>,
    sample_rate: u32,
    initial_a: Option<HighlightModel>,
    initial_b: Option<HighlightModel>,
    hermes: Option<HermesHandle>,
    voice: Option<VoiceHandle>,
    session: SessionKind,
    automix_start: bool,
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
        mix_label: None,
        eq: [[1.0; 2]; 3],
        flt_pos: 0.5,
        delay_wet: 0.0,
        input: String::new(),
        history: Vec::new(),
        history_idx: None,
        suggest_idx: 0,
        suggest_dismissed: false,
        log: VecDeque::new(),
        help_open: false,
        voice_phase: VoicePhase::Idle,
        prev_lines: Vec::new(),
        prev_cols: 0,
        prev_rows: 0,
        xf_hit: zero_hit,
        eq_hits: [[zero_hit; 2]; 3],
        flt_hit: zero_hit,
        dly_hit: zero_hit,
        mute_hits: Vec::new(),
        drag: DragTarget::None,
        vfx_on: true,
        fx: FxState::new(),
        vfx_hit: zero_hit,
        last_frame: None,
        session,
    };
    if let Some(model) = initial_a {
        // Viz model is filled on first engine sync / load; seed highlight only here.
        state.set_model(0, model);
    }
    if let Some(model) = initial_b {
        state.set_model(1, model);
    }
    let play = session.single_deck();
    if hermes.is_some() {
        if let Some(ref v) = voice {
            match v.mode() {
                VoiceMode::Vad => {
                    state.push_log("live UI · VAD 音声→Hermes  F12 一時停止  /cmd ローカル");
                }
                VoiceMode::Push if play => {
                    state.push_log("live UI · F12 音声→Hermes  /load  /cmd ローカル  drag EQ");
                }
                VoiceMode::Push => {
                    state.push_log("live UI · F12 音声→Hermes  /cmd ローカル  drag xf/EQ");
                }
            }
        } else if play {
            state.push_log("live UI · 自然文→Hermes  /load  /cmd ローカル  drag EQ");
        } else {
            state.push_log("live UI · 自然文→Hermes  /cmd ローカル  drag xf/EQ");
        }
    } else if play {
        state.push_log("live UI · drag Hi Mid Lo EQ (Hermes off)");
    } else {
        state.push_log("live UI · drag xfader / Hi Mid Lo EQ (Hermes off)");
    }

    let automix = match hermes.as_ref() {
        Some(h) if !session.single_deck() => Some(AutomixHandle::start(
            h.config().bin.clone(),
            h.config().profile.clone(),
        )),
        _ => None,
    };
    if automix_start {
        if let Some(a) = &automix {
            a.force_on();
        } else {
            state.push_log("automix: skipped (Hermes off)");
        }
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
    let mut snap_err: Option<String> = None;
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
                                    snapshot_dj_then_hush(session, &engine, &tx, &mut snap_err);
                                    return Ok(());
                                }
                                _ => {}
                            }
                            continue;
                        }

                        let hermes_on = hermes.is_some();
                        let suggest = suggest_for_state(&state, hermes_on);
                        let suggest_on = suggest_is_open(&state, &suggest);

                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                snapshot_dj_then_hush(session, &engine, &tx, &mut snap_err);
                                return Ok(());
                            }
                            KeyCode::F(12) => {
                                if let Some(ref v) = voice {
                                    v.toggle();
                                } else if hermes.is_some() {
                                    state.push_log("voice: 無効（マイクなし / Hermes Python 未検出 / --no-voice）");
                                } else {
                                    state.push_log("voice: Hermes off では使えません");
                                }
                            }
                            // Suggest open: Esc only dismisses the overlay.
                            KeyCode::Esc if suggest_on => {
                                state.suggest_dismissed = true;
                                state.invalidate_frame();
                            }
                            KeyCode::Esc if state.input.is_empty() => {
                                snapshot_dj_then_hush(session, &engine, &tx, &mut snap_err);
                                return Ok(());
                            }
                            KeyCode::F(9) if state.voice_phase != VoicePhase::Recording => {
                                let msg = state.toggle_vfx();
                                state.push_log(msg);
                            }
                            // F10 toggles punchcard / highlight body (not a typeable char).
                            KeyCode::F(10) if state.voice_phase != VoicePhase::Recording => {
                                let msg = state.toggle_body_mode();
                                state.push_log(msg);
                            }
                            KeyCode::Up
                                if suggest_on
                                    && !suggest.candidates.is_empty()
                                    && state.voice_phase != VoicePhase::Recording =>
                            {
                                let n = suggest.candidates.len();
                                state.suggest_idx = if state.suggest_idx == 0 {
                                    n - 1
                                } else {
                                    state.suggest_idx - 1
                                };
                                state.invalidate_frame();
                            }
                            KeyCode::Down
                                if suggest_on
                                    && !suggest.candidates.is_empty()
                                    && state.voice_phase != VoicePhase::Recording =>
                            {
                                let n = suggest.candidates.len();
                                state.suggest_idx = (state.suggest_idx + 1) % n;
                                state.invalidate_frame();
                            }
                            KeyCode::Tab
                                if suggest_on
                                    && !suggest.candidates.is_empty()
                                    && state.voice_phase != VoicePhase::Recording =>
                            {
                                apply_suggest_selection(&mut state, &suggest);
                            }
                            KeyCode::Enter => {
                                if state.voice_phase == VoicePhase::Recording {
                                    continue;
                                }
                                // Apply highlighted candidate before dispatch when overlay is open.
                                if suggest_on && !suggest.candidates.is_empty() {
                                    apply_suggest_selection(&mut state, &suggest);
                                }
                                let line = state.input.trim().to_string();
                                state.input.clear();
                                state.history_idx = None;
                                reset_suggest_edit(&mut state);
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
                                    automix.as_ref(),
                                ) {
                                    snapshot_dj_then_hush(session, &engine, &tx, &mut snap_err);
                                    return Ok(());
                                }
                            }
                            KeyCode::Backspace => {
                                if state.voice_phase != VoicePhase::Recording {
                                    state.input.pop();
                                    reset_suggest_edit(&mut state);
                                }
                            }
                            KeyCode::Up if state.voice_phase != VoicePhase::Recording => {
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
                                reset_suggest_edit(&mut state);
                            }
                            KeyCode::Down if state.voice_phase != VoicePhase::Recording => {
                                if let Some(i) = state.history_idx {
                                    if i + 1 >= state.history.len() {
                                        state.history_idx = None;
                                        state.input.clear();
                                    } else {
                                        state.history_idx = Some(i + 1);
                                        state.input = state.history[i + 1].clone();
                                    }
                                    reset_suggest_edit(&mut state);
                                }
                            }
                            KeyCode::Char(c)
                                if !key.modifiers.contains(KeyModifiers::CONTROL)
                                    && !key.modifiers.contains(KeyModifiers::ALT)
                                    && state.voice_phase != VoicePhase::Recording =>
                            {
                                state.input.push(c);
                                reset_suggest_edit(&mut state);
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
                        handle_mouse(&mut state, &tx, m, automix.as_ref());
                    }
                    Ok(Event::Resize(_, _)) => {
                        state.prev_lines.clear();
                        state.prev_cols = 0;
                        state.prev_rows = 0;
                    }
                    _ => {}
                }
            }

            if let Some(ref v) = voice {
                drain_voice_events(&mut state, v, hermes.as_ref(), automix.as_ref());
            }
            if let Some(ref h) = hermes {
                drain_hermes_events(&mut state, h);
            }
            if let Some(a) = &automix {
                drain_automix_events(&mut state, a);
            }

            if let Ok(eng) = engine.try_lock() {
                sync_models_from_engine(&mut state, &eng, sample_rate);
            }

            draw_frame(
                &mut out,
                &mut state,
                &playhead,
                sample_rate,
                hermes.is_some(),
            )?;
            std::thread::sleep(frame);
        }
    })();

    if let Some(a) = &automix {
        a.shutdown();
    }

    let _ = execute!(
        out,
        DisableMouseCapture,
        EnableLineWrap,
        cursor::Show,
        LeaveAlternateScreen
    );
    let _ = disable_raw_mode();
    if let Some(e) = snap_err {
        eprintln!("resume snapshot: {e}");
    }
    result
}

fn handle_mouse(
    state: &mut LiveState,
    tx: &Sender<Command>,
    m: crossterm::event::MouseEvent,
    automix: Option<&AutomixHandle>,
) {
    match m.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if state.vfx_hit.contains(m.column, m.row) {
                let msg = state.toggle_vfx();
                state.push_log(msg);
                state.drag = DragTarget::None;
                return;
            }
            if !state.session.single_deck() && state.xf_hit.contains(m.column, m.row) {
                if let Some(a) = automix {
                    a.touch();
                }
                state.drag = DragTarget::Xf;
                apply_drag(state, tx, m.column);
                return;
            }
            if state.flt_hit.contains(m.column, m.row) {
                if let Some(a) = automix {
                    a.touch();
                }
                state.drag = DragTarget::Flt;
                apply_drag(state, tx, m.column);
                return;
            }
            if state.dly_hit.contains(m.column, m.row) {
                if let Some(a) = automix {
                    a.touch();
                }
                state.drag = DragTarget::Dly;
                apply_drag(state, tx, m.column);
                return;
            }
            for mh in &state.mute_hits {
                if mh.hit.contains(m.column, m.row) {
                    if let Some(a) = automix {
                        a.touch();
                    }
                    let muted = !track_is_muted(state, mh.deck, &mh.track);
                    let _ = tx.send(Command::SetTrackMute {
                        deck: mh.deck,
                        track: mh.track.clone(),
                        muted,
                    });
                    state.push_log(format!(
                        "{} {} {}",
                        if muted { "mute" } else { "unmute" },
                        if mh.deck == 0 { "A" } else { "B" },
                        mh.track
                    ));
                    state.drag = DragTarget::None;
                    return;
                }
            }
            for band in 0..3 {
                let decks = if state.session.single_deck() { 1 } else { 2 };
                for deck in 0..decks {
                    if state.eq_hits[band][deck].contains(m.column, m.row) {
                        if let Some(a) = automix {
                            a.touch();
                        }
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
        DragTarget::Flt => {
            let pos = state.flt_hit.pos_from_col(col);
            state.flt_pos = pos;
            let (lpf, hpf) = bipolar_to_filters(pos);
            let _ = tx.send(Command::SetMixerLpf(lpf));
            let _ = tx.send(Command::SetMixerHpf(hpf));
        }
        DragTarget::Dly => {
            let pos = state.dly_hit.pos_from_col(col);
            state.delay_wet = pos;
            let _ = tx.send(Command::SetMixerDelay {
                wet: pos,
                feedback: None,
            });
        }
    }
}

fn track_is_muted(state: &LiveState, deck: usize, name: &str) -> bool {
    let model = if deck == 0 {
        state.model_a.as_ref()
    } else {
        state.model_b.as_ref()
    };
    model
        .and_then(|m| m.tracks.iter().find(|t| t.name == name))
        .map(|t| t.muted)
        .unwrap_or(false)
}

fn sync_models_from_engine(state: &mut LiveState, eng: &Engine, sample_rate: u32) {
    state.cycle_offset_a = eng.decks[0].cycle_offset();
    state.cycle_offset_b = eng.decks[1].cycle_offset();
    if state.drag != DragTarget::Xf {
        state.xfade_pos = eng.mixer.crossfader_pos();
    }
    if state.drag != DragTarget::Flt {
        state.flt_pos = filters_to_bipolar(eng.mixer.lpf_hz, eng.mixer.hpf_hz);
    }
    if state.drag != DragTarget::Dly {
        state.delay_wet = eng.mixer.held_delay_wet();
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
    state.mix_label = eng.mixer.mix_status().map(|s| match &s.kind {
        Some(k) => format!("MIX {}/{}→{} {}", s.move_name, k, s.to, s.bars_left),
        None => format!("MIX {}→{} {}", s.move_name, s.to, s.bars_left),
    });
}

fn draw_frame(
    out: &mut std::io::Stdout,
    state: &mut LiveState,
    playhead: &AtomicU64,
    sample_rate: u32,
    hermes_enabled: bool,
) -> Result<(), String> {
    let (cols_u, rows_u) = terminal::size().unwrap_or((80, 24));
    let cols = cols_u as usize;
    let rows = rows_u as usize;
    if cols == 0 || rows == 0 {
        return Ok(());
    }

    // Footer: EQ×3 + (xfade) + FLT + DLY + log×3 (reserved) + help + prompt
    let single = state.session.single_deck();
    let xf_rows = if single { 0 } else { 1 };
    let footer_rows = (3 + xf_rows + 2 + LOG_LINES + 2).min(rows.saturating_sub(2));
    let body_rows = rows.saturating_sub(footer_rows);
    let gs = playhead.load(Ordering::Relaxed);

    let gutter = 1usize;
    let half = if single {
        cols
    } else {
        cols.saturating_sub(gutter) / 2
    };
    let right_w = if single {
        0
    } else {
        cols.saturating_sub(half + gutter)
    };

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
    let right_lines = if single {
        Vec::new()
    } else {
        pane_lines(PaneArgs {
            mode: state.body_mode,
            model: state.model_b.as_ref(),
            viz: state.viz_b.as_ref(),
            deck_label: "B",
            gs,
            sample_rate,
            cycle_offset: state.cycle_offset_b,
            width: right_w,
            height: body_rows,
        })
    };

    let mut left_lines = left_lines;
    let mut right_lines = right_lines;
    for line in &mut left_lines {
        *line = pad_clip_ansi(line, half);
    }
    for line in &mut right_lines {
        *line = pad_clip_ansi(line, right_w);
    }

    if state.vfx_on {
        apply_vfx(
            state,
            gs,
            sample_rate,
            half,
            right_w,
            body_rows,
            &mut left_lines,
            &mut right_lines,
        );
    }

    let mut lines: Vec<String> = Vec::with_capacity(rows);

    if single {
        for i in 0..body_rows {
            let left = left_lines.get(i).map(String::as_str).unwrap_or("");
            lines.push(left.to_string());
        }
    } else {
        for i in 0..body_rows {
            let left = left_lines.get(i).map(String::as_str).unwrap_or("");
            let right = right_lines.get(i).map(String::as_str).unwrap_or("");
            lines.push(format!("{left}│{right}"));
        }
    }

    // EQ rows: Hi / Mid / Lo  (A and B side by side, or A only in play)
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
            !single,
        );
        state.eq_hits[band][0] = hit_a;
        state.eq_hits[band][1] = hit_b;
        lines.push(line);
    }

    // Crossfader (short track ≤ 10) — DJ only
    if single {
        state.xf_hit = SliderHit {
            row: 0,
            col0: 0,
            cols: 0,
        };
    } else {
        let xf_row = lines.len();
        if lines.len() < rows {
            let (xf_line, hit) = format_crossfader_line(
                state.xfade_pos,
                cols,
                xf_row as u16,
                state.mix_label.as_deref(),
            );
            state.xf_hit = hit;
            lines.push(xf_line);
        }
    }

    state.mute_hits = collect_mute_hits(state, half, right_w, body_rows);

    // Post-mix FLT (bipolar) + DLY wet
    if lines.len() < rows {
        let flt_row = lines.len();
        let (flt_line, hit) = format_knob_line(
            "FLT",
            state.flt_pos,
            cols,
            flt_row as u16,
            Some("HPF  bypass  LPF"),
        );
        state.flt_hit = hit;
        lines.push(flt_line);
    }
    if lines.len() < rows {
        let dly_row = lines.len();
        let (dly_line, hit) = format_knob_line("DLY", state.delay_wet, cols, dly_row as u16, None);
        state.dly_hit = hit;
        lines.push(dly_line);
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
        let help_row = lines.len() as u16;
        let (help, vfx_hit) = format_help_line(
            state.vfx_on,
            state.voice_phase,
            state.session,
            cols,
            help_row,
        );
        state.vfx_hit = vfx_hit;
        lines.push(help);
        let prompt = format!("» {}", state.input);
        lines.push(pad_clip_ansi(&prompt, cols));
    }
    while lines.len() < rows {
        lines.push(pad_clip_ansi("", cols));
    }
    lines.truncate(rows);

    // Help takes priority; otherwise show local-command suggest overlay.
    if state.help_open {
        overlay_help_modal(&mut lines, cols, rows, state.session);
    } else {
        let suggest = suggest_for_state(state, hermes_enabled);
        if suggest_is_open(state, &suggest) {
            // Keep index in range if the filtered list shrank.
            if !suggest.candidates.is_empty() {
                state.suggest_idx %= suggest.candidates.len();
            }
            overlay_suggest_modal(&mut lines, cols, rows, &suggest, state.suggest_idx);
        }
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
        let suggest = suggest_for_state(state, hermes_enabled);
        let overlay = state.help_open || suggest_is_open(state, &suggest);
        if overlay {
            // Hide cursor while a modal covers the prompt.
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

#[allow(clippy::too_many_arguments)]
fn apply_vfx(
    state: &mut LiveState,
    gs: u64,
    sample_rate: u32,
    half: usize,
    right_w: usize,
    body_rows: usize,
    left: &mut [String],
    right: &mut [String],
) {
    let now = Instant::now();
    let dt = state
        .last_frame
        .map(|t| now.saturating_duration_since(t).as_secs_f32())
        .unwrap_or(0.033)
        .clamp(0.001, 0.10);
    state.last_frame = Some(now);

    let (hits_a, spc_a) = gather_fx_hits(state, 0, gs, sample_rate, half, body_rows);
    state.fx.observe(0, &hits_a, spc_a);
    if right_w > 0 {
        let (hits_b, spc_b) = gather_fx_hits(state, 1, gs, sample_rate, right_w, body_rows);
        state.fx.observe(1, &hits_b, spc_b);
    }
    state.fx.advance(dt);
    if state.fx.has_visuals() {
        state.fx.composite_lines(0, left, half);
        if right_w > 0 {
            state.fx.composite_lines(1, right, right_w);
        }
    }
}

fn gather_fx_hits(
    state: &LiveState,
    deck: usize,
    gs: u64,
    sample_rate: u32,
    width: usize,
    height: usize,
) -> (Vec<live_fx::FxHit>, f32) {
    let (viz, model, offset) = if deck == 0 {
        (
            state.viz_a.as_ref(),
            state.model_a.as_ref(),
            state.cycle_offset_a,
        )
    } else {
        (
            state.viz_b.as_ref(),
            state.model_b.as_ref(),
            state.cycle_offset_b,
        )
    };
    let bpm = viz
        .map(|v| v.bpm)
        .or_else(|| model.map(|m| m.bpm))
        .unwrap_or(120.0);
    let secs_per_cycle = (240.0 / bpm.max(1.0)) as f32;
    let hits = match state.body_mode {
        BodyMode::Highlight => gather_highlight_hits(model, gs, sample_rate, offset),
        BodyMode::Viz => gather_punchcard_hits(viz, gs, sample_rate, offset, width, height),
    };
    (hits, secs_per_cycle)
}

fn gather_highlight_hits(
    model: Option<&HighlightModel>,
    gs: u64,
    sample_rate: u32,
    offset: i64,
) -> Vec<live_fx::FxHit> {
    let Some(model) = model else {
        return Vec::new();
    };
    let sr = model.sample_rate.max(sample_rate);
    let pattern_bar = (bar_index(gs, sr, model.bpm) as i64 + offset).max(0) as u64;
    let pos = bar_pos(gs, sr, model.bpm);
    let atoms = active_atoms(model, pattern_bar, pos);
    atoms
        .into_iter()
        .map(|a| {
            let midi = if a.is_note {
                note_to_midi(&a.value).ok()
            } else {
                None
            };
            let (x, y, atom_cols) = if let Some(sp) = a.span {
                let (x, y) = live_fx::source_byte_xy(&model.source, sp.start, 2);
                let cols = model
                    .source
                    .get(sp.start..sp.end)
                    .map(|s| s.chars().count() as u16)
                    .unwrap_or(a.value.chars().count() as u16);
                (x, y, cols.max(1))
            } else {
                (8.0, 3.0, a.value.chars().count() as u16)
            };
            live_fx::FxHit {
                track_idx: a.track_idx,
                label: a.value,
                midi,
                is_note: a.is_note,
                start_cycle: a.start_cycle,
                x,
                y,
                atom_cols,
            }
        })
        .collect()
}

fn gather_punchcard_hits(
    viz: Option<&VizModel>,
    gs: u64,
    sample_rate: u32,
    offset: i64,
    width: usize,
    height: usize,
) -> Vec<live_fx::FxHit> {
    let Some(viz) = viz else {
        return Vec::new();
    };
    let sr = viz.sample_rate.max(sample_rate);
    let cycle = viz::play_cycle(gs, sr, viz.bpm) + offset as f64;
    viz::hits_sounding_at(viz, cycle)
        .into_iter()
        .map(|h| {
            let (x, y) = viz::hit_origin(viz, &h, gs, offset, sample_rate, width, height);
            live_fx::FxHit {
                track_idx: h.track_idx,
                label: h.label,
                midi: h.midi,
                is_note: h.midi.is_some(),
                start_cycle: h.start_cycle,
                x,
                y,
                atom_cols: 1,
            }
        })
        .collect()
}

fn format_help_line(
    vfx_on: bool,
    voice: VoicePhase,
    session: SessionKind,
    cols: usize,
    row: u16,
) -> (String, SliderHit) {
    let left = match voice {
        VoicePhase::Listening => HELP_LINE_LISTEN,
        VoicePhase::Recording => HELP_LINE_REC,
        VoicePhase::Stt => HELP_LINE_STT,
        VoicePhase::Idle if session.single_deck() => HELP_LINE_PLAY,
        VoicePhase::Idle => HELP_LINE,
    };
    let btn = if vfx_on { "[VFX:ON]" } else { "[VFX:OFF]" };
    let btn_len = btn.chars().count();
    let left_len = left.chars().count();
    let (raw, col0) = if cols > left_len + btn_len + 1 {
        let pad = cols - left_len - btn_len;
        (
            format!("{left}{}{btn}", " ".repeat(pad)),
            (cols - btn_len) as u16,
        )
    } else {
        (format!("{left} {btn}"), (left_len + 1) as u16)
    };
    let line = pad_clip_ansi(&raw, cols);
    (
        line,
        SliderHit {
            row,
            col0,
            cols: btn_len as u16,
        },
    )
}

fn track_names(viz: &Option<VizModel>) -> Vec<String> {
    viz.as_ref()
        .map(|v| v.tracks.iter().map(|t| t.name.clone()).collect())
        .unwrap_or_default()
}

fn suggest_for_state(state: &LiveState, hermes_enabled: bool) -> CompleteResult {
    let tracks_a = track_names(&state.viz_a);
    let tracks_b = track_names(&state.viz_b);
    let ctx = CompleteCtx {
        hermes_enabled,
        tracks_a: &tracks_a,
        tracks_b: &tracks_b,
        session: state.session,
    };
    complete::suggest(&state.input, &ctx)
}

fn suggest_is_open(state: &LiveState, result: &CompleteResult) -> bool {
    !state.help_open && !state.suggest_dismissed && result.has_display()
}

/// Reset selection when the user edits the prompt (also re-enables overlay after Esc).
fn reset_suggest_edit(state: &mut LiveState) {
    state.suggest_idx = 0;
    state.suggest_dismissed = false;
    state.invalidate_frame();
}

/// Insert the highlighted candidate into the prompt (trailing space via complete).
fn apply_suggest_selection(state: &mut LiveState, result: &CompleteResult) {
    if result.candidates.is_empty() {
        return;
    }
    let idx = state.suggest_idx % result.candidates.len();
    if let Some(next) = complete::apply_candidate(&state.input, result, idx) {
        state.input = next;
        state.suggest_idx = 0;
        state.suggest_dismissed = false;
        state.history_idx = None;
        state.invalidate_frame();
    }
}

/// Build visible body lines for the suggest list (selection + scroll window).
fn suggest_body_lines(result: &CompleteResult, selected: usize, max_rows: usize) -> Vec<String> {
    if result.candidates.is_empty() {
        if let Some(ref h) = result.hint {
            return vec![h.clone()];
        }
        return Vec::new();
    }
    let n = result.candidates.len();
    let sel = selected % n;
    let max_rows = max_rows.max(1);
    let start = if n <= max_rows {
        0
    } else {
        (sel + 1).saturating_sub(max_rows)
    };
    let end = (start + max_rows).min(n);
    let mut out = Vec::with_capacity(end - start + 1);
    for i in start..end {
        let mark = if i == sel { "▸ " } else { "  " };
        out.push(format!("{mark}{}", result.candidates[i]));
    }
    if start > 0 || end < n {
        out.push(format!("  … {}/{} …", sel + 1, n));
    }
    out
}

/// Centered help window overlaid on the current frame (does not use the log area).
fn overlay_help_modal(lines: &mut [String], cols: usize, rows: usize, session: SessionKind) {
    let body: Vec<String> = cmd::help_text(session)
        .trim_end()
        .lines()
        .map(|s| s.to_string())
        .collect();
    paint_overlay_box(
        lines,
        cols,
        rows,
        " help ",
        &body,
        "Esc / Enter / Space で閉じる",
    );
}

/// Local-command candidate list (same chrome as help).
fn overlay_suggest_modal(
    lines: &mut [String],
    cols: usize,
    rows: usize,
    result: &CompleteResult,
    selected: usize,
) {
    let body = suggest_body_lines(result, selected, SUGGEST_MAX_ROWS);
    if body.is_empty() {
        return;
    }
    let footer = if result.candidates.is_empty() {
        "Esc で閉じる"
    } else {
        "↑↓ 選択  Tab 適用  Enter 適用+実行  Esc 閉じる"
    };
    paint_overlay_box(lines, cols, rows, " suggest ", &body, footer);
}

/// Shared floating panel used by help and suggest.
fn paint_overlay_box(
    lines: &mut [String],
    cols: usize,
    rows: usize,
    title: &str,
    body: &[String],
    footer: &str,
) {
    if cols < 12 || rows < 6 {
        return;
    }
    if body.is_empty() {
        return;
    }

    let content_w = body
        .iter()
        .map(|l| visible_width(l))
        .chain(std::iter::once(visible_width(footer)))
        .chain(std::iter::once(visible_width(title) + 2))
        .max()
        .unwrap_or(24);
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
    let top = title_border(&hline, title, box_w);

    let mut box_lines: Vec<String> = Vec::with_capacity(box_h);
    box_lines.push(top);
    for line in body.iter().take(body_show) {
        box_lines.push(box_content_row(line, inner));
    }
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
        let left = " ".repeat(col0);
        let mid = pad_clip_ansi(bline, box_w);
        let right_pad = cols.saturating_sub(col0 + box_w);
        let right = " ".repeat(right_pad);
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

/// One EQ row: A track left, B track **right-aligned** (DJ).
/// Play (`show_b == false`): A track only.
/// e.g. `Hi  A ──□──                              B ──□──`
fn format_eq_band_line(
    band: &str,
    pos_a: f32,
    pos_b: f32,
    cols: usize,
    row: u16,
    show_b: bool,
) -> (String, SliderHit, SliderHit) {
    let track_w = MAX_SLIDER_TRACK.min(10);
    let band_pad = format!("{band:<3}");
    let left_lab = format!("{band_pad} A ");
    if !show_b {
        let track_a = format_track(pos_a, track_w);
        let raw = format!("{left_lab}{track_a}");
        let line = pad_clip_ansi(&raw, cols);
        let hit_a = SliderHit {
            row,
            col0: left_lab.chars().count() as u16,
            cols: track_w as u16,
        };
        let hit_b = SliderHit {
            row,
            col0: 0,
            cols: 0,
        };
        return (line, hit_a, hit_b);
    }
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
fn format_crossfader_line(
    pos: f32,
    cols: usize,
    row: u16,
    mix_label: Option<&str>,
) -> (String, SliderHit) {
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
    let mut raw = format!("{}{content}", " ".repeat(pad_left));
    if let Some(s) = mix_label.filter(|s| !s.is_empty()) {
        raw.push(' ');
        raw.push_str("\x1b[2m");
        raw.push_str(s);
        raw.push_str("\x1b[0m");
    }
    let line = pad_clip_ansi(&raw, cols);
    let hit = SliderHit {
        row,
        col0: (pad_left + prefix.chars().count()) as u16,
        cols: track_w as u16,
    };
    (line, hit)
}

/// Short knob like the crossfader. `hint` is dim text after the track.
fn format_knob_line(
    name: &str,
    pos: f32,
    cols: usize,
    row: u16,
    hint: Option<&str>,
) -> (String, SliderHit) {
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
    let prefix = format!("{name} ");
    let pct = format!(" {:3.0}%", pos * 100.0);
    let fixed = prefix.chars().count() + pct.chars().count();
    let track_w = cols.saturating_sub(fixed).clamp(3, MAX_SLIDER_TRACK);
    let track = format_track(pos, track_w);
    let content = format!("{prefix}{track}{pct}");
    let content_vis = prefix.chars().count() + track_w + pct.chars().count();
    let pad_left = cols.saturating_sub(content_vis) / 2;
    let mut raw = format!("{}{content}", " ".repeat(pad_left));
    if let Some(s) = hint.filter(|s| !s.is_empty()) {
        raw.push(' ');
        raw.push_str("\x1b[2m");
        raw.push_str(s);
        raw.push_str("\x1b[0m");
    }
    let line = pad_clip_ansi(&raw, cols);
    let hit = SliderHit {
        row,
        col0: (pad_left + prefix.chars().count()) as u16,
        cols: track_w as u16,
    };
    (line, hit)
}

/// Bipolar master filter: 0.5 = bypass, left = HPF, right = LPF.
fn bipolar_to_filters(pos: f32) -> (Option<f32>, Option<f32>) {
    let p = pos.clamp(0.0, 1.0);
    if (p - 0.5).abs() < 0.04 {
        return (None, None);
    }
    if p < 0.5 {
        let t = (0.5 - p) / 0.5;
        let hz = 40.0 * (2000.0_f32 / 40.0).powf(t);
        (None, Some(hz))
    } else {
        let t = (p - 0.5) / 0.5;
        let hz = 12_000.0 * (200.0_f32 / 12_000.0).powf(t);
        (Some(hz), None)
    }
}

fn filters_to_bipolar(lpf: Option<f32>, hpf: Option<f32>) -> f32 {
    match (lpf, hpf) {
        (None, None) => 0.5,
        (None, Some(hz)) => {
            let span = (2000.0_f32 / 40.0).ln();
            let t = ((hz / 40.0).ln() / span).clamp(0.0, 1.0);
            0.5 - 0.5 * t
        }
        (Some(hz), _) => {
            let span = (200.0_f32 / 12_000.0).ln();
            let t = ((hz / 12_000.0).ln() / span).clamp(0.0, 1.0);
            0.5 + 0.5 * t
        }
    }
}

fn collect_mute_hits(
    state: &LiveState,
    half: usize,
    right_w: usize,
    body_rows: usize,
) -> Vec<MuteHit> {
    let mut out = Vec::new();
    collect_pane_mute_hits(
        &mut out,
        0,
        0,
        half,
        body_rows,
        state.body_mode,
        state.model_a.as_ref(),
        state.viz_a.as_ref(),
    );
    if !state.session.single_deck() && right_w > 0 {
        collect_pane_mute_hits(
            &mut out,
            1,
            (half + 1) as u16,
            right_w,
            body_rows,
            state.body_mode,
            state.model_b.as_ref(),
            state.viz_b.as_ref(),
        );
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn collect_pane_mute_hits(
    out: &mut Vec<MuteHit>,
    deck: usize,
    pane_x: u16,
    pane_w: usize,
    body_rows: usize,
    mode: BodyMode,
    model: Option<&HighlightModel>,
    viz: Option<&VizModel>,
) {
    match mode {
        BodyMode::Viz => {
            let Some(viz) = viz else { return };
            for (row, name) in viz::sample_lane_mute_rows(viz, body_rows) {
                if row >= body_rows {
                    continue;
                }
                out.push(MuteHit {
                    hit: SliderHit {
                        row: row as u16,
                        col0: pane_x,
                        cols: 6.min(pane_w as u16).max(1),
                    },
                    deck,
                    track: name,
                });
            }
        }
        BodyMode::Highlight => {
            let Some(model) = model else { return };
            for (i, line) in model.source.lines().enumerate() {
                let row = 2 + i;
                if row >= body_rows {
                    break;
                }
                let trimmed = line.trim();
                for t in &model.tracks {
                    let tag = format!("// {}", t.name);
                    if trimmed == tag {
                        let cols = t.name.chars().count().clamp(1, pane_w) as u16;
                        out.push(MuteHit {
                            hit: SliderHit {
                                row: row as u16,
                                col0: pane_x,
                                cols,
                            },
                            deck,
                            track: t.name.clone(),
                        });
                        break;
                    }
                }
            }
        }
    }
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
                    "[{}] (empty)\n────────────────────────────────────────\n/{} house 01\n",
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

#[allow(clippy::too_many_arguments)]
fn dispatch_line(
    line: &str,
    tx: &Sender<Command>,
    deck_paths: &DeckPaths,
    engine: &Arc<Mutex<Engine>>,
    sample_rate: u32,
    state: &mut LiveState,
    hermes: Option<&HermesHandle>,
    automix: Option<&AutomixHandle>,
) -> bool {
    let hermes_enabled = hermes.is_some();
    match cmd::classify_live_input(line, hermes_enabled) {
        LiveInput::Empty => true,
        LiveInput::LocalCommand(body) => {
            if let Some(a) = automix {
                a.touch();
            }
            if cmd::is_help_body(&body) {
                state.help_open = true;
                state.invalidate_frame();
                return true;
            }
            exec_local(&body, tx, deck_paths, engine, sample_rate, state, automix)
        }
        LiveInput::HermesPrompt(prompt) => {
            if let Some(a) = automix {
                a.touch();
            }
            let Some(h) = hermes else {
                // Should not happen: classify only returns Hermes when enabled.
                return exec_local(&prompt, tx, deck_paths, engine, sample_rate, state, automix);
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
    automix: Option<&AutomixHandle>,
) -> bool {
    if let Some(handle) = automix {
        if body.split_whitespace().next() == Some("automix") {
            if let Some(msg) = apply_automix_command(body, handle) {
                state.push_log(msg);
            }
            return true;
        }
    }
    // UI-only: VFX overlay (issue #42) — not sent to engine.
    if let Some(msg) = apply_vfx_command(body, state) {
        state.push_log(msg);
        return true;
    }
    // UI-only: body highlight ⇔ punchcard (not sent to engine).
    if let Some(msg) = apply_viz_command(body, state) {
        state.push_log(msg);
        return true;
    }
    let result = cmd::exec_in(body, tx, deck_paths, Some(engine), state.session);
    if let Some((deck, song)) = result.loaded {
        state.set_from_song(deck, &song, sample_rate);
    }
    for m in result.messages {
        state.push_log(m);
    }
    !result.quit
}

/// Handle `automix` / `automix on|off|status`. Live DJ TUI only.
fn apply_automix_command(body: &str, handle: &AutomixHandle) -> Option<String> {
    let mut parts = body.split_whitespace();
    let head = parts.next()?;
    if head != "automix" {
        return None;
    }
    match parts.next() {
        None | Some("status") => {
            let m = match handle.mode() {
                AutomixMode::IdleWait => "idle-wait",
                AutomixMode::Armed => "armed",
                AutomixMode::Disabled => "off",
            };
            Some(format!("automix: {m}"))
        }
        Some("on") => {
            handle.force_on();
            None
        }
        Some("off") => {
            handle.force_off();
            None
        }
        Some(other) => Some(format!("automix: unknown arg `{other}` (on|off)")),
    }
}

/// Handle `vfx` / `dopa` / `flash` (on|off|toggle). Live TUI only.
fn apply_vfx_command(body: &str, state: &mut LiveState) -> Option<String> {
    let mut parts = body.split_whitespace();
    let head = parts.next()?;
    if !matches!(head, "vfx" | "dopa" | "flash") {
        return None;
    }
    let msg = match parts.next() {
        None | Some("toggle") => state.toggle_vfx(),
        Some("on") | Some("1") => state.set_vfx(true),
        Some("off") | Some("0") => state.set_vfx(false),
        Some(other) => {
            return Some(format!("vfx: unknown arg `{other}` (on|off|toggle)"));
        }
    };
    Some(msg.to_string())
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

fn drain_automix_events(state: &mut LiveState, automix: &AutomixHandle) {
    for ev in automix.drain_events() {
        match ev {
            AutomixEvent::Paused => state.push_log("automix: off"),
            AutomixEvent::Resumed => state.push_log("Hermes cronは起動中です"),
            AutomixEvent::Failed(msg) => state.push_log(format!("automix: fail {msg}")),
            AutomixEvent::GatewayDown => state.push_log(GATEWAY_DOWN_MSG),
        }
    }
}

fn drain_voice_events(
    state: &mut LiveState,
    voice: &VoiceHandle,
    hermes: Option<&HermesHandle>,
    automix: Option<&AutomixHandle>,
) {
    for ev in voice.drain_events() {
        match ev {
            VoiceEvent::ListeningStarted => {
                state.voice_phase = VoicePhase::Listening;
                state.invalidate_frame();
                state.push_log("voice: VAD 待ち（F12 で一時停止）");
            }
            VoiceEvent::ListeningPaused => {
                state.voice_phase = VoicePhase::Idle;
                state.invalidate_frame();
                state.push_log("voice: VAD 一時停止（F12 で再開）");
            }
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
                if state.voice_phase != VoicePhase::Listening {
                    state.voice_phase = VoicePhase::Idle;
                }
                state.invalidate_frame();
                state.push_log(format!("voice: 「{text}」"));
                if let Some(h) = hermes {
                    match h.enqueue(&text) {
                        Ok(()) => {
                            if let Some(a) = automix {
                                a.touch();
                            }
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
        let (line, hit) = format_crossfader_line(0.5, 120, 5, None);
        assert!(hit.cols <= MAX_SLIDER_TRACK as u16, "cols={}", hit.cols);
        assert!(line.contains("\x1b[47m"));
        assert!(line.contains("XF"));
    }

    #[test]
    fn crossfader_line_shows_mix_label() {
        let (line, hit) = format_crossfader_line(0.5, 120, 5, Some("MIX fill/echo→B 1"));
        assert!(line.contains("echo"), "{line}");
        assert!(line.contains("XF"), "{line}");
        assert!(hit.cols <= MAX_SLIDER_TRACK as u16, "cols={}", hit.cols);
    }

    #[test]
    fn eq_band_line_has_a_and_b_tracks() {
        let cols = 80usize;
        let (line, ha, hb) = format_eq_band_line("Hi", 0.5, 0.5, cols, 3, true);
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
    fn bipolar_center_is_bypass() {
        let (lpf, hpf) = bipolar_to_filters(0.5);
        assert!(lpf.is_none() && hpf.is_none());
        assert!((filters_to_bipolar(None, None) - 0.5).abs() < 1e-5);
        let (lpf, hpf) = bipolar_to_filters(0.0);
        assert!(hpf.is_some() && lpf.is_none());
        let (lpf, hpf) = bipolar_to_filters(1.0);
        assert!(lpf.is_some() && hpf.is_none());
    }

    #[test]
    fn format_knob_has_hit() {
        let (line, hit) = format_knob_line("FLT", 0.5, 80, 10, Some("bypass"));
        assert!(line.contains("FLT"), "{line}");
        assert!(hit.cols > 0);
        assert_eq!(hit.row, 10);
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
            mix_label: None,
            eq: [[1.0; 2]; 3],
            flt_pos: 0.5,
            delay_wet: 0.0,
            input: String::new(),
            history: Vec::new(),
            history_idx: None,
            suggest_idx: 0,
            suggest_dismissed: false,
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
            flt_hit: SliderHit {
                row: 0,
                col0: 0,
                cols: 0,
            },
            dly_hit: SliderHit {
                row: 0,
                col0: 0,
                cols: 0,
            },
            mute_hits: Vec::new(),
            drag: DragTarget::None,
            vfx_on: true,
            fx: FxState::new(),
            vfx_hit: SliderHit {
                row: 0,
                col0: 0,
                cols: 0,
            },
            last_frame: None,
            session: SessionKind::Dj,
        }
    }

    #[test]
    fn eq_band_line_play_hides_b() {
        let cols = 80usize;
        let (line, ha, hb) = format_eq_band_line("Hi", 0.5, 0.5, cols, 3, false);
        assert!(line.contains("Hi"), "{line}");
        assert!(line.contains("A "), "{line}");
        assert!(!line.contains("B "), "{line}");
        assert_eq!(ha.cols, MAX_SLIDER_TRACK as u16);
        assert_eq!(hb.cols, 0);
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
    fn vfx_command_toggles_overlay() {
        let mut state = empty_state();
        assert!(state.vfx_on);
        let msg = apply_vfx_command("vfx", &mut state).unwrap();
        assert!(msg.contains("off"), "{msg}");
        assert!(!state.vfx_on);
        let msg = apply_vfx_command("dopa on", &mut state).unwrap();
        assert!(msg.contains("on"), "{msg}");
        assert!(state.vfx_on);
        assert!(apply_vfx_command("flash off", &mut state)
            .unwrap()
            .contains("off"));
        assert!(apply_vfx_command("bpm 120", &mut state).is_none());
    }

    #[test]
    fn help_line_has_clickable_vfx_button() {
        let (line, hit) = format_help_line(true, VoicePhase::Idle, SessionKind::Dj, 80, 20);
        assert!(line.contains("[VFX:ON]"), "{line}");
        assert_eq!(hit.row, 20);
        assert!(hit.cols >= 8);
        let (line_off, _) = format_help_line(false, VoicePhase::Idle, SessionKind::Dj, 80, 20);
        assert!(line_off.contains("[VFX:OFF]"), "{line_off}");
    }

    #[test]
    fn help_modal_draws_box_with_commands() {
        let cols = 80usize;
        let rows = 24usize;
        let mut lines: Vec<String> = (0..rows).map(|_| pad_clip_ansi("", cols)).collect();
        overlay_help_modal(&mut lines, cols, rows, SessionKind::Dj);
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
    fn suggest_body_marks_selection_and_scrolls() {
        let result = CompleteResult {
            candidates: (0..15).map(|i| format!("song{i}")).collect(),
            replace_from: 0,
            hint: None,
        };
        let lines = suggest_body_lines(&result, 0, 5);
        assert!(lines[0].starts_with("▸ "), "{lines:?}");
        assert!(lines[1].starts_with("  "), "{lines:?}");
        assert!(lines.iter().any(|l| l.contains('…')), "{lines:?}");

        let lines = suggest_body_lines(&result, 12, 5);
        assert!(lines.iter().any(|l| l.contains("▸ song12")), "{lines:?}");
    }

    #[test]
    fn suggest_modal_draws_candidates() {
        let cols = 80usize;
        let rows = 24usize;
        let mut lines: Vec<String> = (0..rows).map(|_| pad_clip_ansi("", cols)).collect();
        let result = CompleteResult {
            candidates: vec!["house-01".into(), "four-on-the-floor-01".into()],
            replace_from: 8,
            hint: None,
        };
        overlay_suggest_modal(&mut lines, cols, rows, &result, 1);
        let joined = lines.join("\n");
        assert!(joined.contains("suggest"), "{joined}");
        assert!(joined.contains("house-01"), "{joined}");
        assert!(joined.contains("four-on-the-floor-01"), "{joined}");
        assert!(joined.contains("↑↓") || joined.contains("選択"), "{joined}");
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
