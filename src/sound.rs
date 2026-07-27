//! Sound name resolution: waveform list first, then SampleBank.

use crate::sample::SampleBank;
use crate::synth::{NoiseKind, Wave};

/// Tier-A built-in waveform / noise names.
#[allow(dead_code)]
pub const WAVEFORM_NAMES: &[&str] = &[
    "sine", "sawtooth", "saw", "square", "triangle", "tri", "white", "pink", "brown",
];

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedSound {
    Wave(Wave),
    Noise(NoiseKind),
    Sample(String),
}

/// Resolve against fixed waveforms/noise, then sample bank presence.
pub fn resolve_sound(name: &str, bank: &SampleBank) -> Result<ResolvedSound, String> {
    let key = name.trim().to_ascii_lowercase();
    if let Some(w) = Wave::parse(&key) {
        return Ok(ResolvedSound::Wave(w));
    }
    if let Some(n) = NoiseKind::parse(&key) {
        return Ok(ResolvedSound::Noise(n));
    }
    if bank.has(&key) {
        return Ok(ResolvedSound::Sample(key));
    }
    Err(format!(
        "unknown sound: {name} (not a waveform and not in sample bank)"
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
    use crate::sample::{write_test_wav, SampleBank};
    use std::path::PathBuf;

    #[test]
    fn resolves_waves() {
        let bank = SampleBank::empty();
        assert!(matches!(
            resolve_sound("sawtooth", &bank).unwrap(),
            ResolvedSound::Wave(Wave::Saw)
        ));
        assert!(matches!(
            resolve_sound("tri", &bank).unwrap(),
            ResolvedSound::Wave(Wave::Triangle)
        ));
        assert!(matches!(
            resolve_sound("pink", &bank).unwrap(),
            ResolvedSound::Noise(NoiseKind::Pink)
        ));
    }

    #[test]
    fn unknown_is_error() {
        let bank = SampleBank::empty();
        let e = resolve_sound("bd", &bank).unwrap_err();
        assert!(e.contains("unknown sound"));
    }

    #[test]
    fn resolves_sample_from_bank() {
        let dir: PathBuf = std::env::temp_dir().join("strudel_test_sound_bank");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        write_test_wav(&dir.join("bd.wav"), 48_000);
        let bank = SampleBank::load_dir(&dir, 48_000);
        assert!(matches!(
            resolve_sound("bd", &bank).unwrap(),
            ResolvedSound::Sample(s) if s == "bd"
        ));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
