//! Deck: schedule song tracks for one bar and mix a voice pool.

use crate::code::{note_to_hz, Adsr, PatternCode};
use crate::mini;
use crate::sample::{SampleBank, SampleVoice, VoiceKind, SAMPLE_ROOT_HZ};
use crate::song::Song;
use crate::sound::{resolve_sound, ResolvedSound};
use crate::synth::{OscSource, Voice};
use crate::transport::Transport;

pub const MAX_VOICES: usize = 32;

struct ScheduledHit {
    at_sample: u64,
    len_samples: u64,
    sound: String,
    freq: f32,
    gain: f32,
    lpf: Option<f32>,
    adsr: Adsr,
    is_note: bool,
    begin: f32,
    end: f32,
    sample_speed: f32,
    sample_n: Option<i32>,
}

pub struct Deck {
    pub name: String,
    song: Option<Song>,
    pub gain: f32,
    voices: Vec<Option<VoiceKind>>,
    /// Round-robin steal index when pool is full.
    steal_cursor: usize,
    scheduled: Vec<ScheduledHit>,
    next_event: usize,
    scheduled_bar: u64,
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
    }

    pub fn song_title(&self) -> Option<&str> {
        self.song.as_ref().map(|s| s.title.as_str())
    }

    pub fn song_mut(&mut self) -> Option<&mut Song> {
        self.song.as_mut()
    }

    /// Mute/unmute a track and force reschedule on next process.
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
        // Clone track codes to avoid borrow conflict with `self.scheduled`.
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
    for ev in mini::events(&pc.pattern, bar) {
        if ev.value == "~" || ev.value.is_empty() {
            continue;
        }
        let at = bar_start + ((ev.start * spb) / speed) as u64;
        let len = (((ev.dur * spb) / speed) as u64).max(64);
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
            lpf: pc.lpf,
            adsr: pc.adsr,
            is_note,
            begin: pc.begin,
            end: pc.end,
            sample_speed: pc.sample_speed,
            sample_n: pc.sample_n,
        });
    }
}

impl Deck {
    fn alloc_voice(&mut self, voice: VoiceKind) {
        if let Some(slot) = self.voices.iter_mut().find(|v| v.is_none()) {
            *slot = Some(voice);
            return;
        }
        // steal oldest by round-robin
        let i = self.steal_cursor % MAX_VOICES;
        self.voices[i] = Some(voice);
        self.steal_cursor = self.steal_cursor.wrapping_add(1);
    }

    fn spawn_hit(&mut self, hit: &ScheduledHit, samples: &SampleBank, sr: f32) {
        let Ok(resolved) = resolve_sound(&hit.sound, samples) else {
            return;
        };
        match resolved {
            ResolvedSound::Wave(w) => {
                let v = Voice::new(
                    OscSource::Wave(w),
                    hit.freq.max(1.0),
                    hit.gain,
                    hit.len_samples,
                    hit.lpf,
                    hit.adsr,
                )
                .with_adsr_timing(sr, hit.len_samples);
                self.alloc_voice(VoiceKind::Synth(v));
            }
            ResolvedSound::Noise(n) => {
                let v = Voice::new(
                    OscSource::Noise(n),
                    hit.freq.max(1.0),
                    hit.gain,
                    hit.len_samples,
                    hit.lpf,
                    hit.adsr,
                )
                .with_adsr_timing(sr, hit.len_samples);
                self.alloc_voice(VoiceKind::Synth(v));
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
                let v = SampleVoice::new(
                    data,
                    hit.gain,
                    hit.begin,
                    hit.end,
                    hit.sample_speed,
                    pitch_ratio,
                );
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
                // clone fields needed; avoid holding borrow across spawn
                let hit = ScheduledHit {
                    at_sample: self.scheduled[self.next_event].at_sample,
                    len_samples: self.scheduled[self.next_event].len_samples,
                    sound: self.scheduled[self.next_event].sound.clone(),
                    freq: self.scheduled[self.next_event].freq,
                    gain: self.scheduled[self.next_event].gain,
                    lpf: self.scheduled[self.next_event].lpf,
                    adsr: self.scheduled[self.next_event].adsr,
                    is_note: self.scheduled[self.next_event].is_note,
                    begin: self.scheduled[self.next_event].begin,
                    end: self.scheduled[self.next_event].end,
                    sample_speed: self.scheduled[self.next_event].sample_speed,
                    sample_n: self.scheduled[self.next_event].sample_n,
                };
                self.next_event += 1;
                self.spawn_hit(&hit, samples, sr);
            }

            let mut mix = 0f32;
            for slot in self.voices.iter_mut() {
                if let Some(voice) = slot {
                    match voice.next_sample(sr) {
                        Some(x) => mix += x,
                        None => *slot = None,
                    }
                }
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
}
