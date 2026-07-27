//! Method-chain pattern code: `note("...").s("sawtooth").lpf(800)...`

use crate::dsp::CompressorParams;
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
    let input = input.trim();
    let mut pc = PatternCode {
        pattern: Node::Rest,
        sound: "triangle".into(),
        gain: 0.5,
        velocity: 1.0,
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
        raw: input.to_string(),
        mini_src: String::new(),
        mini_base: 0,
    };

    let open = input.find('(').ok_or_else(|| "missing (".to_string())?;
    let head = input[..open].trim();
    if !matches!(head, "note" | "n" | "s" | "sound") {
        return Err(format!("unknown head: {head} (use note/s)"));
    }

    if !parens_balanced(input) {
        return Err("unbalanced parens".into());
    }

    let (first_str, content_start_in_tail) = extract_first_string(&input[open..])?;
    pc.mini_base = trim_start + open + content_start_in_tail;
    pc.mini_src = first_str.clone();
    pc.pattern = mini::parse(&first_str)?;
    pc.is_note = matches!(head, "note" | "n");
    if matches!(head, "s" | "sound") {
        pc.sound = first_word(&first_str);
        pc.is_note = false;
    }

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
            let list = parse_colon_list(args);
            let (cut, q) = apply_cutoff_q(&list)?;
            pc.filter.lpf = Some(cut);
            if let Some(q) = q {
                pc.filter.lpq = q;
            }
            Ok(())
        }
        "lpq" | "resonance" => {
            pc.filter.lpq = parse_num(args)? as f32;
            Ok(())
        }
        "hpf" | "hp" | "hcutoff" => {
            let list = parse_colon_list(args);
            let (cut, q) = apply_cutoff_q(&list)?;
            pc.filter.hpf = Some(cut);
            if let Some(q) = q {
                pc.filter.hpq = q;
            }
            Ok(())
        }
        "hpq" | "hresonance" => {
            pc.filter.hpq = parse_num(args)? as f32;
            Ok(())
        }
        "bpf" | "bp" | "bandf" => {
            let list = parse_colon_list(args);
            let (cut, q) = apply_cutoff_q(&list)?;
            pc.filter.bpf = Some(cut);
            if let Some(q) = q {
                pc.filter.bpq = q;
            }
            Ok(())
        }
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
        "note" => Ok(()),
        "n" => {
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
    let midi = (oct + 1) * 12 + semis;
    Ok(440.0 * 2f32.powf((midi - 69) as f32 / 12.0))
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
}
