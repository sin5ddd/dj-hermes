//! SampleBank (WAV) + SampleVoice playback (tier A).

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::synth::Voice;

/// Default root pitch for `note().s("sample")` speed scaling (C3).
pub const SAMPLE_ROOT_HZ: f32 = 261.6256;

/// Loaded PCM held as `Arc` so voices only refcount-clone.
pub struct SampleBank {
    /// sound name → variation list (sorted by path name)
    samples: HashMap<String, Vec<Arc<Vec<f32>>>>,
}

impl SampleBank {
    pub fn empty() -> Self {
        Self {
            samples: HashMap::new(),
        }
    }

    /// Load `dir/<name>.wav` and `dir/<name>/*.wav` into memory, resampled to `target_sr`.
    pub fn load_dir(dir: &Path, target_sr: u32) -> Self {
        type NamedVars = Vec<(String, Arc<Vec<f32>>)>;
        let mut samples: HashMap<String, NamedVars> = HashMap::new();

        let Ok(rd) = std::fs::read_dir(dir) else {
            return Self::empty();
        };

        for entry in rd.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = match path.file_name().and_then(|s| s.to_str()) {
                    Some(n) => n.to_string(),
                    None => continue,
                };
                let mut vars = Vec::new();
                if let Ok(inner) = std::fs::read_dir(&path) {
                    for e in inner.flatten() {
                        let p = e.path();
                        if is_wav(&p) {
                            if let Ok(data) = load_wav(&p, target_sr) {
                                let key = p
                                    .file_name()
                                    .map(|s| s.to_string_lossy().into_owned())
                                    .unwrap_or_default();
                                vars.push((key, Arc::new(data)));
                            }
                        }
                    }
                }
                vars.sort_by(|a, b| a.0.cmp(&b.0));
                if !vars.is_empty() {
                    samples.insert(name, vars);
                }
            } else if is_wav(&path) {
                let name = match path.file_stem().and_then(|s| s.to_str()) {
                    Some(n) => n.to_string(),
                    None => continue,
                };
                if let Ok(data) = load_wav(&path, target_sr) {
                    samples
                        .entry(name)
                        .or_default()
                        .push((String::new(), Arc::new(data)));
                }
            }
        }

        let samples = samples
            .into_iter()
            .map(|(k, mut v)| {
                // Flat files may share a name with a folder; prefer folder vars if both exist.
                if v.iter().any(|(n, _)| !n.is_empty()) {
                    v.retain(|(n, _)| !n.is_empty());
                    v.sort_by(|a, b| a.0.cmp(&b.0));
                }
                let key = k.to_ascii_lowercase();
                (
                    key,
                    v.into_iter()
                        .map(|(_, a)| a)
                        .collect::<Vec<Arc<Vec<f32>>>>(),
                )
            })
            .collect();

        Self { samples }
    }

    pub fn has(&self, name: &str) -> bool {
        self.samples.contains_key(&name.to_ascii_lowercase())
    }

    pub fn names(&self) -> Vec<String> {
        let mut n: Vec<_> = self.samples.keys().cloned().collect();
        n.sort();
        n
    }

    /// Pick variation by `n % len` (negative n uses rem_euclid).
    pub fn get(&self, name: &str, n: Option<i32>) -> Option<Arc<Vec<f32>>> {
        let vars = self.samples.get(&name.to_ascii_lowercase())?;
        if vars.is_empty() {
            return None;
        }
        let idx = match n {
            None => 0,
            Some(i) => {
                let len = vars.len() as i32;
                i.rem_euclid(len) as usize
            }
        };
        vars.get(idx).cloned()
    }
}

fn is_wav(p: &Path) -> bool {
    p.extension()
        .and_then(|s| s.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("wav"))
}

/// 16-bit PCM WAV → mono f32, nearest-neighbour resample to `target_sr`.
pub fn load_wav(path: &Path, target_sr: u32) -> Result<Vec<f32>, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    decode_wav_bytes(&bytes, target_sr)
}

fn decode_wav_bytes(bytes: &[u8], target_sr: u32) -> Result<Vec<f32>, String> {
    if bytes.len() < 44 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err("not wav".into());
    }

    let mut channels = 1usize;
    let mut src_sr = 44_100u32;
    let mut data: Option<&[u8]> = None;

    let mut i = 12usize;
    while i + 8 <= bytes.len() {
        let id = &bytes[i..i + 4];
        let size =
            u32::from_le_bytes([bytes[i + 4], bytes[i + 5], bytes[i + 6], bytes[i + 7]]) as usize;
        let body_start = i + 8;
        let body_end = (body_start + size).min(bytes.len());
        if id == b"fmt " && size >= 16 {
            let fmt = &bytes[body_start..body_end];
            let audio_format = u16::from_le_bytes([fmt[0], fmt[1]]);
            if audio_format != 1 {
                return Err("only PCM wav".into());
            }
            channels = u16::from_le_bytes([fmt[2], fmt[3]]) as usize;
            src_sr = u32::from_le_bytes([fmt[4], fmt[5], fmt[6], fmt[7]]);
            let bits = u16::from_le_bytes([fmt[14], fmt[15]]);
            if bits != 16 {
                return Err("only 16bit wav".into());
            }
        } else if id == b"data" {
            data = Some(&bytes[body_start..body_end]);
        }
        i = body_end + (size % 2); // word align
    }

    let data = data.ok_or_else(|| "no data chunk".to_string())?;
    if channels == 0 {
        return Err("bad channels".into());
    }
    let frame_bytes = 2 * channels;
    let mut out: Vec<f32> = data
        .chunks_exact(frame_bytes)
        .map(|c| i16::from_le_bytes([c[0], c[1]]) as f32 / 32768.0)
        .collect();

    if src_sr != target_sr && !out.is_empty() && src_sr > 0 {
        let ratio = src_sr as f64 / target_sr as f64;
        let n = ((out.len() as f64) / ratio).round().max(1.0) as usize;
        out = (0..n)
            .map(|k| {
                let src_i = ((k as f64) * ratio) as usize;
                out[src_i.min(out.len() - 1)]
            })
            .collect();
    }
    Ok(out)
}

/// One-shot / pitched sample voice with begin/end/speed and linear interpolation.
pub struct SampleVoice {
    data: Arc<Vec<f32>>,
    pos: f64,
    step: f64,
    end: f64,
    gain: f32,
}

impl SampleVoice {
    /// `begin`/`end` in 0..1 of the buffer. `speed` multiplies playback rate.
    /// `pitch_ratio` is 1.0 for unpitched hits; `target_hz / SAMPLE_ROOT_HZ` for notes.
    pub fn new(
        data: Arc<Vec<f32>>,
        gain: f32,
        begin: f32,
        end: f32,
        speed: f32,
        pitch_ratio: f32,
    ) -> Self {
        let len = data.len() as f64;
        let b = begin.clamp(0.0, 1.0) as f64;
        let e = end.clamp(0.0, 1.0) as f64;
        let (start, end) = if e > b {
            (b * len, e * len)
        } else {
            (0.0, len)
        };
        let step = (speed.max(1e-6) as f64) * (pitch_ratio.max(1e-6) as f64);
        Self {
            data,
            pos: start,
            step,
            end,
            gain,
        }
    }

    pub fn next_sample(&mut self) -> Option<f32> {
        if self.pos >= self.end || self.data.is_empty() {
            return None;
        }
        let i0 = self.pos.floor() as usize;
        if i0 >= self.data.len() {
            return None;
        }
        let frac = self.pos - i0 as f64;
        let s0 = self.data[i0];
        let s1 = self.data.get(i0 + 1).copied().unwrap_or(s0);
        let x = (s0 as f64 + (s1 as f64 - s0 as f64) * frac) as f32 * self.gain;
        self.pos += self.step;
        Some(x)
    }
}

/// Synth or sample voice for the deck pool.
pub enum VoiceKind {
    Synth(Voice),
    Sample(SampleVoice),
}

impl VoiceKind {
    pub fn next_sample(&mut self, sr: f32) -> Option<f32> {
        match self {
            VoiceKind::Synth(v) => v.next_sample(sr),
            VoiceKind::Sample(v) => v.next_sample(),
        }
    }
}

/// Helper for tests: write a short 16-bit mono sine WAV (100 ms).
#[cfg(test)]
pub fn write_test_wav(path: &Path, sr: u32) {
    write_test_wav_secs(path, sr, 0.1);
}

/// Helper for tests: write a 16-bit mono sine WAV of `secs` duration.
#[cfg(test)]
pub fn write_test_wav_secs(path: &Path, sr: u32, secs: f32) {
    use std::io::Write;
    let n = (sr as f32 * secs).max(1.0) as usize;
    let data: Vec<u8> = (0..n)
        .flat_map(|i| {
            let v = ((i as f32 / sr as f32 * 440.0 * 2.0 * std::f32::consts::PI).sin() * 30_000.0)
                as i16;
            v.to_le_bytes()
        })
        .collect();
    let mut f = Vec::new();
    f.write_all(b"RIFF").unwrap();
    f.write_all(&(36u32 + data.len() as u32).to_le_bytes())
        .unwrap();
    f.write_all(b"WAVEfmt ").unwrap();
    f.write_all(&16u32.to_le_bytes()).unwrap();
    f.write_all(&1u16.to_le_bytes()).unwrap();
    f.write_all(&1u16.to_le_bytes()).unwrap();
    f.write_all(&sr.to_le_bytes()).unwrap();
    f.write_all(&(sr * 2).to_le_bytes()).unwrap();
    f.write_all(&2u16.to_le_bytes()).unwrap();
    f.write_all(&16u16.to_le_bytes()).unwrap();
    f.write_all(b"data").unwrap();
    f.write_all(&(data.len() as u32).to_le_bytes()).unwrap();
    f.write_all(&data).unwrap();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::write(path, f).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_wav() {
        let dir = std::env::temp_dir().join("strudel_test_samples_load");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        write_test_wav(&dir.join("bd.wav"), 48_000);
        let bank = SampleBank::load_dir(&dir, 48_000);
        let s = bank.get("bd", None).expect("bd missing");
        assert!(s.len() > 4000);
        assert!(s.iter().any(|x| x.abs() > 0.5));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn n_picks_variation() {
        let dir = std::env::temp_dir().join("strudel_test_samples_n");
        let _ = std::fs::remove_dir_all(&dir);
        let bd = dir.join("bd");
        std::fs::create_dir_all(&bd).unwrap();
        write_test_wav_secs(&bd.join("00.wav"), 48_000, 0.1);
        write_test_wav_secs(&bd.join("01.wav"), 48_000, 0.2);
        let bank = SampleBank::load_dir(&dir, 48_000);
        let a = bank.get("bd", Some(0)).unwrap();
        let b = bank.get("bd", Some(1)).unwrap();
        assert_ne!(a.len(), b.len());
        let c = bank.get("bd", Some(2)).unwrap(); // wraps
        assert_eq!(a.len(), c.len());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn sample_voice_plays() {
        let data = Arc::new(vec![0.0, 0.5, 1.0, 0.5, 0.0]);
        let mut v = SampleVoice::new(data, 1.0, 0.0, 1.0, 1.0, 1.0);
        let mut n = 0;
        while v.next_sample().is_some() {
            n += 1;
            if n > 20 {
                break;
            }
        }
        assert!(n >= 5);
    }
}
