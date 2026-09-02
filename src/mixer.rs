//! Mixer layer: A/B faders, per-deck 3-band EQ, master 1-pole LPF/HPF, compressor, equal-power xfade.
//! DJ mix macros (fill / switch) live here as a preallocated MixJob.

use std::sync::Arc;

use crate::dsp::{eq_pos_to_db, Biquad, BiquadKind, DelayLine, MAX_DELAY_SEC};

/// Isolator-style Lo kill: high-pass so kick/bass does not leak through a −12 dB shelf.
const LO_KILL_HZ: f32 = 250.0;
const LPF_SWEEP_START_HZ: f32 = 12_000.0;
const LPF_SWEEP_END_HZ: f32 = 200.0;
const DELAY_WET: f32 = 0.5;
const ECHO_WET_START: f32 = 0.25;
const ECHO_WET_END: f32 = 0.70;
const ECHO_FB_START: f32 = 0.20;
const ECHO_FB_END: f32 = 0.80;
const HPF_SWEEP_START_HZ: f32 = 40.0;
const HPF_SWEEP_END_HZ: f32 = 2_000.0;
const RISER_GAIN: f32 = 0.35;
/// Click-soften on switch/cut snaps (AGENTS.md). Not a buffer split.
const GAIN_RAMP_SEC: f32 = 0.005;
const DELAY_MAX_SR: f32 = 96_000.0;

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

/// `strudel_mix` / `POST /mix` action (hold is a separate immediate command).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MixAction {
    Long,
    Cut,
    Fill,
}

impl MixAction {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.trim().to_ascii_lowercase().as_str() {
            "long" | "blend" => Ok(Self::Long),
            "cut" | "cutin" | "cut-in" => Ok(Self::Cut),
            "fill" => Ok(Self::Fill),
            other => Err(format!(
                "move must be long, cut, or fill (hold is a separate call): {other}"
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Long => "long",
            Self::Cut => "cut",
            Self::Fill => "fill",
        }
    }

    pub fn default_bars(self, fill: Option<FillKind>) -> u32 {
        match self {
            Self::Long => 8,
            Self::Cut => 1,
            Self::Fill => fill.map(FillKind::default_bars).unwrap_or(1),
        }
    }
}

/// Fill / switch flavour for [`MixAction::Fill`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillKind {
    Delay,
    Lpf,
    Flash,
    Riser,
    Switch,
    Echo,
    Hpf,
    Roll,
    Drop,
}

impl FillKind {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.trim().to_ascii_lowercase().as_str() {
            "delay" => Ok(Self::Delay),
            "lpf" | "filter" => Ok(Self::Lpf),
            "flash" => Ok(Self::Flash),
            "riser" => Ok(Self::Riser),
            "switch" => Ok(Self::Switch),
            "echo" | "echoout" | "echo-out" => Ok(Self::Echo),
            "hpf" => Ok(Self::Hpf),
            "roll" | "loop" | "repeat" => Ok(Self::Roll),
            "drop" | "impact" => Ok(Self::Drop),
            other => Err(format!(
                "kind must be delay, lpf, flash, riser, switch, echo, hpf, roll, or drop: {other}"
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Delay => "delay",
            Self::Lpf => "lpf",
            Self::Flash => "flash",
            Self::Riser => "riser",
            Self::Switch => "switch",
            Self::Echo => "echo",
            Self::Hpf => "hpf",
            Self::Roll => "roll",
            Self::Drop => "drop",
        }
    }

    pub fn default_bars(self) -> u32 {
        match self {
            Self::Riser => 2,
            _ => 1,
        }
    }
}

/// Beat grid for switch / flash. Delay fill stays 8th-note regardless.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MixGrid {
    Eighth,
    Quarter,
}

impl MixGrid {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.trim().to_ascii_lowercase().as_str() {
            "8n" | "8" | "eighth" | "8th" => Ok(Self::Eighth),
            "4n" | "4" | "quarter" | "4th" => Ok(Self::Quarter),
            other => Err(format!("grid must be 8n or 4n: {other}")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Eighth => "8n",
            Self::Quarter => "4n",
        }
    }

    pub fn divisions_per_bar(self) -> u32 {
        match self {
            Self::Eighth => 8,
            Self::Quarter => 4,
        }
    }
}

/// Queued DJ mix macro (bar-quantized except [`Command::HoldXFade`](crate::engine::Command)).
#[derive(Clone, Debug)]
pub struct MixCommand {
    pub action: MixAction,
    pub to_deck: usize,
    pub bars: u32,
    pub eq: bool,
    pub reset_eq: bool,
    pub fill: Option<FillKind>,
    pub grid: MixGrid,
    pub mute_track: Option<String>,
    pub phrase: u32,
}

/// Snapshot for `/status` while a mix macro is running.
#[derive(Clone, Debug, PartialEq)]
pub struct MixStatus {
    pub move_name: String,
    pub kind: Option<String>,
    pub to: String,
    pub bars_left: u32,
}

fn deck_letter(deck: usize) -> String {
    if deck == 1 {
        "B".into()
    } else {
        "A".into()
    }
}

#[derive(Debug)]
enum MixJob {
    Delay {
        to_deck: usize,
        reset_eq: bool,
        end_sample: u64,
        samples_per_bar: u64,
    },
    Lpf {
        to_deck: usize,
        reset_eq: bool,
        start_sample: u64,
        end_sample: u64,
        start_hz: f32,
        end_hz: f32,
        samples_per_bar: u64,
    },
    Flash {
        to_deck: usize,
        outgoing: usize,
        reset_eq: bool,
        start_sample: u64,
        end_sample: u64,
        period: u64,
        last_step: i64,
        samples_per_bar: u64,
    },
    Riser {
        to_deck: usize,
        reset_eq: bool,
        end_sample: u64,
        samples_per_bar: u64,
    },
    Switch {
        to_deck: usize,
        first: usize,
        reset_eq: bool,
        start_sample: u64,
        end_sample: u64,
        period: u64,
        last_step: i64,
        samples_per_bar: u64,
    },
    Echo {
        to_deck: usize,
        reset_eq: bool,
        start_sample: u64,
        end_sample: u64,
        samples_per_bar: u64,
    },
    Hpf {
        to_deck: usize,
        reset_eq: bool,
        start_sample: u64,
        end_sample: u64,
        start_hz: f32,
        end_hz: f32,
        samples_per_bar: u64,
    },
    Drop {
        to_deck: usize,
        reset_eq: bool,
        end_sample: u64,
        samples_per_bar: u64,
    },
    Roll {
        to_deck: usize,
        reset_eq: bool,
        end_sample: u64,
        samples_per_bar: u64,
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
    kill_l: Biquad,
    kill_r: Biquad,
    lo_kill: bool,
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
            kill_l: Biquad::bypass(),
            kill_r: Biquad::bypass(),
            lo_kill: false,
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
        self.kill_l
            .set_coeffs(BiquadKind::HighPass, LO_KILL_HZ, q, sr);
        self.kill_r
            .set_coeffs(BiquadKind::HighPass, LO_KILL_HZ, q, sr);
    }

    fn set_lo_kill(&mut self, kill: bool) {
        self.lo_kill = kill;
    }

    #[inline]
    fn process_stereo(&mut self, l: f32, r: f32) -> (f32, f32) {
        // Lo shelf skipped while isolator kill is on (HPF only).
        let l = if self.lo_kill {
            self.kill_l.process(l)
        } else {
            self.lo_l.process(l)
        };
        let r = if self.lo_kill {
            self.kill_r.process(r)
        } else {
            self.lo_r.process(r)
        };
        let l = self.hi_l.process(self.mid_l.process(l));
        let r = self.hi_r.process(self.mid_r.process(r));
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
    job: Option<MixJob>,
    mix_status: Option<MixStatus>,
    delay_l: DelayLine,
    delay_r: DelayLine,
    delay_wet: f32,
    delay_fb: f32,
    flash_mul: [f32; 2],
    riser_pcm: Option<Arc<Vec<f32>>>,
    riser_idx: usize,
    roll_l: Vec<f32>,
    roll_r: Vec<f32>,
    roll_cap: usize,
    roll_i: usize,
    roll_target: usize,
    /// Linear gain ramp toward `gain_a`/`gain_b` (switch/cut snaps).
    ramp_from_a: f32,
    ramp_from_b: f32,
    ramp_i: u32,
    ramp_n: u32,
}

impl Default for Mixer {
    fn default() -> Self {
        Self::new()
    }
}

impl Mixer {
    pub fn new() -> Self {
        let delay_n = (MAX_DELAY_SEC * DELAY_MAX_SR).ceil() as usize;
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
            job: None,
            mix_status: None,
            delay_l: DelayLine::new(delay_n),
            delay_r: DelayLine::new(delay_n),
            delay_wet: 0.0,
            delay_fb: 0.0,
            flash_mul: [1.0, 1.0],
            riser_pcm: None,
            riser_idx: 0,
            roll_l: vec![0.0; delay_n],
            roll_r: vec![0.0; delay_n],
            roll_cap: 0,
            roll_i: 0,
            roll_target: 0,
            ramp_from_a: 1.0,
            ramp_from_b: 0.0,
            ramp_i: 0,
            ramp_n: 0,
        }
    }

    pub fn xfade(&self) -> Option<&XFadeState> {
        self.xfade.as_ref()
    }

    pub fn mix_status(&self) -> Option<&MixStatus> {
        self.mix_status.as_ref()
    }

    pub fn has_mix_job(&self) -> bool {
        self.job.is_some()
    }

    pub fn lo_kill(&self, deck: usize) -> bool {
        match deck {
            0 => self.eq_a.lo_kill,
            1 => self.eq_b.lo_kill,
            _ => false,
        }
    }

    pub fn set_lo_kill(&mut self, deck: usize, kill: bool) {
        match deck {
            0 => self.eq_a.set_lo_kill(kill),
            1 => self.eq_b.set_lo_kill(kill),
            _ => {}
        }
    }

    /// Bass-swap isolator preset for a long mix into `to_deck`.
    pub fn apply_long_eq_preset(&mut self, to_deck: usize) {
        if to_deck > 1 {
            return;
        }
        let from = 1 - to_deck;
        self.set_lo_kill(from, true);
        self.set_lo_kill(to_deck, false);
        self.set_deck_eq(from, 0, 0.5);
        self.set_deck_eq(from, 1, 0.5);
        self.set_deck_eq(from, 2, 0.5);
        self.set_deck_eq(to_deck, 0, 0.5);
        self.set_deck_eq(to_deck, 1, 0.0);
        self.set_deck_eq(to_deck, 2, 0.5);
    }

    pub fn reset_eq_flat(&mut self) {
        for d in 0..2 {
            self.set_lo_kill(d, false);
            for b in 0..3 {
                self.set_deck_eq(d, b, 0.5);
            }
        }
    }

    /// Freeze current equal-power gains and cancel an in-progress xfade animation.
    pub fn hold_xfade(&mut self) {
        self.xfade = None;
    }

    pub fn set_riser_pcm(&mut self, pcm: Option<Arc<Vec<f32>>>) {
        self.riser_pcm = pcm;
        self.riser_idx = 0;
    }

    pub fn oneshot_needs_pcm(&self) -> Option<&'static str> {
        match self.job {
            Some(MixJob::Riser { .. }) if self.riser_pcm.is_none() => Some("riser"),
            Some(MixJob::Drop { .. }) if self.riser_pcm.is_none() => Some("drop"),
            _ => None,
        }
    }

    /// Stop fill inserts (delay/LPF/flash/riser) without changing faders.
    pub fn clear_job(&mut self) {
        self.job = None;
        self.mix_status = None;
        self.delay_wet = 0.0;
        self.delay_fb = 0.0;
        self.delay_l.clear();
        self.delay_r.clear();
        self.flash_mul = [1.0, 1.0];
        self.riser_pcm = None;
        self.riser_idx = 0;
        self.lpf_hz = None;
        self.hpf_hz = None;
        self.roll_target = 0;
        self.roll_cap = 0;
        self.roll_i = 0;
    }

    fn begin_gain_ramp(&mut self, sr: f32) {
        self.ramp_from_a = self.current_gain_a();
        self.ramp_from_b = self.current_gain_b();
        let n = (GAIN_RAMP_SEC * sr.max(1.0)).round() as u32;
        self.ramp_n = n.max(1);
        self.ramp_i = 0;
    }

    fn current_gain_a(&self) -> f32 {
        if self.ramp_n == 0 || self.ramp_i >= self.ramp_n {
            self.gain_a
        } else {
            let t = self.ramp_i as f32 / self.ramp_n as f32;
            self.ramp_from_a + (self.gain_a - self.ramp_from_a) * t
        }
    }

    fn current_gain_b(&self) -> f32 {
        if self.ramp_n == 0 || self.ramp_i >= self.ramp_n {
            self.gain_b
        } else {
            let t = self.ramp_i as f32 / self.ramp_n as f32;
            self.ramp_from_b + (self.gain_b - self.ramp_from_b) * t
        }
    }

    fn snap_crossfader(&mut self, pos: f32, ramp: bool, sr: f32) {
        self.xfade = None;
        let t = pos.clamp(0.0, 1.0) as f64;
        let theta = t * std::f64::consts::FRAC_PI_2;
        let na = theta.cos() as f32;
        let nb = theta.sin() as f32;
        if ramp {
            self.begin_gain_ramp(sr);
        } else {
            self.ramp_n = 0;
            self.ramp_i = 0;
        }
        self.gain_a = na;
        self.gain_b = nb;
    }

    fn finish_job(&mut self, to_deck: usize, reset_eq: bool, sr: f32) {
        let to = if to_deck > 1 { 0 } else { to_deck };
        self.delay_wet = 0.0;
        self.delay_fb = 0.0;
        self.delay_l.clear();
        self.delay_r.clear();
        self.flash_mul = [1.0, 1.0];
        self.riser_pcm = None;
        self.riser_idx = 0;
        self.lpf_hz = None;
        self.hpf_hz = None;
        self.roll_target = 0;
        self.roll_cap = 0;
        self.roll_i = 0;
        self.job = None;
        self.mix_status = None;
        if reset_eq {
            self.reset_eq_flat();
        }
        self.snap_crossfader(if to == 1 { 1.0 } else { 0.0 }, true, sr);
    }

    fn set_mix_status(&mut self, move_name: &str, kind: Option<&str>, to_deck: usize, bars: u32) {
        self.mix_status = Some(MixStatus {
            move_name: move_name.into(),
            kind: kind.map(|s| s.to_string()),
            to: deck_letter(to_deck),
            bars_left: bars,
        });
    }

    fn update_bars_left(&mut self, global_sample: u64, end: u64, spb: u64) {
        if let Some(st) = self.mix_status.as_mut() {
            let remain = end.saturating_sub(global_sample);
            st.bars_left = if spb == 0 {
                0
            } else {
                remain.div_ceil(spb) as u32
            };
        }
    }

    /// Start a fill MixJob at `start_sample`. `bars` is duration before cut-in.
    #[allow(clippy::too_many_arguments)]
    pub fn start_fill(
        &mut self,
        kind: FillKind,
        to_deck: usize,
        bars: u32,
        reset_eq: bool,
        grid: MixGrid,
        start_sample: u64,
        samples_per_bar: f64,
        bpm: f64,
        sr: f32,
    ) {
        if to_deck > 1 {
            return;
        }
        self.clear_job();
        let bars = bars.clamp(1, 32);
        let spb = samples_per_bar.max(1.0) as u64;
        let len = (bars as f64 * samples_per_bar).max(1.0) as u64;
        let end = start_sample.saturating_add(len);
        let period = (spb / grid.divisions_per_bar() as u64).max(1);
        let outgoing = 1 - to_deck;
        self.set_mix_status("fill", Some(kind.as_str()), to_deck, bars);
        match kind {
            FillKind::Delay => {
                let eighth_sec = (60.0 / bpm.max(1.0) / 2.0) as f32;
                self.delay_l.set_time_sec(eighth_sec, sr);
                self.delay_r.set_time_sec(eighth_sec, sr);
                self.delay_l.clear();
                self.delay_r.clear();
                self.delay_wet = DELAY_WET;
                self.delay_fb = 0.0;
                self.job = Some(MixJob::Delay {
                    to_deck,
                    reset_eq,
                    end_sample: end,
                    samples_per_bar: spb,
                });
            }
            FillKind::Lpf => {
                self.lpf_hz = Some(LPF_SWEEP_START_HZ);
                self.job = Some(MixJob::Lpf {
                    to_deck,
                    reset_eq,
                    start_sample,
                    end_sample: end,
                    start_hz: LPF_SWEEP_START_HZ,
                    end_hz: LPF_SWEEP_END_HZ,
                    samples_per_bar: spb,
                });
            }
            FillKind::Flash => {
                self.flash_mul = [1.0, 1.0];
                self.job = Some(MixJob::Flash {
                    to_deck,
                    outgoing,
                    reset_eq,
                    start_sample,
                    end_sample: end,
                    period,
                    last_step: -1,
                    samples_per_bar: spb,
                });
            }
            FillKind::Riser => {
                self.riser_idx = 0;
                self.job = Some(MixJob::Riser {
                    to_deck,
                    reset_eq,
                    end_sample: end,
                    samples_per_bar: spb,
                });
            }
            FillKind::Switch => {
                self.job = Some(MixJob::Switch {
                    to_deck,
                    first: outgoing,
                    reset_eq,
                    start_sample,
                    end_sample: end,
                    period,
                    last_step: -1,
                    samples_per_bar: spb,
                });
            }
            FillKind::Echo => {
                let eighth_sec = (60.0 / bpm.max(1.0) / 2.0) as f32;
                self.delay_l.set_time_sec(eighth_sec, sr);
                self.delay_r.set_time_sec(eighth_sec, sr);
                self.delay_l.clear();
                self.delay_r.clear();
                self.delay_wet = ECHO_WET_START;
                self.delay_fb = ECHO_FB_START;
                self.job = Some(MixJob::Echo {
                    to_deck,
                    reset_eq,
                    start_sample,
                    end_sample: end,
                    samples_per_bar: spb,
                });
            }
            FillKind::Hpf => {
                self.hpf_hz = Some(HPF_SWEEP_START_HZ);
                self.job = Some(MixJob::Hpf {
                    to_deck,
                    reset_eq,
                    start_sample,
                    end_sample: end,
                    start_hz: HPF_SWEEP_START_HZ,
                    end_hz: HPF_SWEEP_END_HZ,
                    samples_per_bar: spb,
                });
            }
            FillKind::Drop => {
                self.riser_idx = 0;
                self.job = Some(MixJob::Drop {
                    to_deck,
                    reset_eq,
                    end_sample: end,
                    samples_per_bar: spb,
                });
            }
            FillKind::Roll => {
                self.roll_target = (period as usize).min(self.roll_l.len()).max(1);
                self.roll_cap = 0;
                self.roll_i = 0;
                self.roll_l.fill(0.0);
                self.roll_r.fill(0.0);
                self.job = Some(MixJob::Roll {
                    to_deck,
                    reset_eq,
                    end_sample: end,
                    samples_per_bar: spb,
                });
            }
        }
    }

    /// Advance MixJob at buffer head. Returns true if a cut-in just happened.
    pub fn tick_job(&mut self, global_sample: u64, sr: f32) -> bool {
        let finished = self.job.as_ref().and_then(|job| {
            let end = match job {
                MixJob::Delay { end_sample, .. }
                | MixJob::Lpf { end_sample, .. }
                | MixJob::Flash { end_sample, .. }
                | MixJob::Riser { end_sample, .. }
                | MixJob::Switch { end_sample, .. }
                | MixJob::Echo { end_sample, .. }
                | MixJob::Hpf { end_sample, .. }
                | MixJob::Roll { end_sample, .. }
                | MixJob::Drop { end_sample, .. } => *end_sample,
            };
            if global_sample < end {
                return None;
            }
            match job {
                MixJob::Delay {
                    to_deck, reset_eq, ..
                }
                | MixJob::Lpf {
                    to_deck, reset_eq, ..
                }
                | MixJob::Flash {
                    to_deck, reset_eq, ..
                }
                | MixJob::Riser {
                    to_deck, reset_eq, ..
                }
                | MixJob::Switch {
                    to_deck, reset_eq, ..
                }
                | MixJob::Echo {
                    to_deck, reset_eq, ..
                }
                | MixJob::Hpf {
                    to_deck, reset_eq, ..
                }
                | MixJob::Roll {
                    to_deck, reset_eq, ..
                }
                | MixJob::Drop {
                    to_deck, reset_eq, ..
                } => Some((*to_deck, *reset_eq)),
            }
        });
        if let Some((to, reset)) = finished {
            self.finish_job(to, reset, sr);
            return true;
        }

        let Some(job) = self.job.as_ref() else {
            return false;
        };

        // Copy what we need so we can mutate mixer fields without aliasing `job`.
        enum Step {
            Delay {
                end: u64,
                spb: u64,
            },
            Lpf {
                start: u64,
                end: u64,
                start_hz: f32,
                end_hz: f32,
                spb: u64,
            },
            Flash {
                outgoing: usize,
                start: u64,
                end: u64,
                period: u64,
                last_step: i64,
                spb: u64,
            },
            Riser {
                end: u64,
                spb: u64,
            },
            Switch {
                first: usize,
                start: u64,
                end: u64,
                period: u64,
                last_step: i64,
                spb: u64,
            },
            Echo {
                start: u64,
                end: u64,
                spb: u64,
            },
            Hpf {
                start: u64,
                end: u64,
                start_hz: f32,
                end_hz: f32,
                spb: u64,
            },
            Roll {
                end: u64,
                spb: u64,
            },
            Drop {
                end: u64,
                spb: u64,
            },
        }
        let step = match job {
            MixJob::Delay {
                end_sample,
                samples_per_bar,
                ..
            } => Step::Delay {
                end: *end_sample,
                spb: *samples_per_bar,
            },
            MixJob::Lpf {
                start_sample,
                end_sample,
                start_hz,
                end_hz,
                samples_per_bar,
                ..
            } => Step::Lpf {
                start: *start_sample,
                end: *end_sample,
                start_hz: *start_hz,
                end_hz: *end_hz,
                spb: *samples_per_bar,
            },
            MixJob::Flash {
                outgoing,
                start_sample,
                end_sample,
                period,
                last_step,
                samples_per_bar,
                ..
            } => Step::Flash {
                outgoing: *outgoing,
                start: *start_sample,
                end: *end_sample,
                period: *period,
                last_step: *last_step,
                spb: *samples_per_bar,
            },
            MixJob::Riser {
                end_sample,
                samples_per_bar,
                ..
            } => Step::Riser {
                end: *end_sample,
                spb: *samples_per_bar,
            },
            MixJob::Switch {
                first,
                start_sample,
                end_sample,
                period,
                last_step,
                samples_per_bar,
                ..
            } => Step::Switch {
                first: *first,
                start: *start_sample,
                end: *end_sample,
                period: *period,
                last_step: *last_step,
                spb: *samples_per_bar,
            },
            MixJob::Echo {
                start_sample,
                end_sample,
                samples_per_bar,
                ..
            } => Step::Echo {
                start: *start_sample,
                end: *end_sample,
                spb: *samples_per_bar,
            },
            MixJob::Hpf {
                start_sample,
                end_sample,
                start_hz,
                end_hz,
                samples_per_bar,
                ..
            } => Step::Hpf {
                start: *start_sample,
                end: *end_sample,
                start_hz: *start_hz,
                end_hz: *end_hz,
                spb: *samples_per_bar,
            },
            MixJob::Roll {
                end_sample,
                samples_per_bar,
                ..
            } => Step::Roll {
                end: *end_sample,
                spb: *samples_per_bar,
            },
            MixJob::Drop {
                end_sample,
                samples_per_bar,
                ..
            } => Step::Drop {
                end: *end_sample,
                spb: *samples_per_bar,
            },
        };

        match step {
            Step::Delay { end, spb } => self.update_bars_left(global_sample, end, spb),
            Step::Lpf {
                start,
                end,
                start_hz,
                end_hz,
                spb,
            } => {
                let denom = (end - start).max(1) as f32;
                let t = (global_sample.saturating_sub(start) as f32 / denom).clamp(0.0, 1.0);
                self.lpf_hz = Some(start_hz + (end_hz - start_hz) * t);
                self.update_bars_left(global_sample, end, spb);
            }
            Step::Flash {
                outgoing,
                start,
                end,
                period,
                last_step,
                spb,
            } => {
                let cur = (global_sample.saturating_sub(start) / period.max(1)) as i64;
                if cur != last_step {
                    if let Some(MixJob::Flash { last_step: ls, .. }) = self.job.as_mut() {
                        *ls = cur;
                    }
                    let on = cur % 2 == 0;
                    self.flash_mul = [1.0, 1.0];
                    if outgoing < 2 {
                        self.flash_mul[outgoing] = if on { 1.0 } else { 0.0 };
                    }
                }
                self.update_bars_left(global_sample, end, spb);
            }
            Step::Riser { end, spb } => self.update_bars_left(global_sample, end, spb),
            Step::Switch {
                first,
                start,
                end,
                period,
                last_step,
                spb,
            } => {
                let cur = (global_sample.saturating_sub(start) / period.max(1)) as i64;
                if cur != last_step {
                    if let Some(MixJob::Switch { last_step: ls, .. }) = self.job.as_mut() {
                        *ls = cur;
                    }
                    let deck = if cur % 2 == 0 { first } else { 1 - first };
                    self.snap_crossfader(if deck == 1 { 1.0 } else { 0.0 }, true, sr);
                }
                self.update_bars_left(global_sample, end, spb);
            }
            Step::Echo { start, end, spb } => {
                let denom = (end - start).max(1) as f32;
                let t = (global_sample.saturating_sub(start) as f32 / denom).clamp(0.0, 1.0);
                self.delay_wet = ECHO_WET_START + (ECHO_WET_END - ECHO_WET_START) * t;
                self.delay_fb = ECHO_FB_START + (ECHO_FB_END - ECHO_FB_START) * t;
                self.update_bars_left(global_sample, end, spb);
            }
            Step::Hpf {
                start,
                end,
                start_hz,
                end_hz,
                spb,
            } => {
                let denom = (end - start).max(1) as f32;
                let t = (global_sample.saturating_sub(start) as f32 / denom).clamp(0.0, 1.0);
                self.hpf_hz = Some(start_hz + (end_hz - start_hz) * t);
                self.update_bars_left(global_sample, end, spb);
            }
            Step::Roll { end, spb } => self.update_bars_left(global_sample, end, spb),
            Step::Drop { end, spb } => self.update_bars_left(global_sample, end, spb),
        }
        false
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
        self.clear_job();
        self.snap_crossfader(pos, false, 48_000.0);
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
        self.clear_job();
        let len = len_samples.max(1);
        self.xfade = Some(XFadeState {
            to_deck,
            start_sample,
            end_sample: start_sample.saturating_add(len),
        });
    }

    pub fn note_long_mix(&mut self, to_deck: usize, bars: u32) {
        self.set_mix_status("long", None, to_deck, bars.max(1));
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
            if self.job.is_none() {
                self.mix_status = None;
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
        let sr = sample_rate.max(1.0);
        self.eq_a.ensure_sr(sr);
        self.eq_b.ensure_sr(sr);
        let delay_wet = self.delay_wet;
        let delay_fb = self.delay_fb;
        let fa = self.flash_mul[0];
        let fb = self.flash_mul[1];
        let ramping = self.ramp_n > 0 && self.ramp_i < self.ramp_n;

        let lpf_k = self.lpf_hz.map(|cut| {
            let cut = cut.clamp(20.0, sr * 0.45);
            1.0 - (-2.0 * std::f32::consts::PI * cut / sr).exp()
        });
        let hpf_k = self.hpf_hz.map(|cut| {
            let cut = cut.clamp(20.0, sr * 0.45);
            (-2.0 * std::f32::consts::PI * cut / sr).exp()
        });

        for i in 0..n {
            let (ga, gb) = if ramping && self.ramp_i < self.ramp_n {
                let t = self.ramp_i as f32 / self.ramp_n as f32;
                self.ramp_i += 1;
                (
                    self.ramp_from_a + (self.gain_a - self.ramp_from_a) * t,
                    self.ramp_from_b + (self.gain_b - self.ramp_from_b) * t,
                )
            } else {
                (self.gain_a, self.gain_b)
            };
            // Channel EQ then fader (DJ mixer style), dual-mono.
            let (ea_l, ea_r) = self.eq_a.process_stereo(a_l[i], a_r[i]);
            let (eb_l, eb_r) = self.eq_b.process_stereo(b_l[i], b_r[i]);
            let mut l = ea_l * ga * fa + eb_l * gb * fb;
            let mut r = ea_r * ga * fa + eb_r * gb * fb;

            if delay_wet > 0.0 {
                let dl = self.delay_l.process(l, delay_fb);
                let dr = self.delay_r.process(r, delay_fb);
                l += dl * delay_wet;
                r += dr * delay_wet;
            }

            if let Some(pcm) = self.riser_pcm.as_ref() {
                if self.riser_idx < pcm.len() {
                    let s = pcm[self.riser_idx] * RISER_GAIN;
                    self.riser_idx += 1;
                    l += s;
                    r += s;
                }
            }

            if self.roll_target > 0 {
                if self.roll_cap < self.roll_target {
                    let i = self.roll_i;
                    if i < self.roll_l.len() {
                        self.roll_l[i] = l;
                        self.roll_r[i] = r;
                    }
                    self.roll_i += 1;
                    if self.roll_i >= self.roll_target {
                        self.roll_cap = self.roll_target;
                        self.roll_i = 0;
                    }
                } else {
                    let cap = self.roll_cap.max(1);
                    let i = self.roll_i % cap;
                    l = self.roll_l[i];
                    r = self.roll_r[i];
                    self.roll_i = i + 1;
                }
            }

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
    fn lo_kill_drops_bass_more_than_shelf() {
        let sr = 48_000.0f32;
        let n = 8000;
        let mut a = vec![0.0f32; n];
        for (i, s) in a.iter_mut().enumerate() {
            *s = (2.0 * std::f32::consts::PI * 80.0 * i as f32 / sr).sin() * 0.5;
        }
        let b = vec![0.0f32; n];

        let mut shelf = Mixer::new();
        shelf.gain_a = 1.0;
        shelf.set_deck_eq(0, 2, 0.0);
        let mut out_shelf = vec![0.0f32; n];
        mix_center(&mut shelf, &mut out_shelf, &a, &b, sr);

        let mut kill = Mixer::new();
        kill.gain_a = 1.0;
        kill.set_lo_kill(0, true);
        let mut out_kill = vec![0.0f32; n];
        mix_center(&mut kill, &mut out_kill, &a, &b, sr);

        let e_shelf: f32 = out_shelf[2000..].iter().map(|x| x * x).sum();
        let e_kill: f32 = out_kill[2000..].iter().map(|x| x * x).sum();
        assert!(
            e_kill < e_shelf * 0.5,
            "isolator kill should drop bass more than -12 dB shelf: kill={e_kill} shelf={e_shelf}"
        );
        assert!(kill.lo_kill(0));
    }

    #[test]
    fn hold_xfade_keeps_mid_gains() {
        let mut m = Mixer::new();
        m.start_xfade(1, 0, 1000);
        let _ = m.tick_xfade(500);
        let a = m.gain_a;
        let b = m.gain_b;
        assert!(m.xfade().is_some());
        m.hold_xfade();
        assert!(m.xfade().is_none());
        assert!((m.gain_a - a).abs() < 1e-5);
        assert!((m.gain_b - b).abs() < 1e-5);
        assert!(m.crossfader_pos() > 0.2 && m.crossfader_pos() < 0.8);
    }

    #[test]
    fn fill_delay_wet_then_cut() {
        let mut m = Mixer::new();
        m.gain_a = 1.0;
        m.gain_b = 0.0;
        m.start_fill(
            FillKind::Delay,
            1,
            1,
            true,
            MixGrid::Eighth,
            0,
            96_000.0,
            120.0,
            48_000.0,
        );
        assert!(m.has_mix_job());
        assert!(m.delay_wet > 0.0);
        let done = m.tick_job(96_000, 48_000.0);
        assert!(done);
        assert!(!m.has_mix_job());
        assert!(m.gain_a.abs() < 1e-5);
        assert!((m.gain_b - 1.0).abs() < 1e-5);
        assert!((m.deck_eq(0)[1] - 0.5).abs() < 1e-5);
        assert!(!m.lo_kill(0) && !m.lo_kill(1));
        assert!(m.lpf_hz.is_none());
    }

    #[test]
    fn fill_lpf_sweeps_down() {
        let mut m = Mixer::new();
        m.start_fill(
            FillKind::Lpf,
            1,
            1,
            true,
            MixGrid::Eighth,
            0,
            1000.0,
            120.0,
            48_000.0,
        );
        let _ = m.tick_job(0, 48_000.0);
        let h0 = m.lpf_hz.unwrap();
        let _ = m.tick_job(500, 48_000.0);
        let h1 = m.lpf_hz.unwrap();
        assert!(h1 < h0, "lpf should fall: {h0} → {h1}");
        let done = m.tick_job(1000, 48_000.0);
        assert!(done);
        assert!(m.lpf_hz.is_none());
    }

    #[test]
    fn fill_switch_snaps_endpoints() {
        let mut m = Mixer::new();
        m.gain_a = 0.0;
        m.gain_b = 1.0;
        // 1 bar = 8 samples so 8n period = 1 sample (toy).
        m.start_fill(
            FillKind::Switch,
            0,
            1,
            true,
            MixGrid::Eighth,
            0,
            8.0,
            120.0,
            48_000.0,
        );
        let _ = m.tick_job(0, 48_000.0);
        // first cell = opposite of to=A → B
        assert!(m.gain_b > m.gain_a);
        let _ = m.tick_job(1, 48_000.0);
        assert!(m.gain_a > m.gain_b);
        let done = m.tick_job(8, 48_000.0);
        assert!(done);
        assert!(m.gain_a > 0.9);
        assert!(m.gain_b.abs() < 0.1);
    }

    #[test]
    fn fill_flash_gates_outgoing_only() {
        let mut m = Mixer::new();
        m.gain_a = 1.0;
        m.gain_b = 1.0;
        m.start_fill(
            FillKind::Flash,
            1,
            1,
            false,
            MixGrid::Eighth,
            0,
            8.0,
            120.0,
            48_000.0,
        );
        let _ = m.tick_job(0, 48_000.0);
        assert!((m.flash_mul[0] - 1.0).abs() < 1e-5);
        assert!((m.flash_mul[1] - 1.0).abs() < 1e-5);
        let _ = m.tick_job(1, 48_000.0);
        assert!(m.flash_mul[0].abs() < 1e-5);
        assert!((m.flash_mul[1] - 1.0).abs() < 1e-5);
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

    #[test]
    fn fill_kind_parse_v2() {
        assert_eq!(FillKind::parse("echo").unwrap(), FillKind::Echo);
        assert_eq!(FillKind::parse("hpf").unwrap(), FillKind::Hpf);
        assert_eq!(FillKind::parse("roll").unwrap(), FillKind::Roll);
        assert_eq!(FillKind::parse("drop").unwrap(), FillKind::Drop);
        assert_eq!(FillKind::parse("impact").unwrap(), FillKind::Drop);
        assert_eq!(FillKind::parse("echoout").unwrap(), FillKind::Echo);
        assert_eq!(FillKind::parse("loop").unwrap(), FillKind::Roll);
        assert!(FillKind::parse("scratch").is_err());
        assert_eq!(FillKind::Echo.as_str(), "echo");
        assert_eq!(FillKind::Hpf.as_str(), "hpf");
        assert_eq!(FillKind::Roll.as_str(), "roll");
        assert_eq!(FillKind::Drop.as_str(), "drop");
        assert_eq!(FillKind::Echo.default_bars(), 1);
        assert_eq!(FillKind::Hpf.default_bars(), 1);
        assert_eq!(FillKind::Roll.default_bars(), 1);
        assert_eq!(FillKind::Drop.default_bars(), 1);
    }

    #[test]
    fn fill_echo_ramps_then_cuts() {
        let mut m = Mixer::new();
        m.start_fill(
            FillKind::Echo,
            1,
            1,
            true,
            MixGrid::Eighth,
            0,
            1000.0,
            120.0,
            48_000.0,
        );
        let _ = m.tick_job(0, 48_000.0);
        let w0 = m.delay_wet;
        let _ = m.tick_job(500, 48_000.0);
        assert!(m.delay_wet > w0);
        assert!(m.delay_fb > ECHO_FB_START);
        let done = m.tick_job(1000, 48_000.0);
        assert!(done);
        assert!((m.gain_b - 1.0).abs() < 1e-5);
        assert!(m.delay_wet.abs() < 1e-5);
        assert!(m.delay_fb.abs() < 1e-5);
    }

    #[test]
    fn fill_hpf_sweeps_up() {
        let mut m = Mixer::new();
        m.start_fill(
            FillKind::Hpf,
            1,
            1,
            true,
            MixGrid::Eighth,
            0,
            1000.0,
            120.0,
            48_000.0,
        );
        let _ = m.tick_job(0, 48_000.0);
        let h0 = m.hpf_hz.unwrap();
        let _ = m.tick_job(500, 48_000.0);
        let h1 = m.hpf_hz.unwrap();
        assert!(h1 > h0, "hpf should rise: {h0} → {h1}");
        let done = m.tick_job(1000, 48_000.0);
        assert!(done);
        assert!(m.hpf_hz.is_none());
    }

    #[test]
    fn fill_drop_cuts_without_pcm() {
        let mut m = Mixer::new();
        m.start_fill(
            FillKind::Drop,
            1,
            1,
            true,
            MixGrid::Eighth,
            0,
            8.0,
            120.0,
            48_000.0,
        );
        assert_eq!(m.oneshot_needs_pcm(), Some("drop"));
        let done = m.tick_job(8, 48_000.0);
        assert!(done);
        assert!((m.gain_b - 1.0).abs() < 1e-5);
    }

    #[test]
    fn fill_roll_repeats_then_cuts() {
        let mut m = Mixer::new();
        m.gain_a = 1.0;
        m.gain_b = 0.0;
        m.start_fill(
            FillKind::Roll,
            1,
            1,
            true,
            MixGrid::Eighth,
            0,
            8.0,
            120.0,
            48_000.0,
        );
        let a_l = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
        let a_r = a_l;
        let b = [0.0f32; 8];
        let mut out_l = [0.0f32; 8];
        let mut out_r = [0.0f32; 8];
        m.mix(&mut out_l, &mut out_r, &a_l, &a_r, &b, &b, 48_000.0);
        assert!(out_l[0].abs() > 1e-6);
        for x in &out_l[1..] {
            assert!(
                (x - out_l[0]).abs() < 1e-3,
                "roll should repeat first sample: first={} got={x}",
                out_l[0]
            );
        }
        assert!((out_l[7] - 0.8).abs() > 1e-3);
        let done = m.tick_job(8, 48_000.0);
        assert!(done);
        assert!((m.gain_b - 1.0).abs() < 1e-5);
    }
}
