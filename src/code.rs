//! Method-chain pattern code: `note("...").s("sawtooth").lpf(800)...`

use crate::mini::{self, Node};

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

#[derive(Debug, Clone)]
pub struct PatternCode {
    pub pattern: Node,
    /// Waveform name or sample name (resolved at play time).
    pub sound: String,
    pub gain: f32,
    /// Multiplier applied with gain (from velocity/vel).
    pub velocity: f32,
    pub lpf: Option<f32>,
    /// Accumulated slow/fast factor (slow divides, fast multiplies).
    pub speed: f64,
    pub is_note: bool,
    pub adsr: Adsr,
    pub begin: f32,
    pub end: f32,
    pub sample_speed: f32,
    pub sample_n: Option<i32>,
    /// Original source text (for display / error context).
    #[allow(dead_code)]
    pub raw: String,
}

impl PatternCode {
    #[allow(dead_code)]
    pub fn effective_gain(&self) -> f32 {
        self.gain * self.velocity
    }
}

pub fn parse_code(input: &str) -> Result<PatternCode, String> {
    let input = input.trim();
    let mut pc = PatternCode {
        pattern: Node::Rest,
        sound: "triangle".into(),
        gain: 0.5,
        velocity: 1.0,
        lpf: None,
        speed: 1.0,
        is_note: false,
        adsr: Adsr::default(),
        begin: 0.0,
        end: 1.0,
        sample_speed: 1.0,
        sample_n: None,
        raw: input.to_string(),
    };

    let open = input.find('(').ok_or_else(|| "missing (".to_string())?;
    let head = input[..open].trim();
    if !matches!(head, "note" | "n" | "s" | "sound") {
        return Err(format!("unknown head: {head} (use note/s)"));
    }

    // Require balanced parens for the whole expression (catches `note("c3"`).
    if !parens_balanced(input) {
        return Err("unbalanced parens".into());
    }

    let first_str = extract_first_string(&input[open..])?;
    pc.pattern = mini::parse(&first_str)?;
    pc.is_note = matches!(head, "note" | "n");
    if matches!(head, "s" | "sound") {
        pc.sound = first_word(&first_str);
        pc.is_note = false;
    }

    // Walk `.name(args)` chain
    let mut cur = &input[open..];
    while let Some(dot) = cur.find('.') {
        let after = &cur[dot + 1..];
        let paren = after
            .find('(')
            .ok_or_else(|| "missing ( in method".to_string())?;
        let name = after[..paren].trim();
        let mut depth = 0i32;
        let mut end = paren;
        for (i, ch) in after[paren..].char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = paren + i;
                        break;
                    }
                }
                _ => {}
            }
        }
        if depth != 0 {
            return Err("unbalanced parens".into());
        }
        let args = &after[paren + 1..end];
        apply_method(&mut pc, name, args)?;
        cur = &after[end + 1..];
    }
    Ok(pc)
}

fn parens_balanced(s: &str) -> bool {
    let mut depth = 0i32;
    for ch in s.chars() {
        match ch {
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

fn extract_first_string(s: &str) -> Result<String, String> {
    let start = s.find('"').ok_or_else(|| "missing \"".to_string())?;
    let end = s[start + 1..]
        .find('"')
        .ok_or_else(|| "missing closing \"".to_string())?
        + start
        + 1;
    Ok(s[start + 1..end].to_string())
}

fn parse_num(a: &str) -> Result<f64, String> {
    a.trim()
        .trim_matches('"')
        .parse::<f64>()
        .map_err(|_| format!("bad number: {a}"))
}

fn apply_method(pc: &mut PatternCode, name: &str, args: &str) -> Result<(), String> {
    match name {
        "s" | "sound" => {
            pc.sound = args.trim().trim_matches('"').to_string();
            Ok(())
        }
        "gain" => {
            pc.gain = parse_num(args)? as f32;
            Ok(())
        }
        "velocity" | "vel" => {
            pc.velocity = parse_num(args)? as f32;
            Ok(())
        }
        "lpf" | "cutoff" | "lp" | "ctf" => {
            pc.lpf = Some(parse_num(args)? as f32);
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
            // "a:d:s:r" or a d s r
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
        "note" => Ok(()),
        "n" => {
            // sample index when used as method; note head already handled
            if let Ok(i) = parse_num(args) {
                pc.sample_n = Some(i as i32);
            }
            Ok(())
        }
        other => Err(format!("unsupported method: {other}")),
    }
}

/// Single note name → Hz (A4 = 440).
pub fn note_to_hz(note: &str) -> Result<f32, String> {
    if note.is_empty() {
        return Err("empty note".into());
    }
    // Strip chord suffix if present (caller should use expand_chord)
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
                // flat only if not start of octave number; 'b' after letter is flat
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
    // MIDI: C-1 = 0, C4 = 60. formula: (oct + 1) * 12 + semis
    let midi = (oct + 1) * 12 + semis;
    Ok(440.0 * 2f32.powf((midi - 69) as f32 / 12.0))
}

/// `c3'maj` / `c3'min7` → chord tones as note names. Plain notes → one element.
/// Supported qualities: maj, min, maj7, min7, dim, aug, sus2, sus4.
pub fn expand_chord(token: &str) -> Result<Vec<String>, String> {
    if !token.contains('\'') {
        // validate as a note
        note_to_hz(token)?;
        return Ok(vec![token.to_string()]);
    }
    let (root_part, quality) = token
        .split_once('\'')
        .ok_or_else(|| format!("bad chord token: {token}"))?;
    let root_hz_check = note_to_hz(root_part)?;
    let _ = root_hz_check;

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
    while i < bytes.len() && (bytes[i] == b'#' || (bytes[i] == b'b' && i == 1)) {
        // allow # or b accidentals after letter
        if bytes[i] == b'#' || bytes[i] == b'b' {
            i += 1;
        } else {
            break;
        }
    }
    // also consume multi accidentals simply
    while i < bytes.len() && (bytes[i] == b'#' || bytes[i] == b'b') {
        // only if not digit
        if bytes[i].is_ascii_digit() {
            break;
        }
        if bytes[i] == b'#' || bytes[i] == b'b' {
            i += 1;
        }
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
        assert_eq!(pc.lpf, Some(800.0));
        assert!((pc.speed - 0.5).abs() < 1e-9);
    }

    #[test]
    fn sample_head() {
        let pc = parse_code(r#"s("bd sd hh*2").gain(0.8)"#).unwrap();
        assert!(!pc.is_note);
        assert_eq!(pc.sound, "bd");
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
    fn rejects_bad() {
        assert!(parse_code("stack(\"a\")").is_err());
        assert!(parse_code("note(\"c3\"").is_err());
        assert!(parse_code(r#"note("c3").unknown(1)"#).is_err());
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
        let freqs: Vec<f32> = notes.iter().map(|n| note_to_hz(n).unwrap()).collect();
        assert!((freqs[0] - 130.81).abs() < 0.5);
    }

    #[test]
    fn expand_min7() {
        let notes = expand_chord("a2'min7").unwrap();
        assert_eq!(notes, vec!["a2", "c3", "e3", "g3"]);
    }

    #[test]
    fn expand_plain_note() {
        assert_eq!(expand_chord("d#4").unwrap(), vec!["d#4".to_string()]);
    }

    #[test]
    fn expand_unknown_quality() {
        assert!(expand_chord("c3'foo").is_err());
    }
}
