//! Soft synth voice: basic waveforms, noise, ADSR, one-pole LPF (tier A).

use crate::code::Adsr;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Wave {
    Sine,
    Saw,
    Square,
    Triangle,
}

impl Wave {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "sine" => Some(Wave::Sine),
            "sawtooth" | "saw" => Some(Wave::Saw),
            "square" => Some(Wave::Square),
            "triangle" | "tri" => Some(Wave::Triangle),
            _ => None,
        }
    }

    fn sample(self, phase: f32) -> f32 {
        match self {
            Wave::Sine => (phase * 2.0 * std::f32::consts::PI).sin(),
            Wave::Saw => 2.0 * phase - 1.0,
            Wave::Square => {
                if phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Wave::Triangle => 4.0 * (phase - 0.5).abs() - 1.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NoiseKind {
    White,
    Pink,
    Brown,
}

impl NoiseKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "white" => Some(NoiseKind::White),
            "pink" => Some(NoiseKind::Pink),
            "brown" | "brownian" => Some(NoiseKind::Brown),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum OscSource {
    Wave(Wave),
    Noise(NoiseKind),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EnvStage {
    Attack,
    Decay,
    Sustain,
    Release,
    Done,
}

/// Per-voice oscillator + ADSR + optional LPF.
pub struct Voice {
    pub source: OscSource,
    pub freq: f32,
    pub gain: f32,
    phase: f32,
    pos: u64,
    /// Total note length in samples (includes release tail after gate-off).
    len: u64,
    /// Sample index where gate goes off (start of release).
    gate_off: u64,
    adsr: Adsr,
    stage: EnvStage,
    env_level: f32,
    attack_samples: u64,
    decay_samples: u64,
    release_samples: u64,
    lpf: Option<f32>,
    lpf_y: f32,
    // Pink / brown state
    pink_b: [f32; 7],
    brown_y: f32,
    rng: u32,
}

impl Voice {
    pub fn new_wave(wave: Wave, freq: f32, gain: f32, len: u64, lpf: Option<f32>) -> Self {
        Self::new(OscSource::Wave(wave), freq, gain, len, lpf, Adsr::default())
    }

    pub fn new(
        source: OscSource,
        freq: f32,
        gain: f32,
        len: u64,
        lpf: Option<f32>,
        adsr: Adsr,
    ) -> Self {
        // gate_off defaults to full length; release still runs if len allows
        let gate_off = len;
        Self {
            source,
            freq,
            gain,
            phase: 0.0,
            pos: 0,
            len,
            gate_off,
            adsr,
            stage: EnvStage::Attack,
            env_level: 0.0,
            attack_samples: 1,
            decay_samples: 1,
            release_samples: 1,
            lpf,
            lpf_y: 0.0,
            pink_b: [0.0; 7],
            brown_y: 0.0,
            rng: 0x1234_5678,
        }
    }

    /// Configure envelope sample counts from seconds at `sr`, and set gate-off point.
    /// `gate_samples` is how long the note is held before release (excluding release tail).
    pub fn with_adsr_timing(mut self, sr: f32, gate_samples: u64) -> Self {
        self.attack_samples = sec_to_samples(self.adsr.attack, sr).max(1);
        self.decay_samples = sec_to_samples(self.adsr.decay, sr).max(1);
        self.release_samples = sec_to_samples(self.adsr.release, sr).max(1);
        self.gate_off = gate_samples;
        // Ensure voice lives through release
        let need = gate_samples.saturating_add(self.release_samples);
        if self.len < need {
            self.len = need;
        }
        self
    }

    pub fn with_adsr(mut self, adsr: Adsr) -> Self {
        self.adsr = adsr;
        self
    }

    pub fn next_sample(&mut self, sr: f32) -> Option<f32> {
        if self.stage == EnvStage::Done || self.pos >= self.len {
            self.stage = EnvStage::Done;
            return None;
        }

        // Lazy init timing if caller used new_wave without with_adsr_timing
        if self.pos == 0 && self.attack_samples == 1 && self.adsr.attack > 0.0 {
            self.attack_samples = sec_to_samples(self.adsr.attack, sr).max(1);
            self.decay_samples = sec_to_samples(self.adsr.decay, sr).max(1);
            self.release_samples = sec_to_samples(self.adsr.release, sr).max(1);
        }

        if self.pos >= self.gate_off && self.stage != EnvStage::Release {
            self.stage = EnvStage::Release;
        }

        let env = self.advance_env();
        let raw = self.osc_sample();
        let mut x = raw * env * self.gain;

        // advance phase for waves
        if matches!(self.source, OscSource::Wave(_)) {
            self.phase = (self.phase + self.freq / sr) % 1.0;
        }

        if let Some(cut) = self.lpf {
            let k = 1.0 - (-2.0 * std::f32::consts::PI * cut / sr).exp();
            self.lpf_y += k * (x - self.lpf_y);
            x = self.lpf_y;
        }

        self.pos += 1;
        if self.stage == EnvStage::Done {
            return None;
        }
        Some(x)
    }

    fn advance_env(&mut self) -> f32 {
        match self.stage {
            EnvStage::Attack => {
                let t = self.pos as f32 / self.attack_samples as f32;
                self.env_level = t.min(1.0);
                if self.pos + 1 >= self.attack_samples {
                    self.stage = EnvStage::Decay;
                }
                self.env_level
            }
            EnvStage::Decay => {
                let start = self.attack_samples;
                let elapsed = self.pos.saturating_sub(start) as f32;
                let t = (elapsed / self.decay_samples as f32).min(1.0);
                self.env_level = 1.0 + (self.adsr.sustain - 1.0) * t;
                if self.pos + 1 >= start + self.decay_samples {
                    self.stage = EnvStage::Sustain;
                    self.env_level = self.adsr.sustain;
                }
                self.env_level
            }
            EnvStage::Sustain => {
                self.env_level = self.adsr.sustain;
                self.env_level
            }
            EnvStage::Release => {
                // linear release from current level
                let release_start_level = if self.env_level <= 0.0 {
                    self.adsr.sustain
                } else {
                    self.env_level
                };
                // count release progress from gate_off
                let elapsed = self.pos.saturating_sub(self.gate_off) as f32;
                let t = (elapsed / self.release_samples as f32).min(1.0);
                self.env_level = release_start_level * (1.0 - t);
                if elapsed + 1.0 >= self.release_samples as f32 {
                    self.stage = EnvStage::Done;
                    self.env_level = 0.0;
                }
                self.env_level
            }
            EnvStage::Done => 0.0,
        }
    }

    fn osc_sample(&mut self) -> f32 {
        match self.source {
            OscSource::Wave(w) => w.sample(self.phase),
            OscSource::Noise(NoiseKind::White) => self.white(),
            OscSource::Noise(NoiseKind::Pink) => self.pink(),
            OscSource::Noise(NoiseKind::Brown) => self.brown(),
        }
    }

    fn white(&mut self) -> f32 {
        // xorshift32
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

    /// Paul Kellet style pink-ish filter on white.
    fn pink(&mut self) -> f32 {
        let w = self.white();
        self.pink_b[0] = 0.99886 * self.pink_b[0] + w * 0.055_517_9;
        self.pink_b[1] = 0.99332 * self.pink_b[1] + w * 0.075_075_9;
        self.pink_b[2] = 0.969 * self.pink_b[2] + w * 0.153_852;
        self.pink_b[3] = 0.8665 * self.pink_b[3] + w * 0.310_485_6;
        self.pink_b[4] = 0.55 * self.pink_b[4] + w * 0.532_952_2;
        self.pink_b[5] = -0.7616 * self.pink_b[5] - w * 0.016_898;
        let pink = self.pink_b[0]
            + self.pink_b[1]
            + self.pink_b[2]
            + self.pink_b[3]
            + self.pink_b[4]
            + self.pink_b[5]
            + self.pink_b[6]
            + w * 0.5362;
        self.pink_b[6] = w * 0.115926;
        (pink * 0.11).clamp(-1.0, 1.0)
    }

    fn brown(&mut self) -> f32 {
        let w = self.white();
        self.brown_y = (self.brown_y + w * 0.02).clamp(-1.0, 1.0);
        self.brown_y
    }
}

fn sec_to_samples(sec: f32, sr: f32) -> u64 {
    (sec.max(0.0) * sr) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voice_produces_sound_and_ends() {
        let mut v =
            Voice::new_wave(Wave::Sine, 440.0, 0.5, 4800, None).with_adsr_timing(48_000.0, 4000);
        let mut energy = 0f32;
        let mut n = 0;
        while let Some(s) = v.next_sample(48_000.0) {
            energy += s.abs();
            n += 1;
            if n > 20_000 {
                break;
            }
        }
        assert!(energy > 10.0, "energy={energy}");
        assert!(v.next_sample(48_000.0).is_none());
    }

    #[test]
    fn lpf_reduces_energy() {
        let render = |lpf: Option<f32>| {
            let mut v = Voice::new_wave(Wave::Square, 2000.0, 1.0, 4800, lpf)
                .with_adsr(Adsr {
                    attack: 0.001,
                    decay: 0.0,
                    sustain: 1.0,
                    release: 0.001,
                })
                .with_adsr_timing(48_000.0, 4800);
            let mut e = 0f32;
            while let Some(s) = v.next_sample(48_000.0) {
                e += s.abs();
            }
            e
        };
        assert!(render(Some(200.0)) < render(None));
    }

    #[test]
    fn noise_voice_has_energy() {
        let mut v = Voice::new(
            OscSource::Noise(NoiseKind::White),
            0.0,
            0.5,
            2000,
            None,
            Adsr {
                attack: 0.001,
                decay: 0.0,
                sustain: 1.0,
                release: 0.01,
            },
        )
        .with_adsr_timing(48_000.0, 1500);
        let mut energy = 0f32;
        while let Some(s) = v.next_sample(48_000.0) {
            energy += s.abs();
        }
        assert!(energy > 1.0);
    }

    #[test]
    fn release_decays() {
        let mut v = Voice::new_wave(Wave::Sine, 440.0, 1.0, 10_000, None)
            .with_adsr(Adsr {
                attack: 0.001,
                decay: 0.0,
                sustain: 1.0,
                release: 0.05,
            })
            .with_adsr_timing(48_000.0, 1000);
        // skip attack
        for _ in 0..500 {
            let _ = v.next_sample(48_000.0);
        }
        let mid = v.next_sample(48_000.0).unwrap().abs();
        // run into release
        for _ in 0..2000 {
            let _ = v.next_sample(48_000.0);
        }
        let late = v.next_sample(48_000.0).map(|s| s.abs()).unwrap_or(0.0);
        assert!(late < mid, "late={late} mid={mid}");
    }
}
