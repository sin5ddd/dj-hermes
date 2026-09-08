//! Deck: schedule song tracks for one bar, orbit buses, duck, voice pool.

use crate::code::{Adsr, DuckParams, FilterParams, ModParams, PatternCode};
use crate::dsp::{equal_power_pan, orbit_index, CompressorParams, DuckState, OrbitFx, NUM_ORBITS};
use crate::midi::{self, MidiEvent};
use crate::mini;
use crate::sample::{SampleBank, SampleVoice, VoiceKind, SAMPLE_ROOT_HZ};
use crate::scale::resolve_pitch_with_add;
use crate::song::Song;
use crate::sound::{resolve_sound_with_bank, split_sound_selector, ResolvedSound, SampleSelector};
use crate::synth::{OscSource, Voice};
use crate::transport::Transport;
use crossbeam::channel::Sender;

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
    /// 1-based override from `// @midi ch=N`.
    midi_ch: Option<u8>,
    /// 0 = first synth `$:`, 1 = second (MIDI ch 8 / 9).
    synth_ordinal: u8,
    /// `$:` index for `.cut` (choke is per-track, not global).
    cut_track: u16,
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
    /// Last *play* bar we scheduled against (relative clock during time-repeat).
    scheduled_bar: u64,
    /// Previous `map_play` sample, for wrap / jump detection.
    last_play_sample: Option<u64>,
    /// Pattern cycle = (global_bar as i64 + cycle_offset).max(0).
    /// Set by head/cue so a deck can play a different song bar while transport stays locked.
    cycle_offset: i64,
    ducks: [DuckState; NUM_ORBITS],
    /// Per-orbit global delay / room (Deck-local; not shared across A/B).
    orbit_fx: [OrbitFx; NUM_ORBITS],
    /// Last compressor params seen on a spawned hit (Engine may promote to Mixer).
    pub pending_compressor: Option<CompressorParams>,
    midi_tx: Option<Sender<MidiEvent>>,
    midi_slots: Vec<Option<(u8, u8)>>,
    midi_timed: Vec<(u64, u8, u8)>,
    /// Last CC value per channel (0..=15) and controller (0..=127). Dedup sends.
    midi_cc_last: [[Option<u8>; 128]; 16],
    /// Send Bank Select + PC once on the bar after `load`.
    midi_prog_pending: bool,
    /// `--midi-only`: emit MIDI, do not spawn synth/sample voices.
    midi_only: bool,
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
            last_play_sample: None,
            cycle_offset: 0,
            ducks: [DuckState::default(); NUM_ORBITS],
            orbit_fx: std::array::from_fn(|_| OrbitFx::new()),
            pending_compressor: None,
            midi_tx: None,
            midi_slots: vec![None; MAX_VOICES],
            midi_timed: Vec::new(),
            midi_cc_last: [[None; 128]; 16],
            midi_prog_pending: false,
            midi_only: false,
        }
    }

    /// Attach a MIDI sender (play deck A). Audio thread only `try_send`s.
    pub fn set_midi(&mut self, tx: Option<Sender<MidiEvent>>) {
        self.midi_tx = tx;
    }

    /// Skip synth/sample voices (exhibit: SEQTRAK only). MIDI still fires.
    pub fn set_midi_only(&mut self, yes: bool) {
        self.midi_only = yes;
    }

    /// Panic notes then drop the sender so `MidiHandle` join is not stuck on this clone.
    pub fn shutdown_midi(&mut self) {
        self.midi_panic();
        self.midi_tx = None;
    }

    /// Replace the loaded song and force a clean reschedule.
    ///
    /// Clears ringing voices and the event queue (same policy as [`Self::head_to_bar`] /
    /// [`Self::unload`]) so hot-reload does not stack a new bar-head onset on top of
    /// leftover voices (heard as ~2× level on the first hit after load).
    pub fn load(&mut self, song: Song) {
        self.midi_panic();
        self.midi_cc_last = [[None; 128]; 16];
        self.midi_prog_pending = true;
        self.song = Some(song);
        for v in &mut self.voices {
            *v = None;
        }
        self.scheduled.clear();
        self.next_event = 0;
        self.scheduled_bar = u64::MAX;
        self.last_play_sample = None;
        self.cycle_offset = 0;
        self.ducks = [DuckState::default(); NUM_ORBITS];
        for fx in &mut self.orbit_fx {
            fx.clear();
        }
        self.pending_compressor = None;
    }

    pub fn unload(&mut self) {
        self.midi_panic();
        self.song = None;
        for v in &mut self.voices {
            *v = None;
        }
        self.scheduled.clear();
        self.next_event = 0;
        self.scheduled_bar = u64::MAX;
        self.last_play_sample = None;
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
        self.midi_panic();
        let target_cycle = bar_1based.saturating_sub(1);
        self.cycle_offset = target_cycle as i64 - apply_global_bar as i64;
        for v in &mut self.voices {
            *v = None;
        }
        self.scheduled.clear();
        self.next_event = 0;
        self.scheduled_bar = u64::MAX;
        self.last_play_sample = None;
        self.ducks = [DuckState::default(); NUM_ORBITS];
    }

    fn cut_sounding(&mut self) {
        self.midi_panic();
        for v in &mut self.voices {
            *v = None;
        }
        self.ducks = [DuckState::default(); NUM_ORBITS];
    }

    fn rewind_events(&mut self, now: u64) {
        self.next_event = self.scheduled.partition_point(|e| e.at_sample < now);
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
        let tracks: Vec<(PatternCode, crate::song::MidiAnnot, bool)> = self
            .song
            .as_ref()
            .map(|s| {
                s.tracks
                    .iter()
                    .filter(|t| !t.muted)
                    .map(|t| {
                        let has_fm = t.code.mod_params.fm.abs() > 1e-6;
                        let is_syn = midi::track_is_synth(&t.code.sound, t.code.is_note, has_fm);
                        (
                            t.code.clone(),
                            crate::song::MidiAnnot {
                                ch: t.midi_ch,
                                msb: t.midi_msb,
                                lsb: t.midi_lsb,
                                pc: t.midi_pc,
                            },
                            is_syn,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        if self.midi_prog_pending {
            self.midi_prog_pending = false;
            if self.midi_tx.is_some() {
                self.send_midi_programs(&tracks);
            }
        }
        let mut synth_i = 0u8;
        for (track_i, (pc, annot, is_syn)) in tracks.iter().enumerate() {
            // 0 → ch 8, 1 → ch 9, 2+ → ch 8 (map_channel treats only 1 as SYNTH 2).
            let ord = if *is_syn { synth_i } else { 0 };
            schedule_track_into(
                &mut self.scheduled,
                pc,
                pattern_bar,
                bar_start,
                spb,
                annot.ch,
                ord,
                track_i as u16,
            );
            if *is_syn {
                synth_i = synth_i.saturating_add(1);
            }
        }
        self.scheduled.sort_by_key(|e| e.at_sample);
        self.scheduled_bar = global_bar;
    }
}

#[allow(clippy::too_many_arguments)]
fn schedule_track_into(
    scheduled: &mut Vec<ScheduledHit>,
    pc: &PatternCode,
    bar: u64,
    bar_start: u64,
    spb: f64,
    midi_ch: Option<u8>,
    synth_ordinal: u8,
    cut_track: u16,
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
                midi_ch,
                synth_ordinal,
                cut_track,
            });
        }
    }
}

impl Deck {
    fn alloc_voice(&mut self, voice: VoiceKind) -> usize {
        if let Some(key) = voice.cut_key() {
            for i in 0..self.voices.len() {
                if self.voices[i].as_ref().and_then(|v| v.cut_key()) == Some(key) {
                    self.midi_off_slot(i);
                    if let Some(v) = self.voices[i].as_mut() {
                        v.force_release();
                        v.clear_cut_group();
                    }
                }
            }
        }
        if let Some(i) = self.voices.iter().position(|v| v.is_none()) {
            self.voices[i] = Some(voice);
            return i;
        }
        if let Some(i) = self
            .voices
            .iter()
            .position(|v| v.as_ref().is_some_and(|x| x.is_releasing()))
        {
            self.midi_off_slot(i);
            self.voices[i] = Some(voice);
            return i;
        }
        let i = self.steal_cursor % MAX_VOICES;
        self.midi_off_slot(i);
        self.voices[i] = Some(voice);
        self.steal_cursor = self.steal_cursor.wrapping_add(1);
        i
    }

    fn push_midi(&self, ev: MidiEvent) {
        if let Some(tx) = &self.midi_tx {
            midi::try_push(tx, ev);
        }
    }

    fn push_cc(&mut self, ch: u8, cc: u8, val: u8) {
        let chi = (ch & 0x0F) as usize;
        let cci = cc as usize;
        if self.midi_cc_last[chi][cci] == Some(val) {
            return;
        }
        self.midi_cc_last[chi][cci] = Some(val);
        self.push_midi(MidiEvent::Cc {
            ch: chi as u8,
            cc,
            val,
        });
    }

    fn send_midi_programs(&mut self, tracks: &[(PatternCode, crate::song::MidiAnnot, bool)]) {
        let mut synth_i = 0u8;
        for (pc, annot, is_syn) in tracks {
            let ord = if *is_syn { synth_i } else { 0 };
            if *is_syn {
                synth_i = synth_i.saturating_add(1);
            }
            if annot.msb.is_none() && annot.lsb.is_none() && annot.pc.is_none() {
                continue;
            }
            let has_fm = pc.mod_params.fm.abs() > 1e-6;
            let ch = midi::map_channel(&pc.sound, has_fm, annot.ch, ord);
            let msb = annot.msb.unwrap_or(0);
            let lsb = annot.lsb.unwrap_or(0);
            // SOUND SELECT runs on Program Change; always send Bank then PC.
            self.push_midi(MidiEvent::Cc {
                ch,
                cc: midi::CC_BANK_MSB,
                val: msb,
            });
            self.push_midi(MidiEvent::Cc {
                ch,
                cc: midi::CC_BANK_LSB,
                val: lsb,
            });
            self.midi_cc_last[ch as usize][midi::CC_BANK_MSB as usize] = Some(msb);
            self.midi_cc_last[ch as usize][midi::CC_BANK_LSB as usize] = Some(lsb);
            self.push_midi(MidiEvent::ProgramChange {
                ch,
                program: annot.pc.unwrap_or(0),
            });
        }
    }

    fn midi_off_slot(&mut self, i: usize) {
        let pair = self.midi_slots.get_mut(i).and_then(Option::take);
        if let Some((ch, note)) = pair {
            self.push_midi(MidiEvent::NoteOff { ch, note });
        }
    }

    fn midi_panic(&mut self) {
        for i in 0..self.midi_slots.len() {
            self.midi_off_slot(i);
        }
        let timed = std::mem::take(&mut self.midi_timed);
        for (_, ch, note) in timed {
            self.push_midi(MidiEvent::NoteOff { ch, note });
        }
    }

    fn midi_note_on(&mut self, hit: &ScheduledHit, now: u64, slot: Option<usize>) {
        let has_fm = hit.mods.fm.abs() > 1e-6;
        let (ch, note, vel) = midi::map_hit(
            &hit.sound,
            hit.freq,
            hit.is_note,
            has_fm,
            hit.midi_ch,
            hit.synth_ordinal,
            hit.gain,
        );
        let (ccs, n) = midi::hit_ccs(midi::HitCcInput {
            ch,
            gain: hit.gain,
            pan: hit.pan,
            lpf: hit.filter.lpf,
            lpq: hit.filter.lpq,
            attack: hit.adsr.attack,
            decay: hit.adsr.decay,
            release: hit.adsr.release,
            room: hit.room,
            delay: hit.delay,
            fm: hit.mods.fm,
        });
        for &(cc, val) in &ccs[..n] {
            self.push_cc(ch, cc, val);
        }
        self.push_midi(MidiEvent::NoteOn { ch, note, vel });
        if let Some(i) = slot {
            if i < self.midi_slots.len() {
                self.midi_slots[i] = Some((ch, note));
            }
        } else {
            self.midi_timed
                .push((now.saturating_add(hit.len_samples), ch, note));
        }
    }

    fn flush_timed_midi(&mut self, now: u64) {
        let mut i = 0;
        while i < self.midi_timed.len() {
            if self.midi_timed[i].0 <= now {
                let (_, ch, note) = self.midi_timed.remove(i);
                self.push_midi(MidiEvent::NoteOff { ch, note });
            } else {
                i += 1;
            }
        }
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

    fn spawn_hit(&mut self, hit: &ScheduledHit, samples: &SampleBank, sr: f32, now: u64) {
        if self.midi_only {
            self.midi_note_on(hit, now, None);
            return;
        }
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
        let Some((base, sel)) = split_sound_selector(&hit.sound) else {
            self.midi_note_on(hit, now, None);
            return;
        };
        let Ok(resolved) = resolve_sound_with_bank(&base, bank_ref, samples) else {
            self.midi_note_on(hit, now, None);
            return;
        };
        let slot = match resolved {
            ResolvedSound::Wave(w) => {
                if !matches!(sel, SampleSelector::Default) {
                    self.midi_note_on(hit, now, None);
                    return;
                }
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
                .with_pan(hit.pan)
                .with_cut_track(hit.cut_track);
                Some(self.alloc_voice(VoiceKind::Synth(Box::new(v))))
            }
            ResolvedSound::Noise(n) => {
                if !matches!(sel, SampleSelector::Default) {
                    self.midi_note_on(hit, now, None);
                    return;
                }
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
                .with_pan(hit.pan)
                .with_cut_track(hit.cut_track);
                Some(self.alloc_voice(VoiceKind::Synth(Box::new(v))))
            }
            ResolvedSound::Wavetable(table) => {
                if !matches!(sel, SampleSelector::Default) {
                    self.midi_note_on(hit, now, None);
                    return;
                }
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
                .with_pan(hit.pan)
                .with_cut_track(hit.cut_track);
                Some(self.alloc_voice(VoiceKind::Synth(Box::new(v))))
            }
            ResolvedSound::Sample(name) => {
                let data = match &sel {
                    SampleSelector::Default => samples.get(&name, hit.sample_n),
                    SampleSelector::Index(i) => samples.get(&name, Some(*i)),
                    SampleSelector::Stem(slug) => samples.get_stem(&name, slug),
                };
                let Some(data) = data else {
                    self.midi_note_on(hit, now, None);
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
                .with_pan(hit.pan)
                .with_cut_track(hit.cut_track);
                Some(self.alloc_voice(VoiceKind::Sample(v)))
            }
        };
        if let Some(i) = slot {
            self.midi_note_on(hit, now, Some(i));
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

        let n = out_l.len().min(out_r.len());
        // Schedule from the buffer-head play bar only. Crossing an absolute bar
        // mid-buffer must not spawn the next bar before Engine applies pending
        // (mute / load) at the next process() head.
        let head_play = transport.map_play(transport.global_sample);
        let head_bar = transport.bar_index_of(head_play);
        if head_bar != self.scheduled_bar {
            self.schedule_bar(head_bar, transport);
        }

        for i in 0..n {
            let abs = transport.global_sample + i as u64;
            let now = transport.map_play(abs);
            if let Some(prev) = self.last_play_sample {
                if now < prev {
                    self.cut_sounding();
                    self.rewind_events(now);
                } else if now > prev.saturating_add(1) {
                    self.cut_sounding();
                    let play_bar = transport.bar_index_of(now);
                    self.schedule_bar(play_bar, transport);
                }
            }
            self.last_play_sample = Some(now);

            while self.next_event < self.scheduled.len()
                && self.scheduled[self.next_event].at_sample <= now
            {
                let hit = self.scheduled[self.next_event].clone();
                self.next_event += 1;
                self.spawn_hit(&hit, samples, sr, now);
            }
            self.flush_timed_midi(now);

            let mut acc_l = [0.0f32; NUM_ORBITS];
            let mut acc_r = [0.0f32; NUM_ORBITS];
            for vi in 0..self.voices.len() {
                let stepped = self.voices[vi].as_mut().map(|voice| {
                    let oi = voice.orbit_index();
                    let pan = voice.pan();
                    (voice.next_sample(sr), oi, pan)
                });
                let Some((sample, oi, pan)) = stepped else {
                    continue;
                };
                match sample {
                    Some(x) => {
                        let (gl, gr) = equal_power_pan(pan);
                        acc_l[oi] += x * gl;
                        acc_r[oi] += x * gr;
                    }
                    None => {
                        self.midi_off_slot(vi);
                        self.voices[vi] = None;
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
    use crate::midi::MidiEvent;
    use crate::sample::{write_test_wav, write_test_wav_secs};
    use crate::song::parse_song;
    use crate::transport::RepeatDiv;
    use crossbeam::channel::unbounded;
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
        schedule_track_into(&mut hits, &pc, 0, 0, 48_000.0, None, 0, 0);
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
        schedule_track_into(&mut h0, &base, 0, 0, 48_000.0, None, 0, 0);
        schedule_track_into(&mut h2, &shifted, 0, 0, 48_000.0, None, 0, 0);
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
        schedule_track_into(&mut hits, &pc, 0, 0, 48_000.0, None, 0, 0);
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
        schedule_track_into(&mut hits, &pc, 0, 0, 48_000.0, None, 0, 0);
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
        schedule_track_into(&mut hits, &pc, 0, 0, 48_000.0, None, 0, 0);
        assert_eq!(hits.len(), 1, "deck does not expand chord suffixes");
        assert!((hits[0].freq - note_to_hz("c3").unwrap()).abs() < 1.0);
    }

    #[test]
    fn schedule_dyn_lpf_pattern_and_lfo() {
        use crate::code::parse_code;
        let pat = parse_code(r#"s("bd bd").lpf("100 900").gain(0.5)"#).unwrap();
        let mut hits = Vec::new();
        schedule_track_into(&mut hits, &pat, 0, 0, 48_000.0, None, 0, 0);
        assert_eq!(hits.len(), 2);
        assert!((hits[0].filter.lpf.unwrap() - 100.0).abs() < 1.0);
        assert!((hits[1].filter.lpf.unwrap() - 900.0).abs() < 1.0);

        let lfo =
            parse_code(r#"note("c3").s("sawtooth").lpf(sine.rangex(500,4000)).gain(0.4)"#).unwrap();
        let mut h2 = Vec::new();
        schedule_track_into(&mut h2, &lfo, 0, 0, 48_000.0, None, 0, 0);
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
        let dir = std::env::temp_dir().join("dj_hermes_deck_samples");
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
    fn deck_renders_part_slug_sample() {
        let dir = std::env::temp_dir().join("dj_hermes_deck_slug");
        let _ = std::fs::remove_dir_all(&dir);
        let bd = dir.join("bd");
        std::fs::create_dir_all(&bd).unwrap();
        write_test_wav_secs(&bd.join("00.wav"), 48_000, 0.05);
        write_test_wav_secs(&bd.join("8b.wav"), 48_000, 0.4);
        let bank = SampleBank::load_dir(&dir, 48_000);

        let song = parse_song(
            r#"---
kick: s("bd:8b").gain(0.9)
"#,
            "t",
        )
        .unwrap();
        let mut d = Deck::new("A");
        d.load(song);
        let t = Transport::new(48_000, 120.0);
        let buf = process_mid(&mut d, 48_000, &t, &bank);
        assert!(
            buf.iter().any(|s| s.abs() > 0.01),
            "bd:8b should play the named stem"
        );

        let miss = parse_song(
            r#"---
kick: s("bd:nope").gain(0.9)
"#,
            "t",
        )
        .unwrap();
        let mut d2 = Deck::new("A");
        d2.load(miss);
        let silent = process_mid(&mut d2, 48_000, &t, &bank);
        let peak = silent.iter().fold(0.0f32, |a, x| a.max(x.abs()));
        assert!(peak < 0.001, "unknown slug should be silent, peak={peak}");
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
        if bank.get_stem("bd", "hf").is_some() {
            assert!(
                bank.get_stem("hh", "cl").is_some(),
                "catalog hh:cl should ship with bd:hf"
            );
        }
    }

    #[test]
    fn deck_plays_catalog_part_slug_if_present() {
        let path = Path::new("samples");
        if !path.exists() {
            return;
        }
        let bank = SampleBank::load_dir(path, 48_000);
        if bank.get_stem("bd", "hf").is_none() {
            return;
        }
        let song = parse_song(
            r#"---
kick: s("bd:hf*4").gain(0.9)
"#,
            "t",
        )
        .unwrap();
        let mut d = Deck::new("A");
        d.load(song);
        let t = Transport::new(48_000, 120.0);
        let buf = process_mid(&mut d, 48_000, &t, &bank);
        assert!(
            buf.iter().any(|s| s.abs() > 0.01),
            "bd:hf catalog kick should produce energy"
        );
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

    #[test]
    fn cut_releases_instead_of_hard_stop() {
        let song = parse_song(
            r#"---
lead: note("c4 c4").s("sine").gain(0.9).cut(1).attack(0.001).decay(0).sustain(1).release(0.05)
"#,
            "t",
        )
        .unwrap();
        let mut d = Deck::new("A");
        d.load(song);
        let t = Transport::new(48_000, 120.0);
        let bank = SampleBank::empty();
        let buf = process_mid(&mut d, 52_000, &t, &bank);
        let before = buf[47_000].abs();
        let at_cut = buf[48_000].abs();
        assert!(before > 0.05, "sustain before second note, before={before}");
        assert!(
            at_cut > before * 0.3,
            "cut should ADSR-release, not drop to 0: before={before} at_cut={at_cut}"
        );
    }

    #[test]
    fn cut_group_does_not_choke_other_tracks() {
        let one = parse_song(
            r#"---
a: note("c4").s("sine").gain(0.8).cut(1).attack(0.001).decay(0).sustain(1).release(0.05)
"#,
            "t",
        )
        .unwrap();
        let two = parse_song(
            r#"---
a: note("c4").s("sine").gain(0.8).cut(1).attack(0.001).decay(0).sustain(1).release(0.05)
b: note("e4").s("sine").gain(0.8).cut(1).attack(0.001).decay(0).sustain(1).release(0.05)
"#,
            "t",
        )
        .unwrap();
        let t = Transport::new(48_000, 120.0);
        let bank = SampleBank::empty();
        let mut d1 = Deck::new("A");
        d1.load(one);
        let e1: f32 = process_mid(&mut d1, 8_000, &t, &bank)
            .iter()
            .map(|s| s.abs())
            .sum();
        let mut d2 = Deck::new("A");
        d2.load(two);
        let e2: f32 = process_mid(&mut d2, 8_000, &t, &bank)
            .iter()
            .map(|s| s.abs())
            .sum();
        assert!(
            e2 > e1 * 1.15,
            "shared .cut(1) must not mute the other $:  one={e1} two={e2}"
        );
    }

    fn count_note_ons(rx: &crossbeam::channel::Receiver<MidiEvent>) -> usize {
        let mut n = 0;
        while let Ok(ev) = rx.try_recv() {
            if matches!(ev, MidiEvent::NoteOn { .. }) {
                n += 1;
            }
        }
        n
    }

    #[test]
    fn time_repeat_retriggers_slice_hits_sixteen_times() {
        let song = parse_song(
            r#"---
lead: note("c3").s("sine").gain(0.9)
"#,
            "t",
        )
        .unwrap();
        let mut d = Deck::new("A");
        d.load(song);
        d.set_midi_only(true);
        let (tx, rx) = unbounded();
        d.set_midi(Some(tx));
        let mut t = Transport::new(48_000, 120.0);
        t.start_repeat(RepeatDiv::Sixteenth);
        let bank = SampleBank::empty();
        let mut l = vec![0f32; 96_000];
        let mut r = vec![0f32; 96_000];
        d.process(&mut l, &mut r, &t, &bank);
        assert_eq!(
            count_note_ons(&rx),
            16,
            "16th-note repeat of a bar-head hit should fire 16 times in one absolute bar"
        );
    }

    #[test]
    fn time_repeat_skips_hits_outside_slice() {
        let song = parse_song(
            r#"---
lead: note("~ c3").s("sine").gain(0.9)
"#,
            "t",
        )
        .unwrap();
        let mut d = Deck::new("A");
        d.load(song);
        d.set_midi_only(true);
        let (tx, rx) = unbounded();
        d.set_midi(Some(tx));
        let mut t = Transport::new(48_000, 120.0);
        t.start_repeat(RepeatDiv::Sixteenth);
        let bank = SampleBank::empty();
        let mut l = vec![0f32; 96_000];
        let mut r = vec![0f32; 96_000];
        d.process(&mut l, &mut r, &t, &bank);
        assert_eq!(
            count_note_ons(&rx),
            0,
            "hit in the second half of the bar is outside the first 16th slice"
        );
    }

    #[test]
    fn time_repeat_keeps_play_bar_while_absolute_bar_advances() {
        let song = parse_song(
            r#"---
lead: note("c3").s("sine").gain(0.9)
"#,
            "t",
        )
        .unwrap();
        let mut d = Deck::new("A");
        d.load(song);
        let mut t = Transport::new(48_000, 120.0);
        t.global_sample = 48_000;
        t.start_repeat(RepeatDiv::Quarter);
        let bank = SampleBank::empty();
        let mut l = vec![0f32; 24_000];
        let mut r = vec![0f32; 24_000];
        d.process(&mut l, &mut r, &t, &bank);
        t.advance(24_000);
        d.process(&mut l, &mut r, &t, &bank);
        t.advance(24_000);
        assert_eq!(t.bar_index(), 1);
        assert_eq!(d.scheduled_bar, 0);
        assert_eq!(t.play_bar_index(), 0);
    }
}
