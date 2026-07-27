//! Soft synth voice: waveforms, noise, wavetable, FM/vib, ADSR, biquad filters.

use std::sync::Arc;

use crate::code::{Adsr, FilterParams, ModParams};
use crate::dsp::{Biquad, BiquadKind};

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

#[derive(Clone, Debug)]
pub enum OscSource {
    Wave(Wave),
    Noise(NoiseKind),
    Wavetable(Arc<Vec<f32>>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EnvStage {
    Attack,
    Decay,
    Sustain,
    Release,
    Done,
}

/// Per-voice oscillator + modulation + ADSR + biquad chain.
pub struct Voice {
    pub source: OscSource,
    pub freq: f32,
    pub gain: f32,
    pub orbit: u8,
    pub cut: Option<i32>,
    phase: f32,
    mod_phase: f32,
    vib_phase: f32,
    pos: u64,
    len: u64,
    gate_off: u64,
    adsr: Adsr,
    stage: EnvStage,
    env_level: f32,
    attack_samples: u64,
    decay_samples: u64,
    release_samples: u64,
    filter: FilterParams,
    mods: ModParams,
    lpf: Biquad,
    hpf: Biquad,
    bpf: Biquad,
    use_lpf: bool,
    use_hpf: bool,
    use_bpf: bool,
    base_lpf: Option<f32>,
    sr_cached: f32,
    // filter env
    lp_env_level: f32,
    lp_attack_s: u64,
    lp_decay_s: u64,
    // pitch env
    p_attack_s: u64,
    p_decay_s: u64,
    // fm env
    fm_attack_s: u64,
    fm_decay_s: u64,
    pink_b: [f32; 7],
    brown_y: f32,
    rng: u32,
}

impl Voice {
    pub fn new_wave(wave: Wave, freq: f32, gain: f32, len: u64, lpf: Option<f32>) -> Self {
        let mut filter = FilterParams::default();
        filter.lpf = lpf;
        Self::new(
            OscSource::Wave(wave),
            freq,
            gain,
            len,
            filter,
            Adsr::default(),
            ModParams::default(),
            1,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source: OscSource,
        freq: f32,
        gain: f32,
        len: u64,
        filter: FilterParams,
        adsr: Adsr,
        mods: ModParams,
        orbit: u8,
        cut: Option<i32>,
    ) -> Self {
        Self {
            source,
            freq,
            gain,
            orbit,
            cut,
            phase: 0.0,
            mod_phase: 0.0,
            vib_phase: 0.0,
            pos: 0,
            len,
            gate_off: len,
            adsr,
            stage: EnvStage::Attack,
            env_level: 0.0,
            attack_samples: 1,
            decay_samples: 1,
            release_samples: 1,
            filter,
            mods,
            lpf: Biquad::bypass(),
            hpf: Biquad::bypass(),
            bpf: Biquad::bypass(),
            use_lpf: filter.lpf.is_some() || mods.lpenv.abs() > 1e-6,
            use_hpf: filter.hpf.is_some(),
            use_bpf: filter.bpf.is_some(),
            base_lpf: filter.lpf,
            sr_cached: 48_000.0,
            lp_env_level: 0.0,
            lp_attack_s: 1,
            lp_decay_s: 1,
            p_attack_s: 1,
            p_decay_s: 1,
            fm_attack_s: 1,
            fm_decay_s: 1,
            pink_b: [0.0; 7],
            brown_y: 0.0,
            rng: 0x1234_5678,
        }
    }

    pub fn with_adsr_timing(mut self, sr: f32, gate_samples: u64) -> Self {
        self.sr_cached = sr;
        self.attack_samples = sec_to_samples(self.adsr.attack, sr).max(1);
        self.decay_samples = sec_to_samples(self.adsr.decay, sr).max(1);
        self.release_samples = sec_to_samples(self.adsr.release, sr).max(1);
        self.lp_attack_s = sec_to_samples(self.mods.lpa, sr).max(1);
        self.lp_decay_s = sec_to_samples(self.mods.lpd, sr).max(1);
        self.p_attack_s = sec_to_samples(self.mods.patt, sr).max(1);
        self.p_decay_s = sec_to_samples(self.mods.pdec.max(0.001), sr).max(1);
        self.fm_attack_s = sec_to_samples(self.mods.fm_attack, sr).max(1);
        self.fm_decay_s = sec_to_samples(self.mods.fm_decay, sr).max(1);
        self.gate_off = gate_samples;
        let need = gate_samples.saturating_add(self.release_samples);
        if self.len < need {
            self.len = need;
        }
        self.rebuild_filters(sr);
        self
    }

    pub fn with_adsr(mut self, adsr: Adsr) -> Self {
        self.adsr = adsr;
        self
    }

    fn rebuild_filters(&mut self, sr: f32) {
        if let Some(cut) = self.effective_lpf_hz() {
            self.lpf
                .set_coeffs(BiquadKind::LowPass, cut, self.filter.lpq, sr);
            self.use_lpf = true;
        } else {
            self.use_lpf = false;
        }
        if let Some(cut) = self.filter.hpf {
            self.hpf
                .set_coeffs(BiquadKind::HighPass, cut, self.filter.hpq, sr);
            self.use_hpf = true;
        } else {
            self.use_hpf = false;
        }
        if let Some(cut) = self.filter.bpf {
            self.bpf
                .set_coeffs(BiquadKind::BandPass, cut, self.filter.bpq, sr);
            self.use_bpf = true;
        } else {
            self.use_bpf = false;
        }
    }

    fn effective_lpf_hz(&self) -> Option<f32> {
        let base = match self.base_lpf {
            Some(hz) => hz,
            None if self.mods.lpenv.abs() > 1e-6 => 500.0,
            None => return None,
        };
        if self.mods.lpenv.abs() < 1e-6 {
            return Some(base);
        }
        // lpenv depth in "octaves-ish": multiply by 2^(lpenv * env)
        let mult = 2f32.powf(self.mods.lpenv * self.lp_env_level);
        Some((base * mult).clamp(20.0, 20_000.0))
    }

    fn pitch_env_semis(&self) -> f32 {
        if self.mods.penv.abs() < 1e-6 {
            return 0.0;
        }
        let a = self.p_attack_s as f32;
        let d = self.p_decay_s as f32;
        let p = self.pos as f32;
        let level = if p < a {
            p / a
        } else if self.mods.pdec > 0.0 {
            let t = ((p - a) / d).clamp(0.0, 1.0);
            1.0 - t
        } else {
            1.0
        };
        self.mods.penv * level.clamp(0.0, 1.0)
    }

    fn fm_env_level(&self) -> f32 {
        if self.mods.fm.abs() < 1e-6 {
            return 0.0;
        }
        let a = self.fm_attack_s as f32;
        let d = self.fm_decay_s as f32;
        let p = self.pos as f32;
        if p < a {
            p / a
        } else {
            let t = ((p - a) / d).clamp(0.0, 1.0);
            1.0 + (self.mods.fm_sustain - 1.0) * t
        }
    }

    fn advance_lp_env(&mut self) {
        if self.mods.lpenv.abs() < 1e-6 {
            self.lp_env_level = 0.0;
            return;
        }
        let a = self.lp_attack_s as f32;
        let d = self.lp_decay_s as f32;
        let p = self.pos as f32;
        if p < a {
            self.lp_env_level = p / a;
        } else {
            let t = ((p - a) / d).clamp(0.0, 1.0);
            self.lp_env_level = 1.0 + (self.mods.lps - 1.0) * t;
        }
    }

    pub fn next_sample(&mut self, sr: f32) -> Option<f32> {
        if self.stage == EnvStage::Done || self.pos >= self.len {
            self.stage = EnvStage::Done;
            return None;
        }

        if self.pos == 0 && self.attack_samples == 1 && self.adsr.attack > 0.0 {
            self.with_adsr_timing_inplace(sr, self.gate_off);
        }

        if self.pos >= self.gate_off && self.stage != EnvStage::Release {
            self.stage = EnvStage::Release;
        }

        self.advance_lp_env();
        // Update LPF coeffs when filter envelope is active (state preserved).
        if self.mods.lpenv.abs() > 1e-6 {
            if let Some(cut) = self.effective_lpf_hz() {
                self.lpf
                    .set_coeffs(BiquadKind::LowPass, cut, self.filter.lpq, sr);
                self.use_lpf = true;
            }
        }

        let env = self.advance_env();
        let t = self.pos as f32 / sr;

        // pitch + vib
        let mut freq = self.freq.max(0.1);
        let semis = self.pitch_env_semis();
        if semis.abs() > 1e-6 {
            freq *= 2f32.powf(semis / 12.0);
        }
        if self.mods.vib_hz > 0.0 {
            let vib = (self.vib_phase * 2.0 * std::f32::consts::PI).sin();
            freq *= 2f32.powf(self.mods.vibmod * vib / 12.0);
            self.vib_phase = (self.vib_phase + self.mods.vib_hz / sr) % 1.0;
        }

        let raw = self.osc_sample_modulated(freq, sr, t);
        let mut x = raw * env * self.gain;

        // filter chain: LPF → HPF → BPF
        if self.use_lpf {
            x = self.lpf.process(x);
        }
        if self.use_hpf {
            x = self.hpf.process(x);
        }
        if self.use_bpf {
            x = self.bpf.process(x);
        }

        self.pos += 1;
        if self.stage == EnvStage::Done {
            return None;
        }
        Some(x)
    }

    fn with_adsr_timing_inplace(&mut self, sr: f32, gate_samples: u64) {
        self.sr_cached = sr;
        self.attack_samples = sec_to_samples(self.adsr.attack, sr).max(1);
        self.decay_samples = sec_to_samples(self.adsr.decay, sr).max(1);
        self.release_samples = sec_to_samples(self.adsr.release, sr).max(1);
        self.lp_attack_s = sec_to_samples(self.mods.lpa, sr).max(1);
        self.lp_decay_s = sec_to_samples(self.mods.lpd, sr).max(1);
        self.p_attack_s = sec_to_samples(self.mods.patt, sr).max(1);
        self.p_decay_s = sec_to_samples(self.mods.pdec.max(0.001), sr).max(1);
        self.fm_attack_s = sec_to_samples(self.mods.fm_attack, sr).max(1);
        self.fm_decay_s = sec_to_samples(self.mods.fm_decay, sr).max(1);
        self.gate_off = gate_samples;
        let need = gate_samples.saturating_add(self.release_samples);
        if self.len < need {
            self.len = need;
        }
        self.rebuild_filters(sr);
    }

    fn osc_sample_modulated(&mut self, freq: f32, sr: f32, _t: f32) -> f32 {
        let fm_idx = self.mods.fm * self.fm_env_level();
        let fmh = self.mods.fmh.max(0.01);

        // advance mod phase for FM
        let mod_sig = if fm_idx.abs() > 1e-6 {
            let m = (self.mod_phase * 2.0 * std::f32::consts::PI).sin();
            self.mod_phase = (self.mod_phase + (fmh * freq) / sr) % 1.0;
            m
        } else {
            0.0
        };

        let inst_freq = (freq + fm_idx * freq * mod_sig).max(0.1);

        let pure = match &self.source {
            OscSource::Wave(w) => {
                let s = w.sample(self.phase);
                self.phase = (self.phase + inst_freq / sr) % 1.0;
                s
            }
            OscSource::Noise(NoiseKind::White) => self.white(),
            OscSource::Noise(NoiseKind::Pink) => self.pink(),
            OscSource::Noise(NoiseKind::Brown) => self.brown(),
            OscSource::Wavetable(table) => {
                let s = wavetable_sample(table, self.phase);
                self.phase = (self.phase + inst_freq / sr) % 1.0;
                s
            }
        };

        if self.mods.noise_mix > 0.0 {
            let n = self.pink();
            pure * (1.0 - self.mods.noise_mix) + n * self.mods.noise_mix
        } else {
            pure
        }
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
                let release_start_level = if self.env_level <= 0.0 {
                    self.adsr.sustain
                } else {
                    self.env_level
                };
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

    fn white(&mut self) -> f32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

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

fn wavetable_sample(table: &[f32], phase: f32) -> f32 {
    if table.is_empty() {
        return 0.0;
    }
    let n = table.len() as f32;
    let pos = phase.fract().rem_euclid(1.0) * n;
    let i0 = pos.floor() as usize % table.len();
    let i1 = (i0 + 1) % table.len();
    let frac = pos - pos.floor();
    table[i0] * (1.0 - frac) + table[i1] * frac
}

/// Procedural single-cycle wavetables for `wt_*` demo sounds.
pub fn builtin_wavetable(name: &str) -> Option<Arc<Vec<f32>>> {
    let key = name.to_ascii_lowercase();
    let n = 2048usize;
    let table = match key.as_str() {
        "wt_sine" | "wt_demo0" => (0..n)
            .map(|i| {
                let p = i as f32 / n as f32;
                (p * 2.0 * std::f32::consts::PI).sin()
            })
            .collect(),
        "wt_bright" | "wt_demo1" => (0..n)
            .map(|i| {
                let p = i as f32 / n as f32;
                let mut s = 0.0f32;
                for h in 1..=8 {
                    s += (p * 2.0 * std::f32::consts::PI * h as f32).sin() / h as f32;
                }
                s * 0.4
            })
            .collect(),
        "wt_organ" | "wt_demo2" => (0..n)
            .map(|i| {
                let p = i as f32 / n as f32;
                let mut s = 0.0f32;
                for h in [1usize, 2, 3, 4, 6, 8] {
                    s += (p * 2.0 * std::f32::consts::PI * h as f32).sin() / h as f32;
                }
                s * 0.35
            })
            .collect(),
        _ => return None,
    };
    Some(Arc::new(table))
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
            let mut filter = FilterParams::default();
            filter.lpf = lpf;
            filter.lpq = 0.707;
            let mut v = Voice::new(
                OscSource::Wave(Wave::Square),
                2000.0,
                1.0,
                4800,
                filter,
                Adsr {
                    attack: 0.001,
                    decay: 0.0,
                    sustain: 1.0,
                    release: 0.001,
                },
                ModParams::default(),
                1,
                None,
            )
            .with_adsr_timing(48_000.0, 4800);
            let mut e = 0f32;
            // skip short attack transient
            for _ in 0..64 {
                let _ = v.next_sample(48_000.0);
            }
            while let Some(s) = v.next_sample(48_000.0) {
                e += s.abs();
            }
            e
        };
        let with = render(Some(200.0));
        let without = render(None);
        assert!(
            with < without * 0.5,
            "lpf energy {with} should be much less than open {without}"
        );
    }

    #[test]
    fn noise_voice_has_energy() {
        let mut v = Voice::new(
            OscSource::Noise(NoiseKind::White),
            0.0,
            0.5,
            2000,
            FilterParams::default(),
            Adsr {
                attack: 0.001,
                decay: 0.0,
                sustain: 1.0,
                release: 0.01,
            },
            ModParams::default(),
            1,
            None,
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
        for _ in 0..500 {
            let _ = v.next_sample(48_000.0);
        }
        let mid = v.next_sample(48_000.0).unwrap().abs();
        for _ in 0..2000 {
            let _ = v.next_sample(48_000.0);
        }
        let late = v.next_sample(48_000.0).map(|s| s.abs()).unwrap_or(0.0);
        assert!(late < mid, "late={late} mid={mid}");
    }

    #[test]
    fn fm_produces_energy() {
        let mut mods = ModParams::default();
        mods.fm = 4.0;
        mods.fmh = 2.0;
        let mut v = Voice::new(
            OscSource::Wave(Wave::Sine),
            220.0,
            0.8,
            4800,
            FilterParams::default(),
            Adsr {
                attack: 0.001,
                decay: 0.0,
                sustain: 1.0,
                release: 0.01,
            },
            mods,
            1,
            None,
        )
        .with_adsr_timing(48_000.0, 4000);
        let mut e = 0f32;
        while let Some(s) = v.next_sample(48_000.0) {
            e += s.abs();
        }
        assert!(e > 5.0);
    }

    #[test]
    fn wavetable_builtin() {
        let table = builtin_wavetable("wt_bright").unwrap();
        let mut v = Voice::new(
            OscSource::Wavetable(table),
            220.0,
            0.5,
            2000,
            FilterParams::default(),
            Adsr {
                attack: 0.001,
                decay: 0.0,
                sustain: 1.0,
                release: 0.01,
            },
            ModParams::default(),
            1,
            None,
        )
        .with_adsr_timing(48_000.0, 1500);
        let mut e = 0f32;
        while let Some(s) = v.next_sample(48_000.0) {
            e += s.abs();
        }
        assert!(e > 1.0);
    }
}
