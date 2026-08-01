//! Mixer layer: A/B faders, per-deck 3-band EQ, master 1-pole LPF/HPF, compressor, equal-power xfade.

use crate::dsp::{eq_pos_to_db, Biquad, BiquadKind};

/// Equal-power crossfade over N bars (sample range).
#[derive(Debug, Clone)]
pub struct XFadeState {
    pub to_deck: usize,
    pub start_sample: u64,
    pub end_sample: u64,
}

/// Result of advancing xfade for one process buffer (buffer-head sample).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XFadeTick {
    Idle,
    Active,
    /// Fade finished this tick (gains snapped to the target deck).
    /// Decks keep their songs; only fader positions change.
    Finished {
        from_deck: usize,
        to_deck: usize,
    },
}

/// Per-deck channel EQ: Hi (shelf) / Mid (peak) / Lo (shelf). Positions 0..=1, 0.5 = flat.
/// Dual-mono state (shared coeffs) so stereo pan is preserved.
#[derive(Clone, Debug)]
struct ChannelEq {
    /// [Hi, Mid, Lo] slider positions.
    pos: [f32; 3],
    hi_l: Biquad,
    mid_l: Biquad,
    lo_l: Biquad,
    hi_r: Biquad,
    mid_r: Biquad,
    lo_r: Biquad,
    sr: f32,
}

impl ChannelEq {
    fn new(sr: f32) -> Self {
        let mut eq = Self {
            pos: [0.5, 0.5, 0.5],
            hi_l: Biquad::bypass(),
            mid_l: Biquad::bypass(),
            lo_l: Biquad::bypass(),
            hi_r: Biquad::bypass(),
            mid_r: Biquad::bypass(),
            lo_r: Biquad::bypass(),
            sr: sr.max(1.0),
        };
        eq.rebuild();
        eq
    }

    fn ensure_sr(&mut self, sr: f32) {
        let sr = sr.max(1.0);
        if (sr - self.sr).abs() > 1.0 {
            self.sr = sr;
            self.rebuild();
        }
    }

    fn set_band(&mut self, band: usize, value: f32) {
        if band >= 3 {
            return;
        }
        self.pos[band] = value.clamp(0.0, 1.0);
        self.rebuild();
    }

    fn rebuild(&mut self) {
        let sr = self.sr;
        let q = 0.707;
        // band 0 = Hi, 1 = Mid, 2 = Lo (matches live_ui)
        for (hi, mid, lo) in [
            (&mut self.hi_l, &mut self.mid_l, &mut self.lo_l),
            (&mut self.hi_r, &mut self.mid_r, &mut self.lo_r),
        ] {
            hi.set_coeffs_gain(
                BiquadKind::HighShelf,
                6000.0,
                q,
                eq_pos_to_db(self.pos[0]),
                sr,
            );
            mid.set_coeffs_gain(
                BiquadKind::Peaking,
                1000.0,
                q,
                eq_pos_to_db(self.pos[1]),
                sr,
            );
            lo.set_coeffs_gain(
                BiquadKind::LowShelf,
                200.0,
                q,
                eq_pos_to_db(self.pos[2]),
                sr,
            );
        }
    }

    #[inline]
    fn process_stereo(&mut self, l: f32, r: f32) -> (f32, f32) {
        // Lo → Mid → Hi per channel
        let l = self.hi_l.process(self.mid_l.process(self.lo_l.process(l)));
        let r = self.hi_r.process(self.mid_r.process(self.lo_r.process(r)));
        (l, r)
    }
}

/// Third layer above decks: how A/B are mixed and filtered.
pub struct Mixer {
    pub gain_a: f32,
    pub gain_b: f32,
    /// Master low-pass cutoff Hz (`None` = bypass).
    pub lpf_hz: Option<f32>,
    /// Master high-pass cutoff Hz (`None` = bypass).
    pub hpf_hz: Option<f32>,
    xfade: Option<XFadeState>,
    lpf_y_l: f32,
    lpf_y_r: f32,
    hpf_y_l: f32,
    hpf_y_r: f32,
    hpf_x_prev_l: f32,
    hpf_x_prev_r: f32,
    compressor: Option<crate::dsp::Compressor>,
    comp_sr: f32,
    eq_a: ChannelEq,
    eq_b: ChannelEq,
}

impl Default for Mixer {
    fn default() -> Self {
        Self::new()
    }
}

impl Mixer {
    pub fn new() -> Self {
        Self {
            gain_a: 1.0,
            gain_b: 0.0,
            lpf_hz: None,
            hpf_hz: None,
            xfade: None,
            lpf_y_l: 0.0,
            lpf_y_r: 0.0,
            hpf_y_l: 0.0,
            hpf_y_r: 0.0,
            hpf_x_prev_l: 0.0,
            hpf_x_prev_r: 0.0,
            compressor: None,
            comp_sr: 48_000.0,
            eq_a: ChannelEq::new(48_000.0),
            eq_b: ChannelEq::new(48_000.0),
        }
    }

    pub fn xfade(&self) -> Option<&XFadeState> {
        self.xfade.as_ref()
    }

    pub fn set_deck_gain(&mut self, deck: usize, gain: f32) {
        match deck {
            0 => self.gain_a = gain,
            1 => self.gain_b = gain,
            _ => {}
        }
    }

    pub fn deck_gain(&self, deck: usize) -> f32 {
        match deck {
            0 => self.gain_a,
            1 => self.gain_b,
            _ => 0.0,
        }
    }

    /// Set one EQ band for a deck. `band`: 0=Hi, 1=Mid, 2=Lo. `value`: 0..=1 (0.5 = flat).
    pub fn set_deck_eq(&mut self, deck: usize, band: usize, value: f32) {
        match deck {
            0 => self.eq_a.set_band(band, value),
            1 => self.eq_b.set_band(band, value),
            _ => {}
        }
    }

    /// Slider positions [Hi, Mid, Lo] for a deck.
    pub fn deck_eq(&self, deck: usize) -> [f32; 3] {
        match deck {
            0 => self.eq_a.pos,
            1 => self.eq_b.pos,
            _ => [0.5, 0.5, 0.5],
        }
    }

    /// Equal-power crossfader position in \[0, 1\] (0 = full A, 1 = full B).
    /// Cancels any in-progress multi-bar xfade animation.
    pub fn set_crossfader(&mut self, pos: f32) {
        self.xfade = None;
        let t = pos.clamp(0.0, 1.0) as f64;
        let theta = t * std::f64::consts::FRAC_PI_2;
        self.gain_a = theta.cos() as f32;
        self.gain_b = theta.sin() as f32;
    }

    /// Inverse of equal-power mapping: `atan2(B, A) / (π/2)`.
    pub fn crossfader_pos(&self) -> f32 {
        let a = self.gain_a.max(0.0) as f64;
        let b = self.gain_b.max(0.0) as f64;
        if a + b < 1e-9 {
            return 0.0;
        }
        ((b.atan2(a)) / std::f64::consts::FRAC_PI_2).clamp(0.0, 1.0) as f32
    }

    pub fn set_compressor(&mut self, params: Option<crate::dsp::CompressorParams>, sr: f32) {
        self.comp_sr = sr.max(1.0);
        self.compressor = params.map(|p| crate::dsp::Compressor::new(p, self.comp_sr));
    }

    pub fn compressor_params(&self) -> Option<crate::dsp::CompressorParams> {
        self.compressor.as_ref().map(|c| c.params)
    }

    /// Start equal-power xfade at `global_sample` over `bars` (length in samples).
    pub fn start_xfade(&mut self, to_deck: usize, start_sample: u64, len_samples: u64) {
        if to_deck > 1 {
            return;
        }
        self.xfade = Some(XFadeState {
            to_deck,
            start_sample,
            end_sample: start_sample.saturating_add(len_samples.max(1)),
        });
    }

    pub fn clear_xfade(&mut self) {
        self.xfade = None;
    }

    /// Update gains from xfade using buffer-head `global_sample`.
    pub fn tick_xfade(&mut self, global_sample: u64) -> XFadeTick {
        let Some(x) = self.xfade.clone() else {
            return XFadeTick::Idle;
        };
        let denom = (x.end_sample - x.start_sample).max(1) as f64;
        let t = ((global_sample.saturating_sub(x.start_sample)) as f64 / denom).clamp(0.0, 1.0);
        let theta = t * std::f64::consts::FRAC_PI_2;
        let (from, to) = if x.to_deck == 1 { (0, 1) } else { (1, 0) };
        if from == 0 {
            self.gain_a = theta.cos() as f32;
            self.gain_b = theta.sin() as f32;
        } else {
            self.gain_b = theta.cos() as f32;
            self.gain_a = theta.sin() as f32;
        }

        if t >= 1.0 {
            self.xfade = None;
            if to == 0 {
                self.gain_a = 1.0;
                self.gain_b = 0.0;
            } else {
                self.gain_a = 0.0;
                self.gain_b = 1.0;
            }
            XFadeTick::Finished {
                from_deck: from,
                to_deck: to,
            }
        } else {
            XFadeTick::Active
        }
    }

    /// Mix A/B stereo buffers with channel EQ, faders, master filter, linked compressor.
    #[allow(clippy::too_many_arguments)]
    pub fn mix(
        &mut self,
        out_l: &mut [f32],
        out_r: &mut [f32],
        a_l: &[f32],
        a_r: &[f32],
        b_l: &[f32],
        b_r: &[f32],
        sample_rate: f32,
    ) {
        let n = out_l
            .len()
            .min(out_r.len())
            .min(a_l.len())
            .min(a_r.len())
            .min(b_l.len())
            .min(b_r.len());
        let ga = self.gain_a;
        let gb = self.gain_b;
        let sr = sample_rate.max(1.0);
        self.eq_a.ensure_sr(sr);
        self.eq_b.ensure_sr(sr);

        let lpf_k = self.lpf_hz.map(|cut| {
            let cut = cut.clamp(20.0, sr * 0.45);
            1.0 - (-2.0 * std::f32::consts::PI * cut / sr).exp()
        });
        let hpf_k = self.hpf_hz.map(|cut| {
            let cut = cut.clamp(20.0, sr * 0.45);
            (-2.0 * std::f32::consts::PI * cut / sr).exp()
        });

        for i in 0..n {
            // Channel EQ then fader (DJ mixer style), dual-mono.
            let (ea_l, ea_r) = self.eq_a.process_stereo(a_l[i], a_r[i]);
            let (eb_l, eb_r) = self.eq_b.process_stereo(b_l[i], b_r[i]);
            let mut l = ea_l * ga + eb_l * gb;
            let mut r = ea_r * ga + eb_r * gb;

            if let Some(k) = hpf_k {
                let yl = k * (self.hpf_y_l + l - self.hpf_x_prev_l);
                self.hpf_x_prev_l = l;
                self.hpf_y_l = yl;
                l = yl;
                let yr = k * (self.hpf_y_r + r - self.hpf_x_prev_r);
                self.hpf_x_prev_r = r;
                self.hpf_y_r = yr;
                r = yr;
            }
            if let Some(k) = lpf_k {
                self.lpf_y_l += k * (l - self.lpf_y_l);
                l = self.lpf_y_l;
                self.lpf_y_r += k * (r - self.lpf_y_r);
                r = self.lpf_y_r;
            }

            if let Some(comp) = self.compressor.as_mut() {
                let (cl, cr) = comp.process_stereo(l, r);
                l = cl;
                r = cr;
            }

            out_l[i] = l.clamp(-1.0, 1.0);
            out_r[i] = r.clamp(-1.0, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::CompressorParams;

    #[test]
    fn set_crossfader_equal_power() {
        let mut m = Mixer::new();
        m.set_crossfader(0.0);
        assert!((m.gain_a - 1.0).abs() < 1e-5);
        assert!(m.gain_b.abs() < 1e-5);
        assert!(m.crossfader_pos() < 0.01);

        m.set_crossfader(1.0);
        assert!(m.gain_a.abs() < 1e-5);
        assert!((m.gain_b - 1.0).abs() < 1e-5);
        assert!(m.crossfader_pos() > 0.99);

        m.set_crossfader(0.5);
        assert!((m.gain_a - m.gain_b).abs() < 1e-4);
        assert!((m.crossfader_pos() - 0.5).abs() < 0.02);
        // Manual set cancels animated xfade.
        m.start_xfade(1, 0, 1000);
        assert!(m.xfade().is_some());
        m.set_crossfader(0.25);
        assert!(m.xfade().is_none());
    }

    #[test]
    fn equal_power_endpoints() {
        let mut m = Mixer::new();
        m.gain_a = 1.0;
        m.gain_b = 0.0;
        m.start_xfade(1, 0, 1000);
        let t0 = m.tick_xfade(0);
        assert_eq!(t0, XFadeTick::Active);
        assert!((m.gain_a - 1.0).abs() < 1e-5);
        assert!(m.gain_b.abs() < 1e-5);

        let done = m.tick_xfade(1000);
        assert_eq!(
            done,
            XFadeTick::Finished {
                from_deck: 0,
                to_deck: 1
            }
        );
        assert!(m.gain_a.abs() < 1e-5);
        assert!((m.gain_b - 1.0).abs() < 1e-5);
    }

    fn mix_center(m: &mut Mixer, out: &mut [f32], a: &[f32], b: &[f32], sr: f32) {
        let n = out.len();
        let mut out_l = vec![0.0f32; n];
        let mut out_r = vec![0.0f32; n];
        // Center mono sources: same on L and R (equal-power center would scale; tests use direct L/R).
        m.mix(&mut out_l, &mut out_r, a, a, b, b, sr);
        for i in 0..n {
            out[i] = 0.5 * (out_l[i] + out_r[i]);
        }
    }

    #[test]
    fn compressor_on_mix_reduces_peak() {
        let mut m = Mixer::new();
        m.gain_a = 1.0;
        m.gain_b = 0.0;
        m.set_compressor(
            Some(CompressorParams {
                threshold_db: -12.0,
                ratio: 20.0,
                knee_db: 0.0,
                attack: 0.0,
                release: 0.05,
            }),
            48_000.0,
        );
        let a = vec![0.9f32; 2000];
        let b = vec![0.0f32; 2000];
        let mut out = vec![0.0f32; 2000];
        mix_center(&mut m, &mut out, &a, &b, 48_000.0);
        let peak = out.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
        assert!(peak < 0.9, "peak={peak}");
    }

    #[test]
    fn eq_flat_near_unity() {
        let mut m = Mixer::new();
        m.gain_a = 1.0;
        m.gain_b = 0.0;
        // 1 kHz tone at mid band center — flat EQ should pass nearly unchanged.
        let sr = 48_000.0f32;
        let n = 4000;
        let mut a = vec![0.0f32; n];
        for (i, s) in a.iter_mut().enumerate() {
            *s = (2.0 * std::f32::consts::PI * 1000.0 * i as f32 / sr).sin() * 0.5;
        }
        let b = vec![0.0f32; n];
        let mut out = vec![0.0f32; n];
        mix_center(&mut m, &mut out, &a, &b, sr);
        // Skip filter settling; compare RMS.
        let rms_in: f32 = a[1000..].iter().map(|x| x * x).sum::<f32>() / (n - 1000) as f32;
        let rms_out: f32 = out[1000..].iter().map(|x| x * x).sum::<f32>() / (n - 1000) as f32;
        let ratio = (rms_out / rms_in.max(1e-12)).sqrt();
        assert!(
            (ratio - 1.0).abs() < 0.15,
            "flat EQ should be near unity, ratio={ratio}"
        );
    }

    #[test]
    fn eq_lo_cut_reduces_bass() {
        let sr = 48_000.0f32;
        let n = 8000;
        // 100 Hz sine (below Lo shelf).
        let mut a = vec![0.0f32; n];
        for (i, s) in a.iter_mut().enumerate() {
            *s = (2.0 * std::f32::consts::PI * 100.0 * i as f32 / sr).sin() * 0.5;
        }
        let b = vec![0.0f32; n];

        let mut flat = Mixer::new();
        flat.gain_a = 1.0;
        let mut out_flat = vec![0.0f32; n];
        mix_center(&mut flat, &mut out_flat, &a, &b, sr);

        let mut cut = Mixer::new();
        cut.gain_a = 1.0;
        cut.set_deck_eq(0, 2, 0.0); // Lo min (-12 dB)
        let mut out_cut = vec![0.0f32; n];
        mix_center(&mut cut, &mut out_cut, &a, &b, sr);

        let e_flat: f32 = out_flat[2000..].iter().map(|x| x * x).sum();
        let e_cut: f32 = out_cut[2000..].iter().map(|x| x * x).sum();
        assert!(
            e_cut < e_flat * 0.5,
            "Lo cut should reduce bass energy: cut={e_cut} flat={e_flat}"
        );
        assert_eq!(cut.deck_eq(0)[2], 0.0);
    }

    #[test]
    fn eq_hi_boost_raises_treble() {
        let sr = 48_000.0f32;
        let n = 8000;
        // 8 kHz sine (above Hi shelf).
        let mut a = vec![0.0f32; n];
        for (i, s) in a.iter_mut().enumerate() {
            *s = (2.0 * std::f32::consts::PI * 8000.0 * i as f32 / sr).sin() * 0.3;
        }
        let b = vec![0.0f32; n];

        let mut flat = Mixer::new();
        flat.gain_a = 1.0;
        let mut out_flat = vec![0.0f32; n];
        mix_center(&mut flat, &mut out_flat, &a, &b, sr);

        let mut boost = Mixer::new();
        boost.gain_a = 1.0;
        boost.set_deck_eq(0, 0, 1.0); // Hi max (+12 dB)
        let mut out_boost = vec![0.0f32; n];
        mix_center(&mut boost, &mut out_boost, &a, &b, sr);

        let e_flat: f32 = out_flat[2000..].iter().map(|x| x * x).sum();
        let e_boost: f32 = out_boost[2000..].iter().map(|x| x * x).sum();
        assert!(
            e_boost > e_flat * 1.5,
            "Hi boost should raise treble: boost={e_boost} flat={e_flat}"
        );
    }
}
