//! Mixer layer: A/B faders, master 1-pole LPF/HPF, equal-power xfade.

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
    /// Fade finished this tick; engine should unload `from_deck`.
    Finished {
        from_deck: usize,
        to_deck: usize,
    },
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
    lpf_y: f32,
    hpf_y: f32,
    hpf_x_prev: f32,
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
            lpf_y: 0.0,
            hpf_y: 0.0,
            hpf_x_prev: 0.0,
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
        // from → cos (1→0), to → sin (0→1)
        if from == 0 {
            self.gain_a = theta.cos() as f32;
            self.gain_b = theta.sin() as f32;
        } else {
            self.gain_b = theta.cos() as f32;
            self.gain_a = theta.sin() as f32;
        }

        if t >= 1.0 {
            self.xfade = None;
            // Snap to destination
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

    /// Mix A/B mono buffers with faders + master EQ into `out` (same length).
    pub fn mix(&mut self, out: &mut [f32], a: &[f32], b: &[f32], sample_rate: f32) {
        let n = out.len().min(a.len()).min(b.len());
        let ga = self.gain_a;
        let gb = self.gain_b;
        let sr = sample_rate.max(1.0);

        let lpf_k = self.lpf_hz.map(|cut| {
            let cut = cut.clamp(20.0, sr * 0.45);
            1.0 - (-2.0 * std::f32::consts::PI * cut / sr).exp()
        });
        let hpf_k = self.hpf_hz.map(|cut| {
            let cut = cut.clamp(20.0, sr * 0.45);
            (-2.0 * std::f32::consts::PI * cut / sr).exp()
        });

        for i in 0..n {
            let mut x = a[i] * ga + b[i] * gb;

            if let Some(k) = hpf_k {
                // One-pole HPF: y[n] = k * (y[n-1] + x[n] - x[n-1])
                let y = k * (self.hpf_y + x - self.hpf_x_prev);
                self.hpf_x_prev = x;
                self.hpf_y = y;
                x = y;
            }
            if let Some(k) = lpf_k {
                self.lpf_y += k * (x - self.lpf_y);
                x = self.lpf_y;
            }

            out[i] = x.clamp(-1.0, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
