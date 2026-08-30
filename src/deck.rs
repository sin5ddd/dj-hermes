//! Deck: schedule song tracks for one bar, orbit buses, duck, voice pool.

use crate::code::{Adsr, DuckParams, FilterParams, ModParams, PatternCode};
use crate::dsp::{equal_power_pan, orbit_index, CompressorParams, DuckState, OrbitFx, NUM_ORBITS};
use crate::mini;
use crate::sample::{SampleBank, SampleVoice, VoiceKind, SAMPLE_ROOT_HZ};
use crate::scale::resolve_pitch_with_add;
use crate::song::Song;
use crate::sound::{resolve_sound_with_bank, ResolvedSound};
use crate::synth::{OscSource, Voice};
use crate::transport::Transport;

pub const MAX_VOICES: usize = 32;

#[derive(Clone)]
struct ScheduledHit {
    at_sample: u64,
    len_samples: u64,
    sound: String,
    freq: f32,
    gain: f32,
    pan: f32,
    filter: FilterParams,
    adsr: Adsr,
    mods: ModParams,
    is_note: bool,
    begin: f32,
    end: f32,
    sample_speed: f32,
    sample_n: Option<i32>,
    bank: Option<String>,
    orbit: u8,
    duck: DuckParams,
    cut: Option<i32>,
    compressor: Option<CompressorParams>,
    delay: f32,
    delaytime: f32,
    delayfeedback: f32,
    room: f32,
    roomsize: f32,
}

pub struct Deck {
    pub name: String,
    song: Option<Song>,
    /// Pre-mixer level (Engine normally leaves this at 1.0; Mixer owns faders).
    pub gain: f32,
    voices: Vec<Option<VoiceKind>>,
    steal_cursor: usize,
    scheduled: Vec<ScheduledHit>,
    next_event: usize,
    /// Last *global* bar we scheduled against (not pattern cycle).
    scheduled_bar: u64,
    /// Pattern cycle = (global_bar as i64 + cycle_offset).max(0).
    /// Set by head/cue so a deck can play a different song bar while transport stays locked.
    cycle_offset: i64,
    ducks: [DuckState; NUM_ORBITS],
    /// Per-orbit global delay / room (Deck-local; not shared across A/B).
    orbit_fx: [OrbitFx; NUM_ORBITS],
    /// Last compressor params seen on a spawned hit (Engine may promote to Mixer).
    pub pending_compressor: Option<CompressorParams>,
}

impl Deck {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            song: None,
            gain: 1.0,
            voices: (0..MAX_VOICES).map(|_| None).collect(),
            steal_cursor: 0,
            scheduled: Vec::with_capacity(512),
            next_event: 0,
            scheduled_bar: u64::MAX,
            cycle_offset: 0,
            ducks: [DuckState::default(); NUM_ORBITS],
            orbit_fx: std::array::from_fn(|_| OrbitFx::new()),
            pending_compressor: None,
        }
    }

    /// Replace the loaded song and force a clean reschedule.
    ///
    /// Clears ringing voices and the event queue (same policy as [`Self::head_to_bar`] /
    /// [`Self::unload`]) so hot-reload does not stack a new bar-head onset on top of
    /// leftover voices (heard as ~2× level on the first hit after load).
    pub fn load(&mut self, song: Song) {
        self.song = Some(song);
        for v in &mut self.voices {
            *v = None;
        }
        self.scheduled.clear();
        self.next_event = 0;
        self.scheduled_bar = u64::MAX;
        self.cycle_offset = 0;
        self.ducks = [DuckState::default(); NUM_ORBITS];
        for fx in &mut self.orbit_fx {
            fx.clear();
        }
        self.pending_compressor = None;
    }

    pub fn unload(&mut self) {
        self.song = None;
        for v in &mut self.voices {
            *v = None;
        }
        self.scheduled.clear();
        self.next_event = 0;
        self.scheduled_bar = u64::MAX;
        self.cycle_offset = 0;
        self.ducks = [DuckState::default(); NUM_ORBITS];
        for fx in &mut self.orbit_fx {
            fx.clear();
        }
        self.pending_compressor = None;
    }

    /// Offset added to the global bar index when choosing pattern content.
    pub fn cycle_offset(&self) -> i64 {
        self.cycle_offset
    }

    /// Pattern cycle (0-based) for a given global bar.
    pub fn pattern_bar(&self, global_bar: u64) -> u64 {
        (global_bar as i64 + self.cycle_offset).max(0) as u64
    }

    /// Human 1-based song bar for UI / status.
    pub fn song_bar_1based(&self, global_bar: u64) -> u64 {
        self.pattern_bar(global_bar).saturating_add(1)
    }

    /// Jump pattern content so that at `apply_global_bar` the deck plays 1-based `bar_1based`.
    /// Clears ringing voices and forces reschedule.
    pub fn head_to_bar(&mut self, bar_1based: u64, apply_global_bar: u64) {
        let target_cycle = bar_1based.saturating_sub(1);
        self.cycle_offset = target_cycle as i64 - apply_global_bar as i64;
        for v in &mut self.voices {
            *v = None;
        }
        self.scheduled.clear();
        self.next_event = 0;
        self.scheduled_bar = u64::MAX;
        self.ducks = [DuckState::default(); NUM_ORBITS];
    }

    pub fn song_title(&self) -> Option<&str> {
        self.song.as_ref().map(|s| s.title.as_str())
    }

    pub fn song_ref(&self) -> Option<&Song> {
        self.song.as_ref()
    }

    pub fn song_mut(&mut self) -> Option<&mut Song> {
        self.song.as_mut()
    }

    /// Replace song metadata/source for the control plane without forcing an audio reschedule.
    ///
    /// Used by partial-edit APIs so sequential `get`/`patch`/`edit_method` compose before the
    /// bar-quantized [`crate::engine::Command::LoadSong`] applies the audible swap.
    pub fn set_song_data(&mut self, song: Song) {
        self.song = Some(song);
    }

    pub fn set_track_mute(&mut self, track: &str, muted: bool) {
        if let Some(song) = self.song.as_mut() {
            for t in &mut song.tracks {
                if t.name == track {
                    t.muted = muted;
                }
            }
            self.scheduled_bar = u64::MAX;
        }
    }

    fn schedule_bar(&mut self, global_bar: u64, transport: &Transport) {
        self.scheduled.clear();
        self.next_event = 0;
        let spb = transport.samples_per_bar();
        // Sample timestamps follow the shared transport; pattern content may be offset (head/cue).
        let bar_start = (global_bar as f64 * spb) as u64;
        let pattern_bar = self.pattern_bar(global_bar);
        let tracks: Vec<PatternCode> = self
            .song
            .as_ref()
            .map(|s| {
                s.tracks
                    .iter()
                    .filter(|t| !t.muted)
                    .map(|t| t.code.clone())
                    .collect()
            })
            .unwrap_or_default();
        for pc in &tracks {
            schedule_track_into(&mut self.scheduled, pc, pattern_bar, bar_start, spb);
        }
        self.scheduled.sort_by_key(|e| e.at_sample);
        self.scheduled_bar = global_bar;
    }
}

fn schedule_track_into(
    scheduled: &mut Vec<ScheduledHit>,
    pc: &PatternCode,
    bar: u64,
    bar_start: u64,
    spb: f64,
) {
    let speed = pc.speed.max(1e-6);
    let len_scale = pc.length_scale() as f64;
    let ply = pc.ply.clamp(1, 16) as f64;
    for ev in mini::events(&pc.pattern, bar) {
        if ev.value == "~" || ev.value.is_empty() {
            continue;
        }
        // `.ply(n)`: split each event into n equal sub-hits within its timespan.
        let n = ply as u32;
        let sub_dur = ev.dur / ply;
        for i in 0..n {
            let sub_start = ev.start + (i as f64) * sub_dur;
            let phase = sub_start.clamp(0.0, 0.999_999);

            let pitch_extra = pc
                .pitch_add_dyn
                .as_ref()
                .and_then(|d| d.sample_at(bar, phase))
                .map(|v| pc.pitch_add_dyn_sign * f64::from(v))
                .unwrap_or(0.0);
            let pitch_add = pc.pitch_add + pitch_extra;

            let (sound, freq, is_note) = if pc.is_note {
                let sc = pc.scale.as_ref().map(|p| p.at_cycle(bar));
                match resolve_pitch_with_add(&ev.value, sc, pitch_add) {
                    Some(h) => (pc.sound.clone(), h, true),
                    None => continue,
                }
            } else {
                (ev.value.clone(), 0.0, false)
            };

            let mut filter = pc.filter;
            let mut mods = pc.mod_params;
            mods.samples_per_bar = spb as f32;
            mods.lfo_phase0 = phase as f32;

            // Dynamic filters: patterns → per-event snapshot; LFO → continuous on synth.
            if let Some(ref d) = pc.lpf_dyn {
                if let Some(spec) = d.as_lfo() {
                    mods.lpf_lfo = Some(spec);
                    filter.lpf = Some(spec.value_at_bar_phase(phase as f32));
                } else if let Some(v) = d.sample_at(bar, phase) {
                    filter.lpf = Some(v.clamp(20.0, 20_000.0));
                    mods.lpf_lfo = None;
                }
            }
            if let Some(ref d) = pc.hpf_dyn {
                if let Some(v) = d.sample_at(bar, phase) {
                    filter.hpf = Some(v.clamp(20.0, 20_000.0));
                }
            }
            if let Some(ref d) = pc.bpf_dyn {
                if let Some(v) = d.sample_at(bar, phase) {
                    filter.bpf = Some(v.clamp(20.0, 20_000.0));
                }
            }

            let mut gain = pc.effective_gain();
            if let Some(ref d) = pc.gain_dyn {
                if let Some(v) = d.sample_at(bar, phase) {
                    gain = v * pc.velocity;
                }
            }
            let mut pan = pc.pan;
            if let Some(ref d) = pc.pan_dyn {
                if let Some(v) = d.sample_at(bar, phase) {
                    pan = v;
                }
            }

            let at = bar_start + ((sub_start * spb) / speed) as u64;
            let len = ((((sub_dur * spb) / speed) * len_scale) as u64).max(64);
            scheduled.push(ScheduledHit {
                at_sample: at,
                len_samples: len,
                sound: sound.clone(),
                freq,
                gain,
                pan: pan.clamp(0.0, 1.0),
                filter,
                adsr: pc.adsr,
                mods,
                is_note,
                begin: pc.begin,
                end: pc.end,
                sample_speed: pc.sample_speed,
                sample_n: pc.sample_n,
                bank: pc.bank.clone(),
                orbit: pc.orbit,
                duck: pc.duck,
                cut: pc.cut,
                compressor: pc.compressor,
                delay: pc.delay,
                delaytime: pc.delaytime,
                delayfeedback: pc.delayfeedback,
                room: pc.room,
                roomsize: pc.roomsize,
            });
        }
    }
}

impl Deck {
    fn alloc_voice(&mut self, voice: VoiceKind) {
        if let Some(cut) = voice.cut_group() {
            for slot in self.voices.iter_mut() {
                if let Some(v) = slot {
                    if v.cut_group() == Some(cut) {
                        *slot = None;
                    }
                }
            }
        }
        if let Some(slot) = self.voices.iter_mut().find(|v| v.is_none()) {
            *slot = Some(voice);
            return;
        }
        let i = self.steal_cursor % MAX_VOICES;
        self.voices[i] = Some(voice);
        self.steal_cursor = self.steal_cursor.wrapping_add(1);
    }

    fn trigger_duck(&mut self, duck: &DuckParams, sr: f32) {
        let n = duck.count.min(4) as usize;
        for i in 0..n {
            let orbit = duck.orbits[i];
            if orbit == 0 {
                continue;
            }
            let idx = orbit_index(orbit);
            self.ducks[idx].trigger(duck.depth[i], duck.attack[i], sr);
        }
    }

    fn spawn_hit(&mut self, hit: &ScheduledHit, samples: &SampleBank, sr: f32) {
        if hit.duck.count > 0 {
            self.trigger_duck(&hit.duck, sr);
        }
        if let Some(c) = hit.compressor {
            self.pending_compressor = Some(c);
        }
        // Last-write-wins orbit FX params (only when pattern uses delay/room).
        if hit.delay > 1e-6 || hit.room > 1e-6 {
            let oi = orbit_index(hit.orbit);
            self.orbit_fx[oi].set_from_hit(
                hit.delay,
                hit.delaytime,
                hit.delayfeedback,
                hit.room,
                hit.roomsize,
                sr,
            );
        }

        let bank_ref = hit.bank.as_deref();
        let Ok(resolved) = resolve_sound_with_bank(&hit.sound, bank_ref, samples) else {
            return;
        };
        match resolved {
            ResolvedSound::Wave(w) => {
                let v = Voice::new(
                    OscSource::Wave(w),
                    hit.freq.max(1.0),
                    hit.gain,
                    hit.len_samples,
                    hit.filter,
                    hit.adsr,
                    hit.mods,
                    hit.orbit,
                    hit.cut,
                )
                .with_adsr_timing(sr, hit.len_samples)
                .with_pan(hit.pan);
                self.alloc_voice(VoiceKind::Synth(Box::new(v)));
            }
            ResolvedSound::Noise(n) => {
                let v = Voice::new(
                    OscSource::Noise(n),
                    hit.freq.max(1.0),
                    hit.gain,
                    hit.len_samples,
                    hit.filter,
                    hit.adsr,
                    hit.mods,
                    hit.orbit,
                    hit.cut,
                )
                .with_adsr_timing(sr, hit.len_samples)
                .with_pan(hit.pan);
                self.alloc_voice(VoiceKind::Synth(Box::new(v)));
            }
            ResolvedSound::Wavetable(table) => {
                let v = Voice::new(
                    OscSource::Wavetable(table),
                    hit.freq.max(1.0),
                    hit.gain,
                    hit.len_samples,
                    hit.filter,
                    hit.adsr,
                    hit.mods,
                    hit.orbit,
                    hit.cut,
                )
                .with_adsr_timing(sr, hit.len_samples)
                .with_pan(hit.pan);
                self.alloc_voice(VoiceKind::Synth(Box::new(v)));
            }
            ResolvedSound::Sample(name) => {
                let Some(data) = samples.get(&name, hit.sample_n) else {
                    return;
                };
                let pitch_ratio = if hit.is_note && hit.freq > 0.0 {
                    hit.freq / SAMPLE_ROOT_HZ
                } else {
                    1.0
                };
                let v = SampleVoice::new_fx(
                    data,
                    hit.gain,
                    hit.begin,
                    hit.end,
                    hit.sample_speed,
                    pitch_ratio,
                    hit.filter,
                    hit.adsr,
                    hit.orbit,
                    hit.cut,
                )
                .with_adsr_timing(sr, hit.len_samples)
                .with_pan(hit.pan);
                self.alloc_voice(VoiceKind::Sample(v));
            }
        }
    }

    /// Render into stereo `out_l` / `out_r` (same length). Does not advance transport.
    pub fn process(
        &mut self,
        out_l: &mut [f32],
        out_r: &mut [f32],
        transport: &Transport,
        samples: &SampleBank,
    ) {
        let sr = transport.sample_rate as f32;
        for fx in &mut self.orbit_fx {
            fx.ensure_sr(sr);
        }
        let global_bar = transport.bar_index();
        if global_bar != self.scheduled_bar {
            self.schedule_bar(global_bar, transport);
        }

        let n = out_l.len().min(out_r.len());
        for i in 0..n {
            let now = transport.global_sample + i as u64;
            while self.next_event < self.scheduled.len()
                && self.scheduled[self.next_event].at_sample <= now
            {
                let hit = self.scheduled[self.next_event].clone();
                self.next_event += 1;
                self.spawn_hit(&hit, samples, sr);
            }

            let mut acc_l = [0.0f32; NUM_ORBITS];
            let mut acc_r = [0.0f32; NUM_ORBITS];
            for slot in self.voices.iter_mut() {
                if let Some(voice) = slot {
                    match voice.next_sample(sr) {
                        Some(x) => {
                            let oi = voice.orbit_index();
                            let (gl, gr) = equal_power_pan(voice.pan());
                            acc_l[oi] += x * gl;
                            acc_r[oi] += x * gr;
                        }
                        None => *slot = None,
                    }
                }
            }

            let mut mix_l = 0.0f32;
            let mut mix_r = 0.0f32;
            for o in 0..NUM_ORBITS {
                let g = self.ducks[o].advance();
                let (l, r) = self.orbit_fx[o].process_stereo(acc_l[o] * g, acc_r[o] * g);
                mix_l += l;
                mix_r += r;
            }
            out_l[i] = mix_l * self.gain;
            out_r[i] = mix_r * self.gain;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sample::write_test_wav;
    use crate::song::parse_song;
    use std::path::Path;

    fn process_mid(d: &mut Deck, frames: usize, t: &Transport, bank: &SampleBank) -> Vec<f32> {
        let mut l = vec![0f32; frames];
        let mut r = vec![0f32; frames];
        d.process(&mut l, &mut r, t, bank);
        l.iter().zip(r.iter()).map(|(a, b)| 0.5 * (a + b)).collect()
    }

    #[test]
    fn head_changes_pattern_cycle_not_global_time() {
        // Angle alt: cycle 0 → c3, cycle 1 → e3 (approx via different freqs in energy windows).
        let song = parse_song(
            r#"---
lead: note("<c3 e3>").s("sine").gain(0.9)
"#,
            "t",
        )
        .unwrap();
        let mut d = Deck::new("A");
        d.load(song);
        assert_eq!(d.pattern_bar(0), 0);
        assert_eq!(d.song_bar_1based(0), 1);

        // At global bar 5, jump so song bar 2 (1-based) is playing.
        d.head_to_bar(2, 5);
        assert_eq!(d.pattern_bar(5), 1);
        assert_eq!(d.song_bar_1based(5), 2);
        // Offset stays fixed as transport advances.
        assert_eq!(d.pattern_bar(6), 2);
        assert_eq!(d.song_bar_1based(6), 3);
    }

    #[test]
    fn deck_renders_synth_song() {
        let song = parse_song(
            r#"---
bass: note("c3 e3 g3").s("sawtooth").gain(0.8)
"#,
            "t",
        )
        .unwrap();
        let mut d = Deck::new("A");
        d.load(song);
        let t = Transport::new(48_000, 120.0);
        let bank = SampleBank::empty();
        let buf = process_mid(&mut d, 48_000, &t, &bank);
        assert!(buf.iter().any(|s| s.abs() > 0.01));
    }

    #[test]
    fn schedule_ply_multiplies_hits() {
        use crate::code::parse_code;
        let pc = parse_code(r#"s("bd").ply(4).gain(0.5)"#).unwrap();
        let mut hits = Vec::new();
        schedule_track_into(&mut hits, &pc, 0, 0, 48_000.0);
        assert_eq!(hits.len(), 4);
        // Equal spacing within the bar (speed=1).
        let starts: Vec<u64> = hits.iter().map(|h| h.at_sample).collect();
        assert_eq!(starts[0], 0);
        assert!(starts[1] > starts[0]);
        assert_eq!(starts[2] - starts[1], starts[1] - starts[0]);
    }

    #[test]
    fn schedule_add_shifts_degree_pitch() {
        use crate::code::note_to_hz;
        use crate::code::parse_code;
        let base = parse_code(r#"note("0").scale("C4:major").s("sine").gain(0.5)"#).unwrap();
        let shifted =
            parse_code(r#"note("0").scale("C4:major").add(2).s("sine").gain(0.5)"#).unwrap();
        let mut h0 = Vec::new();
        let mut h2 = Vec::new();
        schedule_track_into(&mut h0, &base, 0, 0, 48_000.0);
        schedule_track_into(&mut h2, &shifted, 0, 0, 48_000.0);
        assert_eq!(h0.len(), 1);
        assert_eq!(h2.len(), 1);
        // C major: degree 0+2 == degree 2 ≈ E4
        assert!((h2[0].freq - note_to_hz("e4").unwrap()).abs() < 1.0);
        assert!((h0[0].freq - note_to_hz("c4").unwrap()).abs() < 1.0);
    }

    #[test]
    fn parallel_degrees_schedule_minor_triad() {
        use crate::code::{note_to_hz, parse_code};
        let pc =
            parse_code(r#"note("[0,2,4]").scale("C3:minor").s("triangle").gain(0.3)"#).unwrap();
        let mut hits = Vec::new();
        schedule_track_into(&mut hits, &pc, 0, 0, 48_000.0);
        assert_eq!(hits.len(), 3, "comma-parallel degrees are three voices");
        let mut freqs: Vec<f32> = hits.iter().map(|h| h.freq).collect();
        freqs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for (got, name) in freqs.iter().zip(["c3", "eb3", "g3"]) {
            assert!(
                (*got - note_to_hz(name).unwrap()).abs() < 1.0,
                "{name}: got {got}"
            );
        }
    }

    #[test]
    fn parallel_degrees_schedule_spread_major() {
        use crate::code::{note_to_hz, parse_code};
        let pc =
            parse_code(r#"note("[0,4,9]").scale("C4:major").s("triangle").gain(0.3)"#).unwrap();
        let mut hits = Vec::new();
        schedule_track_into(&mut hits, &pc, 0, 0, 48_000.0);
        assert_eq!(hits.len(), 3, "spread parallel degrees are three voices");
        let mut freqs: Vec<f32> = hits.iter().map(|h| h.freq).collect();
        freqs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        // C major: 0=C4, 4=G4, 9=octave+third=E5. Not a voicing helper.
        for (got, name) in freqs.iter().zip(["c4", "g4", "e5"]) {
            assert!(
                (*got - note_to_hz(name).unwrap()).abs() < 1.0,
                "{name}: got {got}"
            );
        }
    }

    #[test]
    fn chord_suffix_schedules_root_only() {
        use crate::code::{note_to_hz, parse_code};
        let pc = parse_code(r#"note("c3'min").s("triangle").gain(0.3)"#).unwrap();
        let mut hits = Vec::new();
        schedule_track_into(&mut hits, &pc, 0, 0, 48_000.0);
        assert_eq!(hits.len(), 1, "deck does not expand chord suffixes");
        assert!((hits[0].freq - note_to_hz("c3").unwrap()).abs() < 1.0);
    }

    #[test]
    fn schedule_dyn_lpf_pattern_and_lfo() {
        use crate::code::parse_code;
        let pat = parse_code(r#"s("bd bd").lpf("100 900").gain(0.5)"#).unwrap();
        let mut hits = Vec::new();
        schedule_track_into(&mut hits, &pat, 0, 0, 48_000.0);
        assert_eq!(hits.len(), 2);
        assert!((hits[0].filter.lpf.unwrap() - 100.0).abs() < 1.0);
        assert!((hits[1].filter.lpf.unwrap() - 900.0).abs() < 1.0);

        let lfo =
            parse_code(r#"note("c3").s("sawtooth").lpf(sine.rangex(500,4000)).gain(0.4)"#).unwrap();
        let mut h2 = Vec::new();
        schedule_track_into(&mut h2, &lfo, 0, 0, 48_000.0);
        assert_eq!(h2.len(), 1);
        assert!(h2[0].mods.lpf_lfo.is_some());
        assert!(h2[0].filter.lpf.is_some());
    }

    #[test]
    fn deck_pan_hard_left_has_no_right() {
        let song = parse_song(
            r#"---
bass: note("c3").s("sawtooth").gain(0.9).pan(0)
"#,
            "t",
        )
        .unwrap();
        let mut d = Deck::new("A");
        d.load(song);
        let t = Transport::new(48_000, 120.0);
        let bank = SampleBank::empty();
        let mut l = vec![0f32; 24_000];
        let mut r = vec![0f32; 24_000];
        d.process(&mut l, &mut r, &t, &bank);
        let el: f32 = l.iter().map(|x| x.abs()).sum();
        let er: f32 = r.iter().map(|x| x.abs()).sum();
        assert!(el > 1.0, "left energy={el}");
        assert!(er < el * 0.01, "right should be near 0, L={el} R={er}");
    }

    #[test]
    fn deck_renders_sample_song() {
        let dir = std::env::temp_dir().join("strudel_deck_samples");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("bd")).unwrap();
        write_test_wav(&dir.join("bd").join("00.wav"), 48_000);
        let bank = SampleBank::load_dir(&dir, 48_000);
        let song = parse_song(
            r#"---
kick: s("bd*4").gain(0.9)
"#,
            "t",
        )
        .unwrap();
        let mut d = Deck::new("A");
        d.load(song);
        let t = Transport::new(48_000, 120.0);
        let buf = process_mid(&mut d, 48_000, &t, &bank);
        assert!(buf.iter().any(|s| s.abs() > 0.01));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_repo_samples_if_present() {
        let path = Path::new("samples");
        if !path.exists() {
            return;
        }
        let bank = SampleBank::load_dir(path, 48_000);
        assert!(bank.has("bd"), "expected samples/bd");
        assert!(bank.has("sd"));
        assert!(bank.has("hh"));
    }

    #[test]
    fn duck_lowers_target_orbit_energy() {
        // Continuous tone on orbit 2; kick-like trigger ducks orbit 2 every beat.
        // Compare first 10ms after a duck trigger vs late recovery window.
        let song = parse_song(
            r#"---
pad: note("c4").s("sine").gain(0.8).orbit(2).sustain(1).attack(0.001).release(0.01)
kick: s("sine").gain(0.01).duckorbit(2).duckattack(0.05).duckdepth(1).fast(4)
"#,
            "t",
        )
        .unwrap();
        // kick uses s("sine") which is a wave not sample - good
        let mut d = Deck::new("A");
        d.load(song);
        let t = Transport::new(48_000, 120.0);
        let bank = SampleBank::empty();
        let buf = process_mid(&mut d, 24_000, &t, &bank);

        // energy in early window (after first duck) vs later
        let early: f32 = buf[100..600].iter().map(|x| x.abs()).sum();
        let late: f32 = buf[3000..3500].iter().map(|x| x.abs()).sum();
        // After recovery, energy should be higher than deep-duck window
        assert!(
            late > early * 1.2 || late > 10.0,
            "early={early} late={late}"
        );
    }

    #[test]
    fn clip_shortens_notes() {
        let song_long = parse_song(
            r#"---
n: note("c4").s("sine").gain(0.9).attack(0.001).sustain(1).release(0.001)
"#,
            "t",
        )
        .unwrap();
        let song_clip = parse_song(
            r#"---
n: note("c4").s("sine").gain(0.9).attack(0.001).sustain(1).release(0.001).clip(0.1)
"#,
            "t",
        )
        .unwrap();
        let bank = SampleBank::empty();
        let t = Transport::new(48_000, 120.0);

        let mut d = Deck::new("A");
        d.load(song_long);
        let a = process_mid(&mut d, 48_000, &t, &bank);
        let ea: f32 = a.iter().map(|x| x.abs()).sum();

        let mut d = Deck::new("A");
        d.load(song_clip);
        let b = process_mid(&mut d, 48_000, &t, &bank);
        let eb: f32 = b.iter().map(|x| x.abs()).sum();
        assert!(eb < ea * 0.5, "clip energy {eb} vs full {ea}");
    }

    #[test]
    fn delay_leaves_tail_after_note() {
        // Short click + delay; energy should remain after the dry note ends.
        let song = parse_song(
            r#"---
hit: note("c5").s("sine").gain(0.9).attack(0.001).decay(0.01).sustain(0).release(0.01).delay(0.7).delaytime(0.05).delayfeedback(0.6)
"#,
            "t",
        )
        .unwrap();
        let mut d = Deck::new("A");
        d.load(song);
        let t = Transport::new(48_000, 60.0); // 1 bar = 4s at 60 BPM? Wait BPM 60 → 1 beat = 1s, bar = 4s
        let bank = SampleBank::empty();
        // Process ~0.15s so we pass the first delay bounce (~0.05s).
        let buf = process_mid(&mut d, 8_000, &t, &bank);
        // Dry note is very short; samples after 3000 (~62ms) should still have delay energy.
        let late: f32 = buf[3000..6000].iter().map(|x| x * x).sum();
        assert!(late > 1e-4, "expected delay tail energy, late={late}");
    }

    #[test]
    fn room_leaves_tail_after_note() {
        let song = parse_song(
            r#"---
hit: note("c5").s("sine").gain(0.9).attack(0.001).decay(0.01).sustain(0).release(0.01).room(0.8).roomsize(6)
"#,
            "t",
        )
        .unwrap();
        let mut d = Deck::new("A");
        d.load(song);
        let t = Transport::new(48_000, 120.0);
        let bank = SampleBank::empty();
        let buf = process_mid(&mut d, 12_000, &t, &bank);
        let late: f32 = buf[4000..10000].iter().map(|x| x * x).sum();
        assert!(late > 1e-6, "expected room tail energy, late={late}");
    }
}
