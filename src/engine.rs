//! Engine: dual decks, bar-quantized Command queue, mixer layer.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::deck::Deck;
use crate::mixer::{MixAction, MixCommand, Mixer};
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
    /// DJ mix macro (long / cut / fill). Hold is [`Command::HoldXFade`].
    Mix(MixCommand),
    /// Freeze current xfade gains immediately (no bar wait).
    HoldXFade,
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
    Mix(MixCommand),
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
    scratch_a_l: Vec<f32>,
    scratch_a_r: Vec<f32>,
    scratch_b_l: Vec<f32>,
    scratch_b_r: Vec<f32>,
    scratch_out_l: Vec<f32>,
    scratch_out_r: Vec<f32>,
    /// Lock-free playhead for UI highlight (UI re-evaluates patterns; audio only stores).
    pub playhead: Arc<AtomicU64>,
    /// Once true, further `LoadSong` paths must not overwrite master BPM from `song.bpm`.
    /// Seeded by the first song that carries a BPM, or by an explicit [`Command::SetBpm`].
    bpm_seeded: bool,
}

impl Engine {
    pub fn new(sample_rate: u32, bpm: f64) -> Self {
        Self {
            transport: Transport::new(sample_rate, bpm),
            decks: [Deck::new("A"), Deck::new("B")],
            mixer: Mixer::new(),
            pending: Vec::new(),
            scratch_a_l: Vec::new(),
            scratch_a_r: Vec::new(),
            scratch_b_l: Vec::new(),
            scratch_b_r: Vec::new(),
            scratch_out_l: Vec::new(),
            scratch_out_r: Vec::new(),
            playhead: Arc::new(AtomicU64::new(0)),
            bpm_seeded: false,
        }
    }

    /// Apply `song.bpm` to the shared transport only while master BPM is still unseeded.
    fn maybe_seed_bpm_from_song(&mut self, song: &Song) {
        if self.bpm_seeded {
            return;
        }
        if let Some(bpm) = song.bpm {
            self.transport.set_bpm(bpm);
            self.bpm_seeded = true;
        }
    }

    /// Shared playhead for highlight / viz UI threads.
    pub fn playhead_handle(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.playhead)
    }

    fn next_bar(&self) -> u64 {
        self.transport.bar_index().saturating_add(1)
    }

    /// Next bar that is a multiple of `phrase` (1 = next bar, 4 = next 4-bar boundary).
    fn phrase_target_bar(&self, phrase: u32) -> u64 {
        let n = match phrase {
            8 => 8u64,
            4 => 4,
            _ => 1,
        };
        let next = self.next_bar();
        if n <= 1 {
            return next;
        }
        next.div_ceil(n) * n
    }

    /// Load a song on a deck immediately (no bar wait). For CLI play / startup UX.
    /// Live hot-swap should keep using `push_command(LoadSong { .. })`.
    ///
    /// Song BPM is applied only while master BPM is still unseeded (first song wins).
    pub fn load_song_immediate(&mut self, deck: usize, song: Song) {
        if deck >= 2 {
            return;
        }
        self.maybe_seed_bpm_from_song(&song);
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
                self.mixer.clear_job();
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
            Command::HoldXFade => {
                self.mixer.hold_xfade();
            }
            Command::Mix(spec) => {
                self.mixer.clear_job();
                self.pending.retain(|q| !matches!(q.kind, Pending::Mix(_)));
                let target_bar = self.phrase_target_bar(spec.phrase);
                if let Some(track) = spec.mute_track.clone() {
                    if !track.is_empty() && spec.to_deck < 2 {
                        let from = 1 - spec.to_deck;
                        self.pending.push(Queued {
                            target_bar,
                            kind: Pending::TrackMute {
                                deck: from,
                                track,
                                muted: true,
                            },
                        });
                    }
                }
                self.pending.push(Queued {
                    target_bar,
                    kind: Pending::Mix(spec),
                });
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
                    self.maybe_seed_bpm_from_song(&song);
                    self.decks[deck].load(*song);
                    // Ensure loaded deck is audible if its fader is zero and the other is also silent.
                    if self.mixer.deck_gain(deck) <= 0.0 {
                        let other = 1 - deck;
                        if self.mixer.deck_gain(other) <= 0.0 {
                            self.mixer.set_deck_gain(deck, 1.0);
                        }
                    }
                }
                Pending::SetBpm(b) => {
                    self.transport.set_bpm(b);
                    self.bpm_seeded = true;
                }
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
                Pending::Mix(spec) => self.apply_mix(spec),
            }
        }
    }

    fn apply_mix(&mut self, spec: MixCommand) {
        let to = spec.to_deck;
        if to > 1 {
            return;
        }
        let start = self.transport.global_sample;
        let spb = self.transport.samples_per_bar();
        let sr = self.transport.sample_rate as f32;
        match spec.action {
            MixAction::Long => {
                let bars = spec.bars.clamp(1, 32);
                if spec.eq {
                    self.mixer.apply_long_eq_preset(to);
                }
                let len = (bars as f64 * spb) as u64;
                self.mixer.start_xfade(to, start, len.max(1));
                self.mixer.note_long_mix(to, bars);
            }
            MixAction::Cut => {
                if spec.reset_eq {
                    self.mixer.reset_eq_flat();
                }
                self.mixer.set_crossfader(if to == 1 { 1.0 } else { 0.0 });
                // set_crossfader clears jobs; cut is a snap at bar head (no MixJob).
            }
            MixAction::Fill => {
                let Some(kind) = spec.fill else {
                    return;
                };
                let bars = spec.bars.clamp(1, 32);
                self.mixer.start_fill(
                    kind,
                    to,
                    bars,
                    spec.reset_eq,
                    spec.grid,
                    start,
                    spb,
                    self.transport.bpm,
                    sr,
                );
                let _ = self.mixer.tick_job(start, sr);
            }
        }
    }

    /// Process `frames = out.len() / 2` of **interleaved stereo** into `out` (`[L,R,L,R,…]`).
    /// Time advances by `frames` (not `out.len()`).
    pub fn process(&mut self, out: &mut [f32], samples: &SampleBank) {
        self.apply_pending_at_bar_boundary();

        // XFade only moves gains; both decks keep their songs so the DJ can
        // fade back (or cue the quiet deck) without reloading.
        let _ = self.mixer.tick_xfade(self.transport.global_sample);
        let sr = self.transport.sample_rate as f32;
        let _ = self.mixer.tick_job(self.transport.global_sample, sr);

        let frames = out.len() / 2;
        if frames == 0 {
            return;
        }
        if self.scratch_a_l.len() < frames {
            self.scratch_a_l.resize(frames, 0.0);
            self.scratch_a_r.resize(frames, 0.0);
            self.scratch_b_l.resize(frames, 0.0);
            self.scratch_b_r.resize(frames, 0.0);
            self.scratch_out_l.resize(frames, 0.0);
            self.scratch_out_r.resize(frames, 0.0);
        }
        self.scratch_a_l[..frames].fill(0.0);
        self.scratch_a_r[..frames].fill(0.0);
        self.scratch_b_l[..frames].fill(0.0);
        self.scratch_b_r[..frames].fill(0.0);

        // Deck outputs are pre-fader; mixer applies gain_a/gain_b.
        self.decks[0].gain = 1.0;
        self.decks[1].gain = 1.0;
        self.decks[0].process(
            &mut self.scratch_a_l[..frames],
            &mut self.scratch_a_r[..frames],
            &self.transport,
            samples,
        );
        self.decks[1].process(
            &mut self.scratch_b_l[..frames],
            &mut self.scratch_b_r[..frames],
            &self.transport,
            samples,
        );

        let sr = self.transport.sample_rate as f32;
        if self.mixer.riser_needs_pcm() {
            let pcm = samples
                .get_stem("fx", "up")
                .or_else(|| samples.get_stem("fx", "nr"));
            self.mixer.set_riser_pcm(pcm);
        }
        // Last-write-wins compressor params from either deck's pattern hits.
        if let Some(c) = self.decks[0]
            .pending_compressor
            .or(self.decks[1].pending_compressor)
        {
            self.mixer.set_compressor(Some(c), sr);
        }
        self.mixer.mix(
            &mut self.scratch_out_l[..frames],
            &mut self.scratch_out_r[..frames],
            &self.scratch_a_l[..frames],
            &self.scratch_a_r[..frames],
            &self.scratch_b_l[..frames],
            &self.scratch_b_r[..frames],
            sr,
        );

        for i in 0..frames {
            out[i * 2] = self.scratch_out_l[i];
            out[i * 2 + 1] = self.scratch_out_r[i];
        }

        self.transport.advance(frames);
        // Publish after advance so UI sees the end-of-buffer position.
        self.playhead
            .store(self.transport.global_sample, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mixer::{FillKind, MixAction, MixCommand};
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

    /// Interleaved stereo buffer for `frames` time samples.
    fn stereo_buf(frames: usize) -> Vec<f32> {
        vec![0f32; frames * 2]
    }

    #[test]
    fn song_loads_and_plays_after_bar_boundary() {
        // 120 BPM @ 48k → 1 bar = 96000 frames.
        let mut e = Engine::new(48_000, 120.0);
        let bank = SampleBank::empty();

        e.push_command(Command::LoadSong {
            deck: 0,
            song: Box::new(test_song("c3")),
        });

        // Still in bar 0: pending targets bar 1 → silent.
        let mut buf = stereo_buf(9_600);
        e.process(&mut buf, &bank);
        assert!(
            buf.iter().all(|s| s.abs() < 1e-6),
            "should be silent before next bar"
        );

        // Finish bar 0 (remaining ~86400). Still bar 0 at buffer head → silent.
        let mut buf = stereo_buf(86_400);
        e.process(&mut buf, &bank);
        assert!(
            buf.iter().all(|s| s.abs() < 1e-6),
            "should stay silent until bar 1 starts"
        );

        // Now global_sample == 96000 → bar_index == 1 → apply + sound.
        let mut buf = stereo_buf(48_000);
        e.process(&mut buf, &bank);
        assert!(
            buf.iter().any(|s| s.abs() > 0.001),
            "should sound after bar boundary"
        );
        // Default pan center → both channels have energy.
        let l: f32 = buf.iter().step_by(2).map(|s| s.abs()).sum();
        let r: f32 = buf.iter().skip(1).step_by(2).map(|s| s.abs()).sum();
        assert!(l > 0.01 && r > 0.01, "L={l} R={r}");
    }

    #[test]
    fn bpm_change_waits_for_bar() {
        let mut e = Engine::new(48_000, 120.0);
        let bank = SampleBank::empty();
        e.push_command(Command::SetBpm(60.0));
        let mut buf = stereo_buf(4_800); // mid-bar
        e.process(&mut buf, &bank);
        assert!(
            (e.transport.bpm - 120.0).abs() < 1e-9,
            "bpm must not change mid-bar"
        );
        // Finish the rest of bar 0 (still applied at buffer head = mid-bar → no change).
        let mut buf = stereo_buf(96_000 - 4_800);
        e.process(&mut buf, &bank);
        assert!(
            (e.transport.bpm - 120.0).abs() < 1e-9,
            "bpm still unchanged until a buffer starts on the next bar"
        );
        // Buffer head is now exactly at bar 1 → apply pending.
        let mut buf = stereo_buf(1_000);
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
        let mut buf = stereo_buf(96_000);
        e.process(&mut buf, &bank);
        let mut buf = stereo_buf(48_000);
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
        let mut buf = stereo_buf(4_800);
        e.process(&mut buf, &bank);
        assert!(
            buf.iter().any(|s| s.abs() > 0.0),
            "should still sound mid-bar after mute request"
        );

        // Cross bar boundary → muted (voices may tail; process full bars to drain).
        let mut buf = stereo_buf(96_000);
        e.process(&mut buf, &bank);
        // One more bar: no new notes.
        let mut buf = stereo_buf(96_000);
        e.process(&mut buf, &bank);
        assert!(
            buf.iter().all(|s| s.abs() < 1e-4),
            "should be silent after mute at bar boundary"
        );
    }

    #[test]
    fn head_applies_at_next_bar_boundary() {
        // 120 BPM @ 48k → 1 bar = 96000 frames.
        let mut e = Engine::new(48_000, 120.0);
        let bank = SampleBank::empty();
        let bar = 96_000usize;
        e.load_song_immediate(0, test_song("c3"));
        assert_eq!(e.decks[0].song_bar_1based(0), 1);

        // Advance into bar 1, then mid-bar queue head 10.
        let mut buf = stereo_buf(bar);
        e.process(&mut buf, &bank);
        assert_eq!(e.transport.bar_index(), 1);

        let mut buf = stereo_buf(bar / 2);
        e.process(&mut buf, &bank);
        e.push_command(Command::Head { deck: 0, bar: 10 });
        assert_eq!(e.decks[0].cycle_offset(), 0, "pending until next bar");

        // Cross next bar boundary so Head applies at buffer head.
        let mut buf = stereo_buf(bar);
        e.process(&mut buf, &bank);
        if e.decks[0].cycle_offset() == 0 {
            let mut buf = stereo_buf(bar);
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
        let mut buf = stereo_buf(bar);
        e.process(&mut buf, &bank);
        let g2 = e.transport.bar_index();
        assert_eq!(e.decks[0].song_bar_1based(g2), song + (g2 - g));
        assert_eq!(e.decks[0].cycle_offset(), offset);
    }

    #[test]
    fn xfade_transitions_between_decks() {
        // 120 BPM @ 48k → 1 bar = 96000 frames. Pending applies only at buffer head.
        let mut e = Engine::new(48_000, 120.0);
        let bank = SampleBank::empty();
        let bar = 96_000usize;

        e.push_command(Command::LoadSong {
            deck: 0,
            song: Box::new(test_song("c3")),
        });
        // Reach bar 1 head and apply load A.
        let mut buf = stereo_buf(bar);
        e.process(&mut buf, &bank);
        let mut buf = stereo_buf(bar);
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
        let mut buf = stereo_buf(bar);
        e.process(&mut buf, &bank);
        // If still pending (buffer head was mid-timeline), one more bar.
        if e.mixer.xfade().is_none() && e.decks[1].song_title().is_none() {
            let mut buf = stereo_buf(bar);
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
            let mut buf = stereo_buf(bar);
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
        assert_eq!(
            e.decks[0].song_title(),
            Some("t"),
            "source deck should stay loaded after xfade"
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
        let mut buf = stereo_buf(96_000);
        e.process(&mut buf, &bank); // apply at bar 1
        let mut buf = stereo_buf(48_000);
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
            "songs/house-01.strudel",
        )
        .unwrap();
        let mut e = Engine::new(48_000, 120.0);
        e.push_command(Command::LoadSong {
            deck: 0,
            song: Box::new(song),
        });
        // Skip bar 0, play bar 1.
        let mut buf = stereo_buf(96_000);
        e.process(&mut buf, &bank);
        let mut buf = stereo_buf(48_000);
        e.process(&mut buf, &bank);
        assert!(buf.iter().any(|s| s.abs() > 0.001), "smoke should sound");
    }

    fn peak(buf: &[f32]) -> f32 {
        buf.iter().fold(0f32, |m, s| m.max(s.abs()))
    }

    fn song_with_bpm(n: &str, bpm: f64) -> Song {
        parse_song(
            &format!(
                r#"bpm: {bpm}
title: t
---
b: note("{n}").s("sawtooth").gain(0.8)
"#
            ),
            "t",
        )
        .unwrap()
    }

    /// Process one full bar at the engine's current BPM (bar length follows transport).
    fn process_one_bar(e: &mut Engine, bank: &SampleBank) -> Vec<f32> {
        let frames = e.transport.samples_per_bar().round() as usize;
        let mut buf = stereo_buf(frames.max(1));
        e.process(&mut buf, bank);
        buf
    }

    /// Drain until a pending command queued for `next_bar` has been applied.
    fn process_until_next_bar_applied(e: &mut Engine, bank: &SampleBank) {
        // Advance through the remainder of the current bar, then one buffer on the target head.
        let _ = process_one_bar(e, bank);
        let mut buf = stereo_buf(1_000);
        e.process(&mut buf, bank);
    }

    /// Hot-reload on the same deck must not stack a new bar-head onset on leftover voices.
    #[test]
    fn load_clears_voices_no_double_onset() {
        let mut e = Engine::new(48_000, 120.0);
        let bank = SampleBank::empty();
        let bar = 96_000usize;
        let song = test_song("c3");

        e.load_song_immediate(0, song.clone());
        // Play one full bar to establish steady onset energy.
        let mut buf = stereo_buf(bar);
        e.process(&mut buf, &bank);
        let steady = peak(&buf);
        assert!(
            steady > 0.05,
            "expected sound after first bar, peak={steady}"
        );

        // Reload same content at next bar boundary.
        e.push_command(Command::LoadSong {
            deck: 0,
            song: Box::new(song),
        });
        let mut buf = stereo_buf(bar);
        e.process(&mut buf, &bank);
        let after_reload = peak(&buf);
        assert!(
            after_reload > 0.05,
            "should still sound after reload, peak={after_reload}"
        );
        assert!(
            after_reload <= steady * 1.25 + 1e-3,
            "reload onset must not roughly double: steady={steady} after={after_reload}"
        );
    }

    /// Silent-side load must not boost the audible deck's main mix.
    #[test]
    fn silent_deck_load_does_not_boost_main() {
        let mut e = Engine::new(48_000, 120.0);
        let bank = SampleBank::empty();
        let bar = 96_000usize;
        e.mixer.gain_a = 1.0;
        e.mixer.gain_b = 0.0;

        e.load_song_immediate(0, test_song("c3"));
        let mut buf = stereo_buf(bar);
        e.process(&mut buf, &bank);
        let before = peak(&buf);
        assert!(before > 0.05, "deck A should sound, peak={before}");

        e.push_command(Command::LoadSong {
            deck: 1,
            song: Box::new(test_song("c3")),
        });
        process_until_next_bar_applied(&mut e, &bank);
        let after_buf = process_one_bar(&mut e, &bank);
        let after = peak(&after_buf);
        assert!(
            (after - before).abs() <= before * 0.15 + 1e-3,
            "silent deck load must not change main peak much: before={before} after={after}"
        );
        assert_eq!(e.decks[1].song_title(), Some("t"));
        assert!(e.mixer.gain_b.abs() < 1e-5, "B fader must stay silent");
    }

    #[test]
    fn first_load_applies_song_bpm() {
        let mut e = Engine::new(48_000, 120.0);
        let bank = SampleBank::empty();
        assert!(!e.bpm_seeded);
        e.push_command(Command::LoadSong {
            deck: 0,
            song: Box::new(song_with_bpm("c3", 90.0)),
        });
        // Apply at next bar head.
        let mut buf = stereo_buf(96_000);
        e.process(&mut buf, &bank);
        let mut buf = stereo_buf(1_000);
        e.process(&mut buf, &bank);
        assert!(e.bpm_seeded);
        assert!(
            (e.transport.bpm - 90.0).abs() < 1e-9,
            "first song BPM should seed master, got {}",
            e.transport.bpm
        );
    }

    #[test]
    fn second_load_keeps_master_bpm() {
        let mut e = Engine::new(48_000, 120.0);
        let bank = SampleBank::empty();
        e.load_song_immediate(0, song_with_bpm("c3", 100.0));
        assert!(e.bpm_seeded);
        assert!((e.transport.bpm - 100.0).abs() < 1e-9);

        e.push_command(Command::LoadSong {
            deck: 1,
            song: Box::new(song_with_bpm("g3", 140.0)),
        });
        process_until_next_bar_applied(&mut e, &bank);
        assert!(
            (e.transport.bpm - 100.0).abs() < 1e-9,
            "second load must not overwrite master BPM, got {}",
            e.transport.bpm
        );
        assert_eq!(e.decks[1].song_title(), Some("t"));
    }

    #[test]
    fn set_bpm_still_works_after_seed() {
        let mut e = Engine::new(48_000, 120.0);
        let bank = SampleBank::empty();
        e.load_song_immediate(0, song_with_bpm("c3", 100.0));
        assert!(e.bpm_seeded);

        e.push_command(Command::SetBpm(80.0));
        process_until_next_bar_applied(&mut e, &bank);
        assert!(
            (e.transport.bpm - 80.0).abs() < 1e-9,
            "explicit SetBpm must apply, got {}",
            e.transport.bpm
        );
        assert!(e.bpm_seeded);
    }

    fn mix_fill(kind: FillKind, to: usize) -> MixCommand {
        MixCommand {
            action: MixAction::Fill,
            to_deck: to,
            bars: 1,
            eq: true,
            reset_eq: true,
            fill: Some(kind),
            grid: crate::mixer::MixGrid::Eighth,
            mute_track: None,
            phrase: 1,
        }
    }

    #[test]
    fn mix_long_eq_waits_for_bar() {
        let mut e = Engine::new(48_000, 120.0);
        let bank = SampleBank::empty();
        e.load_song_immediate(0, test_song("c3"));
        e.load_song_immediate(1, test_song("g3"));
        e.mixer.set_crossfader(0.0);
        e.push_command(Command::Mix(MixCommand {
            action: MixAction::Long,
            to_deck: 1,
            bars: 2,
            eq: true,
            reset_eq: true,
            fill: None,
            grid: crate::mixer::MixGrid::Eighth,
            mute_track: None,
            phrase: 1,
        }));
        let mut buf = stereo_buf(4_800);
        e.process(&mut buf, &bank);
        assert!(!e.mixer.lo_kill(0), "eq must not apply mid-bar");
        assert!(e.mixer.xfade().is_none());
        process_until_next_bar_applied(&mut e, &bank);
        assert!(e.mixer.lo_kill(0));
        assert!(!e.mixer.lo_kill(1));
        assert!(e.mixer.deck_eq(1)[1] < 0.1);
        assert!(e.mixer.xfade().is_some() || e.mixer.gain_b > 0.0);
    }

    #[test]
    fn mix_cut_snaps_at_bar() {
        let mut e = Engine::new(48_000, 120.0);
        let bank = SampleBank::empty();
        e.load_song_immediate(0, test_song("c3"));
        e.load_song_immediate(1, test_song("g3"));
        e.mixer.set_crossfader(0.0);
        e.mixer.set_lo_kill(0, true);
        e.push_command(Command::Mix(MixCommand {
            action: MixAction::Cut,
            to_deck: 1,
            bars: 1,
            eq: true,
            reset_eq: true,
            fill: None,
            grid: crate::mixer::MixGrid::Eighth,
            mute_track: None,
            phrase: 1,
        }));
        let mut buf = stereo_buf(4_800);
        e.process(&mut buf, &bank);
        assert!(e.mixer.gain_a > 0.9);
        process_until_next_bar_applied(&mut e, &bank);
        assert!(e.mixer.gain_a.abs() < 1e-3);
        assert!((e.mixer.gain_b - 1.0).abs() < 1e-3);
        assert!(!e.mixer.lo_kill(0));
        assert!((e.mixer.deck_eq(0)[1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn mix_hold_is_immediate() {
        let mut e = Engine::new(48_000, 120.0);
        e.mixer.start_xfade(1, 0, 100_000);
        let _ = e.mixer.tick_xfade(50_000);
        assert!(e.mixer.xfade().is_some());
        e.push_command(Command::HoldXFade);
        assert!(e.mixer.xfade().is_none());
        assert!(e.mixer.gain_a > 0.1 && e.mixer.gain_b > 0.1);
    }

    #[test]
    fn mix_switch_then_lands_on_to() {
        let mut e = Engine::new(48_000, 120.0);
        let bank = SampleBank::empty();
        e.load_song_immediate(0, test_song("c3"));
        e.load_song_immediate(1, test_song("g3"));
        e.mixer.set_crossfader(1.0);
        e.push_command(Command::Mix(mix_fill(FillKind::Switch, 0)));
        process_until_next_bar_applied(&mut e, &bank);
        assert!(e.mixer.has_mix_job());
        // First cell is opposite of A → B full.
        assert!(e.mixer.gain_b > e.mixer.gain_a);
        // Job length is 1 bar; apply_pending already ate 1000 frames of it.
        let mut buf = stereo_buf(96_000);
        e.process(&mut buf, &bank);
        if e.mixer.has_mix_job() {
            let mut buf = stereo_buf(96_000);
            e.process(&mut buf, &bank);
        }
        assert!(!e.mixer.has_mix_job());
        assert!(e.mixer.gain_a > 0.9);
        assert!(e.mixer.gain_b.abs() < 0.1);
    }

    #[test]
    fn hush_clears_mix_job() {
        let mut e = Engine::new(48_000, 120.0);
        let bank = SampleBank::empty();
        e.push_command(Command::Mix(mix_fill(FillKind::Delay, 1)));
        process_until_next_bar_applied(&mut e, &bank);
        assert!(e.mixer.has_mix_job());
        e.push_command(Command::Hush);
        assert!(!e.mixer.has_mix_job());
        assert!(e.mixer.xfade().is_none());
    }
}
