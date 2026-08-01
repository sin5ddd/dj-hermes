//! Dynamic method arguments: mini-notation number patterns and continuous LFOs.
//!
//! Supported forms (method args):
//! - scalar: `800`, `"800"`
//! - mini pattern: `"<400 1200>"`, `"400 800 1200"`
//! - LFO: `sine.rangex(500, 4000)`, `sine.range(200, 2000)`, optional `.slow(n)` / `.fast(n)`
//!
//! Strudel-compatible notes:
//! - `sine` / `tri` / `saw` / `square` / `cosine` produce **0..1** unit signals (1 cycle per bar by default).
//! - `.range(min, max)` maps linearly; `.rangex(min, max)` maps exponentially (good for Hz).

use crate::mini::{self, Node};

/// Waveform for continuous control signals (0..1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LfoShape {
    Sine,
    Cosine,
    Tri,
    Saw,
    Square,
}

/// Continuous LFO mapped into `[min, max]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LfoSpec {
    pub shape: LfoShape,
    pub min: f32,
    pub max: f32,
    /// `true` = exponential map (Strudel `rangex`); `false` = linear (`range`).
    pub exponential: bool,
    /// LFO cycles per bar (cycle). Default 1.0; `.slow(4)` → 0.25; `.fast(2)` → 2.0.
    pub cycles_per_bar: f32,
}

impl LfoSpec {
    /// Unit signal 0..1 at phase `p` (unbounded; uses `fract`).
    pub fn unit_at_phase(&self, phase: f32) -> f32 {
        let p = phase.fract().rem_euclid(1.0);
        match self.shape {
            LfoShape::Sine => 0.5 + 0.5 * (p * std::f32::consts::TAU).sin(),
            LfoShape::Cosine => 0.5 + 0.5 * (p * std::f32::consts::TAU).cos(),
            LfoShape::Saw => p,
            LfoShape::Square => {
                if p < 0.5 {
                    0.0
                } else {
                    1.0
                }
            }
            LfoShape::Tri => {
                if p < 0.5 {
                    p * 2.0
                } else {
                    2.0 - p * 2.0
                }
            }
        }
    }

    /// Mapped value at cycle phase `bar_phase` in 0..1 (position within bar) plus free-run offset.
    ///
    /// `bar_phase`: event start or continuous (pos / samples_per_bar) fraction within the bar.
    pub fn value_at_bar_phase(&self, bar_phase: f32) -> f32 {
        let phase = bar_phase * self.cycles_per_bar.max(1e-6);
        let u = self.unit_at_phase(phase).clamp(0.0, 1.0);
        map_unit(u, self.min, self.max, self.exponential)
    }
}

fn map_unit(u: f32, min: f32, max: f32, exponential: bool) -> f32 {
    let u = u.clamp(0.0, 1.0);
    if exponential {
        let lo = min.max(1e-3);
        let hi = max.max(lo * 1.001);
        lo * (hi / lo).powf(u)
    } else {
        min + (max - min) * u
    }
}

/// Scalar, mini number pattern, or continuous LFO.
#[derive(Debug, Clone)]
pub enum DynF32 {
    Const(f32),
    Pattern(Node),
    Lfo(LfoSpec),
}

impl DynF32 {
    /// Sample for one event at bar `cycle`, phase `t` in 0..1 within the bar.
    pub fn sample_at(&self, cycle: u64, t: f64) -> Option<f32> {
        match self {
            DynF32::Const(v) => Some(*v),
            DynF32::Pattern(node) => sample_pattern_f32(node, cycle, t),
            DynF32::Lfo(spec) => Some(spec.value_at_bar_phase(t as f32)),
        }
    }

    pub fn as_lfo(&self) -> Option<LfoSpec> {
        match self {
            DynF32::Lfo(s) => Some(*s),
            _ => None,
        }
    }
}

/// Find the mini event covering time `t` and parse its atom as f32.
pub fn sample_pattern_f32(node: &Node, cycle: u64, t: f64) -> Option<f32> {
    let evs = mini::events(node, cycle);
    if evs.is_empty() {
        return None;
    }
    let t = t.clamp(0.0, 0.999_999);
    for e in &evs {
        if e.value == "~" || e.value.is_empty() {
            continue;
        }
        if t + 1e-12 >= e.start && t < e.start + e.dur {
            return parse_atom_f32(&e.value);
        }
    }
    // Fallback: last non-rest before t, else first numeric.
    let mut last = None;
    for e in &evs {
        if e.value == "~" || e.value.is_empty() {
            continue;
        }
        if e.start <= t + 1e-12 {
            last = parse_atom_f32(&e.value);
        }
    }
    last.or_else(|| {
        evs.iter()
            .find_map(|e| parse_atom_f32(&e.value).filter(|_| e.value != "~"))
    })
}

fn parse_atom_f32(s: &str) -> Option<f32> {
    s.trim().parse::<f32>().ok()
}

/// Parse a method argument into [`DynF32`].
///
/// Accepts scalars, quoted mini patterns, and `sine.rangex(a,b)[.slow(n)|.fast(n)]…`.
pub fn parse_dyn_f32(args: &str) -> Result<DynF32, String> {
    let raw = args.trim();
    if raw.is_empty() {
        return Err("empty dynamic argument".into());
    }

    // Whole-string quotes → mini or scalar string contents.
    if raw.starts_with('"') {
        let (inner, _) = extract_quoted(raw)?;
        let inner = inner.trim();
        if let Ok(n) = inner.parse::<f32>() {
            return Ok(DynF32::Const(n));
        }
        let node = mini::parse(inner).map_err(|e| format!("dyn pattern: {e}"))?;
        return Ok(DynF32::Pattern(node));
    }

    // Bare scalar.
    if let Ok(n) = raw.parse::<f32>() {
        return Ok(DynF32::Const(n));
    }

    // LFO expression.
    if let Some(spec) = try_parse_lfo(raw)? {
        return Ok(DynF32::Lfo(spec));
    }

    // Bare mini (rare unquoted).
    if raw.contains('<')
        || raw.contains('[')
        || raw.contains('*')
        || raw.split_whitespace().count() > 1
    {
        let node = mini::parse(raw).map_err(|e| format!("dyn pattern: {e}"))?;
        return Ok(DynF32::Pattern(node));
    }

    Err(format!(
        "expected number, mini pattern, or sine.rangex(min,max); got {raw:?}"
    ))
}

fn extract_quoted(s: &str) -> Result<(String, usize), String> {
    let start = s.find('"').ok_or_else(|| "missing \"".to_string())?;
    let end = s[start + 1..]
        .find('"')
        .ok_or_else(|| "missing closing \"".to_string())?
        + start
        + 1;
    Ok((s[start + 1..end].to_string(), start + 1))
}

/// `sine.rangex(500, 4000)`, `tri.range(0,1).slow(4)`, etc.
fn try_parse_lfo(s: &str) -> Result<Option<LfoSpec>, String> {
    let s = s.trim();
    let shape = if let Some(rest) = s.strip_prefix("sine") {
        (LfoShape::Sine, rest)
    } else if let Some(rest) = s.strip_prefix("cosine") {
        (LfoShape::Cosine, rest)
    } else if let Some(rest) = s.strip_prefix("tri") {
        (LfoShape::Tri, rest)
    } else if let Some(rest) = s.strip_prefix("saw") {
        (LfoShape::Saw, rest)
    } else if let Some(rest) = s.strip_prefix("square") {
        (LfoShape::Square, rest)
    } else {
        return Ok(None);
    };
    let (shape, mut rest) = shape;
    rest = rest.trim_start();

    // Require .range or .rangex
    let (exponential, after_range) = if let Some(r) = rest.strip_prefix(".rangex") {
        (true, r.trim_start())
    } else if let Some(r) = rest.strip_prefix(".range") {
        (false, r.trim_start())
    } else {
        return Err(format!(
            "LFO needs .range(min,max) or .rangex(min,max), got {s:?}"
        ));
    };

    let (range_args, after_call) =
        match_parens(after_range).ok_or_else(|| format!("unbalanced parens in LFO range: {s}"))?;
    let (min, max) = parse_two_nums(range_args)?;
    let mut cycles = 1.0f32;
    let mut cur = after_call.trim_start();
    // Optional chain: .slow(n) / .fast(n)
    while let Some(dot) = cur.strip_prefix('.') {
        let paren = dot
            .find('(')
            .ok_or_else(|| format!("missing ( after LFO method in {s}"))?;
        let name = dot[..paren].trim();
        let (args, rest) =
            match_parens(&dot[paren..]).ok_or_else(|| format!("unbalanced parens in .{name}"))?;
        let n = args
            .trim()
            .parse::<f32>()
            .map_err(|_| format!("bad number in .{name}({args})"))?;
        match name {
            "slow" => {
                if n <= 0.0 {
                    return Err(".slow expects > 0".into());
                }
                cycles /= n;
            }
            "fast" => {
                if n <= 0.0 {
                    return Err(".fast expects > 0".into());
                }
                cycles *= n;
            }
            other => {
                return Err(format!("unsupported LFO chain .{other} (only .slow/.fast)"));
            }
        }
        cur = rest.trim_start();
    }
    if !cur.is_empty() {
        return Err(format!("trailing junk after LFO: {cur:?}"));
    }

    Ok(Some(LfoSpec {
        shape,
        min,
        max,
        exponential,
        cycles_per_bar: cycles.max(1e-6),
    }))
}

fn parse_two_nums(args: &str) -> Result<(f32, f32), String> {
    let parts: Vec<&str> = args
        .split(',')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();
    if parts.len() != 2 {
        return Err(format!("range expects min, max; got {args:?}"));
    }
    let a = parts[0]
        .parse::<f32>()
        .map_err(|_| format!("bad range min: {}", parts[0]))?;
    let b = parts[1]
        .parse::<f32>()
        .map_err(|_| format!("bad range max: {}", parts[1]))?;
    Ok((a, b))
}

fn match_parens(s: &str) -> Option<(&str, &str)> {
    let s = s.trim_start();
    if !s.starts_with('(') {
        return None;
    }
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
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

/// True if args look like `"a:b"` numeric colon list (not a mini pattern).
pub fn is_colon_scalar_list(args: &str) -> bool {
    let raw = args.trim().trim_matches('"');
    if !raw.contains(':') {
        return false;
    }
    raw.chars()
        .all(|c| c.is_ascii_digit() || matches!(c, '.' | ':' | '-' | '+' | 'e' | 'E' | ' '))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_const_and_pattern() {
        match parse_dyn_f32("800").unwrap() {
            DynF32::Const(v) => assert!((v - 800.0).abs() < 0.1),
            _ => panic!("const"),
        }
        match parse_dyn_f32("\"<400 1200>\"").unwrap() {
            DynF32::Pattern(_) => {}
            _ => panic!("pattern"),
        }
    }

    #[test]
    fn parse_sine_rangex_slow() {
        let d = parse_dyn_f32("sine.rangex(500, 4000).slow(2)").unwrap();
        match d {
            DynF32::Lfo(s) => {
                assert!(s.exponential);
                assert!((s.min - 500.0).abs() < 0.1);
                assert!((s.max - 4000.0).abs() < 0.1);
                assert!((s.cycles_per_bar - 0.5).abs() < 1e-5);
                assert_eq!(s.shape, LfoShape::Sine);
            }
            _ => panic!("lfo"),
        }
    }

    #[test]
    fn rangex_maps_endpoints() {
        let s = LfoSpec {
            shape: LfoShape::Saw, // unit_at 0 = 0, at ~1 = ~1
            min: 100.0,
            max: 1000.0,
            exponential: true,
            cycles_per_bar: 1.0,
        };
        // phase 0 → unit 0 for saw → min
        assert!((s.value_at_bar_phase(0.0) - 100.0).abs() < 1.0);
        // phase ~1 → near max (use 0.999)
        let v = s.value_at_bar_phase(0.999);
        assert!(v > 900.0, "got {v}");
    }

    #[test]
    fn pattern_samples_by_phase() {
        let node = mini::parse("100 900").unwrap();
        let a = sample_pattern_f32(&node, 0, 0.1).unwrap();
        let b = sample_pattern_f32(&node, 0, 0.6).unwrap();
        assert!((a - 100.0).abs() < 0.1);
        assert!((b - 900.0).abs() < 0.1);
    }

    #[test]
    fn stack_pattern_cycles() {
        let node = mini::parse("<100 900>").unwrap();
        let a = sample_pattern_f32(&node, 0, 0.5).unwrap();
        let b = sample_pattern_f32(&node, 1, 0.5).unwrap();
        assert!((a - 100.0).abs() < 0.1);
        assert!((b - 900.0).abs() < 0.1);
    }
}
