//! Sound name resolution: waveform list first, SampleBank later (Task 9).

use crate::synth::{NoiseKind, Wave};

/// Tier-A built-in waveform / noise names.
#[allow(dead_code)]
pub const WAVEFORM_NAMES: &[&str] = &[
    "sine",
    "sawtooth",
    "saw",
    "square",
    "triangle",
    "tri",
    "white",
    "pink",
    "brown",
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResolvedSound {
    Wave(Wave),
    Noise(NoiseKind),
    /// Placeholder until SampleBank is wired in Task 9.
    #[allow(dead_code)]
    SampleName,
}

/// Resolve a sound name against the fixed waveform list only.
/// Sample fallback is Task 9; unknown names error with a clear message.
pub fn resolve_sound(name: &str) -> Result<ResolvedSound, String> {
    let key = name.trim().to_ascii_lowercase();
    if let Some(w) = Wave::from_str(&key) {
        return Ok(ResolvedSound::Wave(w));
    }
    if let Some(n) = NoiseKind::from_str(&key) {
        return Ok(ResolvedSound::Noise(n));
    }
    // Task 9: SampleBank lookup goes here.
    Err(format!(
        "unknown sound: {name} (not a waveform and no sample bank yet)"
    ))
}

#[allow(dead_code)]
pub fn is_waveform_name(name: &str) -> bool {
    WAVEFORM_NAMES
        .iter()
        .any(|w| w.eq_ignore_ascii_case(name.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_waves() {
        assert!(matches!(
            resolve_sound("sawtooth").unwrap(),
            ResolvedSound::Wave(Wave::Saw)
        ));
        assert!(matches!(
            resolve_sound("tri").unwrap(),
            ResolvedSound::Wave(Wave::Triangle)
        ));
        assert!(matches!(
            resolve_sound("pink").unwrap(),
            ResolvedSound::Noise(NoiseKind::Pink)
        ));
    }

    #[test]
    fn unknown_is_error() {
        let e = resolve_sound("bd").unwrap_err();
        assert!(e.contains("unknown sound"));
    }
}
