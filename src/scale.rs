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

/// Resolve a mini-notation atom to Hz.
///
/// With a scale: integer tokens are scale degrees (negative allowed).
/// Named notes always go through `note_to_hz` when not pure integers.
pub fn resolve_pitch(value: &str, scale: Option<&Scale>) -> Option<f32> {
    let v = value.trim();
    if v.is_empty() || v == "~" {
        return None;
    }
    if let Some(sc) = scale {
        if let Ok(deg) = parse_integer_degree(v) {
            return Some(sc.degree_to_hz(deg));
        }
    }
    crate::code::note_to_hz(v).ok()
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
    fn unknown_mode_errors() {
        assert!(parse_scale("C:foo").is_err());
    }

    #[test]
    fn compound_minor_pentatonic() {
        let s = parse_scale("A2:minor:pentatonic").unwrap();
        assert_eq!(s.intervals, vec![0, 3, 5, 7, 10]);
    }
}
