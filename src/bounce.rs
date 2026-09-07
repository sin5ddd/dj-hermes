//! Offline bounce: `Engine::process` to interleaved stereo, write 16-bit WAV.
//! Factory / LUFS gate only. Not used on the audio thread.

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::engine::{Command, Engine};
use crate::sample::SampleBank;
use crate::song::Song;

/// Match e2e / factory renders.
pub const RENDER_SR: u32 = 48_000;
/// `LoadSong` is bar-quantized; discard this many bars before measuring.
pub const DEFAULT_WARMUP_BARS: usize = 1;
/// Default measure length (covers 16-bar genre cycles).
pub const DEFAULT_MEASURE_BARS: usize = 16;

/// One offline bounce of a song on deck A.
#[derive(Debug, Clone)]
pub struct Bounce {
    /// Interleaved stereo `[L, R, L, R, …]`.
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub bpm: f64,
    pub warmup_bars: usize,
    pub bars: usize,
}

impl Bounce {
    pub fn peak_abs(&self) -> f32 {
        peak_abs(&self.samples)
    }

    pub fn rms(&self) -> f32 {
        rms(&self.samples)
    }

    pub fn frames(&self) -> usize {
        self.samples.len() / 2
    }
}

pub fn peak_abs(samples: &[f32]) -> f32 {
    samples.iter().fold(0.0f32, |m, s| m.max(s.abs()))
}

pub fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let acc: f32 = samples.iter().map(|s| s * s).sum();
    (acc / samples.len() as f32).sqrt()
}

/// Samples in one bar (4 beats). Ceil so a `process` call crosses the bar head.
pub fn samples_per_bar(sample_rate: u32, bpm: f64) -> usize {
    let bpm = if bpm.is_finite() && bpm > 0.0 {
        bpm
    } else {
        120.0
    };
    (f64::from(sample_rate) * 60.0 / bpm * 4.0).ceil() as usize
}

/// Render `warmup_bars` (discarded) then `bars` of audio on deck A.
///
/// One `Engine::process` per bar so bar-quantized `LoadSong` applies at a buffer head
/// (same as `tests/e2e.rs`).
pub fn bounce_song(
    song: Song,
    bank: &SampleBank,
    sample_rate: u32,
    warmup_bars: usize,
    bars: usize,
) -> Result<Bounce, String> {
    if bars == 0 {
        return Err("bars must be >= 1".into());
    }
    let bpm = song
        .bpm
        .filter(|b| b.is_finite() && *b > 0.0)
        .unwrap_or(120.0);
    let mut engine = Engine::new(sample_rate, bpm);
    engine.push_command(Command::LoadSong {
        deck: 0,
        song: Box::new(song),
    });
    let bar_len = samples_per_bar(sample_rate, bpm);
    let _ = process_bars(&mut engine, bank, warmup_bars, bar_len);
    let samples = process_bars(&mut engine, bank, bars, bar_len);
    Ok(Bounce {
        samples,
        sample_rate,
        bpm,
        warmup_bars,
        bars,
    })
}

fn process_bars(
    engine: &mut Engine,
    bank: &SampleBank,
    bars: usize,
    frames_per_bar: usize,
) -> Vec<f32> {
    let mut all = Vec::with_capacity(bars.saturating_mul(frames_per_bar).saturating_mul(2));
    let mut buf = vec![0f32; frames_per_bar.saturating_mul(2)];
    for _ in 0..bars {
        buf.fill(0.0);
        engine.process(&mut buf, bank);
        all.extend_from_slice(&buf);
    }
    all
}

/// Write interleaved f32 stereo as 16-bit PCM WAV (little-endian).
pub fn write_wav_i16_stereo(
    path: &Path,
    sample_rate: u32,
    interleaved: &[f32],
) -> Result<(), String> {
    if !interleaved.len().is_multiple_of(2) {
        return Err("stereo wav needs an even sample count (interleaved L/R)".into());
    }
    if sample_rate == 0 {
        return Err("sample_rate must be > 0".into());
    }
    let n_bytes = (interleaved.len() * 2) as u32;
    let mut f = Vec::with_capacity(44 + interleaved.len() * 2);
    f.write_all(b"RIFF").map_err(|e| e.to_string())?;
    f.write_all(&(36u32 + n_bytes).to_le_bytes())
        .map_err(|e| e.to_string())?;
    f.write_all(b"WAVEfmt ").map_err(|e| e.to_string())?;
    f.write_all(&16u32.to_le_bytes())
        .map_err(|e| e.to_string())?;
    f.write_all(&1u16.to_le_bytes())
        .map_err(|e| e.to_string())?; // PCM
    f.write_all(&2u16.to_le_bytes())
        .map_err(|e| e.to_string())?; // stereo
    f.write_all(&sample_rate.to_le_bytes())
        .map_err(|e| e.to_string())?;
    let byte_rate = sample_rate.saturating_mul(2).saturating_mul(2);
    f.write_all(&byte_rate.to_le_bytes())
        .map_err(|e| e.to_string())?;
    f.write_all(&4u16.to_le_bytes())
        .map_err(|e| e.to_string())?; // block align
    f.write_all(&16u16.to_le_bytes())
        .map_err(|e| e.to_string())?;
    f.write_all(b"data").map_err(|e| e.to_string())?;
    f.write_all(&n_bytes.to_le_bytes())
        .map_err(|e| e.to_string())?;
    for &x in interleaved {
        let s = (x.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        f.write_all(&s.to_le_bytes()).map_err(|e| e.to_string())?;
    }
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
        }
    }
    fs::write(path, f).map_err(|e| format!("write {}: {e}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::song::parse_song;

    fn saw_loop() -> Song {
        parse_song(
            r#"setcpm(120/4)
$: note("0 4 7 4").scale("C3:minor").s("sawtooth").gain(0.5)
"#,
            "saw-loop.strudel",
        )
        .expect("parse")
    }

    #[test]
    fn samples_per_bar_120_bpm() {
        assert_eq!(samples_per_bar(48_000, 120.0), 96_000);
    }

    #[test]
    fn warmup_required_for_bar_quantized_load() {
        let bank = SampleBank::empty();
        let silent = bounce_song(saw_loop(), &bank, RENDER_SR, 0, 1).unwrap();
        let sounding = bounce_song(saw_loop(), &bank, RENDER_SR, 1, 1).unwrap();
        assert!(
            sounding.peak_abs() > 0.05,
            "warmup 1 should play the loaded song, peak={}",
            sounding.peak_abs()
        );
        assert!(
            silent.peak_abs() < sounding.peak_abs() * 0.1,
            "warmup 0 / 1 bar is before LoadSong applies, silent={}, sounding={}",
            silent.peak_abs(),
            sounding.peak_abs()
        );
    }

    #[test]
    fn write_wav_roundtrip_header() {
        let bank = SampleBank::empty();
        let bounce = bounce_song(saw_loop(), &bank, RENDER_SR, 1, 1).unwrap();
        let dir = std::env::temp_dir().join("strudel_bounce_wav");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("saw.wav");
        write_wav_i16_stereo(&path, bounce.sample_rate, &bounce.samples).unwrap();
        let bytes = fs::read(&path).unwrap();
        assert!(bytes.len() > 44);
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(&bytes[22..24], &2u16.to_le_bytes());
        let sr = u32::from_le_bytes(bytes[24..28].try_into().unwrap());
        assert_eq!(sr, RENDER_SR);
        for &s in &bounce.samples {
            assert!(s.is_finite(), "non-finite sample");
        }
    }
}
