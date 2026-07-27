/// Shared clock. Both decks reference the same `global_sample` so bars stay locked.
/// 1 bar = 4 beats fixed.
pub struct Transport {
    pub sample_rate: u32,
    pub bpm: f64,
    pub global_sample: u64,
}

impl Transport {
    pub fn new(sample_rate: u32, bpm: f64) -> Self {
        Self {
            sample_rate,
            bpm,
            global_sample: 0,
        }
    }

    pub fn samples_per_beat(&self) -> f64 {
        self.sample_rate as f64 * 60.0 / self.bpm
    }

    pub fn samples_per_bar(&self) -> f64 {
        self.samples_per_beat() * 4.0
    }

    /// Current bar index (0-based).
    pub fn bar_index(&self) -> u64 {
        (self.global_sample as f64 / self.samples_per_bar()) as u64
    }

    /// Position within the current bar in [0, 1).
    pub fn bar_pos(&self) -> f64 {
        let spb = self.samples_per_bar();
        (self.global_sample as f64 % spb) / spb
    }

    /// Absolute sample position of the bar head `n` bars ahead (n=1 → next bar).
    pub fn bar_boundary(&self, n: u64) -> u64 {
        ((self.bar_index() + n) as f64 * self.samples_per_bar()) as u64
    }

    pub fn advance(&mut self, frames: usize) {
        self.global_sample += frames as u64;
    }

    #[allow(dead_code)]
    pub fn set_bpm(&mut self, bpm: f64) {
        self.bpm = bpm;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_math() {
        let mut t = Transport::new(48_000, 120.0); // 1 bar = 96000 samples
        assert_eq!(t.bar_index(), 0);
        assert_eq!(t.bar_boundary(1), 96_000);
        t.advance(96_000);
        assert_eq!(t.bar_index(), 1);
        assert_eq!(t.bar_boundary(1), 192_000);
        t.advance(48_000); // middle of bar 1
        assert!((t.bar_pos() - 0.5).abs() < 1e-9);
        assert_eq!(t.bar_boundary(1), 192_000);
    }
}
