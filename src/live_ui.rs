//! Combined highlight TUI + command line for `play --repl` (demo-oriented).

use std::collections::VecDeque;
use std::io::{stdout, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossbeam::channel::Sender;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, execute, queue, terminal};

use crate::cmd;
use crate::engine::{Command, Engine};
use crate::highlight::{active_spans, bar_index, bar_pos, render_ansi_ex, HighlightModel};
use crate::watcher::DeckPaths;

const HELP_LINE: &str = "a load <file>  b mute kick  x 4  a x 4  bpm 128  hush  status  help  quit";

struct LiveState {
    model_a: Option<HighlightModel>,
    model_b: Option<HighlightModel>,
    input: String,
    history: Vec<String>,
    history_idx: Option<usize>,
    log: VecDeque<String>,
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
        while self.log.len() > 4 {
            self.log.pop_front();
        }
    }
}

/// Run alternate-screen UI: highlight (top) + command line (bottom).
/// Returns when the user quits.
pub fn run(
    tx: Sender<Command>,
    deck_paths: DeckPaths,
    engine: Arc<Mutex<Engine>>,
    playhead: Arc<AtomicU64>,
    sample_rate: u32,
    initial: Option<(usize, HighlightModel)>,
) -> Result<(), String> {
    let mut state = LiveState {
        model_a: None,
        model_b: None,
        input: String::new(),
        history: Vec::new(),
        history_idx: None,
        log: VecDeque::new(),
    };
    if let Some((deck, model)) = initial {
        state.set_model(deck, model);
    }
    state.push_log("live UI · A left / B right  ·  type help");

    enable_raw_mode().map_err(|e| format!("raw mode: {e}"))?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, cursor::Hide)
        .map_err(|e| format!("enter alternate screen: {e}"))?;

    let frame = Duration::from_millis(33);
    let result = (|| -> Result<(), String> {
        loop {
            // Input (non-blocking drain)
            while event::poll(Duration::from_millis(0)).unwrap_or(false) {
                if let Ok(Event::Key(key)) = event::read() {
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
                        KeyCode::Char(c) => {
                            if !key.modifiers.contains(KeyModifiers::CONTROL)
                                && !key.modifiers.contains(KeyModifiers::ALT)
                            {
                                state.input.push(c);
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Sync mute flags / bpm from engine if possible
            if let Ok(eng) = engine.try_lock() {
                sync_models_from_engine(&mut state, &eng, sample_rate);
            }

            draw_frame(&mut out, &state, &playhead, sample_rate)?;
            std::thread::sleep(frame);
        }
    })();

    let _ = execute!(out, cursor::Show, LeaveAlternateScreen);
    let _ = disable_raw_mode();
    result
}

fn sync_models_from_engine(state: &mut LiveState, eng: &Engine, sample_rate: u32) {
    // Refresh muted flags and bpm from live engine songs without replacing source
    // unless titles still match (reload replaces source via :load / watcher path).
    for (deck, model_slot) in [(0, &mut state.model_a), (1, &mut state.model_b)] {
        let Some(model) = model_slot.as_mut() else {
            continue;
        };
        model.bpm = eng.transport.bpm;
        model.sample_rate = sample_rate;
        if let Some(song) = eng.decks[deck].song_title() {
            if song != model.title.as_str() {
                // Engine has a different song (e.g. watcher reload). Rebuild model.
                if let Some(s) = eng.decks[deck].song_ref() {
                    *model = HighlightModel::from_song(s, sample_rate);
                }
            } else if let Some(s) = eng.decks[deck].song_ref() {
                // Same title: still refresh mute flags / patterns if source length matches.
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
    state: &LiveState,
    playhead: &AtomicU64,
    sample_rate: u32,
) -> Result<(), String> {
    let (cols, rows) = terminal::size().unwrap_or((80, 24));
    let cols = cols as usize;
    let rows = rows as usize;
    // Reserve bottom for log + help + prompt
    let footer_rows = 6.min(rows.saturating_sub(3));
    let body_rows = rows.saturating_sub(footer_rows);
    let gs = playhead.load(Ordering::Relaxed);

    // Left = deck A, right = deck B (gutter `|` between panes).
    let gutter = 1usize;
    let half = cols.saturating_sub(gutter) / 2;
    let right_w = cols.saturating_sub(half + gutter);

    let left_lines = pane_lines(
        state.model_a.as_ref(),
        "A",
        gs,
        sample_rate,
        half,
        body_rows,
    );
    let right_lines = pane_lines(
        state.model_b.as_ref(),
        "B",
        gs,
        sample_rate,
        right_w,
        body_rows,
    );

    queue!(out, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All))
        .map_err(|e| format!("draw: {e}"))?;

    for i in 0..body_rows {
        let left = left_lines.get(i).map(String::as_str).unwrap_or("");
        let right = right_lines.get(i).map(String::as_str).unwrap_or("");
        let left_cell = pad_clip_ansi(left, half);
        let right_cell = pad_clip_ansi(right, right_w);
        queue!(out, cursor::MoveTo(0, i as u16)).map_err(|e| format!("draw: {e}"))?;
        write!(out, "{left_cell}│{right_cell}").map_err(|e| format!("draw: {e}"))?;
    }

    let mut row = body_rows as u16;
    if row < rows as u16 {
        queue!(out, cursor::MoveTo(0, row)).map_err(|e| format!("draw: {e}"))?;
        write!(out, "{}", "─".repeat(cols.min(200))).map_err(|e| format!("draw: {e}"))?;
        row += 1;
    }
    for msg in state.log.iter() {
        if row as usize >= rows.saturating_sub(2) {
            break;
        }
        queue!(out, cursor::MoveTo(0, row)).map_err(|e| format!("draw: {e}"))?;
        write!(out, "{}", pad_clip_ansi(msg, cols)).map_err(|e| format!("draw: {e}"))?;
        row += 1;
    }
    if rows >= 2 {
        queue!(out, cursor::MoveTo(0, rows as u16 - 2)).map_err(|e| format!("draw: {e}"))?;
        write!(out, "{}", pad_clip_ansi(HELP_LINE, cols)).map_err(|e| format!("draw: {e}"))?;
    }
    queue!(out, cursor::MoveTo(0, rows as u16 - 1)).map_err(|e| format!("draw: {e}"))?;
    let prompt = format!("» {}", state.input);
    write!(out, "{}", pad_clip_ansi(&prompt, cols)).map_err(|e| format!("draw: {e}"))?;
    let cursor_col = (2 + state.input.chars().count()).min(cols.saturating_sub(1)) as u16;
    queue!(
        out,
        cursor::MoveTo(cursor_col, rows as u16 - 1),
        cursor::Show
    )
    .map_err(|e| format!("draw: {e}"))?;
    out.flush().map_err(|e| format!("draw: {e}"))?;
    let _ = queue!(out, cursor::Hide);
    Ok(())
}

/// Build highlight (or empty placeholder) lines for one half-pane.
fn pane_lines(
    model: Option<&HighlightModel>,
    deck_label: &str,
    gs: u64,
    sample_rate: u32,
    width: usize,
    height: usize,
) -> Vec<String> {
    let text = if let Some(model) = model {
        let sr = model.sample_rate.max(sample_rate);
        let bar = bar_index(gs, sr, model.bpm);
        let pos = bar_pos(gs, sr, model.bpm);
        let spans = active_spans(model, bar, pos);
        let header = format!(
            "[{}] {}  ·  {:.0} BPM  ·  bar {}  ·  pos {:.2}",
            deck_label, model.title, model.bpm, bar, pos
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
    // Soft-wrap is not applied; long lines are clipped by pad_clip_ansi at draw time.
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
            // CSI / SGR sequence: copy through final byte (0x40–0x7E, typically 'm')
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
            // Still need to close reverse if we truncate mid-style? Emit reset.
            break;
        }
        out.push(c);
        visible += 1;
    }
    if visible < width {
        out.push_str(&" ".repeat(width - visible));
    }
    // Ensure styles don't bleed into the other pane.
    out.push_str("\x1b[0m");
    out
}

/// Returns false when UI should exit.
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
        // help is multi-line — keep a few lines in the log pane
        let lines: Vec<&str> = m.lines().collect();
        if lines.len() > 1 {
            for part in lines.iter().take(3) {
                state.push_log(*part);
            }
            if lines.len() > 3 {
                state.push_log("… (type help / see README)");
            }
        } else {
            state.push_log(m);
        }
    }
    !result.quit
}
