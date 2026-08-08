//! Method-chain pattern code: `note("...").s("sawtooth").lpf(800)...`
//!
//! Also supports Strudel factories as the pattern argument:
//! - `note(cat("a", "b"))` / `s(cat("bd ~", "sd ~"))` — one cycle per arg (`"<a b>"`)
//! - nested `cat(cat(...), ...)` flattens to a longer cycle list

use crate::control::{is_colon_scalar_list, parse_dyn_f32, DynF32, LfoSpec};
use crate::dsp::CompressorParams;
use crate::mini::{self, Node};
use crate::scale::ScalePattern;

/// Defaults for amplitude envelope (seconds; sustain is level 0..1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Adsr {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

impl Default for Adsr {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.1,
        }
    }
}

/// Local filter cutoffs / Q (None = bypass that stage).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FilterParams {
    pub lpf: Option<f32>,
    pub lpq: f32,
    pub hpf: Option<f32>,
    pub hpq: f32,
    pub bpf: Option<f32>,
    pub bpq: f32,
}

impl Default for FilterParams {
    fn default() -> Self {
        Self {
            lpf: None,
            lpq: 0.707,
            hpf: None,
            hpq: 0.707,
            bpf: None,
            bpq: 1.0,
        }
    }
}

/// Vibrato / FM / pitch & filter envelope modulation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModParams {
    pub vib_hz: f32,
    pub vibmod: f32,
    pub fm: f32,
    pub fmh: f32,
    pub fm_attack: f32,
    pub fm_decay: f32,
    pub fm_sustain: f32,
    pub noise_mix: f32,
    pub penv: f32,
    pub patt: f32,
    pub pdec: f32,
    pub lpenv: f32,
    pub lpa: f32,
    pub lpd: f32,
    pub lps: f32,
    pub lpr: f32,
    /// Continuous LPF LFO (`sine.rangex` etc.). `None` = off.
    pub lpf_lfo: Option<LfoSpec>,
    /// Bar phase (0..1) at note onset; continuous LFO continues from here.
    pub lfo_phase0: f32,
    /// Samples per bar (cycle) for LFO phase advance; 0 = treat as 1 bar/sec fallback.
    pub samples_per_bar: f32,
}

impl Default for ModParams {
    fn default() -> Self {
        Self {
            vib_hz: 0.0,
            vibmod: 0.5,
            fm: 0.0,
            fmh: 1.0,
            fm_attack: 0.001,
            fm_decay: 0.1,
            fm_sustain: 0.0,
            noise_mix: 0.0,
            penv: 0.0,
            patt: 0.2,
            pdec: 0.0,
            lpenv: 0.0,
            lpa: 0.01,
            lpd: 0.1,
            lps: 0.5,
            lpr: 0.1,
            lpf_lfo: None,
            lfo_phase0: 0.0,
            samples_per_bar: 0.0,
        }
    }
}

/// Up to 4 duck targets (orbit id 1-based; 0 = unused slot).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DuckParams {
    pub count: u8,
    pub orbits: [u8; 4],
    pub attack: [f32; 4],
    pub depth: [f32; 4],
}

impl Default for DuckParams {
    fn default() -> Self {
        Self {
            count: 0,
            orbits: [0; 4],
            attack: [0.2; 4],
            depth: [1.0; 4],
        }
    }
}

#[derive(Debug, Clone)]
pub struct PatternCode {
    pub pattern: Node,
    /// Waveform name or sample name (resolved at play time).
    pub sound: String,
    pub gain: f32,
    /// Multiplier applied with gain (from velocity/vel).
    pub velocity: f32,
    /// Stereo pan 0=left … 0.5=center … 1=right (equal-power).
    pub pan: f32,
    pub filter: FilterParams,
    /// Accumulated slow/fast factor (slow divides, fast multiplies).
    pub speed: f64,
    pub is_note: bool,
    pub adsr: Adsr,
    pub mod_params: ModParams,
    pub begin: f32,
    pub end: f32,
    pub sample_speed: f32,
    pub sample_n: Option<i32>,
    /// Sample bank prefix (`.bank("RolandTR808")` → resolve `rolandtr808_bd`).
    pub bank: Option<String>,
    /// Event length multiplier (clip / legato). None = 1.0.
    pub clip: Option<f32>,
    pub legato: Option<f32>,
    /// Cut group; same group steals previous voices.
    pub cut: Option<i32>,
    /// Orbit id (1-based, default 1).
    pub orbit: u8,
    pub duck: DuckParams,
    pub compressor: Option<CompressorParams>,
    /// Orbit delay wet level 0..=1 (global per orbit).
    pub delay: f32,
    /// Delay time in seconds.
    pub delaytime: f32,
    /// Delay feedback (clamped &lt; 1 at DSP).
    pub delayfeedback: f32,
    /// Orbit reverb wet level 0..=1.
    pub room: f32,
    /// Reverb size 0..=10 (Strudel-compatible range).
    pub roomsize: f32,
    /// Optional scale (fixed or per-cycle progression): integer mini atoms are degrees.
    pub scale: Option<ScalePattern>,
    /// Scalar pitch offset from `.add` / `.sub` (accumulated).
    /// With scale: integer degrees shift by `pitch_add.round()` steps.
    /// Named notes: shift by `pitch_add` semitones (MIDI).
    pub pitch_add: f64,
    /// Event subdivision factor from `.ply(n)` (accumulated by multiply, clamp 1..=16).
    pub ply: u32,
    /// Optional dynamic gain (mini pattern or LFO snapshot at event).
    pub gain_dyn: Option<DynF32>,
    /// Optional dynamic pan.
    pub pan_dyn: Option<DynF32>,
    /// Optional dynamic LPF (pattern per-event, or continuous LFO on synth voice).
    pub lpf_dyn: Option<DynF32>,
    pub hpf_dyn: Option<DynF32>,
    pub bpf_dyn: Option<DynF32>,
    /// Optional per-event pitch_add pattern (added to scalar `pitch_add`).
    pub pitch_add_dyn: Option<DynF32>,
    /// Sign for patterned pitch add from `.sub` (-1) vs `.add` (+1).
    pub pitch_add_dyn_sign: f64,
    /// Original source text (for display / error context).
    pub raw: String,
    /// Mini-notation string (contents of the first `"..."`).
    pub mini_src: String,
    /// Absolute byte offset of `mini_src` in the song file (set by song parser).
    pub mini_base: usize,
}

impl PatternCode {
    pub fn effective_gain(&self) -> f32 {
        self.gain * self.velocity
    }

    /// Length scale from clip (priority) or legato; default 1.0.
    pub fn length_scale(&self) -> f32 {
        self.clip.or(self.legato).unwrap_or(1.0).max(0.01)
    }

    /// Convenience: legacy field name used in tests.
    pub fn lpf(&self) -> Option<f32> {
        self.filter.lpf
    }

    /// Shift mini_base from code-relative to absolute song source offset.
    pub fn with_source_base(mut self, code_abs_start: usize) -> Self {
        self.mini_base = code_abs_start.saturating_add(self.mini_base);
        self
    }
}

pub fn parse_code(input: &str) -> Result<PatternCode, String> {
    let trim_start = leading_ws_bytes(input);
    // Allow trailing `;` (JS/Strudel habit) and surrounding whitespace.
    let input = input.trim().trim_end_matches(';').trim();
    let mut pc = PatternCode {
        pattern: Node::Rest,
        sound: "triangle".into(),
        gain: 0.5,
        velocity: 1.0,
        pan: 0.5,
        filter: FilterParams::default(),
        speed: 1.0,
        is_note: false,
        adsr: Adsr::default(),
        mod_params: ModParams::default(),
        begin: 0.0,
        end: 1.0,
        sample_speed: 1.0,
        sample_n: None,
        bank: None,
        clip: None,
        legato: None,
        cut: None,
        orbit: 1,
        duck: DuckParams::default(),
        compressor: None,
        delay: 0.0,
        delaytime: 0.25,
        delayfeedback: 0.5,
        room: 0.0,
        roomsize: 1.0,
        scale: None,
        pitch_add: 0.0,
        ply: 1,
        gain_dyn: None,
        pan_dyn: None,
        lpf_dyn: None,
        hpf_dyn: None,
        bpf_dyn: None,
        pitch_add_dyn: None,
        pitch_add_dyn_sign: 1.0,
        raw: input.to_string(),
        mini_src: String::new(),
        mini_base: 0,
    };

    let open = input.find('(').ok_or_else(|| "missing (".to_string())?;
    let head = input[..open].trim();
    if !matches!(head, "note" | "n" | "s" | "sound" | "cat" | "slowcat") {
        return Err(format!("unknown head: {head} (use note/s/cat)"));
    }

    if !parens_balanced(input) {
        return Err("unbalanced parens".into());
    }

    let (head_args, after_head) =
        match_parens(&input[open..]).ok_or_else(|| "unbalanced parens in head".to_string())?;
    // `open + 1` is the absolute index of head_args inside trimmed `input`.
    let args_abs = open + 1;

    match head {
        "cat" | "slowcat" => {
            let (pat, mini_src, span_base) = parse_cat_call(head_args, args_abs)?;
            pc.pattern = pat;
            pc.mini_src = mini_src;
            // Spans are relative to trimmed input; mini_base is 0 within trimmed,
            // adjusted by leading whitespace of the original code slice.
            pc.mini_base = trim_start + span_base;
        }
        "note" | "n" | "s" | "sound" => {
            let (pat, mini_src, content_abs) = parse_pattern_arg(head_args, args_abs)?;
            pc.pattern = pat;
            pc.mini_src = mini_src.clone();
            pc.mini_base = trim_start + content_abs;
            pc.is_note = matches!(head, "note" | "n");
            if matches!(head, "s" | "sound") {
                pc.sound = first_sound(&pc.pattern, &mini_src);
                pc.is_note = false;
            }
        }
        _ => unreachable!(),
    }

    // Method chain: `.s("saw").gain(0.5)` after the head call.
    let mut cur = after_head;
    while let Some(dot_rel) = cur.find('.') {
        let after = &cur[dot_rel + 1..];
        let paren = after
            .find('(')
            .ok_or_else(|| "missing ( in method".to_string())?;
        let name = after[..paren].trim();
        let (args, rest) = match_parens(&after[paren..])
            .ok_or_else(|| "unbalanced parens in method".to_string())?;
        apply_method(&mut pc, name, args)?;
        cur = rest;
    }
    Ok(pc)
}

/// Parse a single pattern argument: `"mini"` or `cat(...)` / `slowcat(...)`.
///
/// Returns `(node, mini_src, span_base)` where `span_base` is the byte offset
/// inside the trimmed code input that atom spans are relative to (0 for cat
/// multi-arg patterns whose spans are already absolute within the trimmed input).
fn parse_pattern_arg(args: &str, args_abs: usize) -> Result<(Node, String, usize), String> {
    let args = args.trim();
    if args.is_empty() {
        return Err("empty pattern argument".into());
    }
    if let Some(inner) = call_inner(args, &["cat", "slowcat"]) {
        let open = args.find('(').unwrap();
        let inner_abs = args_abs + leading_ws_bytes(args) + open + 1;
        let (pat, mini_src, _) = parse_cat_call(inner, inner_abs)?;
        // Spans already absolute within trimmed input → base 0 relative to input start.
        return Ok((pat, mini_src, 0));
    }
    // Plain mini string.
    let (s, content_start) = extract_first_string(args)?;
    let node = mini::parse(&s)?;
    Ok((node, s, args_abs + leading_ws_bytes(args) + content_start))
}

/// `cat("a", "b", cat("c", "d"))` → `Stack` of one-cycle items (flattened).
///
/// Returns `(node, display_src, span_base)` where spans in `node` are relative to
/// the trimmed code input (span_base is always 0 for the returned node).
fn parse_cat_call(inner: &str, inner_abs: usize) -> Result<(Node, String, usize), String> {
    let mut items: Vec<Node> = Vec::new();
    let mut src_parts: Vec<String> = Vec::new();
    for (rel, part) in split_args_with_pos(inner) {
        let lead = leading_ws_bytes(part);
        let part_trim = part.trim();
        if part_trim.is_empty() {
            continue;
        }
        if let Some(nested) = call_inner(part_trim, &["cat", "slowcat"]) {
            let open = part_trim.find('(').unwrap();
            let nested_abs = inner_abs + rel + lead + open + 1;
            let (node, nested_src, _) = parse_cat_call(nested, nested_abs)?;
            src_parts.push(nested_src);
            match node {
                Node::Stack(v) => items.extend(v),
                other => items.push(other),
            }
            continue;
        }
        let (content, content_off) = extract_first_string(part_trim)?;
        src_parts.push(content.clone());
        let mut node = mini::parse(&content)?;
        // Atom spans → absolute within trimmed code input.
        let delta = inner_abs + rel + lead + content_off;
        mini::offset_spans(&mut node, delta);
        items.push(node);
    }
    if items.is_empty() {
        return Err("cat() needs at least one argument".into());
    }
    Ok((Node::Stack(items), src_parts.join(" | "), 0))
}

/// If `s` is `name(...)` for one of `names`, return the inside of the outer parens.
fn call_inner<'a>(s: &'a str, names: &[&str]) -> Option<&'a str> {
    let s = s.trim();
    for name in names {
        if let Some(rest) = s.strip_prefix(name) {
            let rest = rest.trim_start();
            if let Some((inner, after)) = match_parens(rest) {
                if after.trim().is_empty() {
                    return Some(inner);
                }
            }
        }
    }
    None
}

/// `s` starts with `(…)` — return `(inner, rest_after_closing)`.
fn match_parens(s: &str) -> Option<(&str, &str)> {
    if !s.starts_with('(') {
        return None;
    }
    let mut depth = 0i32;
    let mut in_str = false;
    for (i, c) in s.char_indices() {
        if in_str {
            if c == '"' {
                in_str = false;
            }
            continue;
        }
        match c {
            '"' => in_str = true,
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some((&s[1..i], &s[i + 1..]));
                }
            }
            _ => {}
        }
    }
    None
}

/// Split top-level comma-separated args; each entry is `(byte_offset, slice)`.
fn split_args_with_pos(s: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut depth = 0i32;
    let mut in_str = false;
    for (i, c) in s.char_indices() {
        if in_str {
            if c == '"' {
                in_str = false;
            }
            continue;
        }
        match c {
            '"' => in_str = true,
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            ',' if depth == 0 => {
                out.push((start, &s[start..i]));
                start = i + c.len_utf8();
            }
            _ => {}
        }
    }
    out.push((start, &s[start..]));
    out
}

fn parens_balanced(s: &str) -> bool {
    let mut depth = 0i32;
    let mut in_str = false;
    for ch in s.chars() {
        if in_str {
            if ch == '"' {
                in_str = false;
            }
            continue;
        }
        match ch {
            '"' => in_str = true,
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0
}

fn first_word(s: &str) -> String {
    s.split_whitespace().next().unwrap_or("sine").to_string()
}

fn first_sound(pattern: &Node, mini_src: &str) -> String {
    if let Some(v) = first_atom_value(pattern) {
        if v != "~" {
            return v;
        }
    }
    first_word(mini_src)
}

fn first_atom_value(node: &Node) -> Option<String> {
    match node {
        Node::Atom { value, .. } => Some(value.clone()),
        Node::Rest => None,
        Node::Seq(items) | Node::Stack(items) | Node::Parallel(items) => {
            items.iter().find_map(first_atom_value)
        }
        Node::Fast(inner, _) | Node::Slow(inner, _) | Node::Elongate(inner, _) => {
            first_atom_value(inner)
        }
    }
}

fn leading_ws_bytes(s: &str) -> usize {
    s.len() - s.trim_start().len()
}

fn extract_first_string(s: &str) -> Result<(String, usize), String> {
    let start = s.find('"').ok_or_else(|| "missing \"".to_string())?;
    let end = s[start + 1..]
        .find('"')
        .ok_or_else(|| "missing closing \"".to_string())?
        + start
        + 1;
    let content_start = start + 1;
    Ok((s[content_start..end].to_string(), content_start))
}

fn parse_num(a: &str) -> Result<f64, String> {
    a.trim()
        .trim_matches('"')
        .parse::<f64>()
        .map_err(|_| format!("bad number: {a}"))
}

/// Split `a:b:c` or whitespace; strips surrounding quotes.
fn parse_colon_list(args: &str) -> Vec<f64> {
    let raw = args.trim().trim_matches('"');
    if raw.is_empty() {
        return Vec::new();
    }
    let parts: Vec<&str> = if raw.contains(':') {
        raw.split(':').collect()
    } else {
        raw.split_whitespace().collect()
    };
    parts
        .into_iter()
        .filter_map(|p| p.trim().parse::<f64>().ok())
        .collect()
}

fn apply_cutoff_q(list: &[f64]) -> Result<(f32, Option<f32>), String> {
    if list.is_empty() {
        return Err("missing cutoff".into());
    }
    let cut = list[0] as f32;
    let q = list.get(1).map(|v| *v as f32);
    Ok((cut, q))
}

enum FilterDynKind {
    Lpf,
    Hpf,
    Bpf,
}

fn apply_filter_dyn(pc: &mut PatternCode, args: &str, kind: FilterDynKind) -> Result<(), String> {
    // Colon form "1000:8" stays fixed cutoff:Q.
    if is_colon_scalar_list(args) {
        let list = parse_colon_list(args);
        let (cut, q) = apply_cutoff_q(&list)?;
        match kind {
            FilterDynKind::Lpf => {
                pc.filter.lpf = Some(cut);
                pc.lpf_dyn = None;
                if let Some(q) = q {
                    pc.filter.lpq = q;
                }
            }
            FilterDynKind::Hpf => {
                pc.filter.hpf = Some(cut);
                pc.hpf_dyn = None;
                if let Some(q) = q {
                    pc.filter.hpq = q;
                }
            }
            FilterDynKind::Bpf => {
                pc.filter.bpf = Some(cut);
                pc.bpf_dyn = None;
                if let Some(q) = q {
                    pc.filter.bpq = q;
                }
            }
        }
        return Ok(());
    }
    match parse_dyn_f32(args)? {
        DynF32::Const(v) => match kind {
            FilterDynKind::Lpf => {
                pc.filter.lpf = Some(v);
                pc.lpf_dyn = None;
            }
            FilterDynKind::Hpf => {
                pc.filter.hpf = Some(v);
                pc.hpf_dyn = None;
            }
            FilterDynKind::Bpf => {
                pc.filter.bpf = Some(v);
                pc.bpf_dyn = None;
            }
        },
        other => match kind {
            FilterDynKind::Lpf => {
                // Base while continuous LFO runs (midpoint-ish).
                if let Some(spec) = other.as_lfo() {
                    pc.filter.lpf = Some(((spec.min + spec.max) * 0.5).clamp(20.0, 20_000.0));
                }
                pc.lpf_dyn = Some(other);
            }
            FilterDynKind::Hpf => {
                if let Some(spec) = other.as_lfo() {
                    pc.filter.hpf = Some(((spec.min + spec.max) * 0.5).clamp(20.0, 20_000.0));
                }
                pc.hpf_dyn = Some(other);
            }
            FilterDynKind::Bpf => {
                if let Some(spec) = other.as_lfo() {
                    pc.filter.bpf = Some(((spec.min + spec.max) * 0.5).clamp(20.0, 20_000.0));
                }
                pc.bpf_dyn = Some(other);
            }
        },
    }
    Ok(())
}

fn apply_pitch_add_dyn(pc: &mut PatternCode, args: &str, sign: f64) -> Result<(), String> {
    match parse_dyn_f32(args)? {
        DynF32::Const(v) => {
            pc.pitch_add += sign * f64::from(v);
            Ok(())
        }
        DynF32::Lfo(_) => Err(
            ".add/.sub LFO is not supported; use a mini pattern or scalar (vib for continuous pitch)"
                .into(),
        ),
        DynF32::Pattern(node) => {
            pc.pitch_add_dyn = Some(DynF32::Pattern(node));
            pc.pitch_add_dyn_sign = sign;
            Ok(())
        }
    }
}

fn apply_method(pc: &mut PatternCode, name: &str, args: &str) -> Result<(), String> {
    match name {
        "s" | "sound" => {
            pc.sound = args.trim().trim_matches('"').to_string();
            Ok(())
        }
        "gain" => match parse_dyn_f32(args)? {
            DynF32::Const(v) => {
                pc.gain = v;
                pc.gain_dyn = None;
                Ok(())
            }
            other => {
                pc.gain_dyn = Some(other);
                Ok(())
            }
        },
        "velocity" | "vel" => {
            pc.velocity = parse_num(args)? as f32;
            Ok(())
        }
        "pan" => match parse_dyn_f32(args)? {
            DynF32::Const(v) => {
                pc.pan = v.clamp(0.0, 1.0);
                pc.pan_dyn = None;
                Ok(())
            }
            other => {
                pc.pan_dyn = Some(other);
                Ok(())
            }
        },
        "lpf" | "cutoff" | "lp" | "ctf" => apply_filter_dyn(pc, args, FilterDynKind::Lpf),
        "lpq" | "resonance" => {
            pc.filter.lpq = parse_num(args)? as f32;
            Ok(())
        }
        "hpf" | "hp" | "hcutoff" => apply_filter_dyn(pc, args, FilterDynKind::Hpf),
        "hpq" | "hresonance" => {
            pc.filter.hpq = parse_num(args)? as f32;
            Ok(())
        }
        "bpf" | "bp" | "bandf" => apply_filter_dyn(pc, args, FilterDynKind::Bpf),
        "bpq" | "bandq" => {
            pc.filter.bpq = parse_num(args)? as f32;
            Ok(())
        }
        "attack" | "att" => {
            pc.adsr.attack = parse_num(args)? as f32;
            Ok(())
        }
        "decay" | "dec" => {
            pc.adsr.decay = parse_num(args)? as f32;
            Ok(())
        }
        "sustain" | "sus" => {
            pc.adsr.sustain = parse_num(args)? as f32;
            Ok(())
        }
        "release" | "rel" => {
            pc.adsr.release = parse_num(args)? as f32;
            Ok(())
        }
        "adsr" => {
            let raw = args.trim().trim_matches('"');
            let parts: Vec<&str> = if raw.contains(':') {
                raw.split(':').collect()
            } else {
                raw.split_whitespace().collect()
            };
            if parts.len() != 4 {
                return Err(format!("adsr expects 4 values, got {raw}"));
            }
            pc.adsr.attack = parse_num(parts[0])? as f32;
            pc.adsr.decay = parse_num(parts[1])? as f32;
            pc.adsr.sustain = parse_num(parts[2])? as f32;
            pc.adsr.release = parse_num(parts[3])? as f32;
            Ok(())
        }
        "vib" | "vibrato" | "v" => {
            let list = parse_colon_list(args);
            if list.is_empty() {
                return Err("vib expects frequency".into());
            }
            pc.mod_params.vib_hz = list[0] as f32;
            if let Some(d) = list.get(1) {
                pc.mod_params.vibmod = *d as f32;
            }
            Ok(())
        }
        "vibmod" | "vmod" => {
            let list = parse_colon_list(args);
            if list.is_empty() {
                return Err("vibmod expects depth".into());
            }
            pc.mod_params.vibmod = list[0] as f32;
            if let Some(f) = list.get(1) {
                pc.mod_params.vib_hz = *f as f32;
            }
            Ok(())
        }
        "fm" => {
            pc.mod_params.fm = parse_num(args)? as f32;
            Ok(())
        }
        "fmh" => {
            pc.mod_params.fmh = parse_num(args)? as f32;
            Ok(())
        }
        "fmattack" | "fmatt" => {
            pc.mod_params.fm_attack = parse_num(args)? as f32;
            Ok(())
        }
        "fmdecay" | "fmdec" => {
            pc.mod_params.fm_decay = parse_num(args)? as f32;
            Ok(())
        }
        "fmsustain" | "fmsus" => {
            pc.mod_params.fm_sustain = parse_num(args)? as f32;
            Ok(())
        }
        "noise" => {
            // Pink noise mix into oscillator (not the white/pink/brown sound sources).
            pc.mod_params.noise_mix = (parse_num(args)? as f32).clamp(0.0, 1.0);
            Ok(())
        }
        "penv" => {
            pc.mod_params.penv = parse_num(args)? as f32;
            Ok(())
        }
        "pattack" | "patt" => {
            pc.mod_params.patt = parse_num(args)? as f32;
            Ok(())
        }
        "pdecay" | "pdec" => {
            pc.mod_params.pdec = parse_num(args)? as f32;
            Ok(())
        }
        "lpenv" | "lpe" => {
            pc.mod_params.lpenv = parse_num(args)? as f32;
            Ok(())
        }
        "lpattack" | "lpa" => {
            pc.mod_params.lpa = parse_num(args)? as f32;
            Ok(())
        }
        "lpdecay" | "lpd" => {
            pc.mod_params.lpd = parse_num(args)? as f32;
            Ok(())
        }
        "lpsustain" | "lps" => {
            pc.mod_params.lps = parse_num(args)? as f32;
            Ok(())
        }
        "lprelease" | "lpr" => {
            pc.mod_params.lpr = parse_num(args)? as f32;
            Ok(())
        }
        "begin" => {
            pc.begin = parse_num(args)? as f32;
            Ok(())
        }
        "end" => {
            pc.end = parse_num(args)? as f32;
            Ok(())
        }
        "speed" => {
            pc.sample_speed = parse_num(args)? as f32;
            Ok(())
        }
        "slow" => {
            pc.speed /= parse_num(args)?;
            Ok(())
        }
        "fast" => {
            pc.speed *= parse_num(args)?;
            Ok(())
        }
        "bank" => {
            pc.bank = Some(args.trim().trim_matches('"').to_ascii_lowercase());
            Ok(())
        }
        "clip" => {
            pc.clip = Some(parse_num(args)? as f32);
            Ok(())
        }
        "legato" => {
            pc.legato = Some(parse_num(args)? as f32);
            Ok(())
        }
        "cut" => {
            pc.cut = Some(parse_num(args)? as i32);
            Ok(())
        }
        "orbit" | "o" => {
            let n = parse_num(args)? as i32;
            if n < 1 {
                return Err(format!("orbit must be >= 1, got {n}"));
            }
            pc.orbit = n.clamp(1, 4) as u8;
            Ok(())
        }
        "duckorbit" | "duck" => {
            let list = parse_colon_list(args);
            if list.is_empty() {
                return Err("duckorbit expects orbit id".into());
            }
            pc.duck.count = 0;
            for (i, v) in list.iter().take(4).enumerate() {
                let id = (*v as i32).clamp(1, 4) as u8;
                pc.duck.orbits[i] = id;
                pc.duck.count = (i + 1) as u8;
            }
            Ok(())
        }
        "duckattack" | "duckatt" | "datt" => {
            let list = parse_colon_list(args);
            if list.is_empty() {
                return Err("duckattack expects time".into());
            }
            for i in 0..4 {
                let v = list.get(i).or_else(|| list.first()).copied().unwrap_or(0.2);
                pc.duck.attack[i] = v as f32;
            }
            Ok(())
        }
        "duckdepth" => {
            let list = parse_colon_list(args);
            if list.is_empty() {
                return Err("duckdepth expects depth".into());
            }
            for i in 0..4 {
                let v = list.get(i).or_else(|| list.first()).copied().unwrap_or(1.0);
                pc.duck.depth[i] = (v as f32).clamp(0.0, 1.0);
            }
            Ok(())
        }
        "compressor" => {
            pc.compressor = Some(CompressorParams::parse(args)?);
            Ok(())
        }
        "delay" => {
            let list = parse_colon_list(args);
            if list.is_empty() {
                return Err("delay expects level".into());
            }
            pc.delay = (list[0] as f32).clamp(0.0, 1.0);
            if let Some(t) = list.get(1) {
                pc.delaytime = (*t as f32).max(0.0);
            }
            if let Some(f) = list.get(2) {
                pc.delayfeedback = (*f as f32).clamp(0.0, 0.95);
            }
            Ok(())
        }
        "delaytime" | "delayt" | "dt" => {
            pc.delaytime = (parse_num(args)? as f32).max(0.0);
            Ok(())
        }
        "delayfeedback" | "delayfb" | "dfb" => {
            pc.delayfeedback = (parse_num(args)? as f32).clamp(0.0, 0.95);
            Ok(())
        }
        "room" => {
            let list = parse_colon_list(args);
            if list.is_empty() {
                return Err("room expects level".into());
            }
            pc.room = (list[0] as f32).clamp(0.0, 1.0);
            if let Some(s) = list.get(1) {
                pc.roomsize = (*s as f32).clamp(0.0, 10.0);
            }
            Ok(())
        }
        "roomsize" | "rsize" | "sz" | "size" => {
            // `size` is Strudel synonym for roomsize; only used as room FX here.
            pc.roomsize = (parse_num(args)? as f32).clamp(0.0, 10.0);
            Ok(())
        }
        "note" => {
            // `cat(...).note()` — values are pitches (Strudel factory chain).
            pc.is_note = true;
            Ok(())
        }
        "scale" => {
            let s = args.trim().trim_matches('"');
            pc.scale = Some(crate::scale::parse_scale_arg(s)?);
            // Degree patterns only make sense as pitched events.
            pc.is_note = true;
            Ok(())
        }
        "add" => apply_pitch_add_dyn(pc, args, 1.0),
        "sub" => apply_pitch_add_dyn(pc, args, -1.0),
        "ply" => {
            let n = parse_num(args)?;
            if n.fract() != 0.0 || n < 1.0 {
                return Err(format!("ply expects integer >= 1, got {n}"));
            }
            let n = (n as u32).clamp(1, 16);
            pc.ply = pc.ply.saturating_mul(n).clamp(1, 16);
            Ok(())
        }
        "n" => {
            if args.trim().is_empty() {
                pc.is_note = true;
                return Ok(());
            }
            if let Ok(i) = parse_num(args) {
                pc.sample_n = Some(i as i32);
            }
            Ok(())
        }
        other => Err(format!("unsupported method: {other}")),
    }
}

// ── Structural chain edit (source-level; independent of PatternCode flatten) ─

/// One method call in a pattern chain: `.name(args)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodCall {
    pub name: String,
    /// Raw argument text inside the parentheses (no surrounding `()`).
    pub args: String,
}

/// Editable view of `note("…").s("saw").lpf(400)` (head + ordered methods).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainEdit {
    pub head: String,
    pub head_args: String,
    pub methods: Vec<MethodCall>,
}

/// How to change a method on a [`ChainEdit`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodEditOp {
    /// Replace the last occurrence of `method`, or append if missing.
    Set,
    /// Always append a new call at the end of the chain.
    Add,
    /// Remove the last occurrence of `method` (error if none).
    Remove,
}

impl MethodEditOp {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.trim().to_ascii_lowercase().as_str() {
            "set" => Ok(Self::Set),
            "add" => Ok(Self::Add),
            "remove" | "rm" | "del" | "delete" => Ok(Self::Remove),
            other => Err(format!(
                "unknown method op: {other} (use set, add, or remove)"
            )),
        }
    }
}

/// Parse a method chain into head + methods without applying semantics.
pub fn parse_chain_for_edit(input: &str) -> Result<ChainEdit, String> {
    let input = input.trim().trim_end_matches(';').trim();
    if input.is_empty() {
        return Err("empty chain".into());
    }
    if !parens_balanced(input) {
        return Err("unbalanced parens".into());
    }
    let open = input.find('(').ok_or_else(|| "missing (".to_string())?;
    let head = input[..open].trim();
    if !matches!(head, "note" | "n" | "s" | "sound" | "cat" | "slowcat") {
        return Err(format!("unknown head: {head} (use note/s/cat)"));
    }
    let (head_args, after_head) =
        match_parens(&input[open..]).ok_or_else(|| "unbalanced parens in head".to_string())?;

    let mut methods = Vec::new();
    let mut cur = after_head;
    while let Some(dot_rel) = cur.find('.') {
        let after = &cur[dot_rel + 1..];
        let paren = after
            .find('(')
            .ok_or_else(|| "missing ( in method".to_string())?;
        let name = after[..paren].trim();
        if name.is_empty() {
            return Err("empty method name".into());
        }
        let (args, rest) = match_parens(&after[paren..])
            .ok_or_else(|| format!("unbalanced parens in method .{name}"))?;
        methods.push(MethodCall {
            name: name.to_string(),
            args: args.to_string(),
        });
        cur = rest;
    }
    let trailing = cur.trim();
    if !trailing.is_empty() {
        return Err(format!("trailing junk after chain: {trailing}"));
    }

    Ok(ChainEdit {
        head: head.to_string(),
        head_args: head_args.to_string(),
        methods,
    })
}

/// Apply a method-level edit. `args` is the inside-parens text (ignored for Remove).
pub fn apply_method_edit(
    chain: &mut ChainEdit,
    op: MethodEditOp,
    method: &str,
    args: &str,
) -> Result<(), String> {
    let method = method.trim();
    if method.is_empty() {
        return Err("method name required".into());
    }
    // Reject head names so callers use patch_track instead.
    if matches!(method, "note" | "n" | "s" | "sound" | "cat" | "slowcat") {
        return Err(format!(
            "cannot edit head factory `{method}` via edit_method; use patch_track"
        ));
    }

    match op {
        MethodEditOp::Add => {
            chain.methods.push(MethodCall {
                name: method.to_string(),
                args: args.to_string(),
            });
        }
        MethodEditOp::Set => {
            if let Some(i) = chain
                .methods
                .iter()
                .rposition(|m| m.name.eq_ignore_ascii_case(method))
            {
                chain.methods[i].name = method.to_string();
                chain.methods[i].args = args.to_string();
            } else {
                chain.methods.push(MethodCall {
                    name: method.to_string(),
                    args: args.to_string(),
                });
            }
        }
        MethodEditOp::Remove => {
            let Some(i) = chain
                .methods
                .iter()
                .rposition(|m| m.name.eq_ignore_ascii_case(method))
            else {
                return Err(format!("method not found: .{method}"));
            };
            chain.methods.remove(i);
        }
    }
    Ok(())
}

/// Serialize a chain back to source text.
pub fn serialize_chain(chain: &ChainEdit) -> String {
    let mut out = format!("{}({})", chain.head, chain.head_args);
    for m in &chain.methods {
        out.push('.');
        out.push_str(&m.name);
        out.push('(');
        out.push_str(&m.args);
        out.push(')');
    }
    out
}

/// Convenience: parse → edit → serialize, then validate with [`parse_code`].
pub fn edit_method_on_code(
    code: &str,
    op: MethodEditOp,
    method: &str,
    args: &str,
) -> Result<String, String> {
    let mut chain = parse_chain_for_edit(code)?;
    apply_method_edit(&mut chain, op, method, args)?;
    let out = serialize_chain(&chain);
    // Reject chains the player cannot parse.
    parse_code(&out)?;
    Ok(out)
}

/// Parse a note name (optional chord suffix stripped) into MIDI number as `i32`.
/// A4 = 69. Chord tokens like `c3'maj` use the root only (matches Deck scheduling).
pub fn note_to_midi_i32(note: &str) -> Result<i32, String> {
    if note.is_empty() {
        return Err("empty note".into());
    }
    let note = note.split('\'').next().unwrap_or(note);
    let bytes = note.as_bytes();
    let mut i = 0;
    let letter = bytes[0].to_ascii_lowercase() as char;
    i += 1;
    let mut semis = match letter {
        'c' => 0,
        'd' => 2,
        'e' => 4,
        'f' => 5,
        'g' => 7,
        'a' => 9,
        'b' => 11,
        _ => return Err(format!("bad note: {note}")),
    };
    while i < bytes.len() {
        match bytes[i] as char {
            '#' => {
                semis += 1;
                i += 1;
            }
            'b' => {
                semis -= 1;
                i += 1;
            }
            _ => break,
        }
    }
    let oct_str = &note[i..];
    let oct: i32 = oct_str
        .parse()
        .map_err(|_| format!("bad octave in {note}"))?;
    Ok((oct + 1) * 12 + semis)
}

/// Single note name → MIDI 0..=127 (A4 = 69). Chord suffix uses root only.
pub fn note_to_midi(note: &str) -> Result<u8, String> {
    let midi = note_to_midi_i32(note)?;
    if (0..=127).contains(&midi) {
        Ok(midi as u8)
    } else {
        Err(format!("midi out of range for {note}: {midi}"))
    }
}

/// MIDI note number → Hz (A4 = 440).
pub fn midi_to_hz(midi: i32) -> f32 {
    440.0 * 2f32.powf((midi - 69) as f32 / 12.0)
}

/// Single note name → Hz (A4 = 440).
pub fn note_to_hz(note: &str) -> Result<f32, String> {
    let midi = note_to_midi_i32(note)?;
    Ok(midi_to_hz(midi))
}

/// `c3'maj` / `c3'min7` → chord tones as note names. Plain notes → one element.
pub fn expand_chord(token: &str) -> Result<Vec<String>, String> {
    if !token.contains('\'') {
        note_to_hz(token)?;
        return Ok(vec![token.to_string()]);
    }
    let (root_part, quality) = token
        .split_once('\'')
        .ok_or_else(|| format!("bad chord token: {token}"))?;
    let _ = note_to_hz(root_part)?;

    let q = quality.to_ascii_lowercase();
    let intervals: &[i32] = match q.as_str() {
        "maj" | "major" => &[0, 4, 7],
        "min" | "minor" | "m" => &[0, 3, 7],
        "maj7" => &[0, 4, 7, 11],
        "min7" | "m7" => &[0, 3, 7, 10],
        "dim" => &[0, 3, 6],
        "aug" => &[0, 4, 8],
        "sus2" => &[0, 2, 7],
        "sus4" => &[0, 5, 7],
        other => return Err(format!("unsupported chord quality: {other}")),
    };

    let (letter_acc, oct) = split_root(root_part)?;
    let root_semi = pitch_class(&letter_acc)?;
    let mut out = Vec::with_capacity(intervals.len());
    for &iv in intervals {
        let total = root_semi + iv;
        let pc = total.rem_euclid(12);
        let oct_off = total.div_euclid(12);
        out.push(format_note(pc, oct + oct_off));
    }
    Ok(out)
}

fn split_root(root: &str) -> Result<(String, i32), String> {
    let bytes = root.as_bytes();
    if bytes.is_empty() {
        return Err("empty root".into());
    }
    let mut i = 1;
    while i < bytes.len() && (bytes[i] == b'#' || bytes[i] == b'b') {
        if bytes[i].is_ascii_digit() {
            break;
        }
        i += 1;
    }
    let letter_acc = root[..i].to_string();
    let oct: i32 = root[i..]
        .parse()
        .map_err(|_| format!("bad octave in {root}"))?;
    Ok((letter_acc, oct))
}

fn pitch_class(name: &str) -> Result<i32, String> {
    let mut chars = name.chars();
    let letter = chars
        .next()
        .ok_or_else(|| "empty pitch".to_string())?
        .to_ascii_lowercase();
    let mut semis: i32 = match letter {
        'c' => 0,
        'd' => 2,
        'e' => 4,
        'f' => 5,
        'g' => 7,
        'a' => 9,
        'b' => 11,
        _ => return Err(format!("bad pitch class: {name}")),
    };
    for c in chars {
        match c {
            '#' => semis += 1,
            'b' => semis -= 1,
            _ => return Err(format!("bad pitch class: {name}")),
        }
    }
    Ok(semis.rem_euclid(12))
}

fn format_note(pc: i32, oct: i32) -> String {
    let names = [
        "c", "c#", "d", "d#", "e", "f", "f#", "g", "g#", "a", "a#", "b",
    ];
    let pc = pc.rem_euclid(12);
    format!("{}{}", names[pc as usize], oct)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_code() {
        let pc =
            parse_code(r#"note("c3 e3 [g3 ~]").s("sawtooth").lpf(800).gain(0.4).slow(2)"#).unwrap();
        assert!(pc.is_note);
        assert_eq!(pc.sound, "sawtooth");
        assert_eq!(pc.lpf(), Some(800.0));
        assert!((pc.speed - 0.5).abs() < 1e-9);
    }

    #[test]
    fn sample_head() {
        let pc = parse_code(r#"s("bd sd hh*2").gain(0.8)"#).unwrap();
        assert!(!pc.is_note);
        assert_eq!(pc.sound, "bd");
        assert!((pc.pan - 0.5).abs() < 1e-6);
    }

    #[test]
    fn parses_pan() {
        let pc = parse_code(r#"s("bd").pan(0)"#).unwrap();
        assert!((pc.pan - 0.0).abs() < 1e-6);
        let pc = parse_code(r#"note("c3").s("sawtooth").pan(1)"#).unwrap();
        assert!((pc.pan - 1.0).abs() < 1e-6);
        let pc = parse_code(r#"s("hh").pan(0.25)"#).unwrap();
        assert!((pc.pan - 0.25).abs() < 1e-6);
    }

    #[test]
    fn note_default_sound_is_triangle() {
        let pc = parse_code(r#"note("c3")"#).unwrap();
        assert_eq!(pc.sound, "triangle");
    }

    #[test]
    fn note_freqs() {
        assert!((note_to_hz("a4").unwrap() - 440.0).abs() < 0.01);
        assert!((note_to_hz("c3").unwrap() - 130.81).abs() < 0.5);
        assert!((note_to_hz("c#4").unwrap() - 277.18).abs() < 0.5);
    }

    #[test]
    fn note_midi_numbers() {
        assert_eq!(note_to_midi("a4").unwrap(), 69);
        assert_eq!(note_to_midi("c4").unwrap(), 60);
        assert_eq!(note_to_midi("c3'maj").unwrap(), 48); // root only
    }

    #[test]
    fn rejects_bad() {
        assert!(parse_code("stack(\"a\")").is_err());
        assert!(parse_code("note(\"c3\"").is_err());
        assert!(parse_code(r#"note("c3").unknown(1)"#).is_err());
    }

    #[test]
    fn cat_factory_note_and_sound() {
        let pc = parse_code(r#"note(cat("c2 eb2", "g2 bb2")).s("sawtooth").gain(0.5)"#).unwrap();
        assert!(pc.is_note);
        assert_eq!(pc.sound, "sawtooth");
        match &pc.pattern {
            Node::Stack(items) => assert_eq!(items.len(), 2),
            other => panic!("expected Stack, got {other:?}"),
        }
        let e0 = mini::events(&pc.pattern, 0);
        assert_eq!(e0[0].value, "c2");
        let e1 = mini::events(&pc.pattern, 1);
        assert_eq!(e1[0].value, "g2");

        let drum = parse_code(r#"s(cat("bd ~ ~ ~", "bd ~ ~ bd")).gain(0.9)"#).unwrap();
        assert!(!drum.is_note);
        assert_eq!(drum.sound, "bd");
        match &drum.pattern {
            Node::Stack(items) => assert_eq!(items.len(), 2),
            other => panic!("expected Stack, got {other:?}"),
        }
    }

    #[test]
    fn cat_nested_flattens() {
        let pc = parse_code(r#"s(cat(cat("bd ~", "sd ~"), "hh ~", "oh ~"))"#).unwrap();
        match &pc.pattern {
            Node::Stack(items) => assert_eq!(items.len(), 4),
            other => panic!("expected Stack, got {other:?}"),
        }
        assert_eq!(mini::events(&pc.pattern, 2)[0].value, "hh");
    }

    #[test]
    fn cat_head_with_note_method() {
        let pc = parse_code(r#"cat("c2", "e2").note().s("square")"#).unwrap();
        assert!(pc.is_note);
        assert_eq!(pc.sound, "square");
    }

    #[test]
    fn trailing_semicolon_ok() {
        let pc = parse_code(r#"s("bd*4").gain(0.9);"#).unwrap();
        assert_eq!(pc.sound, "bd");
    }

    #[test]
    fn adsr_chain() {
        let pc = parse_code(r#"note("c3").adsr("0.02:0.1:0.5:0.2")"#).unwrap();
        assert!((pc.adsr.attack - 0.02).abs() < 1e-6);
        assert!((pc.adsr.sustain - 0.5).abs() < 1e-6);
    }

    #[test]
    fn expand_maj() {
        let notes = expand_chord("c3'maj").unwrap();
        assert_eq!(notes, vec!["c3", "e3", "g3"]);
    }

    #[test]
    fn expand_min7() {
        let notes = expand_chord("a2'min7").unwrap();
        assert_eq!(notes, vec!["a2", "c3", "e3", "g3"]);
    }

    #[test]
    fn parses_scale_degrees() {
        let pc = parse_code(r#"note("0 2 3").scale("C2:minor").s("sawtooth").gain(0.5)"#).unwrap();
        assert!(pc.is_note);
        assert!(pc.scale.is_some());
        let sc = pc.scale.as_ref().unwrap().at_cycle(0);
        assert!((sc.degree_to_hz(0) - note_to_hz("c2").unwrap()).abs() < 0.5);
        assert!((sc.degree_to_hz(-1) - note_to_hz("bb1").unwrap()).abs() < 0.5);
    }

    #[test]
    fn parses_scale_progression() {
        let pc = parse_code(
            r#"note("0 2 4 0").scale("<A2:minor D:dorian G:mixolydian C:major>").s("sawtooth")"#,
        )
        .unwrap();
        let pat = pc.scale.as_ref().unwrap();
        assert_eq!(pat.at_cycle(0).root_midi, note_to_midi_i32("a2").unwrap());
        assert_eq!(pat.at_cycle(2).root_midi, note_to_midi_i32("g4").unwrap());
    }

    #[test]
    fn scale_unknown_mode_errors() {
        assert!(parse_code(r#"note("0").scale("C:nope")"#).is_err());
    }

    #[test]
    fn parses_add_sub_accumulate() {
        let pc = parse_code(r#"note("0 2 4").scale("C2:major").add(2).sub(1).s("sine")"#).unwrap();
        assert!((pc.pitch_add - 1.0).abs() < 1e-9);
        let pc2 = parse_code(r#"note("c4").add(12).s("sine")"#).unwrap();
        assert!((pc2.pitch_add - 12.0).abs() < 1e-9);
    }

    #[test]
    fn parses_dyn_lpf_pattern_and_lfo() {
        let pc = parse_code(r#"s("bd*4").lpf("<400 1200>").gain(0.5)"#).unwrap();
        assert!(matches!(
            pc.lpf_dyn,
            Some(crate::control::DynF32::Pattern(_))
        ));
        let pc2 = parse_code(r#"note("c3").s("sawtooth").lpf(sine.rangex(500, 4000)).gain(0.4)"#)
            .unwrap();
        match pc2.lpf_dyn.as_ref().unwrap() {
            crate::control::DynF32::Lfo(s) => {
                assert!(s.exponential);
                assert!((s.min - 500.0).abs() < 0.1);
            }
            _ => panic!("expected LFO"),
        }
        let pc3 = parse_code(r#"note("0").scale("C4:major").add("<0 2>").s("sine")"#).unwrap();
        assert!(pc3.pitch_add_dyn.is_some());
    }

    #[test]
    fn parses_cutoff_alias_with_lfo() {
        let pc = parse_code(r#"s("hh*8").cutoff(sine.range(200, 8000).slow(2))"#).unwrap();
        match pc.lpf_dyn.as_ref().unwrap() {
            crate::control::DynF32::Lfo(s) => {
                assert!(!s.exponential);
                assert!((s.cycles_per_bar - 0.5).abs() < 1e-5);
            }
            _ => panic!("expected LFO"),
        }
    }

    #[test]
    fn parses_ply_multiply_and_clamp() {
        let pc = parse_code(r#"s("bd").ply(2).ply(2)"#).unwrap();
        assert_eq!(pc.ply, 4);
        let pc2 = parse_code(r#"s("bd").ply(16).ply(2)"#).unwrap();
        assert_eq!(pc2.ply, 16); // clamp
        assert!(parse_code(r#"s("bd").ply(0)"#).is_err());
        assert!(parse_code(r#"s("bd").ply(1.5)"#).is_err());
        assert!(parse_code(r#"s("bd").ply("<2 4>")"#).is_err());
    }

    #[test]
    fn sample_track_accepts_add_without_pitch_use() {
        // Parse succeeds; pitch_add is only applied on note paths at schedule time.
        let pc = parse_code(r#"s("bd").add(2).gain(0.5)"#).unwrap();
        assert!(!pc.is_note);
        assert!((pc.pitch_add - 2.0).abs() < 1e-9);
    }

    #[test]
    fn expand_plain_note() {
        assert_eq!(expand_chord("d#4").unwrap(), vec!["d#4".to_string()]);
    }

    #[test]
    fn expand_unknown_quality() {
        assert!(expand_chord("c3'foo").is_err());
    }

    #[test]
    fn mini_src_and_relative_base() {
        let code = r#"s("bd*4").gain(0.9)"#;
        let pc = parse_code(code).unwrap();
        assert_eq!(pc.mini_src, "bd*4");
        assert_eq!(
            &code[pc.mini_base..pc.mini_base + pc.mini_src.len()],
            "bd*4"
        );
    }

    #[test]
    fn vib_colon_and_orbit_duck() {
        let pc = parse_code(
            r#"note("c3").s("sawtooth").vib("4:12").orbit(2).duckorbit("2:3").duckattack(0.15).duckdepth("1:0.5")"#,
        )
        .unwrap();
        assert!((pc.mod_params.vib_hz - 4.0).abs() < 1e-6);
        assert!((pc.mod_params.vibmod - 12.0).abs() < 1e-6);
        assert_eq!(pc.orbit, 2);
        assert_eq!(pc.duck.count, 2);
        assert_eq!(pc.duck.orbits[0], 2);
        assert_eq!(pc.duck.orbits[1], 3);
        assert!((pc.duck.depth[1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn filter_and_bank_clip() {
        let pc =
            parse_code(r#"s("bd").bank("tr808").hpf(200).lpq(2).lpf("1000:8").clip(0.5).cut(1)"#)
                .unwrap();
        assert_eq!(pc.bank.as_deref(), Some("tr808"));
        assert_eq!(pc.filter.hpf, Some(200.0));
        assert!((pc.filter.lpq - 8.0).abs() < 1e-6);
        assert_eq!(pc.filter.lpf, Some(1000.0));
        assert_eq!(pc.clip, Some(0.5));
        assert_eq!(pc.cut, Some(1));
    }

    #[test]
    fn compressor_method() {
        let pc = parse_code(r#"s("bd").compressor("-20:4:6:.003:.1")"#).unwrap();
        let c = pc.compressor.unwrap();
        assert!((c.threshold_db + 20.0).abs() < 1e-5);
        assert!((c.ratio - 4.0).abs() < 1e-5);
    }

    #[test]
    fn delay_room_parse() {
        let pc = parse_code(r#"s("bd").delay("0.5:0.25:0.8").room(0.3).roomsize(2)"#).unwrap();
        assert!((pc.delay - 0.5).abs() < 1e-6);
        assert!((pc.delaytime - 0.25).abs() < 1e-6);
        assert!((pc.delayfeedback - 0.8).abs() < 1e-6);
        assert!((pc.room - 0.3).abs() < 1e-6);
        assert!((pc.roomsize - 2.0).abs() < 1e-6);

        let pc2 =
            parse_code(r#"s("hh").delay(0.4).delaytime(0.125).delayfeedback(0.6).room("0.9:4")"#)
                .unwrap();
        assert!((pc2.delay - 0.4).abs() < 1e-6);
        assert!((pc2.delaytime - 0.125).abs() < 1e-6);
        assert!((pc2.delayfeedback - 0.6).abs() < 1e-6);
        assert!((pc2.room - 0.9).abs() < 1e-6);
        assert!((pc2.roomsize - 4.0).abs() < 1e-6);
    }

    #[test]
    fn chain_edit_roundtrip_and_set_lpf() {
        let src = r#"note("c2 c2").s("sawtooth").gain(0.7)"#;
        let chain = parse_chain_for_edit(src).unwrap();
        assert_eq!(chain.head, "note");
        assert_eq!(chain.methods.len(), 2);
        assert_eq!(serialize_chain(&chain), src);

        let out = edit_method_on_code(src, MethodEditOp::Set, "lpf", "400").unwrap();
        assert_eq!(out, r#"note("c2 c2").s("sawtooth").gain(0.7).lpf(400)"#);
        let pc = parse_code(&out).unwrap();
        assert_eq!(pc.lpf(), Some(400.0));
    }

    #[test]
    fn chain_edit_set_replaces_last_same_method() {
        let src = r#"note("c3").s("saw").lpf(200).gain(0.5).lpf(800)"#;
        let out = edit_method_on_code(src, MethodEditOp::Set, "lpf", "1200").unwrap();
        assert_eq!(out, r#"note("c3").s("saw").lpf(200).gain(0.5).lpf(1200)"#);
    }

    #[test]
    fn chain_edit_remove_and_nested_args() {
        let src = r#"note("0 2").s("sawtooth").lpf(sine.rangex(500,4000)).gain(0.6)"#;
        let chain = parse_chain_for_edit(src).unwrap();
        assert_eq!(chain.methods[1].args, "sine.rangex(500,4000)");
        let out = edit_method_on_code(src, MethodEditOp::Remove, "gain", "").unwrap();
        assert_eq!(
            out,
            r#"note("0 2").s("sawtooth").lpf(sine.rangex(500,4000))"#
        );
        assert!(edit_method_on_code(src, MethodEditOp::Remove, "room", "").is_err());
    }

    #[test]
    fn chain_edit_add_appends() {
        let src = r#"s("bd*4").gain(0.9)"#;
        let out = edit_method_on_code(src, MethodEditOp::Add, "hpf", "200").unwrap();
        assert_eq!(out, r#"s("bd*4").gain(0.9).hpf(200)"#);
    }
}
