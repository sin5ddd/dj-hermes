//! Mixer layer: A/B faders, per-deck 3-band EQ, master 1-pole LPF/HPF, compressor, equal-power xfade.
//! DJ mix macros (fill / switch) live here as a preallocated MixJob.
//! Vinyl: static band-pass (worn) + post-comp delay-tap pitch wow (tape keeps rate varispeed).

use std::sync::Arc;

use crate::dsp::{
    eq_pos_to_db, Biquad, BiquadKind, DelayLine, EQ_FLAT, MAX_DELAY_FEEDBACK, MAX_DELAY_SEC,
};

/// Isolator-style Lo kill: high-pass so kick/bass does not leak through a shelf cut.
const LO_KILL_HZ: f32 = 250.0;
/// Isolator-style Hi kill: low-pass so air does not leak at slider left.
const HI_KILL_HZ: f32 = 6_000.0;
/// Slider at or below this engages Lo/Hi kill filters.
const EQ_KILL_POS: f32 = 0.02;
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
/// Preallocated tape-stop ring (~4 s at 60 BPM). Never `resize` on the audio thread.
const TAPE_RING_SEC: f32 = 4.0;
/// Worn-vinyl band-pass (static; cutoff is not an LFO).
const VINYL_BP_HZ: f32 = 900.0;
const VINYL_BP_Q: f32 = 0.7;
/// Held wow: one cycle per 33.3 RPM revolution.
const VINYL_WOW_HZ: f64 = 33.3 / 60.0;
/// Vibrato delay midpoint (samples). Must exceed fill amplitude.
const VINYL_VIBRATO_CENTER: f64 = 900.0;
/// Delay amplitude → ≈ ±50 cent at [`VINYL_WOW_HZ`] (48 kHz).
const VINYL_VIBRATO_AMP_HELD: f64 = 400.0;
/// Fill delay amplitude → ≈ ±100 cent.
const VINYL_VIBRATO_AMP_FILL: f64 = 800.0;

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

/// `dj_hermes_mix` / `POST /mix` action (hold is a separate immediate command).
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
    Vinyl,
    /// Hidden mix-lane pattern only, then cut-in. No extra DSP.
    Lane,
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
            "vinyl" => Ok(Self::Vinyl),
            "lane" => Ok(Self::Lane),
            other => Err(format!(
                "kind must be delay, lpf, flash, riser, switch, echo, hpf, roll, drop, vinyl, or lane: {other}"
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
            Self::Vinyl => "vinyl",
            Self::Lane => "lane",
        }
    }

    pub fn default_bars(self) -> u32 {
        match self {
            // Mix job length. Catalog riser WAVs are still ~15 s at 1× from bar head.
            Self::Riser => 4,
            // Vinyl fill: worn BPF wet ramps over 8 bars, then cut-in.
            Self::Vinyl => 8,
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
    /// Mix-lane song (`$: ` from `mixes/<kind>.strudel`). Parsed on the control thread.
    pub lane: Option<Box<crate::song::Song>>,
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
    Vinyl {
        to_deck: usize,
        reset_eq: bool,
        start_sample: u64,
        end_sample: u64,
        samples_per_bar: u64,
    },
    Lane {
        to_deck: usize,
        reset_eq: bool,
        end_sample: u64,
        samples_per_bar: u64,
    },
}

/// Per-deck channel EQ: Hi (shelf) / Mid (peak) / Lo (shelf).
/// Positions 0..=1: **1.0 = 0 dB** (right), 0.0 = band kill (left). No boost.
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
    hi_kill_l: Biquad,
    hi_kill_r: Biquad,
    lo_kill: bool,
    sr: f32,
}

impl ChannelEq {
    fn new(sr: f32) -> Self {
        let mut eq = Self {
            pos: [EQ_FLAT, EQ_FLAT, EQ_FLAT],
            hi_l: Biquad::bypass(),
            mid_l: Biquad::bypass(),
            lo_l: Biquad::bypass(),
            hi_r: Biquad::bypass(),
            mid_r: Biquad::bypass(),
            lo_r: Biquad::bypass(),
            kill_l: Biquad::bypass(),
            kill_r: Biquad::bypass(),
            hi_kill_l: Biquad::bypass(),
            hi_kill_r: Biquad::bypass(),
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
        self.hi_kill_l
            .set_coeffs(BiquadKind::LowPass, HI_KILL_HZ, q, sr);
        self.hi_kill_r
            .set_coeffs(BiquadKind::LowPass, HI_KILL_HZ, q, sr);
    }

    fn set_lo_kill(&mut self, kill: bool) {
        self.lo_kill = kill;
    }

    fn lo_is_kill(&self) -> bool {
        self.lo_kill || self.pos[2] <= EQ_KILL_POS
    }

    fn hi_is_kill(&self) -> bool {
        self.pos[0] <= EQ_KILL_POS
    }

    #[inline]
    fn process_stereo(&mut self, l: f32, r: f32) -> (f32, f32) {
        let lo_kill = self.lo_is_kill();
        let hi_kill = self.hi_is_kill();
        let l = if lo_kill {
            self.kill_l.process(l)
        } else {
            self.lo_l.process(l)
        };
        let r = if lo_kill {
            self.kill_r.process(r)
        } else {
            self.lo_r.process(r)
        };
        let l = self.mid_l.process(l);
        let r = self.mid_r.process(r);
        let l = if hi_kill {
            self.hi_kill_l.process(l)
        } else {
            self.hi_l.process(l)
        };
        let r = if hi_kill {
            self.hi_kill_r.process(r)
        } else {
            self.hi_r.process(r)
        };
        (l, r)
    }
}

#[derive(Clone, Copy)]
struct TapeWindow {
    origin_abs: u64,
    dur: u64,
    shots: u8,
    last_shot: Option<u64>,
}

impl TapeWindow {
    fn total_end(self) -> u64 {
        self.origin_abs
            .saturating_add(self.dur.max(1).saturating_mul(self.shots.max(1) as u64))
    }
}

/// Linear-interpolation varispeed: write 1:1, read at `rate`.
struct Varispeed {
    ring_l: Vec<f32>,
    ring_r: Vec<f32>,
    cap: usize,
    write: usize,
    read: f64,
    last_l: f32,
    last_r: f32,
    fade_i: u32,
    fade_n: u32,
    fade_from_l: f32,
    fade_from_r: f32,
}

impl Varispeed {
    fn new(cap: usize) -> Self {
        let cap = cap.max(4);
        Self {
            ring_l: vec![0.0; cap],
            ring_r: vec![0.0; cap],
            cap,
            write: 0,
            read: 0.0,
            last_l: 0.0,
            last_r: 0.0,
            fade_i: 0,
            fade_n: 0,
            fade_from_l: 0.0,
            fade_from_r: 0.0,
        }
    }

    fn reset(&mut self) {
        self.write = 0;
        self.read = 0.0;
        self.last_l = 0.0;
        self.last_r = 0.0;
        self.fade_i = 0;
        self.fade_n = 0;
        self.fade_from_l = 0.0;
        self.fade_from_r = 0.0;
    }

    fn fading(&self) -> bool {
        self.fade_n > 0 && self.fade_i < self.fade_n
    }

    fn begin_fade_to_dry(&mut self, sr: f32) {
        self.fade_from_l = self.last_l;
        self.fade_from_r = self.last_r;
        let n = (GAIN_RAMP_SEC * sr.max(1.0)).round() as u32;
        self.fade_n = n.max(1);
        self.fade_i = 0;
        self.write = 0;
        self.read = 0.0;
    }

    fn process(&mut self, l: f32, r: f32, rate: f64) -> (f32, f32) {
        let cap = self.cap;
        let wi = self.write % cap;
        self.ring_l[wi] = l;
        self.ring_r[wi] = r;
        self.write = self.write.wrapping_add(1);

        let max_lag = (cap - 2) as f64;
        if (self.write as f64) - self.read > max_lag {
            self.read = self.write as f64 - max_lag;
        }

        let idx = self.read.floor().max(0.0) as usize;
        let frac = (self.read - idx as f64) as f32;
        let i0 = idx % cap;
        let i1 = (i0 + 1) % cap;
        let ol = self.ring_l[i0].mul_add(1.0 - frac, self.ring_l[i1] * frac);
        let or_ = self.ring_r[i0].mul_add(1.0 - frac, self.ring_r[i1] * frac);
        self.read += rate.max(0.0);
        self.last_l = ol;
        self.last_r = or_;
        (ol, or_)
    }

    /// Write 1:1 and read a modulated delay (pitch wow). Tape `process` is rate≤1 only.
    fn process_vibrato(&mut self, l: f32, r: f32, delay: f64) -> (f32, f32) {
        let cap = self.cap;
        let wi = self.write % cap;
        self.ring_l[wi] = l;
        self.ring_r[wi] = r;
        self.write = self.write.wrapping_add(1);

        let d = delay.clamp(1.0, (cap - 2) as f64);
        if (self.write as f64) < d + 2.0 {
            self.last_l = l;
            self.last_r = r;
            return (l, r);
        }
        let pos = self.write as f64 - d;
        let idx = pos.floor().max(0.0) as usize;
        let frac = (pos - idx as f64) as f32;
        let i0 = idx % cap;
        let i1 = (i0 + 1) % cap;
        let ol = self.ring_l[i0].mul_add(1.0 - frac, self.ring_l[i1] * frac);
        let or_ = self.ring_r[i0].mul_add(1.0 - frac, self.ring_r[i1] * frac);
        self.last_l = ol;
        self.last_r = or_;
        (ol, or_)
    }

    fn mix_dry(&mut self, dry_l: f32, dry_r: f32) -> (f32, f32) {
        if !self.fading() {
            return (dry_l, dry_r);
        }
        let t = self.fade_i as f32 / self.fade_n as f32;
        self.fade_i += 1;
        (
            self.fade_from_l + (dry_l - self.fade_from_l) * t,
            self.fade_from_r + (dry_r - self.fade_from_r) * t,
        )
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
    /// Operator-held master FX; fill jobs overlay `lpf_hz` / `hpf_hz` / delay then restore these.
    held_lpf: Option<f32>,
    held_hpf: Option<f32>,
    held_delay_wet: f32,
    held_delay_fb: f32,
    /// Operator-held vinyl (static BPF + wow). Fill overlays then restores this.
    held_vinyl: bool,
    vinyl_l: Biquad,
    vinyl_r: Biquad,
    vinyl_sr: f32,
    /// Held wow phase (radians). Fill uses job progress instead; this still advances.
    vinyl_phase: f64,
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
    tape: Option<TapeWindow>,
    varispeed: Varispeed,
}

impl Default for Mixer {
    fn default() -> Self {
        Self::new()
    }
}

impl Mixer {
    pub fn new() -> Self {
        let delay_n = (MAX_DELAY_SEC * DELAY_MAX_SR).ceil() as usize;
        let tape_n = (TAPE_RING_SEC * DELAY_MAX_SR).ceil() as usize + 2;
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
            compressor: Some(crate::dsp::Compressor::new(
                crate::dsp::CompressorParams::MIXER_DEFAULT,
                48_000.0,
            )),
            comp_sr: 48_000.0,
            eq_a: ChannelEq::new(48_000.0),
            eq_b: ChannelEq::new(48_000.0),
            job: None,
            mix_status: None,
            delay_l: DelayLine::new(delay_n),
            delay_r: DelayLine::new(delay_n),
            delay_wet: 0.0,
            delay_fb: 0.0,
            held_lpf: None,
            held_hpf: None,
            held_delay_wet: 0.0,
            held_delay_fb: 0.0,
            held_vinyl: false,
            vinyl_l: Biquad::bypass(),
            vinyl_r: Biquad::bypass(),
            vinyl_sr: 48_000.0,
            vinyl_phase: 0.0,
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
            tape: None,
            varispeed: Varispeed::new(tape_n),
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
        self.set_deck_eq(from, 0, EQ_FLAT);
        self.set_deck_eq(from, 1, EQ_FLAT);
        self.set_deck_eq(from, 2, EQ_FLAT);
        self.set_deck_eq(to_deck, 0, EQ_FLAT);
        self.set_deck_eq(to_deck, 1, 0.0);
        self.set_deck_eq(to_deck, 2, EQ_FLAT);
    }

    pub fn reset_eq_flat(&mut self) {
        for d in 0..2 {
            self.set_lo_kill(d, false);
            for b in 0..3 {
                self.set_deck_eq(d, b, EQ_FLAT);
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
    /// Restores operator-held master FX (does not zero them).
    pub fn clear_job(&mut self) {
        let was_vinyl = self.job_owns_vinyl();
        self.job = None;
        self.mix_status = None;
        self.flash_mul = [1.0, 1.0];
        self.riser_pcm = None;
        self.riser_idx = 0;
        self.roll_target = 0;
        self.roll_cap = 0;
        self.roll_i = 0;
        self.restore_held_fx();
        if was_vinyl {
            self.fade_vinyl_wow_if_idle(self.comp_sr);
        }
    }

    fn job_owns_lpf(&self) -> bool {
        matches!(self.job, Some(MixJob::Lpf { .. }))
    }

    fn job_owns_hpf(&self) -> bool {
        matches!(self.job, Some(MixJob::Hpf { .. }))
    }

    fn job_owns_delay(&self) -> bool {
        matches!(self.job, Some(MixJob::Delay { .. } | MixJob::Echo { .. }))
    }

    fn job_owns_vinyl(&self) -> bool {
        matches!(self.job, Some(MixJob::Vinyl { .. }))
    }

    fn vinyl_sounding(&self) -> bool {
        self.held_vinyl || self.job_owns_vinyl()
    }

    fn ensure_vinyl_bpf(&mut self, sr: f32) {
        let sr = sr.max(1.0);
        self.vinyl_sr = sr;
        self.vinyl_l
            .set_coeffs(BiquadKind::BandPass, VINYL_BP_HZ, VINYL_BP_Q, sr);
        self.vinyl_r
            .set_coeffs(BiquadKind::BandPass, VINYL_BP_HZ, VINYL_BP_Q, sr);
    }

    fn clear_vinyl_bpf(&mut self) {
        self.vinyl_l = Biquad::bypass();
        self.vinyl_r = Biquad::bypass();
    }

    fn restore_held_fx(&mut self) {
        self.lpf_hz = self.held_lpf;
        self.hpf_hz = self.held_hpf;
        self.delay_wet = self.held_delay_wet;
        self.delay_fb = self.held_delay_fb;
        if self.held_delay_wet <= 0.0 {
            self.delay_l.clear();
            self.delay_r.clear();
        }
        if self.held_vinyl {
            self.ensure_vinyl_bpf(self.vinyl_sr);
        } else {
            self.clear_vinyl_bpf();
        }
    }

    fn fade_vinyl_wow_if_idle(&mut self, sr: f32) {
        if !self.vinyl_sounding() && self.tape.is_none() {
            self.varispeed.begin_fade_to_dry(sr.max(1.0));
        }
    }

    /// Operator master LPF. Fill LPF overlay does not overwrite the held value.
    pub fn set_held_lpf(&mut self, hz: Option<f32>) {
        self.held_lpf = hz;
        if !self.job_owns_lpf() {
            self.lpf_hz = hz;
        }
    }

    /// Operator master HPF. Fill HPF overlay does not overwrite the held value.
    pub fn set_held_hpf(&mut self, hz: Option<f32>) {
        self.held_hpf = hz;
        if !self.job_owns_hpf() {
            self.hpf_hz = hz;
        }
    }

    /// Operator master delay wet/feedback. `time_sec` is the tap (typically an 8th).
    pub fn set_held_delay(&mut self, wet: f32, feedback: Option<f32>, time_sec: f32, sr: f32) {
        self.held_delay_wet = wet.clamp(0.0, 1.0);
        if let Some(fb) = feedback {
            self.held_delay_fb = fb.clamp(0.0, MAX_DELAY_FEEDBACK);
        }
        self.delay_l.set_time_sec(time_sec.max(0.0), sr);
        self.delay_r.set_time_sec(time_sec.max(0.0), sr);
        if !self.job_owns_delay() {
            self.delay_wet = self.held_delay_wet;
            self.delay_fb = self.held_delay_fb;
            if self.held_delay_wet <= 0.0 {
                self.delay_l.clear();
                self.delay_r.clear();
            }
        }
    }

    pub fn held_delay_wet(&self) -> f32 {
        self.held_delay_wet
    }

    /// Operator master vinyl (static BPF + pitch wow). Fill overlays then restores this.
    pub fn set_held_vinyl(&mut self, on: bool, sr: f32) {
        let was = self.vinyl_sounding();
        self.held_vinyl = on;
        if on {
            self.ensure_vinyl_bpf(sr);
        }
        if was && !self.vinyl_sounding() {
            self.clear_vinyl_bpf();
            self.fade_vinyl_wow_if_idle(sr);
        }
    }

    pub fn held_vinyl(&self) -> bool {
        self.held_vinyl
    }

    /// Louder deck (tie → A). Used to isolate before a fill.
    pub fn main_deck(&self) -> usize {
        if self.gain_b > self.gain_a {
            1
        } else {
            0
        }
    }

    fn snap_to_main_if_needed(&mut self, sr: f32) {
        let pos = self.crossfader_pos();
        if pos <= 0.02 || pos >= 0.98 {
            return;
        }
        let target = if self.main_deck() == 1 { 1.0 } else { 0.0 };
        self.snap_crossfader(target, true, sr);
    }

    /// ~5ms equal-power hold; used when time-repeat starts or jumps back to absolute.
    pub fn soften_click(&mut self, sr: f32) {
        self.begin_gain_ramp(sr);
    }

    /// Arm post-mix varispeed. `dur` is one shot; `shots` consecutive stabs snap between them.
    pub fn start_tape(&mut self, origin_abs: u64, dur: u64, shots: u8) {
        self.tape = Some(TapeWindow {
            origin_abs,
            dur: dur.max(1),
            shots: shots.max(1),
            last_shot: None,
        });
        self.varispeed.reset();
    }

    /// Drop unread ring and fade last wet sample to dry (~5ms).
    pub fn stop_tape(&mut self, sr: f32) {
        if self.tape.take().is_some() {
            self.varispeed.begin_fade_to_dry(sr);
        }
    }

    /// Immediate discard (hush). No fade.
    pub fn clear_tape(&mut self) {
        self.tape = None;
        self.varispeed.reset();
    }

    pub fn tape_on(&self) -> bool {
        self.tape.is_some()
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
        let was_vinyl = self.job_owns_vinyl();
        self.flash_mul = [1.0, 1.0];
        self.riser_pcm = None;
        self.riser_idx = 0;
        self.roll_target = 0;
        self.roll_cap = 0;
        self.roll_i = 0;
        self.job = None;
        self.mix_status = None;
        self.restore_held_fx();
        if was_vinyl {
            self.fade_vinyl_wow_if_idle(sr);
        }
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
        self.snap_to_main_if_needed(sr);
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
            FillKind::Vinyl => {
                self.ensure_vinyl_bpf(sr);
                self.soften_click(sr);
                self.job = Some(MixJob::Vinyl {
                    to_deck,
                    reset_eq,
                    start_sample,
                    end_sample: end,
                    samples_per_bar: spb,
                });
            }
            FillKind::Lane => {
                self.job = Some(MixJob::Lane {
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
                | MixJob::Drop { end_sample, .. }
                | MixJob::Vinyl { end_sample, .. }
                | MixJob::Lane { end_sample, .. } => *end_sample,
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
                }
                | MixJob::Vinyl {
                    to_deck, reset_eq, ..
                }
                | MixJob::Lane {
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
            Vinyl {
                end: u64,
                spb: u64,
            },
            Lane {
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
            MixJob::Vinyl {
                end_sample,
                samples_per_bar,
                ..
            } => Step::Vinyl {
                end: *end_sample,
                spb: *samples_per_bar,
            },
            MixJob::Lane {
                end_sample,
                samples_per_bar,
                ..
            } => Step::Lane {
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
            Step::Vinyl { end, spb } => self.update_bars_left(global_sample, end, spb),
            Step::Lane { end, spb } => self.update_bars_left(global_sample, end, spb),
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

    /// Set one EQ band for a deck. `band`: 0=Hi, 1=Mid, 2=Lo. `value`: 0..=1 (1.0 = 0 dB).
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
            _ => [EQ_FLAT, EQ_FLAT, EQ_FLAT],
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
    /// `head_sample` is the absolute sample of `out[0]` (tape-stop varispeed).
    #[allow(clippy::too_many_arguments)]
    pub fn mix(
        &mut self,
        out_l: &mut [f32],
        out_r: &mut [f32],
        a_l: &[f32],
        a_r: &[f32],
        b_l: &[f32],
        b_r: &[f32],
        lane_l: &[f32],
        lane_r: &[f32],
        sample_rate: f32,
        head_sample: u64,
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
        if let Some(comp) = self.compressor.as_mut() {
            if (sr - self.comp_sr).abs() > 1.0 {
                let p = comp.params;
                comp.set_params(p, sr);
                self.comp_sr = sr;
            }
        }
        let delay_wet = self.delay_wet;
        let delay_fb = self.delay_fb;
        let fa = self.flash_mul[0];
        let fb = self.flash_mul[1];
        let ramping = self.ramp_n > 0 && self.ramp_i < self.ramp_n;
        let vinyl_on = self.vinyl_sounding();
        let vinyl_fill = match self.job {
            Some(MixJob::Vinyl {
                start_sample,
                end_sample,
                ..
            }) => Some((start_sample, end_sample)),
            _ => None,
        };
        if vinyl_on && (sr - self.vinyl_sr).abs() > 1.0 {
            self.ensure_vinyl_bpf(sr);
        }

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

            if i < lane_l.len() && i < lane_r.len() {
                l += lane_l[i];
                r += lane_r[i];
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

            if vinyl_on {
                let bl = self.vinyl_l.process(l);
                let br = self.vinyl_r.process(r);
                let wet = if let Some((start, end)) = vinyl_fill {
                    let denom = end.saturating_sub(start).max(1) as f32;
                    let abs = head_sample.saturating_add(i as u64);
                    (abs.saturating_sub(start) as f32 / denom).clamp(0.0, 1.0)
                } else {
                    1.0
                };
                l = l.mul_add(1.0 - wet, bl * wet);
                r = r.mul_add(1.0 - wet, br * wet);
            }

            if let Some(comp) = self.compressor.as_mut() {
                let (cl, cr) = comp.process_stereo(l, r);
                l = cl;
                r = cr;
            }

            if self.tape.is_some() || self.varispeed.fading() || vinyl_on {
                let abs = head_sample.saturating_add(i as u64);
                let window = self.tape.filter(|w| abs < w.total_end());
                if let Some(w) = window {
                    let dur = w.dur.max(1);
                    let shot = abs.saturating_sub(w.origin_abs) / dur;
                    if w.last_shot != Some(shot) {
                        if w.last_shot.is_some() {
                            self.varispeed.reset();
                        }
                        if let Some(t) = self.tape.as_mut() {
                            t.last_shot = Some(shot);
                        }
                    }
                    let shot_start = w.origin_abs.saturating_add(shot.saturating_mul(dur));
                    let rate = crate::transport::tape_rate_at_abs(
                        abs,
                        shot_start,
                        shot_start.saturating_add(dur),
                    );
                    let (ol, or_) = self.varispeed.process(l, r, rate);
                    l = ol;
                    r = or_;
                } else if vinyl_on {
                    let _ = self.tape.take();
                    let amp = if vinyl_fill.is_some() {
                        VINYL_VIBRATO_AMP_FILL
                    } else {
                        VINYL_VIBRATO_AMP_HELD
                    };
                    let delay = VINYL_VIBRATO_CENTER + amp * self.vinyl_phase.sin();
                    let (ol, or_) = self.varispeed.process_vibrato(l, r, delay);
                    l = ol;
                    r = or_;
                } else {
                    if self.tape.take().is_some() {
                        self.varispeed.begin_fade_to_dry(sr);
                    }
                    let (ol, or_) = self.varispeed.mix_dry(l, r);
                    l = ol;
                    r = or_;
                }
            }

            if vinyl_on {
                self.vinyl_phase += std::f64::consts::TAU * VINYL_WOW_HZ / f64::from(sr).max(1.0);
                if self.vinyl_phase >= std::f64::consts::TAU {
                    self.vinyl_phase -= std::f64::consts::TAU;
                }
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
        m.mix(&mut out_l, &mut out_r, a, a, b, b, &[], &[], sr, 0);
        for i in 0..n {
            out[i] = 0.5 * (out_l[i] + out_r[i]);
        }
    }

    #[test]
    fn mixer_starts_with_master_compressor() {
        let m = Mixer::new();
        let p = m
            .compressor_params()
            .expect("Mixer::new should enable master compressor");
        assert!((p.threshold_db - CompressorParams::MIXER_DEFAULT.threshold_db).abs() < 1e-5);
        assert!((p.ratio - CompressorParams::MIXER_DEFAULT.ratio).abs() < 1e-5);
        assert!(p.threshold_db <= -24.0);
        assert!(p.ratio >= 8.0);
    }

    #[test]
    fn default_compressor_reduces_sustained_peak() {
        let sr = 48_000.0f32;
        let n = 4000;
        let a = vec![0.8f32; n];
        let b = vec![0.0f32; n];

        let mut dry = Mixer::new();
        dry.gain_a = 1.0;
        dry.set_compressor(None, sr);
        let mut out_dry = vec![0.0f32; n];
        mix_center(&mut dry, &mut out_dry, &a, &b, sr);

        let mut wet = Mixer::new();
        wet.gain_a = 1.0;
        let mut out_wet = vec![0.0f32; n];
        mix_center(&mut wet, &mut out_wet, &a, &b, sr);

        let peak = |buf: &[f32]| buf[2000..].iter().map(|x| x.abs()).fold(0.0f32, f32::max);
        let peak_dry = peak(&out_dry);
        let peak_wet = peak(&out_wet);
        assert!(
            peak_wet < peak_dry * 0.7,
            "default master comp should squash a 0.8 sustain: dry={peak_dry} wet={peak_wet}"
        );
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
        m.set_compressor(None, 48_000.0);
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
        flat.set_compressor(None, sr);
        let mut out_flat = vec![0.0f32; n];
        mix_center(&mut flat, &mut out_flat, &a, &b, sr);

        let mut cut = Mixer::new();
        cut.gain_a = 1.0;
        cut.set_compressor(None, sr);
        cut.set_deck_eq(0, 2, 0.0); // Lo kill
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
        shelf.set_deck_eq(0, 2, 0.75);
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
            "isolator kill should drop bass more than a mid-slider cut: kill={e_kill} shelf={e_shelf}"
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
        assert!((m.deck_eq(0)[1] - EQ_FLAT).abs() < 1e-5);
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
    fn eq_hi_kill_drops_treble() {
        let sr = 48_000.0f32;
        let n = 8000;
        // 8 kHz sine (above Hi shelf / kill LPF).
        let mut a = vec![0.0f32; n];
        for (i, s) in a.iter_mut().enumerate() {
            *s = (2.0 * std::f32::consts::PI * 8000.0 * i as f32 / sr).sin() * 0.3;
        }
        let b = vec![0.0f32; n];

        let mut flat = Mixer::new();
        flat.gain_a = 1.0;
        flat.set_compressor(None, sr);
        let mut out_flat = vec![0.0f32; n];
        mix_center(&mut flat, &mut out_flat, &a, &b, sr);

        let mut cut = Mixer::new();
        cut.gain_a = 1.0;
        cut.set_compressor(None, sr);
        cut.set_deck_eq(0, 0, 0.0);
        let mut out_cut = vec![0.0f32; n];
        mix_center(&mut cut, &mut out_cut, &a, &b, sr);

        let e_flat: f32 = out_flat[2000..].iter().map(|x| x * x).sum();
        let e_cut: f32 = out_cut[2000..].iter().map(|x| x * x).sum();
        assert!(
            e_cut < e_flat * 0.5,
            "Hi kill should drop treble: cut={e_cut} flat={e_flat}"
        );
    }

    #[test]
    fn fill_restores_held_lpf() {
        let mut m = Mixer::new();
        m.set_held_lpf(Some(800.0));
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
        assert!(m.lpf_hz.unwrap() > 1000.0);
        let done = m.tick_job(1000, 48_000.0);
        assert!(done);
        assert!((m.lpf_hz.unwrap() - 800.0).abs() < 1e-3);
    }

    #[test]
    fn fill_snaps_mid_xfader_to_main() {
        let mut m = Mixer::new();
        m.set_crossfader(0.5);
        assert!((m.crossfader_pos() - 0.5).abs() < 0.05);
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
        // Tie at 0.5 → main A; fill isolates A then will cut to B at end.
        assert!(m.gain_a > 0.9, "fill should snap to main A first");
        assert!(m.gain_b.abs() < 0.15);
    }

    #[test]
    fn long_mix_does_not_snap_xfader() {
        let mut m = Mixer::new();
        m.set_crossfader(0.4);
        let pos = m.crossfader_pos();
        m.start_xfade(1, 0, 1000);
        assert!(m.xfade().is_some());
        assert!(
            (m.crossfader_pos() - pos).abs() < 0.02,
            "long xfade must not snap the fader before it ticks"
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
        assert_eq!(FillKind::Riser.default_bars(), 4);
        assert_eq!(FillKind::parse("vinyl").unwrap(), FillKind::Vinyl);
        assert_eq!(FillKind::Vinyl.as_str(), "vinyl");
        assert_eq!(FillKind::Vinyl.default_bars(), 8);
        assert_eq!(FillKind::parse("lane").unwrap(), FillKind::Lane);
        assert_eq!(FillKind::Lane.as_str(), "lane");
        assert_eq!(FillKind::Lane.default_bars(), 1);
    }

    #[test]
    fn mix_adds_lane_bus() {
        let mut m = Mixer::new();
        m.gain_a = 0.0;
        m.gain_b = 0.0;
        let sil = [0.0f32; 4];
        let lane = [0.5f32; 4];
        let mut out_l = [0.0f32; 4];
        let mut out_r = [0.0f32; 4];
        m.mix(
            &mut out_l, &mut out_r, &sil, &sil, &sil, &sil, &lane, &lane, 48_000.0, 0,
        );
        for x in out_l {
            assert!((x - 0.5).abs() < 0.2, "lane should be audible, got {x}");
        }
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
        m.mix(
            &mut out_l,
            &mut out_r,
            &a_l,
            &a_r,
            &b,
            &b,
            &[],
            &[],
            48_000.0,
            0,
        );
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

    #[test]
    fn varispeed_half_rate_doubles_sine_period() {
        let mut v = Varispeed::new(8_000);
        let sr = 48_000.0f32;
        let freq = 440.0f32;
        let n = 12_000usize;
        let mut out = vec![0.0f32; n];
        for (i, slot) in out.iter_mut().enumerate() {
            let s = (2.0 * std::f32::consts::PI * freq * (i as f32) / sr).sin() * 0.5;
            let (l, _) = v.process(s, s, 0.5);
            *slot = l;
        }
        // Skip the first cycle of interpolation settle; measure +zero crossings.
        let start = 2_000usize;
        let mut crossings = Vec::new();
        for (i, pair) in out.windows(2).enumerate().skip(start) {
            if pair[0] < 0.0 && pair[1] >= 0.0 {
                crossings.push(i + 1);
            }
        }
        assert!(
            crossings.len() >= 8,
            "need several crossings, got {}",
            crossings.len()
        );
        let mut gaps = Vec::new();
        for w in crossings.windows(2) {
            gaps.push((w[1] - w[0]) as f64);
        }
        let mean = gaps.iter().sum::<f64>() / gaps.len() as f64;
        let expected = sr as f64 / freq as f64 * 2.0; // period at rate 0.5
        assert!(
            (mean - expected).abs() < expected * 0.08,
            "mean period {mean} expected ~{expected}"
        );
    }

    fn sine_buf(n: usize, hz: f32, sr: f32) -> Vec<f32> {
        (0..n)
            .map(|i| (2.0 * std::f32::consts::PI * hz * i as f32 / sr).sin() * 0.5)
            .collect()
    }

    fn tail_energy(buf: &[f32]) -> f32 {
        let start = buf.len() / 4;
        buf[start..].iter().map(|x| x * x).sum()
    }

    #[test]
    fn vinyl_bpf_cuts_low_sine() {
        let sr = 48_000.0f32;
        let n = 8_000;
        let silent = vec![0.0f32; n];
        let mut dry = Mixer::new();
        dry.gain_a = 1.0;
        dry.set_compressor(None, sr);
        let low = sine_buf(n, 80.0, sr);
        let mid = sine_buf(n, 900.0, sr);
        let mut out_low_dry = vec![0.0f32; n];
        mix_center(&mut dry, &mut out_low_dry, &low, &silent, sr);

        let mut wet = Mixer::new();
        wet.gain_a = 1.0;
        wet.set_compressor(None, sr);
        wet.set_held_vinyl(true, sr);
        let mut out_low = vec![0.0f32; n];
        let mut out_mid = vec![0.0f32; n];
        mix_center(&mut wet, &mut out_low, &low, &silent, sr);
        mix_center(&mut wet, &mut out_mid, &mid, &silent, sr);

        let e_low_dry = tail_energy(&out_low_dry);
        let e_low = tail_energy(&out_low);
        let e_mid = tail_energy(&out_mid);
        assert!(
            e_low < e_low_dry * 0.35,
            "held vinyl BPF should thin 80 Hz: wet={e_low} dry={e_low_dry}"
        );
        assert!(
            e_mid > e_low * 2.0,
            "900 Hz should pass more than 80 Hz: mid={e_mid} low={e_low}"
        );
    }

    #[test]
    fn vinyl_fill_bpf_wet_ramps() {
        let sr = 48_000.0f32;
        let n = 512usize;
        let silent = vec![0.0f32; n];
        let low = sine_buf(n, 80.0, sr);
        let mut m = Mixer::new();
        m.gain_a = 1.0;
        m.set_compressor(None, sr);
        m.start_fill(
            FillKind::Vinyl,
            1,
            8,
            true,
            MixGrid::Eighth,
            0,
            1_000.0,
            120.0,
            sr,
        );
        let mut early_l = vec![0.0f32; n];
        let mut early_r = vec![0.0f32; n];
        m.mix(
            &mut early_l,
            &mut early_r,
            &low,
            &low,
            &silent,
            &silent,
            &[],
            &[],
            sr,
            0,
        );
        let mut late_l = vec![0.0f32; n];
        let mut late_r = vec![0.0f32; n];
        m.mix(
            &mut late_l,
            &mut late_r,
            &low,
            &low,
            &silent,
            &silent,
            &[],
            &[],
            sr,
            7_400,
        );
        let e_early: f32 = early_l.iter().map(|x| x * x).sum();
        let e_late: f32 = late_l.iter().map(|x| x * x).sum();
        assert!(
            e_late < e_early * 0.7,
            "fill かすれ should get stronger: early={e_early} late={e_late}"
        );
        assert!(m.has_mix_job());
        assert!(m.tick_job(8_000, sr));
        assert!(!m.has_mix_job());
        assert!(!m.held_vinyl());
    }

    #[test]
    fn vinyl_held_wow_varies_zero_cross_gaps() {
        let sr = 48_000.0f32;
        let n = 48_000usize;
        let silent = vec![0.0f32; n];
        let tone = sine_buf(n, 1_000.0, sr);
        let mut m = Mixer::new();
        m.gain_a = 1.0;
        m.set_compressor(None, sr);
        m.set_held_vinyl(true, sr);
        let mut out_l = vec![0.0f32; n];
        let mut out_r = vec![0.0f32; n];
        m.mix(
            &mut out_l,
            &mut out_r,
            &tone,
            &tone,
            &silent,
            &silent,
            &[],
            &[],
            sr,
            0,
        );
        let mut crossings = Vec::new();
        for (i, pair) in out_l.windows(2).enumerate().skip(2_000) {
            if pair[0] < 0.0 && pair[1] >= 0.0 {
                crossings.push(i + 1);
            }
        }
        assert!(
            crossings.len() >= 20,
            "need several crossings, got {}",
            crossings.len()
        );
        let gaps: Vec<i64> = crossings
            .windows(2)
            .map(|w| w[1] as i64 - w[0] as i64)
            .collect();
        let min = *gaps.iter().min().unwrap();
        let max = *gaps.iter().max().unwrap();
        assert!(max > min, "wow should stretch period: min={min} max={max}");
    }

    #[test]
    fn vinyl_held_survives_delay_fill() {
        let mut m = Mixer::new();
        m.set_held_vinyl(true, 48_000.0);
        m.start_fill(
            FillKind::Delay,
            1,
            1,
            true,
            MixGrid::Eighth,
            0,
            1_000.0,
            120.0,
            48_000.0,
        );
        assert!(m.held_vinyl());
        assert!(m.tick_job(1_000, 48_000.0));
        assert!(m.held_vinyl());
        assert!(!m.has_mix_job());
    }

    #[test]
    fn vinyl_tape_window_then_held_stays() {
        let sr = 48_000.0f32;
        let n = 256usize;
        let silent = vec![0.0f32; n];
        let low = sine_buf(n, 80.0, sr);
        let mut m = Mixer::new();
        m.gain_a = 1.0;
        m.set_compressor(None, sr);
        m.set_held_vinyl(true, sr);
        m.start_tape(0, 64, 1);
        let mut out_l = vec![0.0f32; n];
        let mut out_r = vec![0.0f32; n];
        m.mix(
            &mut out_l,
            &mut out_r,
            &low,
            &low,
            &silent,
            &silent,
            &[],
            &[],
            sr,
            0,
        );
        assert!(m.tape_on() || m.held_vinyl());
        m.mix(
            &mut out_l,
            &mut out_r,
            &low,
            &low,
            &silent,
            &silent,
            &[],
            &[],
            sr,
            200,
        );
        assert!(m.held_vinyl());
        assert!(!m.tape_on());
    }
}
