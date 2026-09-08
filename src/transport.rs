/// Shared clock. Both decks reference the same `global_sample` so bars stay locked.
/// 1 bar = 4 beats fixed.
///
/// Time-repeat (#60) maps **absolute** `global_sample` to a **play** sample
/// (`map_play`) so hits and MIDI retrigger inside a note-value slice while
/// mix jobs / pending still follow the absolute clock.
pub struct Transport {
    pub sample_rate: u32,
    pub bpm: f64,
    pub global_sample: u64,
    repeat: Option<RepeatState>,
}

/// Note-value grid for time-repeat. Names are American; 4n = quarter note
/// (四分音符), not minutes. In 4/4 that is `divisions_per_bar` slices.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepeatDiv {
    /// Quarter note (四分音符): 4 slices per bar.
    Quarter,
    /// Eighth note (八分音符): 8 slices per bar.
    Eighth,
    /// Sixteenth note (16分音符): 16 slices per bar.
    Sixteenth,
    /// Thirty-second note (32分音符): 32 slices per bar.
    ThirtySecond,
}

impl RepeatDiv {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.trim().to_ascii_lowercase().as_str() {
            "4" | "4n" | "4th" | "quarter" => Ok(Self::Quarter),
            "8" | "8n" | "8th" | "eighth" => Ok(Self::Eighth),
            "16" | "16n" | "16th" | "sixteenth" => Ok(Self::Sixteenth),
            "32" | "32n" | "32nd" | "thirtysecond" | "thirty-second" => Ok(Self::ThirtySecond),
            other => Err(format!(
                "repeat div must be 4n, 8n, 16n, or 32n (quarter/eighth/16th/32nd notes): {other}"
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Quarter => "4n",
            Self::Eighth => "8n",
            Self::Sixteenth => "16n",
            Self::ThirtySecond => "32n",
        }
    }

    pub fn divisions_per_bar(self) -> u32 {
        match self {
            Self::Quarter => 4,
            Self::Eighth => 8,
            Self::Sixteenth => 16,
            Self::ThirtySecond => 32,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct RepeatState {
    end_abs: u64,
    slice_start: u64,
    slice_len: u64,
    div: RepeatDiv,
}

impl Transport {
    pub fn new(sample_rate: u32, bpm: f64) -> Self {
        Self {
            sample_rate,
            bpm,
            global_sample: 0,
            repeat: None,
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
        if self.repeat.is_some_and(|r| self.global_sample >= r.end_abs) {
            self.repeat = None;
        }
    }

    #[allow(dead_code)]
    pub fn set_bpm(&mut self, bpm: f64) {
        self.bpm = bpm;
    }

    /// Absolute sample → sounding play sample. Identity when repeat is off or expired.
    pub fn map_play(&self, abs: u64) -> u64 {
        let Some(r) = self.repeat else {
            return abs;
        };
        if abs >= r.end_abs {
            return abs;
        }
        let len = r.slice_len.max(1);
        r.slice_start + abs.saturating_sub(r.slice_start) % len
    }

    pub fn play_sample(&self) -> u64 {
        self.map_play(self.global_sample)
    }

    /// Bar index of a sample (absolute or play), 0-based.
    pub fn bar_index_of(&self, sample: u64) -> u64 {
        (sample as f64 / self.samples_per_bar()) as u64
    }

    pub fn play_bar_index(&self) -> u64 {
        self.bar_index_of(self.play_sample())
    }

    /// Active note-value grid, or `None` if off / expired.
    pub fn repeat_div(&self) -> Option<RepeatDiv> {
        self.repeat
            .filter(|r| self.global_sample < r.end_abs)
            .map(|r| r.div)
    }

    pub fn repeat_slice(&self) -> Option<(u64, u64)> {
        self.repeat
            .filter(|r| self.global_sample < r.end_abs)
            .map(|r| (r.slice_start, r.slice_len.max(1)))
    }

    pub fn repeat_end_sample(&self) -> Option<u64> {
        self.repeat
            .filter(|r| self.global_sample < r.end_abs)
            .map(|r| r.end_abs)
    }

    /// Snap to the current note-value window and run for one absolute bar.
    pub fn start_repeat(&mut self, div: RepeatDiv) {
        let d = div.divisions_per_bar() as f64;
        let spb = self.samples_per_bar().max(1.0);
        let slice_len = ((spb / d).round() as u64).max(1);
        let abs = self.global_sample;
        let slice_start = (abs / slice_len) * slice_len;
        let end_abs = abs.saturating_add(spb.round() as u64);
        self.repeat = Some(RepeatState {
            end_abs,
            slice_start,
            slice_len,
            div,
        });
    }

    pub fn clear_repeat(&mut self) {
        self.repeat = None;
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

    #[test]
    fn repeat_div_parse_is_note_values_not_minutes() {
        assert_eq!(RepeatDiv::parse("4n").unwrap(), RepeatDiv::Quarter);
        assert_eq!(RepeatDiv::parse("4").unwrap(), RepeatDiv::Quarter);
        assert_eq!(RepeatDiv::parse("16th").unwrap(), RepeatDiv::Sixteenth);
        assert_eq!(RepeatDiv::parse("32").unwrap(), RepeatDiv::ThirtySecond);
        assert_eq!(RepeatDiv::Quarter.divisions_per_bar(), 4);
        assert_eq!(RepeatDiv::Sixteenth.divisions_per_bar(), 16);
        assert!(RepeatDiv::parse("1m").is_err());
    }

    #[test]
    fn map_play_wraps_sixteenth_then_identity_after_one_bar() {
        let mut t = Transport::new(48_000, 120.0); // 1 bar = 96000
        t.global_sample = 3_000; // first 16th window starts at 0
        t.start_repeat(RepeatDiv::Sixteenth);
        assert_eq!(t.repeat_div().unwrap().as_str(), "16n");
        let (slice_start, slice_len) = t.repeat_slice().unwrap();
        assert_eq!(slice_start, 0);
        assert_eq!(slice_len, 6_000);
        assert_eq!(t.map_play(3_000), 3_000);
        assert_eq!(t.map_play(6_000), 0);
        assert_eq!(t.map_play(9_000), 3_000);
        // One absolute bar later: identity.
        assert_eq!(t.map_play(3_000 + 96_000), 99_000);
        t.advance(96_000);
        assert!(t.repeat_div().is_none());
        assert_eq!(t.map_play(t.global_sample), t.global_sample);
    }

    #[test]
    fn map_play_quarter_snaps_to_current_beat() {
        let mut t = Transport::new(48_000, 120.0);
        t.global_sample = 30_000; // beat 2 starts at 24000 (四分音符)
        t.start_repeat(RepeatDiv::Quarter);
        let (slice_start, slice_len) = t.repeat_slice().unwrap();
        assert_eq!(slice_start, 24_000);
        assert_eq!(slice_len, 24_000);
        assert_eq!(t.map_play(48_000), 24_000);
    }
}
