//! Engine: dual decks, bar-quantized Command queue, mixer layer.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::deck::Deck;
use crate::mixer::{Mixer, XFadeTick};
use crate::sample::SampleBank;
use crate::song::Song;
use crate::transport::Transport;

pub enum Command {
    LoadSong {
        deck: usize,
        /// Boxed: `Song` is large (source + tracks); keep the enum small for clippy.
        song: Box<Song>,
    },
    UnloadDeck {
        deck: usize,
    },
    SetBpm(f64),
    Hush,
    XFade {
        to_deck: usize,
        bars: u32,
    },
    SetTrackMute {
        deck: usize,
        track: String,
        muted: bool,
    },
    /// Mixer fader (immediate; not bar-quantized).
    SetDeckGain {
        deck: usize,
        gain: f32,
    },
    /// Equal-power crossfader 0=A … 1=B (immediate; cancels multi-bar xfade).
    SetCrossfader(f32),
    /// Jump a deck's pattern to 1-based bar `bar` at the next bar boundary (shared transport stays put).
    Head {
        deck: usize,
        /// Human bar number (1 = first bar of the song).
        bar: u64,
    },
    /// Master LPF cutoff Hz; `None` via negative/NaN not used — use `SetMixerLpf(None)`.
    SetMixerLpf(Option<f32>),
    SetMixerHpf(Option<f32>),
    /// Per-deck channel EQ band (immediate). `band`: 0=Hi, 1=Mid, 2=Lo; `value`: 0..=1 (0.5=flat).
    SetDeckEq {
        deck: usize,
        band: u8,
        value: f32,
    },
}

enum Pending {
    LoadSong {
        deck: usize,
        song: Box<Song>,
    },
    SetBpm(f64),
    XFade {
        to_deck: usize,
        bars: u32,
    },
    TrackMute {
        deck: usize,
        track: String,
        muted: bool,
    },
    Head {
        deck: usize,
        bar: u64,
    },
}

struct Queued {
    /// Apply when `transport.bar_index() >= target_bar`.
    target_bar: u64,
    kind: Pending,
}

pub struct Engine {
    pub transport: Transport,
    pub decks: [Deck; 2],
    pub mixer: Mixer,
    pending: Vec<Queued>,
    scratch_a: Vec<f32>,
    scratch_b: Vec<f32>,
    /// Lock-free playhead for UI highlight (UI re-evaluates patterns; audio only stores).
    pub playhead: Arc<AtomicU64>,
}

impl Engine {
    pub fn new(sample_rate: u32, bpm: f64) -> Self {
        Self {
            transport: Transport::new(sample_rate, bpm),
            decks: [Deck::new("A"), Deck::new("B")],
            mixer: Mixer::new(),
            pending: Vec::new(),
            scratch_a: Vec::new(),
            scratch_b: Vec::new(),
            playhead: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Shared playhead for highlight / viz UI threads.
    pub fn playhead_handle(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.playhead)
    }

    fn next_bar(&self) -> u64 {
        self.transport.bar_index().saturating_add(1)
    }

    /// Load a song on a deck immediately (no bar wait). For CLI play / startup UX.
    /// Live hot-swap should keep using `push_command(LoadSong { .. })`.
    pub fn load_song_immediate(&mut self, deck: usize, song: Song) {
        if deck >= 2 {
            return;
        }
        if let Some(bpm) = song.bpm {
            self.transport.set_bpm(bpm);
        }
        self.decks[deck].load(song);
        // Audition / startup: ensure the loaded deck is audible.
        if deck == 0 {
            self.mixer.gain_a = 1.0;
        } else if deck == 1 && self.mixer.gain_b <= 0.0 && self.mixer.gain_a <= 0.0 {
            self.mixer.gain_b = 1.0;
        }
    }

    pub fn push_command(&mut self, cmd: Command) {
        match cmd {
            Command::Hush => {
                self.decks[0].unload();
                self.decks[1].unload();
                self.mixer.clear_xfade();
                self.mixer.gain_a = 0.0;
                self.mixer.gain_b = 0.0;
                self.pending.clear();
            }
            Command::UnloadDeck { deck } => {
                if deck < 2 {
                    self.decks[deck].unload();
                    self.mixer.set_deck_gain(deck, 0.0);
                }
            }
            Command::LoadSong { deck, song } => {
                if deck < 2 {
                    self.pending.push(Queued {
                        target_bar: self.next_bar(),
                        kind: Pending::LoadSong { deck, song },
                    });
                }
            }
            Command::SetBpm(b) => {
                self.pending.push(Queued {
                    target_bar: self.next_bar(),
                    kind: Pending::SetBpm(b),
                });
            }
            Command::XFade { to_deck, bars } => {
                if to_deck < 2 {
                    self.pending.push(Queued {
                        target_bar: self.next_bar(),
                        kind: Pending::XFade { to_deck, bars },
                    });
                }
            }
            Command::SetTrackMute { deck, track, muted } => {
                if deck < 2 {
                    self.pending.push(Queued {
                        target_bar: self.next_bar(),
                        kind: Pending::TrackMute { deck, track, muted },
                    });
                }
            }
            Command::Head { deck, bar } => {
                if deck < 2 && bar >= 1 {
                    self.pending.push(Queued {
                        target_bar: self.next_bar(),
                        kind: Pending::Head { deck, bar },
                    });
                }
            }
            Command::SetDeckGain { deck, gain } => {
                if deck < 2 {
                    self.mixer.set_deck_gain(deck, gain);
                }
            }
            Command::SetCrossfader(pos) => {
                self.mixer.set_crossfader(pos);
            }
            Command::SetMixerLpf(hz) => {
                self.mixer.lpf_hz = hz;
            }
            Command::SetMixerHpf(hz) => {
                self.mixer.hpf_hz = hz;
            }
            Command::SetDeckEq { deck, band, value } => {
                if deck < 2 && (band as usize) < 3 {
                    self.mixer.set_deck_eq(deck, band as usize, value);
                }
            }
        }
    }

    /// Apply pending whose target_bar has been reached (buffer head only).
    fn apply_pending_at_bar_boundary(&mut self) {
        let bar_now = self.transport.bar_index();
        if self.pending.is_empty() {
            return;
        }
        let mut rest = Vec::new();
        let ready: Vec<Pending> = self
            .pending
            .drain(..)
            .filter_map(|q| {
                if q.target_bar <= bar_now {
                    Some(q.kind)
                } else {
                    rest.push(q);
                    None
                }
            })
            .collect();
        self.pending = rest;

        for p in ready {
            match p {
                Pending::LoadSong { deck, song } => {
                    if let Some(bpm) = song.bpm {
                        self.transport.set_bpm(bpm);
                    }
                    self.decks[deck].load(*song);
                    // Ensure loaded deck is audible if its fader is zero and the other is also silent.
                    if self.mixer.deck_gain(deck) <= 0.0 {
                        let other = 1 - deck;
                        if self.mixer.deck_gain(other) <= 0.0 {
                            self.mixer.set_deck_gain(deck, 1.0);
                        }
                    }
                }
                Pending::SetBpm(b) => self.transport.set_bpm(b),
                Pending::TrackMute { deck, track, muted } => {
                    self.decks[deck].set_track_mute(&track, muted);
                }
                Pending::XFade { to_deck, bars } => {
                    let start = self.transport.global_sample;
                    let len = (bars as f64 * self.transport.samples_per_bar()) as u64;
                    self.mixer.start_xfade(to_deck, start, len.max(1));
                }
                Pending::Head { deck, bar } => {
                    // Apply at this global bar head so pattern_bar(apply) == bar-1.
                    self.decks[deck].head_to_bar(bar, bar_now);
                }
            }
        }
    }

    pub fn process(&mut self, out: &mut [f32], samples: &SampleBank) {
        self.apply_pending_at_bar_boundary();

        match self.mixer.tick_xfade(self.transport.global_sample) {
            XFadeTick::Finished { from_deck, .. } => {
                self.decks[from_deck].unload();
            }
            XFadeTick::Idle | XFadeTick::Active => {}
        }

        let n = out.len();
        if self.scratch_a.len() < n {
            self.scratch_a.resize(n, 0.0);
            self.scratch_b.resize(n, 0.0);
        }
        self.scratch_a[..n].fill(0.0);
        self.scratch_b[..n].fill(0.0);

        // Deck outputs are pre-fader; mixer applies gain_a/gain_b.
        self.decks[0].gain = 1.0;
        self.decks[1].gain = 1.0;
        self.decks[0].process(&mut self.scratch_a[..n], &self.transport, samples);
        self.decks[1].process(&mut self.scratch_b[..n], &self.transport, samples);

        let sr = self.transport.sample_rate as f32;
        // Last-write-wins compressor params from either deck's pattern hits.
        if let Some(c) = self.decks[0]
            .pending_compressor
            .or(self.decks[1].pending_compressor)
        {
            self.mixer.set_compressor(Some(c), sr);
        }
        self.mixer
            .mix(out, &self.scratch_a[..n], &self.scratch_b[..n], sr);

        self.transport.advance(n);
        // Publish after advance so UI sees the end-of-buffer position.
        self.playhead
            .store(self.transport.global_sample, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::song::parse_song;

    fn test_song(n: &str) -> Song {
        parse_song(
            &format!(
                r#"---
b: note("{n}").s("sawtooth").gain(0.8)
"#
            ),
            "t",
        )
        .unwrap()
    }

    #[test]
    fn song_loads_and_plays_after_bar_boundary() {
        // 120 BPM @ 48k → 1 bar = 96000 samples.
        let mut e = Engine::new(48_000, 120.0);
        let bank = SampleBank::empty();

        e.push_command(Command::LoadSong {
            deck: 0,
            song: Box::new(test_song("c3")),
        });

        // Still in bar 0: pending targets bar 1 → silent.
        let mut buf = vec![0f32; 9_600];
        e.process(&mut buf, &bank);
        assert!(
            buf.iter().all(|s| s.abs() < 1e-6),
            "should be silent before next bar"
        );

        // Finish bar 0 (remaining ~86400). Still bar 0 at buffer head → silent.
        let mut buf = vec![0f32; 86_400];
        e.process(&mut buf, &bank);
        assert!(
            buf.iter().all(|s| s.abs() < 1e-6),
            "should stay silent until bar 1 starts"
        );

        // Now global_sample == 96000 → bar_index == 1 → apply + sound.
        let mut buf = vec![0f32; 48_000];
        e.process(&mut buf, &bank);
        assert!(
            buf.iter().any(|s| s.abs() > 0.001),
            "should sound after bar boundary"
        );
    }

    #[test]
    fn bpm_change_waits_for_bar() {
        let mut e = Engine::new(48_000, 120.0);
        let bank = SampleBank::empty();
        e.push_command(Command::SetBpm(60.0));
        let mut buf = vec![0f32; 4_800]; // mid-bar
        e.process(&mut buf, &bank);
        assert!(
            (e.transport.bpm - 120.0).abs() < 1e-9,
            "bpm must not change mid-bar"
        );
        // Finish the rest of bar 0 (still applied at buffer head = mid-bar → no change).
        let mut buf = vec![0f32; 96_000 - 4_800];
        e.process(&mut buf, &bank);
        assert!(
            (e.transport.bpm - 120.0).abs() < 1e-9,
            "bpm still unchanged until a buffer starts on the next bar"
        );
        // Buffer head is now exactly at bar 1 → apply pending.
        let mut buf = vec![0f32; 1_000];
        e.process(&mut buf, &bank);
        assert!(
            (e.transport.bpm - 60.0).abs() < 1e-9,
            "bpm must change at bar boundary, got {}",
            e.transport.bpm
        );
    }

    #[test]
    fn track_mute_waits_for_bar() {
        let mut e = Engine::new(48_000, 120.0);
        let bank = SampleBank::empty();
        let song = parse_song("---\nb: note(\"c3\").s(\"sawtooth\").gain(0.8)", "t").unwrap();
        e.push_command(Command::LoadSong {
            deck: 0,
            song: Box::new(song),
        });
        // Apply load at bar 1.
        let mut buf = vec![0f32; 96_000];
        e.process(&mut buf, &bank);
        let mut buf = vec![0f32; 48_000];
        e.process(&mut buf, &bank);
        assert!(
            buf.iter().any(|s| s.abs() > 0.001),
            "should sound before mute"
        );

        e.push_command(Command::SetTrackMute {
            deck: 0,
            track: "b".into(),
            muted: true,
        });
        // Still same bar: mute pending for next bar → still sounding.
        let mut buf = vec![0f32; 4_800];
        e.process(&mut buf, &bank);
        assert!(
            buf.iter().any(|s| s.abs() > 0.0),
            "should still sound mid-bar after mute request"
        );

        // Cross bar boundary → muted (voices may tail; process full bars to drain).
        let mut buf = vec![0f32; 96_000];
        e.process(&mut buf, &bank);
        // One more bar: no new notes.
        let mut buf = vec![0f32; 96_000];
        e.process(&mut buf, &bank);
        assert!(
            buf.iter().all(|s| s.abs() < 1e-4),
            "should be silent after mute at bar boundary"
        );
    }

    #[test]
    fn head_applies_at_next_bar_boundary() {
        // 120 BPM @ 48k → 1 bar = 96000 samples.
        let mut e = Engine::new(48_000, 120.0);
        let bank = SampleBank::empty();
        let bar = 96_000usize;
        e.load_song_immediate(0, test_song("c3"));
        assert_eq!(e.decks[0].song_bar_1based(0), 1);

        // Advance into bar 1, then mid-bar queue head 10.
        let mut buf = vec![0f32; bar];
        e.process(&mut buf, &bank);
        assert_eq!(e.transport.bar_index(), 1);

        let mut buf = vec![0f32; bar / 2];
        e.process(&mut buf, &bank);
        e.push_command(Command::Head { deck: 0, bar: 10 });
        assert_eq!(e.decks[0].cycle_offset(), 0, "pending until next bar");

        // Cross next bar boundary so Head applies at buffer head.
        let mut buf = vec![0f32; bar];
        e.process(&mut buf, &bank);
        if e.decks[0].cycle_offset() == 0 {
            let mut buf = vec![0f32; bar];
            e.process(&mut buf, &bank);
        }

        let offset = e.decks[0].cycle_offset();
        assert_ne!(offset, 0, "head should set cycle_offset");
        let g = e.transport.bar_index();
        // offset = (10-1) - G_apply  ⇒  song_bar(g) = g + offset + 1
        assert_eq!(
            e.decks[0].song_bar_1based(g),
            (g as i64 + offset + 1) as u64
        );
        assert!(e.decks[0].song_bar_1based(g) >= 10);

        let song = e.decks[0].song_bar_1based(g);
        let mut buf = vec![0f32; bar];
        e.process(&mut buf, &bank);
        let g2 = e.transport.bar_index();
        assert_eq!(e.decks[0].song_bar_1based(g2), song + (g2 - g));
        assert_eq!(e.decks[0].cycle_offset(), offset);
    }

    #[test]
    fn xfade_transitions_between_decks() {
        // 120 BPM @ 48k → 1 bar = 96000 samples. Pending applies only at buffer head.
        let mut e = Engine::new(48_000, 120.0);
        let bank = SampleBank::empty();
        let bar = 96_000usize;

        e.push_command(Command::LoadSong {
            deck: 0,
            song: Box::new(test_song("c3")),
        });
        // Reach bar 1 head and apply load A.
        let mut buf = vec![0f32; bar];
        e.process(&mut buf, &bank);
        let mut buf = vec![0f32; bar];
        e.process(&mut buf, &bank);
        assert_eq!(e.decks[0].song_title(), Some("t"));
        assert!((e.mixer.gain_a - 1.0).abs() < 1e-5);

        e.push_command(Command::LoadSong {
            deck: 1,
            song: Box::new(test_song("g3")),
        });
        e.push_command(Command::XFade {
            to_deck: 1,
            bars: 2,
        });

        // Process until pending targets are reached (next bar head).
        let mut buf = vec![0f32; bar];
        e.process(&mut buf, &bank);
        // If still pending (buffer head was mid-timeline), one more bar.
        if e.mixer.xfade().is_none() && e.decks[1].song_title().is_none() {
            let mut buf = vec![0f32; bar];
            e.process(&mut buf, &bank);
        }
        assert_eq!(
            e.decks[1].song_title(),
            Some("t"),
            "deck B should be loaded"
        );
        assert!(
            e.mixer.xfade().is_some() || e.mixer.gain_b > 0.0,
            "xfade should have started or completed, a={} b={}",
            e.mixer.gain_a,
            e.mixer.gain_b
        );

        // Run enough bars for a 2-bar xfade to finish.
        for _ in 0..4 {
            let mut buf = vec![0f32; bar];
            e.process(&mut buf, &bank);
        }

        assert!(
            e.mixer.gain_a.abs() < 1e-3,
            "from gain should be 0, got {}",
            e.mixer.gain_a
        );
        assert!(
            (e.mixer.gain_b - 1.0).abs() < 1e-3,
            "to gain should be 1, got {}",
            e.mixer.gain_b
        );
        assert!(
            e.decks[0].song_title().is_none(),
            "old deck unloaded after xfade"
        );
        assert_eq!(e.decks[1].song_title(), Some("t"));
    }

    #[test]
    fn decks_share_transport() {
        let mut e = Engine::new(48_000, 120.0);
        let bank = SampleBank::empty();
        e.mixer.gain_a = 1.0;
        e.mixer.gain_b = 1.0;
        e.push_command(Command::LoadSong {
            deck: 0,
            song: Box::new(test_song("c3")),
        });
        e.push_command(Command::LoadSong {
            deck: 1,
            song: Box::new(test_song("c3")),
        });
        let mut buf = vec![0f32; 96_000];
        e.process(&mut buf, &bank); // apply at bar 1
        let mut buf = vec![0f32; 48_000];
        e.process(&mut buf, &bank);
        let peak = buf.iter().fold(0f32, |m, s| m.max(s.abs()));
        assert!(
            peak > 0.3,
            "expected constructive interference, peak={peak}"
        );
    }

    #[test]
    fn sample_kit_smoke() {
        let path = std::path::Path::new("samples");
        if !path.exists() {
            return;
        }
        let bank = SampleBank::load_dir(path, 48_000);
        let song = parse_song(
            r#"
bpm: 120
title: smoke
---
kick: s("bd*4").gain(0.9)
hat: s("hh*8").gain(0.3)
bass: note("c2").s("sawtooth").lpf(400).gain(0.5)
"#,
            "songs/smoke.strudel",
        )
        .unwrap();
        let mut e = Engine::new(48_000, 120.0);
        e.push_command(Command::LoadSong {
            deck: 0,
            song: Box::new(song),
        });
        // Skip bar 0, play bar 1.
        let mut buf = vec![0f32; 96_000];
        e.process(&mut buf, &bank);
        let mut buf = vec![0f32; 48_000];
        e.process(&mut buf, &bank);
        assert!(buf.iter().any(|s| s.abs() > 0.001), "smoke should sound");
    }
}
