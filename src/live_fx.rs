//! Live TUI visual effects overlay (issue #42).
//!
//! UI thread only. Audio still exposes nothing but the playhead sample counter.
//!
//! Photosensitivity: full-pane washes are dim, rate-capped below 3 Hz, and decay
//! from note-on (no on/off square pulse). Hats and other dense hits stay local.

use std::collections::HashSet;

/// WCAG 2.3.1 general-flash threshold is 3 Hz. Stay clearly under it.
pub const FLASH_MIN_INTERVAL_SECS: f32 = 0.40;
/// Exponential decay rate (1/seconds). Gain = exp(-k * age) from note-on.
const FLASH_DECAY: f32 = 5.0;
/// Drop the wash when gain falls below this (≈ 0.7 s at k=5).
const FLASH_CUTOFF: f32 = 0.03;

const RIPPLE_ACCENT_LIFE: f32 = 0.45;
const RIPPLE_LOCAL_LIFE: f32 = 0.28;
/// Radii are in column-width units (cell_dist).
const RIPPLE_ACCENT_R: f32 = 10.0;
const RIPPLE_LOCAL_R: f32 = 4.0;
/// Half-thickness of the ring, also in column-width units.
const RIPPLE_RING: f32 = 1.35;
/// Peak ripple brightness as a fraction of the 256-color RGB (then decays).
const RIPPLE_PEAK: f32 = 0.6;
/// Monospace cells are typically ~2× as tall as they are wide.
/// Scale `dy` so a Euclidean isosurface looks circular on screen, not a vertical ellipse.
const CELL_ASPECT: f32 = 2.0;
const BEAM_LIFE: f32 = 0.28;
const LONG_FX_CYCLES: f64 = 4.0;

const MAX_RIPPLES: usize = 24;
const MAX_BEAMS: usize = 12;
const MAX_HUES: usize = 8;

/// Peak pane-wash RGB (truecolor). Half of the previous pass.
const FLASH_RGB_BD: (u8, u8, u8) = (30, 5, 7);
const FLASH_RGB_SD: (u8, u8, u8) = (30, 25, 4);
const FLASH_RGB_CP: (u8, u8, u8) = (32, 18, 4);
const FLASH_RGB_IMPACT: (u8, u8, u8) = (28, 7, 10);
const FLASH_RGB_OTHER: (u8, u8, u8) = (6, 8, 20);

/// Brighter atom / ring colors (small area).
const ATOM_BD: u8 = 196;
const ATOM_SD: u8 = 220;
const ATOM_HH: u8 = 51;
const ATOM_OH: u8 = 87;
const ATOM_CP: u8 = 208;
const ATOM_PERC: u8 = 40;
const ATOM_RIM: u8 = 175;
const ATOM_TOM: u8 = 32;
const FALLBACK: [u8; 8] = [139, 107, 73, 67, 133, 101, 137, 109];

/// Slow hue wheel (dark cube) for long-FX track tints.
const HUE_DIM: [u8; 12] = [52, 58, 64, 22, 23, 17, 18, 19, 54, 53, 89, 88];
/// Pitch-class colors for melody beams (one row, not a pane fill).
const NOTE_BEAM: [u8; 12] = [167, 173, 179, 71, 73, 67, 68, 69, 104, 103, 133, 131];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitClass {
    /// Kick / snare / clap / impact: dim pane wash (rate-capped) + larger ripple.
    Accent,
    /// Hats and other percussion: local ripple + atom tint only.
    Local,
    /// Pitched notes: horizontal beam.
    Note,
    /// Risers / uplifters: slow track hue sweep.
    LongFx,
}

#[derive(Clone, Debug)]
pub struct FxHit {
    pub track_idx: usize,
    pub label: String,
    pub midi: Option<u8>,
    pub is_note: bool,
    pub start_cycle: f64,
    pub x: f32,
    pub y: f32,
    pub atom_cols: u16,
}

#[derive(Clone, Copy, Debug)]
struct AtomPaint {
    x: f32,
    y: f32,
    cols: u16,
    color: u8,
}

#[derive(Clone, Hash, Eq, PartialEq)]
struct HitKey {
    track_idx: u16,
    start_q: i32,
    label: String,
}

struct Ripple {
    deck: u8,
    x: f32,
    y: f32,
    color: u8,
    age: f32,
    life: f32,
    radius_max: f32,
}

struct Beam {
    deck: u8,
    x: f32,
    y: f32,
    color: u8,
    age: f32,
}

struct Flash {
    rgb: (u8, u8, u8),
    age: f32,
}

struct HueSweep {
    deck: u8,
    y0: f32,
    y1: f32,
    age: f32,
    life: f32,
}

pub struct FxState {
    clock: f32,
    prev: [HashSet<HitKey>; 2],
    last_flash_at: [f32; 2],
    ripples: Vec<Ripple>,
    beams: Vec<Beam>,
    flashes: [Option<Flash>; 2],
    hues: Vec<HueSweep>,
    atoms: [Vec<AtomPaint>; 2],
    #[cfg(test)]
    flash_spawns: u32,
}

impl Default for FxState {
    fn default() -> Self {
        Self::new()
    }
}

impl FxState {
    pub fn new() -> Self {
        Self {
            clock: 0.0,
            prev: Default::default(),
            last_flash_at: [-FLASH_MIN_INTERVAL_SECS, -FLASH_MIN_INTERVAL_SECS],
            ripples: Vec::new(),
            beams: Vec::new(),
            flashes: [None, None],
            hues: Vec::new(),
            atoms: Default::default(),
            #[cfg(test)]
            flash_spawns: 0,
        }
    }

    pub fn clear(&mut self) {
        self.prev[0].clear();
        self.prev[1].clear();
        self.ripples.clear();
        self.beams.clear();
        self.flashes = [None, None];
        self.hues.clear();
        self.atoms[0].clear();
        self.atoms[1].clear();
    }

    pub fn has_visuals(&self) -> bool {
        !self.ripples.is_empty()
            || !self.beams.is_empty()
            || self.flashes.iter().any(|f| f.is_some())
            || !self.hues.is_empty()
            || !self.atoms[0].is_empty()
            || !self.atoms[1].is_empty()
    }

    /// Ingest this frame's sounding hits for `deck` (0 or 1). Spawns on attacks only.
    pub fn observe(&mut self, deck: usize, hits: &[FxHit], secs_per_cycle: f32) {
        let deck = deck.min(1);
        let mut now = HashSet::with_capacity(hits.len());
        let mut paints = Vec::with_capacity(hits.len());
        for h in hits {
            now.insert(hit_key(h));
            paints.push(AtomPaint {
                x: h.x,
                y: h.y,
                cols: h.atom_cols.max(1),
                color: atom_color(h),
            });
        }
        for h in hits {
            if !self.prev[deck].contains(&hit_key(h)) {
                self.spawn_attack(deck, h, secs_per_cycle);
            }
        }
        self.prev[deck] = now;
        self.atoms[deck] = paints;
    }

    pub fn advance(&mut self, dt: f32) {
        let dt = dt.clamp(0.0, 0.10);
        self.clock += dt;
        for r in &mut self.ripples {
            r.age += dt;
        }
        self.ripples.retain(|r| r.age < r.life);
        for b in &mut self.beams {
            b.age += dt;
        }
        self.beams.retain(|b| b.age < BEAM_LIFE);
        for flash in &mut self.flashes {
            if let Some(f) = flash {
                f.age += dt;
                if flash_gain(f.age) < FLASH_CUTOFF {
                    *flash = None;
                }
            }
        }
        for h in &mut self.hues {
            h.age += dt;
        }
        self.hues.retain(|h| h.age < h.life);
    }

    /// Overlay VFX onto already width-padded pane lines.
    pub fn composite_lines(&self, deck: usize, lines: &mut [String], width: usize) {
        let deck = deck.min(1);
        if width == 0 || lines.is_empty() {
            return;
        }
        if !self.has_visuals() {
            return;
        }
        for (row, line) in lines.iter_mut().enumerate() {
            let mut cells = parse_ansi_line(line);
            while cells.len() < width {
                cells.push(Cell::blank());
            }
            cells.truncate(width);
            self.paint_row(deck as u8, row, &mut cells, width);
            *line = emit_line(&cells);
        }
    }

    #[cfg(test)]
    pub fn flash_spawn_count(&self) -> u32 {
        self.flash_spawns
    }

    #[cfg(test)]
    pub fn flash_active(&self, deck: usize) -> bool {
        self.flashes[deck.min(1)].is_some()
    }

    #[cfg(test)]
    pub fn ripple_count(&self) -> usize {
        self.ripples.len()
    }

    #[cfg(test)]
    pub fn beam_count(&self) -> usize {
        self.beams.len()
    }

    #[cfg(test)]
    pub fn hue_count(&self) -> usize {
        self.hues.len()
    }

    #[cfg(test)]
    pub fn flash_gain_at(&self, deck: usize) -> f32 {
        self.flashes[deck.min(1)]
            .as_ref()
            .map(|f| flash_gain(f.age))
            .unwrap_or(0.0)
    }

    fn spawn_attack(&mut self, deck: usize, h: &FxHit, secs_per_cycle: f32) {
        let class = classify(&h.label, h.is_note);
        match class {
            HitClass::Accent => {
                self.push_ripple(deck, h, RIPPLE_ACCENT_R, RIPPLE_ACCENT_LIFE, atom_color(h));
                self.try_pane_flash(deck, flash_peak_rgb(&h.label));
            }
            HitClass::Local => {
                self.push_ripple(deck, h, RIPPLE_LOCAL_R, RIPPLE_LOCAL_LIFE, atom_color(h));
            }
            HitClass::Note => {
                self.push_beam(deck, h);
            }
            HitClass::LongFx => {
                let life = (LONG_FX_CYCLES as f32) * secs_per_cycle.max(0.25);
                self.push_hue(deck, h, life);
                self.push_ripple(deck, h, RIPPLE_LOCAL_R, RIPPLE_LOCAL_LIFE, atom_color(h));
            }
        }
    }

    fn try_pane_flash(&mut self, deck: usize, rgb: (u8, u8, u8)) {
        // Restarting every 16th would strobe; keep the 3 Hz cap. A new note-on
        // after the interval resets the envelope from peak (decay, not a pulse).
        if self.clock - self.last_flash_at[deck] < FLASH_MIN_INTERVAL_SECS {
            return;
        }
        self.flashes[deck] = Some(Flash { rgb, age: 0.0 });
        self.last_flash_at[deck] = self.clock;
        #[cfg(test)]
        {
            self.flash_spawns += 1;
        }
    }

    fn push_ripple(&mut self, deck: usize, h: &FxHit, radius_max: f32, life: f32, color: u8) {
        if self.ripples.len() >= MAX_RIPPLES {
            self.ripples.remove(0);
        }
        self.ripples.push(Ripple {
            deck: deck as u8,
            x: h.x,
            y: h.y,
            color,
            age: 0.0,
            life,
            radius_max,
        });
    }

    fn push_beam(&mut self, deck: usize, h: &FxHit) {
        if self.beams.len() >= MAX_BEAMS {
            self.beams.remove(0);
        }
        let color = h.midi.map(midi_beam_color).unwrap_or_else(|| atom_color(h));
        self.beams.push(Beam {
            deck: deck as u8,
            x: h.x,
            y: h.y,
            color,
            age: 0.0,
        });
    }

    fn push_hue(&mut self, deck: usize, h: &FxHit, life: f32) {
        self.hues
            .retain(|s| !(s.deck == deck as u8 && (s.y0 - h.y).abs() < 0.6));
        if self.hues.len() >= MAX_HUES {
            self.hues.remove(0);
        }
        self.hues.push(HueSweep {
            deck: deck as u8,
            y0: h.y,
            y1: h.y,
            age: 0.0,
            life: life.max(0.5),
        });
    }

    fn paint_row(&self, deck: u8, row: usize, cells: &mut [Cell], width: usize) {
        let y = row as f32;
        if let Some(flash) = &self.flashes[deck as usize] {
            if let Some(rgb) = flash_rgb_now(flash) {
                // Dim wash only on cells that are not already reverse/atom-lit.
                for c in cells.iter_mut() {
                    if c.bg.is_none() && !c.reverse {
                        c.bg = Some(Color::Rgb(rgb.0, rgb.1, rgb.2));
                    }
                }
            }
        }
        for hue in &self.hues {
            if hue.deck != deck {
                continue;
            }
            if y < hue.y0 - 0.5 || y > hue.y1 + 0.5 {
                continue;
            }
            let t = (hue.age / hue.life).clamp(0.0, 1.0);
            let color = hue_dim_color(t);
            for c in cells.iter_mut() {
                if c.bg.is_none() && !c.reverse {
                    c.bg = Some(Color::Indexed(color));
                }
            }
        }
        for r in &self.ripples {
            if r.deck != deck {
                continue;
            }
            let t = (r.age / r.life).clamp(0.0, 1.0);
            let Some(rgb) = ripple_rgb_now(r) else {
                continue;
            };
            let radius = t * r.radius_max;
            for (col, c) in cells.iter_mut().enumerate() {
                let dx = col as f32 - r.x;
                let dy = y - r.y;
                let dist = cell_dist(dx, dy);
                if (dist - radius).abs() <= RIPPLE_RING {
                    c.bg = Some(Color::Rgb(rgb.0, rgb.1, rgb.2));
                }
            }
        }
        for b in &self.beams {
            if b.deck != deck {
                continue;
            }
            if (y - b.y).abs() > 0.55 {
                continue;
            }
            let t = (b.age / BEAM_LIFE).clamp(0.0, 1.0);
            // Shrink from the edges toward the origin so it is a beam, not a pane blink.
            let half = (width as f32) * 0.45 * (1.0 - t);
            for (col, c) in cells.iter_mut().enumerate() {
                if (col as f32 - b.x).abs() <= half {
                    c.bg = Some(Color::Indexed(b.color));
                }
            }
        }
        for a in &self.atoms[deck as usize] {
            if (y - a.y).abs() > 0.5 {
                continue;
            }
            let x0 = a.x.round().max(0.0) as usize;
            let x1 = (x0 + a.cols as usize).min(cells.len());
            for c in &mut cells[x0..x1] {
                c.bg = Some(Color::Indexed(a.color));
                c.reverse = false;
            }
        }
    }
}

fn flash_gain(age: f32) -> f32 {
    (-FLASH_DECAY * age.max(0.0)).exp()
}

fn flash_rgb_now(flash: &Flash) -> Option<(u8, u8, u8)> {
    scale_rgb(flash.rgb, flash_gain(flash.age))
}

fn ripple_gain(age: f32, life: f32) -> f32 {
    let t = if life <= 1e-6 {
        1.0
    } else {
        (age / life).clamp(0.0, 1.0)
    };
    let decay = 1.0 - t;
    RIPPLE_PEAK * decay * decay
}

fn ripple_rgb_now(r: &Ripple) -> Option<(u8, u8, u8)> {
    scale_rgb(ansi256_to_rgb(r.color), ripple_gain(r.age, r.life))
}

fn scale_rgb(rgb: (u8, u8, u8), gain: f32) -> Option<(u8, u8, u8)> {
    if gain < FLASH_CUTOFF {
        return None;
    }
    Some((
        (rgb.0 as f32 * gain).round() as u8,
        (rgb.1 as f32 * gain).round() as u8,
        (rgb.2 as f32 * gain).round() as u8,
    ))
}

/// xterm 256-color index → RGB (cube + grayscale; basic 0..=15 are VGA).
fn ansi256_to_rgb(n: u8) -> (u8, u8, u8) {
    match n {
        0..=15 => {
            const BASIC: [(u8, u8, u8); 16] = [
                (0, 0, 0),
                (128, 0, 0),
                (0, 128, 0),
                (128, 128, 0),
                (0, 0, 128),
                (128, 0, 128),
                (0, 128, 128),
                (192, 192, 192),
                (128, 128, 128),
                (255, 0, 0),
                (0, 255, 0),
                (255, 255, 0),
                (0, 0, 255),
                (255, 0, 255),
                (0, 255, 255),
                (255, 255, 255),
            ];
            BASIC[n as usize]
        }
        16..=231 => {
            let n = n - 16;
            let r = n / 36;
            let g = (n % 36) / 6;
            let b = n % 6;
            let level = |c: u8| if c == 0 { 0 } else { 40 * c + 55 };
            (level(r), level(g), level(b))
        }
        _ => {
            let v = 8 + (n - 232) * 10;
            (v, v, v)
        }
    }
}

/// Euclidean distance in visual space (column-width units).
///
/// One row is `CELL_ASPECT` columns tall, so equal cell counts would draw a
/// vertical ellipse. Compensate here.
fn cell_dist(dx: f32, dy: f32) -> f32 {
    let vy = dy * CELL_ASPECT;
    dx.mul_add(dx, vy * vy).sqrt()
}

fn hit_key(h: &FxHit) -> HitKey {
    HitKey {
        track_idx: h.track_idx.min(u16::MAX as usize) as u16,
        start_q: (h.start_cycle * 64.0).round() as i32,
        label: h.label.clone(),
    }
}

pub fn classify(label: &str, is_note: bool) -> HitClass {
    if is_note {
        return HitClass::Note;
    }
    let l = norm(label);
    if is_long_fx(&l) {
        return HitClass::LongFx;
    }
    if is_accent(&l) {
        return HitClass::Accent;
    }
    HitClass::Local
}

fn is_long_fx(l: &str) -> bool {
    matches!(l, "fx:up" | "fx:nr")
        || l.starts_with("fx-riser")
        || l.starts_with("fx-uplifter")
        || (l.starts_with("fx-") && (l.contains("riser") || l.contains("uplift")))
}

fn is_accent(l: &str) -> bool {
    matches!(
        l,
        "bd" | "kick" | "bassdrum" | "sd" | "sn" | "snare" | "cp" | "clap" | "fx:id"
    ) || l.starts_with("fx-impact")
}

fn norm(label: &str) -> String {
    label.trim().to_ascii_lowercase()
}

/// Small-area color (atom / ripple). Distinct per drum token.
pub fn atom_color_for_label(label: &str) -> u8 {
    match norm(label).as_str() {
        "bd" | "kick" | "bassdrum" => ATOM_BD,
        "sd" | "sn" | "snare" => ATOM_SD,
        "hh" | "ch" | "chh" | "hat" => ATOM_HH,
        "oh" | "ohh" | "openhat" => ATOM_OH,
        "cp" | "clap" => ATOM_CP,
        "perc" => ATOM_PERC,
        "rim" => ATOM_RIM,
        "tom" => ATOM_TOM,
        other => {
            let mut h = 0u32;
            for b in other.bytes() {
                h = h.wrapping_mul(16777619) ^ u32::from(b);
            }
            FALLBACK[(h as usize) % FALLBACK.len()]
        }
    }
}

/// Peak RGB for a large-area pane wash (before envelope).
pub fn flash_peak_rgb(label: &str) -> (u8, u8, u8) {
    let l = norm(label);
    if is_accent(&l) && (l.starts_with("fx-impact") || l.contains("impact")) {
        return FLASH_RGB_IMPACT;
    }
    match l.as_str() {
        "bd" | "kick" | "bassdrum" => FLASH_RGB_BD,
        "sd" | "sn" | "snare" => FLASH_RGB_SD,
        "cp" | "clap" => FLASH_RGB_CP,
        _ => FLASH_RGB_OTHER,
    }
}

fn atom_color(h: &FxHit) -> u8 {
    if h.is_note {
        if let Some(m) = h.midi {
            return midi_beam_color(m);
        }
    }
    atom_color_for_label(&h.label)
}

pub fn midi_beam_color(midi: u8) -> u8 {
    NOTE_BEAM[(midi % 12) as usize]
}

fn hue_dim_color(t: f32) -> u8 {
    let i = ((t * HUE_DIM.len() as f32).floor() as usize).min(HUE_DIM.len() - 1);
    HUE_DIM[i]
}

/// Map a byte offset in song source to pane cells (after `header_rows` banner lines).
pub fn source_byte_xy(source: &str, byte: usize, header_rows: usize) -> (f32, f32) {
    let byte = byte.min(source.len());
    let mut row = header_rows as f32;
    let mut col = 0.0f32;
    for c in source[..byte].chars() {
        if c == '\n' {
            row += 1.0;
            col = 0.0;
        } else {
            col += 1.0;
        }
    }
    (col, row)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Color {
    Indexed(u8),
    Rgb(u8, u8, u8),
}

#[derive(Clone, Copy)]
struct Cell {
    ch: char,
    fg: Option<u8>,
    bg: Option<Color>,
    reverse: bool,
    dim: bool,
}

impl Cell {
    fn blank() -> Self {
        Self {
            ch: ' ',
            fg: None,
            bg: None,
            reverse: false,
            dim: false,
        }
    }
}

fn parse_ansi_line(s: &str) -> Vec<Cell> {
    let mut cells = Vec::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    let mut fg = None;
    let mut bg = None;
    let mut reverse = false;
    let mut dim = false;
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                let mut params = String::new();
                for c2 in chars.by_ref() {
                    if ('@'..='~').contains(&c2) {
                        apply_sgr(&params, c2, &mut fg, &mut bg, &mut reverse, &mut dim);
                        break;
                    }
                    params.push(c2);
                }
            }
            continue;
        }
        cells.push(Cell {
            ch: c,
            fg,
            bg,
            reverse,
            dim,
        });
    }
    cells
}

fn apply_sgr(
    params: &str,
    end: char,
    fg: &mut Option<u8>,
    bg: &mut Option<Color>,
    reverse: &mut bool,
    dim: &mut bool,
) {
    if end != 'm' {
        return;
    }
    if params.is_empty() {
        *fg = None;
        *bg = None;
        *reverse = false;
        *dim = false;
        return;
    }
    let parts: Vec<i32> = params.split(';').filter_map(|p| p.parse().ok()).collect();
    let mut i = 0;
    while i < parts.len() {
        match parts[i] {
            0 => {
                *fg = None;
                *bg = None;
                *reverse = false;
                *dim = false;
            }
            2 => *dim = true,
            7 => *reverse = true,
            22 => *dim = false,
            27 => *reverse = false,
            39 => *fg = None,
            49 => *bg = None,
            38 if i + 2 < parts.len() && parts[i + 1] == 5 => {
                *fg = Some(parts[i + 2].clamp(0, 255) as u8);
                i += 2;
            }
            48 if i + 2 < parts.len() && parts[i + 1] == 5 => {
                *bg = Some(Color::Indexed(parts[i + 2].clamp(0, 255) as u8));
                i += 2;
            }
            48 if i + 4 < parts.len() && parts[i + 1] == 2 => {
                *bg = Some(Color::Rgb(
                    parts[i + 2].clamp(0, 255) as u8,
                    parts[i + 3].clamp(0, 255) as u8,
                    parts[i + 4].clamp(0, 255) as u8,
                ));
                i += 4;
            }
            _ => {}
        }
        i += 1;
    }
}

fn emit_line(cells: &[Cell]) -> String {
    let mut out = String::with_capacity(cells.len() + 16);
    let mut cur_fg: Option<Option<u8>> = None;
    let mut cur_bg: Option<Option<Color>> = None;
    let mut cur_rev: Option<bool> = None;
    let mut cur_dim: Option<bool> = None;
    for c in cells {
        let style_change = cur_fg != Some(c.fg)
            || cur_bg != Some(c.bg)
            || cur_rev != Some(c.reverse)
            || cur_dim != Some(c.dim);
        if style_change {
            out.push_str("\x1b[0m");
            if c.dim {
                out.push_str("\x1b[2m");
            }
            if c.reverse {
                out.push_str("\x1b[7m");
            }
            if let Some(fg) = c.fg {
                out.push_str("\x1b[38;5;");
                out.push_str(&fg.to_string());
                out.push('m');
            }
            if let Some(bg) = c.bg {
                match bg {
                    Color::Indexed(n) => {
                        out.push_str("\x1b[48;5;");
                        out.push_str(&n.to_string());
                        out.push('m');
                    }
                    Color::Rgb(r, g, b) => {
                        out.push_str("\x1b[48;2;");
                        out.push_str(&r.to_string());
                        out.push(';');
                        out.push_str(&g.to_string());
                        out.push(';');
                        out.push_str(&b.to_string());
                        out.push('m');
                    }
                }
            }
            cur_fg = Some(c.fg);
            cur_bg = Some(c.bg);
            cur_rev = Some(c.reverse);
            cur_dim = Some(c.dim);
        }
        out.push(c.ch);
    }
    out.push_str("\x1b[0m");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(label: &str, is_note: bool, start: f64) -> FxHit {
        FxHit {
            track_idx: 0,
            label: label.into(),
            midi: if is_note { Some(60) } else { None },
            is_note,
            start_cycle: start,
            x: 4.0,
            y: 2.0,
            atom_cols: label.chars().count() as u16,
        }
    }

    #[test]
    fn palette_drums_are_distinct_ch_matches_hh() {
        let bd = atom_color_for_label("bd");
        let sd = atom_color_for_label("sd");
        let hh = atom_color_for_label("hh");
        let ch = atom_color_for_label("ch");
        let oh = atom_color_for_label("oh");
        let cp = atom_color_for_label("cp");
        assert_eq!(ch, hh);
        let set = [bd, sd, hh, oh, cp];
        for i in 0..set.len() {
            for j in 0..set.len() {
                if i != j {
                    assert_ne!(set[i], set[j], "i={i} j={j}");
                }
            }
        }
    }

    #[test]
    fn classify_families() {
        assert_eq!(classify("bd", false), HitClass::Accent);
        assert_eq!(classify("sd", false), HitClass::Accent);
        assert_eq!(classify("cp", false), HitClass::Accent);
        assert_eq!(classify("fx:id", false), HitClass::Accent);
        assert_eq!(classify("hh", false), HitClass::Local);
        assert_eq!(classify("ch", false), HitClass::Local);
        assert_eq!(classify("oh", false), HitClass::Local);
        assert_eq!(classify("c3", true), HitClass::Note);
        assert_eq!(classify("fx:up", false), HitClass::LongFx);
        assert_eq!(classify("fx:nr", false), HitClass::LongFx);
    }

    #[test]
    fn hats_never_spawn_pane_flash() {
        let mut fx = FxState::new();
        for i in 0..16 {
            fx.observe(0, &[hit("hh", false, i as f64 / 16.0)], 2.0);
            fx.advance(0.03);
        }
        assert_eq!(fx.flash_spawn_count(), 0);
        assert!(fx.ripple_count() > 0);
    }

    #[test]
    fn accent_flash_is_rate_capped_not_strobe() {
        let mut fx = FxState::new();
        // 16th-note kicks (~0.05s apart at 120 BPM would be faster; use 50ms).
        for i in 0..12 {
            fx.observe(0, &[hit("bd", false, i as f64)], 2.0);
            fx.advance(0.05);
        }
        // 12 attacks over 0.6s → at most floor(0.6/0.4)+1 = 2, never 3 Hz strobe.
        assert!(
            fx.flash_spawn_count() <= 2,
            "spawns={}",
            fx.flash_spawn_count()
        );
        assert!(fx.flash_spawn_count() >= 1);
    }

    #[test]
    fn no_flash_retrigger_while_fading() {
        let mut fx = FxState::new();
        fx.observe(0, &[hit("bd", false, 0.0)], 2.0);
        assert!(fx.flash_active(0));
        fx.advance(0.05);
        fx.observe(0, &[hit("sd", false, 1.0)], 2.0);
        assert_eq!(
            fx.flash_spawn_count(),
            1,
            "second accent during fade must not strobe"
        );
    }

    #[test]
    fn wash_gain_decays_from_note_on() {
        let mut fx = FxState::new();
        fx.observe(0, &[hit("bd", false, 0.0)], 2.0);
        let a = fx.flash_gain_at(0);
        fx.advance(0.12);
        let b = fx.flash_gain_at(0);
        fx.advance(0.12);
        let c = fx.flash_gain_at(0);
        assert!(a > 0.95, "onset gain={a}");
        assert!(a > b && b > c, "a={a} b={b} c={c}");
        assert!(c > FLASH_CUTOFF);
    }

    #[test]
    fn wash_may_restart_after_min_interval() {
        let mut fx = FxState::new();
        fx.observe(0, &[hit("bd", false, 0.0)], 2.0);
        // advance() caps dt at 0.10 (one frame).
        let mut elapsed = 0.0;
        while elapsed < FLASH_MIN_INTERVAL_SECS + 0.02 {
            fx.advance(0.05);
            elapsed += 0.05;
        }
        fx.observe(0, &[hit("bd", false, 1.0)], 2.0);
        assert_eq!(fx.flash_spawn_count(), 2);
        assert!(fx.flash_gain_at(0) > 0.95, "new note-on resets envelope");
    }

    #[test]
    fn flash_peak_is_dim_not_bright_red() {
        let bd = flash_peak_rgb("bd");
        let sd = flash_peak_rgb("sd");
        assert_ne!(bd, sd);
        assert!(bd.0 <= 32 && bd.1 < 20, "bd={bd:?}");
        assert!(sd.0 <= 32 && sd.1 <= 32, "sd={sd:?}");
    }

    #[test]
    fn same_hit_sustained_spawns_one_ripple() {
        let mut fx = FxState::new();
        let h = hit("bd", false, 0.0);
        fx.observe(0, std::slice::from_ref(&h), 2.0);
        fx.advance(0.03);
        fx.observe(0, std::slice::from_ref(&h), 2.0);
        assert_eq!(fx.ripple_count(), 1);
    }

    #[test]
    fn note_spawns_beam_not_flash() {
        let mut fx = FxState::new();
        fx.observe(0, &[hit("c3", true, 0.0)], 2.0);
        assert_eq!(fx.flash_spawn_count(), 0);
        assert_eq!(fx.beam_count(), 1);
    }

    #[test]
    fn long_fx_spawns_hue() {
        let mut fx = FxState::new();
        fx.observe(0, &[hit("fx:up", false, 0.0)], 2.0);
        assert_eq!(fx.hue_count(), 1);
        assert_eq!(fx.flash_spawn_count(), 0);
        let a = hue_dim_color(0.0);
        let b = hue_dim_color(1.0);
        assert_ne!(a, b);
    }

    #[test]
    fn composite_does_not_panic_on_reverse_video() {
        let mut fx = FxState::new();
        fx.observe(0, &[hit("bd", false, 0.0)], 2.0);
        let mut lines = vec!["\x1b[7mbd\x1b[27m rest".to_string(), "hello".into()];
        fx.composite_lines(0, &mut lines, 20);
        assert!(lines[0].contains('\u{1b}') || !lines[0].is_empty());
        assert!(
            lines
                .iter()
                .any(|l| l.contains("\x1b[48;5;") || l.contains("\x1b[48;2;")),
            "{lines:?}"
        );
    }

    #[test]
    fn source_byte_xy_skips_header() {
        let src = "setcpm(30)\n$: s(\"bd\")\n";
        let (x, y) = source_byte_xy(src, src.find("bd").unwrap(), 2);
        assert!(y >= 2.0);
        assert!(x >= 0.0);
    }

    #[test]
    fn ripple_peak_is_scaled_and_decays() {
        assert!((ripple_gain(0.0, 1.0) - RIPPLE_PEAK).abs() < 1e-5);
        assert!(ripple_gain(0.5, 1.0) < ripple_gain(0.0, 1.0));
        assert!(ripple_gain(0.9, 1.0) < ripple_gain(0.5, 1.0));
        let full = ansi256_to_rgb(ATOM_BD);
        let peak = scale_rgb(full, RIPPLE_PEAK).unwrap();
        let expected = (full.0 as f32 * RIPPLE_PEAK).round() as u8;
        assert_eq!(peak.0, expected, "peak={peak:?} full={full:?}");
        let later = scale_rgb(full, ripple_gain(0.5, 1.0)).unwrap();
        assert!(later.0 < peak.0, "later={later:?} peak={peak:?}");
    }

    #[test]
    fn cell_dist_treats_two_cols_as_one_row() {
        let across = cell_dist(2.0, 0.0);
        let down = cell_dist(0.0, 1.0);
        assert!(
            (across - down).abs() < 1e-5,
            "across={across} down={down} (cells are ~2× taller than wide)"
        );
        // Uncorrected equal cell counts must NOT be treated as a circle.
        assert!((cell_dist(1.0, 0.0) - cell_dist(0.0, 1.0)).abs() > 0.5);
    }
}
