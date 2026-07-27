//! Sound name resolution: waveform list → wavetable → SampleBank.

use std::sync::Arc;

use crate::sample::SampleBank;
use crate::synth::{builtin_wavetable, NoiseKind, Wave};

/// Tier-A/B built-in waveform / noise / wavetable names.
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
    "wt_sine",
    "wt_bright",
    "wt_organ",
    "wt_demo0",
    "wt_demo1",
    "wt_demo2",
];

#[derive(Debug, Clone)]
pub enum ResolvedSound {
    Wave(Wave),
    Noise(NoiseKind),
    Wavetable(Arc<Vec<f32>>),
    Sample(String),
}

/// Resolve bank-prefixed sample name: `{bank}_{sound}` (lowercase).
pub fn banked_name(bank: Option<&str>, sound: &str) -> String {
    let sound = sound.trim().to_ascii_lowercase();
    match bank {
        Some(b) if !b.is_empty() => format!("{}_{sound}", b.trim().to_ascii_lowercase()),
        _ => sound,
    }
}

/// Resolve against fixed waveforms/noise/wavetables, then sample bank presence.
pub fn resolve_sound(name: &str, bank: &SampleBank) -> Result<ResolvedSound, String> {
    resolve_sound_with_bank(name, None, bank)
}

pub fn resolve_sound_with_bank(
    name: &str,
    sample_bank: Option<&str>,
    bank: &SampleBank,
) -> Result<ResolvedSound, String> {
    let key = name.trim().to_ascii_lowercase();
    if let Some(w) = Wave::parse(&key) {
        return Ok(ResolvedSound::Wave(w));
    }
    if let Some(n) = NoiseKind::parse(&key) {
        return Ok(ResolvedSound::Noise(n));
    }
    if let Some(wt) = builtin_wavetable(&key) {
        return Ok(ResolvedSound::Wavetable(wt));
    }
    // Sample bank may also hold wt_* cycles
    if key.starts_with("wt_") {
        if let Some(data) = bank.get(&key, None) {
            return Ok(ResolvedSound::Wavetable(data));
        }
    }
    let sample_key = banked_name(sample_bank, &key);
    if bank.has(&sample_key) {
        return Ok(ResolvedSound::Sample(sample_key));
    }
    // Fall back to unbanked name if bank prefix miss
    if sample_bank.is_some() && bank.has(&key) {
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
    fn resolves_wavetable() {
        let bank = SampleBank::empty();
        assert!(matches!(
            resolve_sound("wt_bright", &bank).unwrap(),
            ResolvedSound::Wavetable(_)
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

    #[test]
    fn banked_name_resolution() {
        let dir: PathBuf = std::env::temp_dir().join("strudel_test_bank_prefix");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        write_test_wav(&dir.join("tr808_bd.wav"), 48_000);
        let bank = SampleBank::load_dir(&dir, 48_000);
        assert!(matches!(
            resolve_sound_with_bank("bd", Some("tr808"), &bank).unwrap(),
            ResolvedSound::Sample(s) if s == "tr808_bd"
        ));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
