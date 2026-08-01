//! Musical scales for degree → pitch resolution (Strudel-compatible, 0-based).
//!
//! Degrees are **relative to the scale root** and may be negative:
//! e.g. `C2:major` degree `-1` → B1 (one step below the root in the scale).

use crate::code::{midi_to_hz, note_to_midi_i32};

/// Parsed scale: root MIDI + interval pattern within one octave.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scale {
    /// MIDI note of degree 0 (the root).
    pub root_midi: i32,
    /// Semitone offsets from root within one octave (sorted, starting with 0).
    pub intervals: Vec<i32>,
}

impl Scale {
    /// Map a zero-based scale degree (may be negative) to MIDI.
    ///
    /// Degrees are relative to the root: for `C2:major`, `-1` is B1.
    pub fn degree_to_midi(&self, degree: i32) -> i32 {
        let n = self.intervals.len() as i32;
        debug_assert!(n > 0);
        // floor_div / rem_euclid so -1 is the last step of the previous octave.
        let oct = degree.div_euclid(n);
        let step = degree.rem_euclid(n) as usize;
        self.root_midi + oct * 12 + self.intervals[step]
    }

    /// Map degree to Hz (A4 = 440).
    pub fn degree_to_hz(&self, degree: i32) -> f32 {
        midi_to_hz(self.degree_to_midi(degree))
    }
}

/// Parse a Strudel-style scale string: `C:minor`, `C2:major`, `bb3:dorian`, `C2:minor:pentatonic`.
/// Whitespace is stripped. Root without octave defaults to octave **4**.
pub fn parse_scale(raw: &str) -> Result<Scale, String> {
    let s: String = raw
        .trim()
        .trim_matches('"')
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    if s.is_empty() {
        return Err("empty scale".into());
    }

    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() < 2 {
        return Err(format!(
            "scale expects Root:mode (e.g. C2:minor), got {raw:?}"
        ));
    }

    let root_part = parts[0];
    let mode_key = parts[1..].join(":");
    let intervals = mode_intervals(&mode_key)?;

    // Root may be `C`, `c2`, `bb3`, `F#`.
    let root_midi = parse_root_midi(root_part)?;
    Ok(Scale {
        root_midi,
        intervals,
    })
}

fn parse_root_midi(root: &str) -> Result<i32, String> {
    if root.is_empty() {
        return Err("empty scale root".into());
    }
    // If last char is not a digit, default octave 4.
    let has_oct = root
        .chars()
        .last()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false);
    let note = if has_oct {
        root.to_string()
    } else {
        format!("{root}4")
    };
    note_to_midi_i32(&note)
}

fn mode_intervals(mode: &str) -> Result<Vec<i32>, String> {
    let m = mode.to_ascii_lowercase();
    let intervals: &[i32] = match m.as_str() {
        "major" | "ionian" => &[0, 2, 4, 5, 7, 9, 11],
        "minor" | "aeolian" | "natural" | "naturalminor" => &[0, 2, 3, 5, 7, 8, 10],
        "dorian" => &[0, 2, 3, 5, 7, 9, 10],
        "phrygian" => &[0, 1, 3, 5, 7, 8, 10],
        "lydian" => &[0, 2, 4, 6, 7, 9, 11],
        "mixolydian" => &[0, 2, 4, 5, 7, 9, 10],
        "locrian" => &[0, 1, 3, 5, 6, 8, 10],
        "major:pentatonic" | "pentatonic" | "majorpentatonic" => &[0, 2, 4, 7, 9],
        "minor:pentatonic" | "minorpentatonic" => &[0, 3, 5, 7, 10],
        "chromatic" => &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        other => {
            return Err(format!(
                "unsupported scale mode: {other} (try major, minor, dorian, …)"
            ))
        }
    };
    Ok(intervals.to_vec())
}

/// Fixed scale or a per-cycle progression (Strudel-style `.scale("<A2:minor D:dorian …>")`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScalePattern {
    Fixed(Scale),
    /// One entry per cycle (bar), from mini `<>` in the scale argument.
    PerCycle(Vec<Scale>),
}

impl ScalePattern {
    /// Scale active for the given cycle (bar index).
    pub fn at_cycle(&self, cycle: u64) -> &Scale {
        match self {
            ScalePattern::Fixed(s) => s,
            ScalePattern::PerCycle(v) => {
                debug_assert!(!v.is_empty());
                &v[(cycle as usize) % v.len()]
            }
        }
    }
}

/// Parse a `.scale(...)` argument: fixed `C2:minor` or progression
/// `<A2:minor D:dorian G:mixolydian C:major>` (one scale per cycle).
pub fn parse_scale_arg(raw: &str) -> Result<ScalePattern, String> {
    let s = raw.trim().trim_matches('"').trim();
    if s.is_empty() {
        return Err("empty scale".into());
    }

    if s.starts_with('<') {
        if !s.ends_with('>') {
            return Err(format!("unclosed scale progression: {raw:?}"));
        }
        let inner = s[1..s.len() - 1].trim();
        if inner.is_empty() {
            return Err("empty scale progression".into());
        }
        // Mini `:` is not a word char; split progression by whitespace only.
        let mut scales = Vec::new();
        for part in inner.split_whitespace() {
            scales.push(parse_scale(part)?);
        }
        if scales.is_empty() {
            return Err("empty scale progression".into());
        }
        if scales.len() == 1 {
            return Ok(ScalePattern::Fixed(scales.remove(0)));
        }
        return Ok(ScalePattern::PerCycle(scales));
    }

    if s.contains('<') || s.contains('>') {
        return Err(format!(
            "scale progression must be fully inside <...>, got {raw:?}"
        ));
    }
    // Spaces without <> are not a multi-scale pattern (would be ambiguous).
    if s.split_whitespace().count() > 1 {
        return Err(format!(
            "multi-scale progression needs <> (e.g. <A2:minor D:dorian>), got {raw:?}"
        ));
    }

    Ok(ScalePattern::Fixed(parse_scale(s)?))
}

/// Resolve a mini-notation atom to Hz (no pitch offset).
///
/// With a scale: integer tokens are scale degrees (negative allowed).
/// Named notes always go through note-name parsing when not pure integers.
pub fn resolve_pitch(value: &str, scale: Option<&Scale>) -> Option<f32> {
    resolve_pitch_with_add(value, scale, 0.0)
}

/// Resolve pitch with a scalar offset from `.add` / `.sub`.
///
/// - Scale + integer atom: degree `deg + pitch_add.round()` then `degree_to_hz`.
/// - Named note: MIDI + `pitch_add` semitones (fractional allowed via float MIDI).
/// - Scale missing + integer atom: still unresolved (degrees need a scale).
pub fn resolve_pitch_with_add(value: &str, scale: Option<&Scale>, pitch_add: f64) -> Option<f32> {
    let v = value.trim();
    if v.is_empty() || v == "~" {
        return None;
    }
    if let Some(sc) = scale {
        if let Ok(deg) = parse_integer_degree(v) {
            let offset = pitch_add.round() as i32;
            return Some(sc.degree_to_hz(deg.saturating_add(offset)));
        }
    }
    // Named notes (and non-degree tokens): chromatic semitone shift.
    let midi = crate::code::note_to_midi_i32(v).ok()?;
    let midi_f = midi as f64 + pitch_add;
    Some(crate::code::midi_to_hz(midi_f.round() as i32))
}

/// True integer degree: `0`, `-1`, `12` — not `1.5`, not `c2`.
fn parse_integer_degree(s: &str) -> Result<i32, ()> {
    // Reject floats and note names.
    if s.contains('.') || s.chars().any(|c| c.is_ascii_alphabetic()) {
        return Err(());
    }
    s.parse::<i32>().map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::code::note_to_hz;

    #[test]
    fn parse_c2_minor() {
        let s = parse_scale("C2:minor").unwrap();
        assert_eq!(s.root_midi, note_to_midi_i32("c2").unwrap());
        assert_eq!(s.intervals, vec![0, 2, 3, 5, 7, 8, 10]);
    }

    #[test]
    fn default_octave_is_4() {
        let s = parse_scale("C:major").unwrap();
        assert_eq!(s.root_midi, note_to_midi_i32("c4").unwrap());
    }

    #[test]
    fn degrees_zero_based_c2_minor() {
        let s = parse_scale("C2:minor").unwrap();
        let c2 = note_to_hz("c2").unwrap();
        let eb2 = note_to_hz("eb2").unwrap();
        let f2 = note_to_hz("f2").unwrap();
        let g2 = note_to_hz("g2").unwrap();
        assert!((s.degree_to_hz(0) - c2).abs() < 0.5);
        assert!((s.degree_to_hz(2) - eb2).abs() < 0.5);
        assert!((s.degree_to_hz(3) - f2).abs() < 0.5);
        assert!((s.degree_to_hz(4) - g2).abs() < 0.5);
    }

    #[test]
    fn negative_degree_below_root() {
        // C2 major: degree -1 → B1 (major scale step below root).
        let s = parse_scale("C2:major").unwrap();
        let b1 = note_to_hz("b1").unwrap();
        assert!((s.degree_to_hz(-1) - b1).abs() < 0.5);
        // C2 minor: degree -1 → Bb1 (aeolian step below root).
        let sm = parse_scale("C2:minor").unwrap();
        let bb1 = note_to_hz("bb1").unwrap();
        assert!((sm.degree_to_hz(-1) - bb1).abs() < 0.5);
    }

    #[test]
    fn parse_scale_arg_fixed() {
        let p = parse_scale_arg("C2:minor").unwrap();
        match p {
            ScalePattern::Fixed(s) => assert_eq!(s.root_midi, note_to_midi_i32("c2").unwrap()),
            ScalePattern::PerCycle(_) => panic!("expected Fixed"),
        }
    }

    #[test]
    fn parse_scale_arg_per_cycle_progression() {
        let p = parse_scale_arg("<A2:minor D:dorian G:mixolydian C:major>").unwrap();
        match p {
            ScalePattern::PerCycle(v) => {
                assert_eq!(v.len(), 4);
                assert_eq!(v[0].root_midi, note_to_midi_i32("a2").unwrap());
                assert_eq!(v[1].root_midi, note_to_midi_i32("d4").unwrap()); // D → D4
                assert_eq!(v[2].root_midi, note_to_midi_i32("g4").unwrap());
                assert_eq!(v[3].root_midi, note_to_midi_i32("c4").unwrap());
                // Cycle 0 vs 3: degree 0 moves A2 → C4
                assert!((v[0].degree_to_hz(0) - note_to_hz("a2").unwrap()).abs() < 0.5);
                assert!((v[3].degree_to_hz(0) - note_to_hz("c4").unwrap()).abs() < 0.5);
            }
            ScalePattern::Fixed(_) => panic!("expected PerCycle"),
        }
        let p = parse_scale_arg("<A2:minor D:dorian G:mixolydian C:major>").unwrap();
        assert_eq!(p.at_cycle(0).root_midi, note_to_midi_i32("a2").unwrap());
        assert_eq!(p.at_cycle(4).root_midi, note_to_midi_i32("a2").unwrap());
        assert_eq!(p.at_cycle(3).root_midi, note_to_midi_i32("c4").unwrap());
    }

    #[test]
    fn degree_7_is_octave() {
        let s = parse_scale("C2:minor").unwrap();
        let c3 = note_to_hz("c3").unwrap();
        assert!((s.degree_to_hz(7) - c3).abs() < 0.5);
    }

    #[test]
    fn resolve_pitch_with_scale() {
        let s = parse_scale("C2:minor").unwrap();
        let hz = resolve_pitch("0", Some(&s)).unwrap();
        assert!((hz - note_to_hz("c2").unwrap()).abs() < 0.5);
        let named = resolve_pitch("g2", Some(&s)).unwrap();
        assert!((named - note_to_hz("g2").unwrap()).abs() < 0.5);
    }

    #[test]
    fn resolve_pitch_add_shifts_degrees() {
        let s = parse_scale("C2:major").unwrap();
        // degree 0 + 2 == degree 2
        let a = resolve_pitch_with_add("0", Some(&s), 2.0).unwrap();
        let b = resolve_pitch("2", Some(&s)).unwrap();
        assert!((a - b).abs() < 0.5);
        // sub 1 from degree 2 → degree 1
        let c = resolve_pitch_with_add("2", Some(&s), -1.0).unwrap();
        let d = resolve_pitch("1", Some(&s)).unwrap();
        assert!((c - d).abs() < 0.5);
    }

    #[test]
    fn resolve_pitch_add_shifts_named_semitones() {
        let c4 = note_to_hz("c4").unwrap();
        let c5 = note_to_hz("c5").unwrap();
        let hz = resolve_pitch_with_add("c4", None, 12.0).unwrap();
        assert!((hz - c5).abs() < 0.5);
        let same = resolve_pitch_with_add("c4", None, 0.0).unwrap();
        assert!((same - c4).abs() < 0.5);
    }

    #[test]
    fn unknown_mode_errors() {
        assert!(parse_scale("C:foo").is_err());
    }

    #[test]
    fn compound_minor_pentatonic() {
        let s = parse_scale("A2:minor:pentatonic").unwrap();
        assert_eq!(s.intervals, vec![0, 3, 5, 7, 10]);
    }
}
