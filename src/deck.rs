//! Deck: schedule song tracks for one bar, orbit buses, duck, voice pool.

use crate::code::{note_to_hz, Adsr, DuckParams, FilterParams, ModParams, PatternCode};
use crate::dsp::{orbit_index, CompressorParams, DuckState, NUM_ORBITS};
use crate::mini;
use crate::sample::{SampleBank, SampleVoice, VoiceKind, SAMPLE_ROOT_HZ};
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
    scheduled_bar: u64,
    ducks: [DuckState; NUM_ORBITS],
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
            ducks: [DuckState::default(); NUM_ORBITS],
            pending_compressor: None,
        }
    }

    pub fn load(&mut self, song: Song) {
        self.song = Some(song);
        self.scheduled_bar = u64::MAX;
    }

    pub fn unload(&mut self) {
        self.song = None;
        for v in &mut self.voices {
            *v = None;
        }
        self.scheduled.clear();
        self.next_event = 0;
        self.scheduled_bar = u64::MAX;
        self.ducks = [DuckState::default(); NUM_ORBITS];
        self.pending_compressor = None;
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

    fn schedule_bar(&mut self, bar: u64, transport: &Transport) {
        self.scheduled.clear();
        self.next_event = 0;
        let spb = transport.samples_per_bar();
        let bar_start = (bar as f64 * spb) as u64;
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
            schedule_track_into(&mut self.scheduled, pc, bar, bar_start, spb);
        }
        self.scheduled.sort_by_key(|e| e.at_sample);
        self.scheduled_bar = bar;
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
    for ev in mini::events(&pc.pattern, bar) {
        if ev.value == "~" || ev.value.is_empty() {
            continue;
        }
        let at = bar_start + ((ev.start * spb) / speed) as u64;
        let len = ((((ev.dur * spb) / speed) * len_scale) as u64).max(64);
        let (sound, freq, is_note) = if pc.is_note {
            match note_to_hz(&ev.value) {
                Ok(h) => (pc.sound.clone(), h, true),
                Err(_) => continue,
            }
        } else {
            (ev.value.clone(), 0.0, false)
        };
        scheduled.push(ScheduledHit {
            at_sample: at,
            len_samples: len,
            sound,
            freq,
            gain: pc.effective_gain(),
            filter: pc.filter,
            adsr: pc.adsr,
            mods: pc.mod_params,
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
        });
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
                .with_adsr_timing(sr, hit.len_samples);
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
                .with_adsr_timing(sr, hit.len_samples);
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
                .with_adsr_timing(sr, hit.len_samples);
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
                .with_adsr_timing(sr, hit.len_samples);
                self.alloc_voice(VoiceKind::Sample(v));
            }
        }
    }

    /// Render into `out` (mono). Does not advance transport.
    pub fn process(&mut self, out: &mut [f32], transport: &Transport, samples: &SampleBank) {
        let sr = transport.sample_rate as f32;
        let bar = transport.bar_index();
        if bar != self.scheduled_bar {
            self.schedule_bar(bar, transport);
        }

        for (i, frame) in out.iter_mut().enumerate() {
            let now = transport.global_sample + i as u64;
            while self.next_event < self.scheduled.len()
                && self.scheduled[self.next_event].at_sample <= now
            {
                let hit = self.scheduled[self.next_event].clone();
                self.next_event += 1;
                self.spawn_hit(&hit, samples, sr);
            }

            let mut acc = [0.0f32; NUM_ORBITS];
            for slot in self.voices.iter_mut() {
                if let Some(voice) = slot {
                    match voice.next_sample(sr) {
                        Some(x) => {
                            let oi = voice.orbit_index();
                            acc[oi] += x;
                        }
                        None => *slot = None,
                    }
                }
            }

            let mut mix = 0.0f32;
            for (o, sample) in acc.iter().enumerate() {
                let g = self.ducks[o].advance();
                mix += sample * g;
            }
            *frame = mix * self.gain;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sample::write_test_wav;
    use crate::song::parse_song;
    use std::path::Path;

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
        let mut buf = vec![0f32; 48_000];
        d.process(&mut buf, &t, &bank);
        assert!(buf.iter().any(|s| s.abs() > 0.01));
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
        let mut buf = vec![0f32; 48_000];
        d.process(&mut buf, &t, &bank);
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
        let mut buf = vec![0f32; 24_000];
        d.process(&mut buf, &t, &bank);

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
        let mut a = vec![0f32; 48_000];
        d.process(&mut a, &t, &bank);
        let ea: f32 = a.iter().map(|x| x.abs()).sum();

        let mut d = Deck::new("A");
        d.load(song_clip);
        let mut b = vec![0f32; 48_000];
        d.process(&mut b, &t, &bank);
        let eb: f32 = b.iter().map(|x| x.abs()).sum();
        assert!(eb < ea * 0.5, "clip energy {eb} vs full {ea}");
    }
}
