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

/// One display row after JS-style soft wrap.
#[derive(Debug, Clone)]
pub struct LayoutLine {
    pub text: String,
    /// Original source byte for each display column, or `None` for indent spaces.
    pub byte_of_col: Vec<Option<usize>>,
}

/// Display rows for a song source (no header / footer).
#[derive(Debug, Clone)]
pub struct SourceLayout {
    pub lines: Vec<LayoutLine>,
}

/// Soft-wrap Strudel/JS source to `width` columns.
///
/// Break at member-access `.` outside strings/comments (not decimals).
/// Continuation lines that start at that `.` get two leading spaces.
/// Segments that still overflow are hard-wrapped at `width` with no extra indent.
pub fn layout_js_source(source: &str, width: usize) -> SourceLayout {
    let width = width.max(1);
    let mut scan = JsScan::new();
    let mut lines = Vec::new();
    let bytes = source.as_bytes();
    let mut byte = 0usize;
    while byte < source.len() {
        let line_start = byte;
        let mut line_end = byte;
        while line_end < source.len() && bytes[line_end] != b'\n' {
            line_end += 1;
        }
        let content_end = if line_end > line_start && bytes[line_end - 1] == b'\r' {
            line_end - 1
        } else {
            line_end
        };
        let line = &source[line_start..content_end];
        lines.extend(wrap_source_line(line, line_start, width, &mut scan));
        scan.at_newline();
        if line_end < source.len() && bytes[line_end] == b'\n' {
            byte = line_end + 1;
        } else {
            break;
        }
    }
    if source.is_empty() {
        lines.push(LayoutLine {
            text: String::new(),
            byte_of_col: Vec::new(),
        });
    }
    SourceLayout { lines }
}

/// Map an original source byte to pane cells after `header_rows` banner lines.
pub fn layout_byte_xy(layout: &SourceLayout, byte: usize, header_rows: usize) -> (f32, f32) {
    let mut best = (0.0f32, header_rows as f32);
    for (row, line) in layout.lines.iter().enumerate() {
        let y = header_rows as f32 + row as f32;
        for (col, b) in line.byte_of_col.iter().enumerate() {
            let Some(ob) = *b else {
                continue;
            };
            if ob == byte {
                return (col as f32, y);
            }
            if ob < byte {
                best = (col as f32, y);
            } else {
                return (col as f32, y);
            }
        }
    }
    best
}

/// Display columns occupied by `[start, end)` on the layout row that contains `start`.
pub fn layout_span_cols(
    layout: &SourceLayout,
    start: usize,
    end: usize,
    header_rows: usize,
) -> u16 {
    let (x, y) = layout_byte_xy(layout, start, header_rows);
    let row = (y as usize).saturating_sub(header_rows);
    let Some(line) = layout.lines.get(row) else {
        return 1;
    };
    let x = x as usize;
    let mut cols = 0u16;
    for (col, b) in line.byte_of_col.iter().enumerate() {
        if col < x {
            continue;
        }
        match *b {
            Some(ob) if ob >= start && ob < end => cols = cols.saturating_add(1),
            Some(ob) if ob >= end => break,
            _ => {}
        }
    }
    cols.max(1)
}

/// Like [`render_ansi_ex`], wrapping the source body to `wrap_width`.
pub fn render_ansi_wrapped(
    model: &HighlightModel,
    active: &[Span],
    header: &str,
    with_quit_footer: bool,
    wrap_width: usize,
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
    let layout = layout_js_source(src, wrap_width);

    let mut out = String::with_capacity(src.len() + header.len() + 64 + active.len() * 16);
    out.push_str(header);
    if !header.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("────────────────────────────────────────\n");
    out.push_str(&paint_layout(&layout, &marks));
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("────────────────────────────────────────\n");
    if with_quit_footer {
        out.push_str("[q] quit  ·  mini-notation highlight\n");
    }
    out
}

fn paint_layout(layout: &SourceLayout, marks: &[bool]) -> String {
    let mut out = String::new();
    for line in &layout.lines {
        let mut on = false;
        for (col, ch) in line.text.chars().enumerate() {
            let want = line
                .byte_of_col
                .get(col)
                .copied()
                .flatten()
                .and_then(|b| marks.get(b).copied())
                .unwrap_or(false);
            if want != on {
                if want {
                    out.push_str("\x1b[7m");
                } else {
                    out.push_str("\x1b[27m");
                }
                on = want;
            }
            out.push(ch);
        }
        if on {
            out.push_str("\x1b[27m");
        }
        out.push('\n');
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum JsState {
    Normal,
    LineComment,
    BlockComment,
    Double,
    Single,
    Backtick,
}

#[derive(Clone, Copy, Debug)]
struct JsScan {
    state: JsState,
    escape: bool,
}

impl JsScan {
    fn new() -> Self {
        Self {
            state: JsState::Normal,
            escape: false,
        }
    }

    fn at_newline(&mut self) {
        if self.state == JsState::LineComment {
            self.state = JsState::Normal;
        }
        self.escape = false;
    }

    /// Consume one token starting at byte `i`. Returns the next byte index.
    fn feed(&mut self, s: &str, i: usize) -> usize {
        let Some(c) = s[i..].chars().next() else {
            return i;
        };
        let n = c.len_utf8();
        match self.state {
            JsState::Normal => {
                if c == '/' {
                    let next = s.get(i + n..).and_then(|t| t.chars().next());
                    if next == Some('/') {
                        self.state = JsState::LineComment;
                        return i + n + 1;
                    }
                    if next == Some('*') {
                        self.state = JsState::BlockComment;
                        return i + n + 1;
                    }
                }
                match c {
                    '"' => self.state = JsState::Double,
                    '\'' => self.state = JsState::Single,
                    '`' => self.state = JsState::Backtick,
                    _ => {}
                }
                i + n
            }
            JsState::LineComment => i + n,
            JsState::BlockComment => {
                if c == '*' && s.get(i + n..).is_some_and(|t| t.starts_with('/')) {
                    self.state = JsState::Normal;
                    return i + n + 1;
                }
                i + n
            }
            JsState::Double | JsState::Single | JsState::Backtick => {
                let quote = match self.state {
                    JsState::Double => '"',
                    JsState::Single => '\'',
                    JsState::Backtick => '`',
                    JsState::Normal | JsState::LineComment | JsState::BlockComment => {
                        unreachable!("string states only")
                    }
                };
                if self.escape {
                    self.escape = false;
                } else if c == '\\' {
                    self.escape = true;
                } else if c == quote {
                    self.state = JsState::Normal;
                }
                i + n
            }
        }
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || c == '$'
}

/// Byte offsets of member-access `.` relative to `line`. Advances `scan` through the line.
fn member_dot_rel(line: &str, scan: &mut JsScan) -> Vec<usize> {
    let mut dots = Vec::new();
    let mut i = 0usize;
    while i < line.len() {
        let Some(c) = line[i..].chars().next() else {
            break;
        };
        if scan.state == JsState::Normal && c == '.' {
            let next = line[i + c.len_utf8()..].chars().next();
            if next.is_some_and(is_ident_start) {
                dots.push(i);
            }
        }
        i = scan.feed(line, i);
    }
    dots
}

fn wrap_source_line(
    line: &str,
    line_byte: usize,
    width: usize,
    scan: &mut JsScan,
) -> Vec<LayoutLine> {
    let mut scan_end = *scan;
    let dots: Vec<usize> = member_dot_rel(line, &mut scan_end)
        .into_iter()
        .map(|rel| line_byte + rel)
        .collect();
    *scan = scan_end;

    let chars: Vec<(usize, char)> = line.char_indices().collect();
    if chars.len() <= width {
        return vec![emit_range(line_byte, &chars, 0, chars.len(), 0)];
    }

    let mut out = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        let indent = if i > 0 && is_abs_dot(&dots, line_byte + chars[i].0) {
            2.min(width.saturating_sub(1))
        } else {
            0
        };
        let budget = width.saturating_sub(indent).max(1);
        let remaining = chars.len() - i;
        if remaining <= budget {
            out.push(emit_range(line_byte, &chars, i, chars.len(), indent));
            break;
        }
        let window_end = (i + budget).min(chars.len().saturating_sub(1));
        let mut chosen = None;
        for (ci, &(b, _)) in chars.iter().enumerate().take(window_end + 1).skip(i + 1) {
            if is_abs_dot(&dots, line_byte + b) {
                chosen = Some(ci);
            }
        }
        if let Some(d) = chosen {
            out.push(emit_range(line_byte, &chars, i, d, indent));
            i = d;
            continue;
        }
        let take = budget.min(remaining);
        out.push(emit_range(line_byte, &chars, i, i + take, indent));
        i += take;
    }
    out
}

fn is_abs_dot(dots: &[usize], abs_byte: usize) -> bool {
    dots.binary_search(&abs_byte).is_ok()
}

fn emit_range(
    line_byte: usize,
    chars: &[(usize, char)],
    start: usize,
    end: usize,
    indent: usize,
) -> LayoutLine {
    let mut text = String::with_capacity(indent + end.saturating_sub(start));
    let mut byte_of_col = Vec::with_capacity(indent + end.saturating_sub(start));
    for _ in 0..indent {
        text.push(' ');
        byte_of_col.push(None);
    }
    for &(b, ch) in &chars[start..end] {
        text.push(ch);
        byte_of_col.push(Some(line_byte + b));
    }
    LayoutLine { text, byte_of_col }
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

    #[test]
    fn wrap_method_chain_at_dot() {
        let src = r#"$: s("bd*4").gain(0.9)"#;
        let layout = layout_js_source(src, 16);
        let texts: Vec<&str> = layout.lines.iter().map(|l| l.text.as_str()).collect();
        assert_eq!(texts, vec![r#"$: s("bd*4")"#, "  .gain(0.9)"]);
    }

    #[test]
    fn wrap_skips_dots_inside_string() {
        let src = r#"$: s("x.y.z").gain(0.5)"#;
        let layout = layout_js_source(src, 18);
        let texts: Vec<&str> = layout.lines.iter().map(|l| l.text.as_str()).collect();
        assert_eq!(texts, vec![r#"$: s("x.y.z")"#, "  .gain(0.5)"]);
        assert!(
            !texts
                .iter()
                .any(|t| t.contains(r#"s("x"#) && t.ends_with("x")),
            "must not wrap at `.y` inside the string: {texts:?}"
        );
    }

    #[test]
    fn wrap_skips_decimal_point() {
        let src = r#"$: s("bd").gain(0.58)"#;
        let layout = layout_js_source(src, 20);
        let joined = layout
            .lines
            .iter()
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            joined.contains(".gain(0.58)"),
            "decimal in 0.58 must stay on the method line: {joined:?}"
        );
        assert!(
            !joined.contains("\n0.58") && !joined.contains("  58"),
            "{joined:?}"
        );
    }

    #[test]
    fn wrap_hard_when_no_dots() {
        let src = r#"$: s("abcdefghijklmnopqrstuvwxyz")"#;
        let layout = layout_js_source(src, 10);
        assert!(
            layout.lines.len() > 1,
            "expected hard wrap, got {:?}",
            layout.lines.iter().map(|l| &l.text).collect::<Vec<_>>()
        );
        assert_eq!(layout.lines[0].text.chars().count(), 10);
        assert!(
            !layout.lines[1].text.starts_with("  "),
            "hard wrap should not indent: {:?}",
            layout.lines[1].text
        );
        let rebuilt: String = layout.lines.iter().map(|l| l.text.as_str()).collect();
        assert_eq!(rebuilt, src);
    }

    #[test]
    fn wrap_short_line_unchanged() {
        let src = "setcpm(30)";
        let layout = layout_js_source(src, 40);
        assert_eq!(layout.lines.len(), 1);
        assert_eq!(layout.lines[0].text, src);
    }

    #[test]
    fn wrap_span_maps_to_same_text() {
        let src = r#"$: s("bd").gain(0.9)"#;
        let bd = src.find("bd").expect("bd");
        let layout = layout_js_source(src, 12);
        let (x, y) = layout_byte_xy(&layout, bd, 0);
        let line = &layout.lines[y as usize];
        let col = x as usize;
        let slice: String = line.text.chars().skip(col).take(2).collect();
        assert_eq!(slice, "bd", "line={:?} x={x} y={y}", line.text);
        assert_eq!(layout_span_cols(&layout, bd, bd + 2, 0), 2);
    }

    #[test]
    fn wrapped_render_keeps_reverse_on_atom() {
        let text = "---\nkick: s(\"bd\").gain(0.9)\n";
        let song = parse_song(text, "t").unwrap();
        let model = HighlightModel::from_song(&song, 48_000);
        let spans = active_spans(&model, 0, 0.0);
        let frame = render_ansi_wrapped(&model, &spans, "header", false, 16);
        assert!(frame.contains("\x1b[7m"));
        assert!(frame.contains("bd"));
        assert!(frame.contains("  .gain"), "{frame}");
    }
}
