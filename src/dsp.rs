//! Shared realtime DSP: biquad filters, dynamics compressor, duck envelopes.

use std::f32::consts::PI;

/// User-facing orbit ids are 1..=NUM_ORBITS; internal index is id - 1.
pub const NUM_ORBITS: usize = 4;

#[inline]
pub fn orbit_index(orbit_1based: u8) -> usize {
    let id = orbit_1based.max(1) as usize;
    (id - 1).min(NUM_ORBITS - 1)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BiquadKind {
    LowPass,
    HighPass,
    BandPass,
}

/// RBJ biquad (Cookbook formulae).
#[derive(Clone, Copy, Debug)]
pub struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}

impl Biquad {
    pub fn bypass() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            z1: 0.0,
            z2: 0.0,
        }
    }

    pub fn new(kind: BiquadKind, cutoff_hz: f32, q: f32, sr: f32) -> Self {
        let mut f = Self::bypass();
        f.set_coeffs(kind, cutoff_hz, q, sr);
        f
    }

    /// Update coefficients without clearing delay state (safe for automation).
    pub fn set_coeffs(&mut self, kind: BiquadKind, cutoff_hz: f32, q: f32, sr: f32) {
        let sr = sr.max(1.0);
        let cutoff = cutoff_hz.clamp(20.0, sr * 0.45);
        let q = q.clamp(0.1, 20.0);
        let w0 = 2.0 * PI * cutoff / sr;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q);

        let (b0, b1, b2, a0, a1, a2) = match kind {
            BiquadKind::LowPass => {
                let b1 = 1.0 - cos_w0;
                let b0 = b1 * 0.5;
                let b2 = b0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            BiquadKind::HighPass => {
                let b1 = -(1.0 + cos_w0);
                let b0 = (1.0 + cos_w0) * 0.5;
                let b2 = b0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            BiquadKind::BandPass => {
                let b0 = sin_w0 / 2.0;
                let b1 = 0.0;
                let b2 = -b0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
        };

        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }
}

/// Soft-knee peak compressor (mono).
#[derive(Clone, Copy, Debug)]
pub struct CompressorParams {
    /// dB, typically negative (e.g. -20).
    pub threshold_db: f32,
    pub ratio: f32,
    pub knee_db: f32,
    pub attack: f32,
    pub release: f32,
}

impl Default for CompressorParams {
    fn default() -> Self {
        Self {
            threshold_db: -20.0,
            ratio: 4.0,
            knee_db: 6.0,
            attack: 0.003,
            release: 0.1,
        }
    }
}

impl CompressorParams {
    /// Parse Strudel-style `threshold:ratio:knee:attack:release`.
    pub fn parse(s: &str) -> Result<Self, String> {
        let raw = s.trim().trim_matches('"');
        let parts: Vec<&str> = if raw.contains(':') {
            raw.split(':').collect()
        } else {
            raw.split_whitespace().collect()
        };
        if parts.len() != 5 {
            return Err(format!(
                "compressor expects threshold:ratio:knee:attack:release, got {raw}"
            ));
        }
        let parse = |p: &str| {
            p.trim()
                .parse::<f32>()
                .map_err(|_| format!("bad compressor number: {p}"))
        };
        Ok(Self {
            threshold_db: parse(parts[0])?,
            ratio: parse(parts[1])?.max(1.0),
            knee_db: parse(parts[2])?.max(0.0),
            attack: parse(parts[3])?.max(0.0),
            release: parse(parts[4])?.max(0.0),
        })
    }
}

#[derive(Clone, Debug)]
pub struct Compressor {
    pub params: CompressorParams,
    env: f32,
    attack_coeff: f32,
    release_coeff: f32,
}

impl Compressor {
    pub fn new(params: CompressorParams, sr: f32) -> Self {
        let sr = sr.max(1.0);
        let mut c = Self {
            params,
            env: 0.0,
            attack_coeff: 0.0,
            release_coeff: 0.0,
        };
        c.recalc_coeffs(sr);
        c
    }

    pub fn set_params(&mut self, params: CompressorParams, sr: f32) {
        self.params = params;
        self.recalc_coeffs(sr);
    }

    fn recalc_coeffs(&mut self, sr: f32) {
        let sr = sr.max(1.0);
        // one-pole envelope: coeff = exp(-1 / (time * sr))
        self.attack_coeff = if self.params.attack <= 0.0 {
            0.0
        } else {
            (-1.0 / (self.params.attack * sr)).exp()
        };
        self.release_coeff = if self.params.release <= 0.0 {
            0.0
        } else {
            (-1.0 / (self.params.release * sr)).exp()
        };
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let level = x.abs();
        if level > self.env {
            self.env = self.attack_coeff * self.env + (1.0 - self.attack_coeff) * level;
        } else {
            self.env = self.release_coeff * self.env + (1.0 - self.release_coeff) * level;
        }

        let env_db = if self.env > 1e-8 {
            20.0 * self.env.log10()
        } else {
            -160.0
        };

        let th = self.params.threshold_db;
        let knee = self.params.knee_db;
        let ratio = self.params.ratio.max(1.0);

        let over = env_db - th;
        let gr_db = if over <= -knee * 0.5 {
            0.0
        } else if over >= knee * 0.5 {
            over * (1.0 - 1.0 / ratio)
        } else {
            // soft knee
            let t = over + knee * 0.5;
            (t * t) / (2.0 * knee.max(1e-6)) * (1.0 - 1.0 / ratio)
        };

        let gain = 10f32.powf(-gr_db / 20.0);
        x * gain
    }
}

/// Per-orbit duck envelope: instant drop then linear recover to 1.0.
#[derive(Clone, Copy, Debug)]
pub struct DuckState {
    pub gain: f32,
    depth: f32,
    recover_samples: u64,
    pos: u64,
    active: bool,
}

impl Default for DuckState {
    fn default() -> Self {
        Self {
            gain: 1.0,
            depth: 0.0,
            recover_samples: 1,
            pos: 0,
            active: false,
        }
    }
}

impl DuckState {
    pub fn trigger(&mut self, depth: f32, attack_sec: f32, sr: f32) {
        let depth = depth.clamp(0.0, 1.0);
        self.depth = depth;
        self.gain = 1.0 - depth;
        self.recover_samples = ((attack_sec.max(0.0) * sr.max(1.0)) as u64).max(1);
        self.pos = 0;
        self.active = true;
    }

    #[inline]
    pub fn advance(&mut self) -> f32 {
        if !self.active {
            return self.gain;
        }
        let t = (self.pos as f32 / self.recover_samples as f32).clamp(0.0, 1.0);
        self.gain = (1.0 - self.depth) + self.depth * t;
        self.pos += 1;
        if t >= 1.0 {
            self.active = false;
            self.gain = 1.0;
        }
        self.gain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn biquad_lpf_stable() {
        let mut f = Biquad::new(BiquadKind::LowPass, 500.0, 0.707, 48_000.0);
        let mut y = 0.0f32;
        for i in 0..1000 {
            let x = if i % 2 == 0 { 1.0 } else { -1.0 };
            y = f.process(x);
            assert!(y.is_finite());
        }
        assert!(y.abs() < 2.0);
    }

    #[test]
    fn compressor_parse() {
        let p = CompressorParams::parse("-20:20:10:.002:.02").unwrap();
        assert!((p.threshold_db + 20.0).abs() < 1e-5);
        assert!((p.ratio - 20.0).abs() < 1e-5);
    }

    #[test]
    fn compressor_reduces_peak() {
        let mut c = Compressor::new(
            CompressorParams {
                threshold_db: -12.0,
                ratio: 20.0,
                knee_db: 0.0,
                attack: 0.0,
                release: 0.05,
            },
            48_000.0,
        );
        let mut peak_in = 0.0f32;
        let mut peak_out = 0.0f32;
        for _ in 0..2000 {
            let x: f32 = 0.9;
            peak_in = peak_in.max(x.abs());
            let y = c.process(x);
            peak_out = peak_out.max(y.abs());
        }
        assert!(peak_out < peak_in, "out={peak_out} in={peak_in}");
    }

    #[test]
    fn duck_recovers() {
        let mut d = DuckState::default();
        d.trigger(1.0, 0.01, 48_000.0);
        assert!((d.gain - 0.0).abs() < 1e-5);
        let mut last = 0.0f32;
        for _ in 0..600 {
            last = d.advance();
        }
        assert!((last - 1.0).abs() < 1e-4);
    }
}
