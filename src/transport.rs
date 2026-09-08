/// Shared clock. Both decks reference the same `global_sample` so bars stay locked.
/// 1 bar = 4 beats fixed.
///
/// Time-repeat (#60) and tape-stop (#61) map **absolute** `global_sample` to a
/// **play** sample (`map_play`) so hits and MIDI follow the relative clock while
/// mix jobs / pending still follow the absolute clock. Only one mapping is
/// active (last-wins).
pub struct Transport {
    pub sample_rate: u32,
    pub bpm: f64,
    pub global_sample: u64,
    repeat: Option<RepeatState>,
    tape: Option<TapeState>,
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

/// Note-value length for one tape-stop shot. 4/4: `1n` = 1 bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TapeLen {
    /// Whole note (全音符): 1 bar.
    Whole,
    /// Half note (二分音符): 2 beats.
    Half,
    /// Quarter note (四分音符): 1 beat.
    Quarter,
    /// Eighth note (八分音符): half a beat.
    Eighth,
}

impl TapeLen {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.trim().to_ascii_lowercase().as_str() {
            "1n" | "1" | "whole" | "bar" | "1bar" => Ok(Self::Whole),
            "2n" | "2" | "half" => Ok(Self::Half),
            "4n" | "4" | "4th" | "quarter" => Ok(Self::Quarter),
            "8n" | "8" | "8th" | "eighth" => Ok(Self::Eighth),
            "1m" => Err("tape len is a note value (1n/2n/4n/8n), not minutes (got 1m)".into()),
            other => Err(format!(
                "tape len must be 1n, 2n, 4n, or 8n (whole/half/quarter/eighth): {other}"
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Whole => "1n",
            Self::Half => "2n",
            Self::Quarter => "4n",
            Self::Eighth => "8n",
        }
    }

    /// How many of this note fit in one 4/4 bar.
    pub fn per_bar(self) -> u32 {
        match self {
            Self::Whole => 1,
            Self::Half => 2,
            Self::Quarter => 4,
            Self::Eighth => 8,
        }
    }
}

/// One tape-stop gesture: length of each shot × how many shots in a row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TapeSpec {
    pub len: TapeLen,
    pub reps: u8,
}

impl TapeSpec {
    pub const ONE_BAR: Self = Self {
        len: TapeLen::Whole,
        reps: 1,
    };
    pub const MAX_REPS: u8 = 8;

    pub fn new(len: TapeLen, reps: u8) -> Result<Self, String> {
        if !(1..=Self::MAX_REPS).contains(&reps) {
            return Err(format!("tape reps must be 1..={}: {reps}", Self::MAX_REPS));
        }
        Ok(Self { len, reps })
    }

    /// `len` may be `4n` or `4n*2`. Optional second token is reps.
    pub fn parse(len: &str, reps: Option<&str>) -> Result<Self, String> {
        let (len_s, star_reps) = match len.split_once('*') {
            Some((l, r)) => (l, Some(r)),
            None => (len, None),
        };
        let len = TapeLen::parse(len_s)?;
        let reps_s = star_reps.or(reps);
        let reps = match reps_s {
            None => 1,
            Some(s) => {
                let n: u32 = s
                    .trim()
                    .parse()
                    .map_err(|_| format!("tape reps must be 1..={}: {s}", Self::MAX_REPS))?;
                if n < 1 || n > Self::MAX_REPS as u32 {
                    return Err(format!("tape reps must be 1..={}: {n}", Self::MAX_REPS));
                }
                n as u8
            }
        };
        Ok(Self { len, reps })
    }

    pub fn as_status(self) -> String {
        if self.reps <= 1 {
            self.len.as_str().to_string()
        } else {
            format!("{}*{}", self.len.as_str(), self.reps)
        }
    }
}

/// Tape-stop playhead: each shot is `play = shot_start + dur * ∫(1-u)^2`.
/// Consecutive shots snap to absolute at the shot boundary then run again.
#[derive(Clone, Copy, Debug)]
struct TapeState {
    origin_abs: u64,
    dur: u64,
    shots: u8,
    spec: TapeSpec,
}

/// Rate curve `rate = (1-u)^2` for tape-stop progress `u` in `[0, 1]`.
pub fn tape_rate(u: f64) -> f64 {
    let x = 1.0 - u.clamp(0.0, 1.0);
    x * x
}

/// Closed-form `∫_0^U (1-u)^2 du = U - U^2 + U^3/3`.
pub fn tape_integral(u: f64) -> f64 {
    let u = u.clamp(0.0, 1.0);
    u - u * u + u * u * u / 3.0
}

/// Instantaneous tape-stop rate at `abs`. `1.0` after `end_abs`.
pub fn tape_rate_at_abs(abs: u64, start_abs: u64, end_abs: u64) -> f64 {
    if abs >= end_abs {
        return 1.0;
    }
    let dur = end_abs.saturating_sub(start_abs).max(1) as f64;
    tape_rate((abs.saturating_sub(start_abs) as f64 / dur).clamp(0.0, 1.0))
}

impl Transport {
    pub fn new(sample_rate: u32, bpm: f64) -> Self {
        Self {
            sample_rate,
            bpm,
            global_sample: 0,
            repeat: None,
            tape: None,
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
        if self
            .tape
            .is_some_and(|t| self.global_sample >= t.total_end())
        {
            self.tape = None;
        }
    }

    #[allow(dead_code)]
    pub fn set_bpm(&mut self, bpm: f64) {
        self.bpm = bpm;
    }

    /// Absolute sample → sounding play sample. Identity when mapping is off or expired.
    pub fn map_play(&self, abs: u64) -> u64 {
        if let Some(t) = self.tape {
            if abs < t.total_end() {
                return t.play_at(abs);
            }
            return abs;
        }
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
    /// Clears tape-stop (last-wins).
    pub fn start_repeat(&mut self, div: RepeatDiv) {
        self.tape = None;
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

    /// Start tape-stop. Clears time-repeat (last-wins).
    pub fn start_tape(&mut self, spec: TapeSpec) {
        self.repeat = None;
        let abs = self.global_sample;
        let per = spec.len.per_bar() as f64;
        let dur = (self.samples_per_bar() / per).round().max(1.0) as u64;
        let shots = spec.reps.max(1);
        self.tape = Some(TapeState {
            origin_abs: abs,
            dur,
            shots,
            spec,
        });
    }

    pub fn clear_tape(&mut self) {
        self.tape = None;
    }

    pub fn tape_on(&self) -> bool {
        self.tape
            .is_some_and(|t| self.global_sample < t.total_end())
    }

    pub fn tape_spec(&self) -> Option<TapeSpec> {
        self.tape
            .filter(|t| self.global_sample < t.total_end())
            .map(|t| t.spec)
    }

    /// `(origin, one-shot samples, shot count)` while running.
    pub fn tape_schedule(&self) -> Option<(u64, u64, u8)> {
        self.tape
            .filter(|t| self.global_sample < t.total_end())
            .map(|t| (t.origin_abs, t.dur.max(1), t.shots.max(1)))
    }

    pub fn tape_end_sample(&self) -> Option<u64> {
        self.tape
            .filter(|t| self.global_sample < t.total_end())
            .map(|t| t.total_end())
    }

    /// Instantaneous tape rate at `abs` (`1.0` when tape is off).
    pub fn tape_rate_at(&self, abs: u64) -> f64 {
        match self.tape {
            Some(t) => t.rate_at(abs),
            None => 1.0,
        }
    }
}

impl TapeState {
    fn total_end(self) -> u64 {
        self.origin_abs
            .saturating_add(self.dur.max(1).saturating_mul(self.shots.max(1) as u64))
    }

    fn shot_window(self, abs: u64) -> Option<(u64, u64)> {
        if abs >= self.total_end() {
            return None;
        }
        let dur = self.dur.max(1);
        let shot = abs.saturating_sub(self.origin_abs) / dur;
        let start = self.origin_abs.saturating_add(shot.saturating_mul(dur));
        Some((start, start.saturating_add(dur)))
    }

    fn play_at(self, abs: u64) -> u64 {
        let Some((start, end)) = self.shot_window(abs) else {
            return abs;
        };
        let dur = end.saturating_sub(start).max(1) as f64;
        let u = (abs.saturating_sub(start) as f64 / dur).clamp(0.0, 1.0);
        (start as f64 + tape_integral(u) * dur).round() as u64
    }

    fn rate_at(self, abs: u64) -> f64 {
        match self.shot_window(abs) {
            Some((start, end)) => tape_rate_at_abs(abs, start, end),
            None => 1.0,
        }
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

    #[test]
    fn tape_integral_matches_closed_form_then_identity() {
        let mut t = Transport::new(48_000, 120.0); // 1 bar = 96000
        t.start_tape(TapeSpec::ONE_BAR);
        assert!(t.tape_on());
        assert_eq!(t.tape_spec().unwrap().as_status(), "1n");
        let dur = 96_000.0;
        for abs in [0_u64, 24_000, 48_000, 72_000, 95_999] {
            let u = abs as f64 / dur;
            let expected = (tape_integral(u) * dur).round() as u64;
            assert_eq!(t.map_play(abs), expected, "abs={abs}");
            let rate = tape_rate(u);
            assert!((t.tape_rate_at(abs) - rate).abs() < 1e-12);
        }
        // At U=1 the playhead has advanced only 1/3 bar, then snaps to absolute.
        let at_end_minus = t.map_play(95_999);
        assert!(
            (at_end_minus as i64 - 32_000).abs() < 4,
            "play at end-1 should be ~dur/3, got {at_end_minus}"
        );
        assert_eq!(t.map_play(96_000), 96_000);
        t.advance(96_000);
        assert!(!t.tape_on());
        assert_eq!(t.map_play(t.global_sample), t.global_sample);
    }

    #[test]
    fn tape_quarter_is_one_beat() {
        let mut t = Transport::new(48_000, 120.0);
        t.start_tape(TapeSpec::parse("4n", None).unwrap());
        let at_end_minus = t.map_play(23_999);
        assert!(
            (at_end_minus as i64 - 8_000).abs() < 4,
            "4n end-1 play ~dur/3, got {at_end_minus}"
        );
        assert_eq!(t.map_play(24_000), 24_000);
        t.advance(24_000);
        assert!(!t.tape_on());
    }

    #[test]
    fn tape_two_quarters_chain_then_identity() {
        let mut t = Transport::new(48_000, 120.0);
        t.start_tape(TapeSpec::parse("4n*2", None).unwrap());
        assert_eq!(t.tape_spec().unwrap().as_status(), "4n*2");
        // End of first shot: play ~ 8000, then snap to 24000 for shot 2.
        assert!(t.map_play(23_999) < 12_000);
        assert_eq!(t.map_play(24_000), 24_000);
        assert!(t.tape_on());
        t.advance(24_000);
        assert!(t.tape_on(), "second 4n still running");
        t.advance(24_000);
        assert!(!t.tape_on());
        assert_eq!(t.map_play(t.global_sample), t.global_sample);
    }

    #[test]
    fn tape_len_parse_rejects_minutes() {
        assert_eq!(TapeLen::parse("4n").unwrap(), TapeLen::Quarter);
        assert_eq!(TapeLen::parse("1n").unwrap(), TapeLen::Whole);
        assert!(TapeLen::parse("1m").is_err());
        assert!(TapeSpec::parse("4n", Some("9")).is_err());
    }

    #[test]
    fn tape_and_repeat_last_wins() {
        let mut t = Transport::new(48_000, 120.0);
        t.start_repeat(RepeatDiv::Sixteenth);
        t.start_tape(TapeSpec::ONE_BAR);
        assert!(t.tape_on());
        assert!(t.repeat_div().is_none());
        t.start_repeat(RepeatDiv::Eighth);
        assert!(!t.tape_on());
        assert_eq!(t.repeat_div(), Some(RepeatDiv::Eighth));
    }
}
