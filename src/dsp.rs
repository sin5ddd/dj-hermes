//! Shared realtime DSP: biquad filters, dynamics compressor, duck envelopes,
//! orbit delay/room, and channel EQ helpers.

use std::f32::consts::PI;

/// User-facing orbit ids are 1..=NUM_ORBITS; internal index is id - 1.
pub const NUM_ORBITS: usize = 4;

/// Max delay time for orbit delay lines (seconds).
pub const MAX_DELAY_SEC: f32 = 2.0;

/// Hard ceiling for delay feedback (&lt; 1 to prevent runaway).
pub const MAX_DELAY_FEEDBACK: f32 = 0.95;

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
    Peaking,
    LowShelf,
    HighShelf,
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

    pub fn new_eq(kind: BiquadKind, cutoff_hz: f32, q: f32, gain_db: f32, sr: f32) -> Self {
        let mut f = Self::bypass();
        f.set_coeffs_gain(kind, cutoff_hz, q, gain_db, sr);
        f
    }

    /// Update coefficients without clearing delay state (safe for automation).
    pub fn set_coeffs(&mut self, kind: BiquadKind, cutoff_hz: f32, q: f32, sr: f32) {
        self.set_coeffs_gain(kind, cutoff_hz, q, 0.0, sr);
    }

    /// Like [`set_coeffs`], with dB gain for peaking / shelf filters.
    pub fn set_coeffs_gain(
        &mut self,
        kind: BiquadKind,
        cutoff_hz: f32,
        q: f32,
        gain_db: f32,
        sr: f32,
    ) {
        let sr = sr.max(1.0);
        let cutoff = cutoff_hz.clamp(20.0, sr * 0.45);
        let q = q.clamp(0.1, 20.0);
        let w0 = 2.0 * PI * cutoff / sr;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q);
        let a_lin = 10f32.powf(gain_db / 40.0);

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
            BiquadKind::Peaking => {
                let b0 = 1.0 + alpha * a_lin;
                let b1 = -2.0 * cos_w0;
                let b2 = 1.0 - alpha * a_lin;
                let a0 = 1.0 + alpha / a_lin;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha / a_lin;
                (b0, b1, b2, a0, a1, a2)
            }
            BiquadKind::LowShelf => {
                let two_sqrt_a_alpha = 2.0 * a_lin.sqrt() * alpha;
                let b0 = a_lin * ((a_lin + 1.0) - (a_lin - 1.0) * cos_w0 + two_sqrt_a_alpha);
                let b1 = 2.0 * a_lin * ((a_lin - 1.0) - (a_lin + 1.0) * cos_w0);
                let b2 = a_lin * ((a_lin + 1.0) - (a_lin - 1.0) * cos_w0 - two_sqrt_a_alpha);
                let a0 = (a_lin + 1.0) + (a_lin - 1.0) * cos_w0 + two_sqrt_a_alpha;
                let a1 = -2.0 * ((a_lin - 1.0) + (a_lin + 1.0) * cos_w0);
                let a2 = (a_lin + 1.0) + (a_lin - 1.0) * cos_w0 - two_sqrt_a_alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            BiquadKind::HighShelf => {
                let two_sqrt_a_alpha = 2.0 * a_lin.sqrt() * alpha;
                let b0 = a_lin * ((a_lin + 1.0) + (a_lin - 1.0) * cos_w0 + two_sqrt_a_alpha);
                let b1 = -2.0 * a_lin * ((a_lin - 1.0) + (a_lin + 1.0) * cos_w0);
                let b2 = a_lin * ((a_lin + 1.0) + (a_lin - 1.0) * cos_w0 - two_sqrt_a_alpha);
                let a0 = (a_lin + 1.0) - (a_lin - 1.0) * cos_w0 + two_sqrt_a_alpha;
                let a1 = 2.0 * ((a_lin - 1.0) - (a_lin + 1.0) * cos_w0);
                let a2 = (a_lin + 1.0) - (a_lin - 1.0) * cos_w0 - two_sqrt_a_alpha;
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

/// Map DJ EQ slider position (0..=1, 0.5 = flat) to gain in dB (±12 dB).
#[inline]
pub fn eq_pos_to_db(pos: f32) -> f32 {
    (pos.clamp(0.0, 1.0) - 0.5) * 24.0
}

/// Circular delay line for orbit delay FX.
#[derive(Clone, Debug)]
pub struct DelayLine {
    buf: Vec<f32>,
    write: usize,
    delay_samples: usize,
}

impl DelayLine {
    pub fn new(max_samples: usize) -> Self {
        let n = max_samples.max(2);
        Self {
            buf: vec![0.0; n],
            write: 0,
            delay_samples: 1,
        }
    }

    pub fn clear(&mut self) {
        self.buf.fill(0.0);
        self.write = 0;
    }

    pub fn set_time_sec(&mut self, sec: f32, sr: f32) {
        let max_d = self.buf.len().saturating_sub(1).max(1);
        let n = (sec.max(0.0) * sr.max(1.0)).round() as usize;
        self.delay_samples = n.clamp(1, max_d);
    }

    /// Write `dry + delayed * feedback`; return the delayed sample (pre-write).
    #[inline]
    pub fn process(&mut self, dry: f32, feedback: f32) -> f32 {
        let len = self.buf.len();
        let read = (self.write + len - self.delay_samples) % len;
        let delayed = self.buf[read];
        let fb = feedback.clamp(0.0, MAX_DELAY_FEEDBACK);
        self.buf[self.write] = dry + delayed * fb;
        self.write = (self.write + 1) % len;
        delayed
    }
}

#[derive(Clone, Debug)]
struct Comb {
    buf: Vec<f32>,
    idx: usize,
    feedback: f32,
    filter: f32,
    damp: f32,
}

impl Comb {
    fn new(size: usize) -> Self {
        Self {
            buf: vec![0.0; size.max(1)],
            idx: 0,
            feedback: 0.5,
            filter: 0.0,
            damp: 0.2,
        }
    }

    fn clear(&mut self) {
        self.buf.fill(0.0);
        self.idx = 0;
        self.filter = 0.0;
    }

    #[inline]
    fn process(&mut self, x: f32) -> f32 {
        let y = self.buf[self.idx];
        self.filter = y * (1.0 - self.damp) + self.filter * self.damp;
        self.buf[self.idx] = x + self.filter * self.feedback;
        self.idx += 1;
        if self.idx >= self.buf.len() {
            self.idx = 0;
        }
        y
    }
}

#[derive(Clone, Debug)]
struct Allpass {
    buf: Vec<f32>,
    idx: usize,
    feedback: f32,
}

impl Allpass {
    fn new(size: usize) -> Self {
        Self {
            buf: vec![0.0; size.max(1)],
            idx: 0,
            feedback: 0.5,
        }
    }

    fn clear(&mut self) {
        self.buf.fill(0.0);
        self.idx = 0;
    }

    #[inline]
    fn process(&mut self, x: f32) -> f32 {
        let buf_out = self.buf[self.idx];
        let y = -x + buf_out;
        self.buf[self.idx] = x + buf_out * self.feedback;
        self.idx += 1;
        if self.idx >= self.buf.len() {
            self.idx = 0;
        }
        y
    }
}

/// Lightweight mono reverb (Schroeder / freeverb-style combs + allpasses).
#[derive(Clone, Debug)]
pub struct SimpleReverb {
    combs: [Comb; 4],
    allpasses: [Allpass; 2],
    size: f32,
    sr: f32,
}

impl SimpleReverb {
    /// Build with delay lengths scaled to `sr` (freeverb base at 44.1 kHz).
    pub fn new(sr: f32) -> Self {
        let mut r = Self {
            combs: [Comb::new(1), Comb::new(1), Comb::new(1), Comb::new(1)],
            allpasses: [Allpass::new(1), Allpass::new(1)],
            size: 1.0,
            sr: 44_100.0,
        };
        r.rebuild(sr);
        r
    }

    pub fn clear(&mut self) {
        for c in &mut self.combs {
            c.clear();
        }
        for a in &mut self.allpasses {
            a.clear();
        }
    }

    fn rebuild(&mut self, sr: f32) {
        let sr = sr.max(1.0);
        self.sr = sr;
        let scale = sr / 44_100.0;
        // Freeverb-ish comb / allpass lengths at 44.1 kHz.
        let comb_lens = [1116, 1188, 1277, 1356];
        let ap_lens = [556, 441];
        for (i, &base) in comb_lens.iter().enumerate() {
            let n = ((base as f32) * scale).round().max(2.0) as usize;
            self.combs[i] = Comb::new(n);
        }
        for (i, &base) in ap_lens.iter().enumerate() {
            let n = ((base as f32) * scale).round().max(2.0) as usize;
            self.allpasses[i] = Allpass::new(n);
        }
        self.apply_size(self.size);
    }

    /// Room size in Strudel-ish range 0..=10.
    pub fn set_size(&mut self, size: f32) {
        self.size = size.clamp(0.0, 10.0);
        self.apply_size(self.size);
    }

    fn apply_size(&mut self, size: f32) {
        // Map 0..10 → feedback ~0.5..0.92 (stable).
        let t = (size / 10.0).clamp(0.0, 1.0);
        let fb = 0.5 + 0.42 * t;
        for c in &mut self.combs {
            c.feedback = fb;
            c.damp = 0.25 - 0.1 * t;
        }
    }

    pub fn ensure_sr(&mut self, sr: f32) {
        if (sr - self.sr).abs() > 1.0 {
            let size = self.size;
            self.rebuild(sr);
            self.size = size;
            self.apply_size(size);
        }
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let mut s = 0.0f32;
        for c in &mut self.combs {
            s += c.process(x);
        }
        s *= 0.25;
        for a in &mut self.allpasses {
            s = a.process(s);
        }
        s
    }
}

/// Per-orbit global FX: dry + delay send + room send.
#[derive(Clone, Debug)]
pub struct OrbitFx {
    pub delay_level: f32,
    pub delay_time: f32,
    pub delay_fb: f32,
    pub room_level: f32,
    pub room_size: f32,
    delay: DelayLine,
    reverb: SimpleReverb,
    sr: f32,
    ready: bool,
}

impl Default for OrbitFx {
    fn default() -> Self {
        Self::new()
    }
}

impl OrbitFx {
    pub fn new() -> Self {
        Self {
            delay_level: 0.0,
            delay_time: 0.25,
            delay_fb: 0.5,
            room_level: 0.0,
            room_size: 1.0,
            delay: DelayLine::new(2),
            reverb: SimpleReverb::new(48_000.0),
            sr: 48_000.0,
            ready: false,
        }
    }

    pub fn ensure_sr(&mut self, sr: f32) {
        let sr = sr.max(1.0);
        if self.ready && (sr - self.sr).abs() <= 1.0 {
            return;
        }
        let max_n = ((MAX_DELAY_SEC * sr).ceil() as usize).max(2);
        self.delay = DelayLine::new(max_n);
        self.delay.set_time_sec(self.delay_time, sr);
        self.reverb = SimpleReverb::new(sr);
        self.reverb.set_size(self.room_size);
        self.sr = sr;
        self.ready = true;
    }

    pub fn clear(&mut self) {
        self.delay.clear();
        self.reverb.clear();
        self.delay_level = 0.0;
        self.room_level = 0.0;
        self.delay_time = 0.25;
        self.delay_fb = 0.5;
        self.room_size = 1.0;
    }

    /// Last-write-wins param update from a pattern hit.
    pub fn set_from_hit(
        &mut self,
        delay: f32,
        delaytime: f32,
        delayfeedback: f32,
        room: f32,
        roomsize: f32,
        sr: f32,
    ) {
        self.ensure_sr(sr);
        self.delay_level = delay.clamp(0.0, 1.0);
        self.delay_time = delaytime.clamp(0.0, MAX_DELAY_SEC);
        self.delay_fb = delayfeedback.clamp(0.0, MAX_DELAY_FEEDBACK);
        self.room_level = room.clamp(0.0, 1.0);
        self.room_size = roomsize.clamp(0.0, 10.0);
        self.delay.set_time_sec(self.delay_time, self.sr);
        self.reverb.set_size(self.room_size);
    }

    /// Process one dry sample: always advances FX state so tails continue.
    #[inline]
    pub fn process(&mut self, dry: f32) -> f32 {
        if !self.ready {
            return dry;
        }
        let delayed = self.delay.process(dry, self.delay_fb);
        let room = self.reverb.process(dry);
        dry + delayed * self.delay_level + room * self.room_level
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

    #[test]
    fn delay_echo_at_time() {
        let sr = 48_000.0f32;
        let mut dl = DelayLine::new((MAX_DELAY_SEC * sr) as usize);
        dl.set_time_sec(0.01, sr); // 480 samples
        let delay_n = 480usize;
        // impulse
        let y0 = dl.process(1.0, 0.0);
        assert!(y0.abs() < 1e-6, "first sample should be empty delay");
        let mut found = 0.0f32;
        for i in 1..=delay_n + 2 {
            let y = dl.process(0.0, 0.0);
            if i == delay_n {
                found = y;
            }
        }
        assert!(found > 0.5, "expected echo near delay time, got {found}");
    }

    #[test]
    fn delay_feedback_clamped() {
        let mut dl = DelayLine::new(1000);
        dl.set_time_sec(0.001, 48_000.0);
        // Even with feedback request of 2.0, energy must stay finite.
        let mut peak = 0.0f32;
        for i in 0..5000 {
            let x = if i == 0 { 0.5 } else { 0.0 };
            let y = dl.process(x, 2.0);
            peak = peak.max(y.abs());
            assert!(y.is_finite());
        }
        assert!(peak < 10.0, "peak={peak}");
    }

    #[test]
    fn reverb_has_tail() {
        let mut r = SimpleReverb::new(48_000.0);
        r.set_size(4.0);
        let _ = r.process(1.0);
        let mut energy = 0.0f32;
        // Skip a few ms then measure residual energy.
        for _ in 0..2000 {
            let _ = r.process(0.0);
        }
        for _ in 0..2000 {
            let y = r.process(0.0);
            energy += y * y;
        }
        assert!(energy > 1e-8, "expected reverb tail energy, got {energy}");
    }

    #[test]
    fn peaking_and_shelf_stable() {
        let mut peak = Biquad::new_eq(BiquadKind::Peaking, 1000.0, 0.707, 6.0, 48_000.0);
        let mut lo = Biquad::new_eq(BiquadKind::LowShelf, 200.0, 0.707, -12.0, 48_000.0);
        let mut hi = Biquad::new_eq(BiquadKind::HighShelf, 6000.0, 0.707, 12.0, 48_000.0);
        for i in 0..2000 {
            let x = (i as f32 * 0.1).sin();
            let y = hi.process(lo.process(peak.process(x)));
            assert!(y.is_finite());
        }
    }

    #[test]
    fn orbit_fx_delay_tail() {
        let mut fx = OrbitFx::new();
        fx.set_from_hit(0.8, 0.02, 0.5, 0.0, 1.0, 48_000.0);
        let _ = fx.process(1.0);
        let mut peak_late = 0.0f32;
        // After ~0.02s (960 samples) echo should appear; keep processing silence.
        for i in 0..2000 {
            let y = fx.process(0.0);
            if i > 900 {
                peak_late = peak_late.max(y.abs());
            }
        }
        assert!(
            peak_late > 0.1,
            "delay wet should leave a tail, peak={peak_late}"
        );
    }

    #[test]
    fn eq_pos_to_db_flat_and_ends() {
        assert!((eq_pos_to_db(0.5)).abs() < 1e-5);
        assert!((eq_pos_to_db(1.0) - 12.0).abs() < 1e-5);
        assert!((eq_pos_to_db(0.0) + 12.0).abs() < 1e-5);
    }
}
