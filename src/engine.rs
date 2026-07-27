//! Engine: dual decks, bar-quantized Command queue, mix to mono.

use crate::deck::Deck;
use crate::sample::SampleBank;
use crate::song::Song;
use crate::transport::Transport;

pub enum Command {
    LoadSong {
        deck: usize,
        song: Song,
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
}

enum Pending {
    LoadSong {
        deck: usize,
        song: Song,
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
}

struct Queued {
    /// Apply when `transport.bar_index() >= target_bar`.
    target_bar: u64,
    kind: Pending,
}

struct XFadeState {
    to_deck: usize,
    start_sample: u64,
    end_sample: u64,
}

pub struct Engine {
    pub transport: Transport,
    pub decks: [Deck; 2],
    pending: Vec<Queued>,
    xfade: Option<XFadeState>,
    scratch_a: Vec<f32>,
    scratch_b: Vec<f32>,
}

impl Engine {
    pub fn new(sample_rate: u32, bpm: f64) -> Self {
        Self {
            transport: Transport::new(sample_rate, bpm),
            decks: [Deck::new("A"), Deck::new("B")],
            pending: Vec::new(),
            xfade: None,
            scratch_a: Vec::new(),
            scratch_b: Vec::new(),
        }
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
        self.decks[deck].gain = 1.0;
    }

    pub fn push_command(&mut self, cmd: Command) {
        match cmd {
            Command::Hush => {
                self.decks[0].unload();
                self.decks[1].unload();
                self.xfade = None;
                self.pending.clear();
            }
            Command::UnloadDeck { deck } => {
                if deck < 2 {
                    self.decks[deck].unload();
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
                    self.decks[deck].load(song);
                }
                Pending::SetBpm(b) => self.transport.set_bpm(b),
                Pending::TrackMute { deck, track, muted } => {
                    self.decks[deck].set_track_mute(&track, muted);
                }
                Pending::XFade { to_deck, bars } => {
                    let start = self.transport.global_sample;
                    let len = (bars as f64 * self.transport.samples_per_bar()) as u64;
                    self.xfade = Some(XFadeState {
                        to_deck,
                        start_sample: start,
                        end_sample: start.saturating_add(len.max(1)),
                    });
                }
            }
        }
    }

    pub fn process(&mut self, out: &mut [f32], samples: &SampleBank) {
        self.apply_pending_at_bar_boundary();

        if let Some(x) = &self.xfade {
            let denom = (x.end_sample - x.start_sample).max(1) as f64;
            let t = ((self.transport.global_sample.saturating_sub(x.start_sample)) as f64 / denom)
                .clamp(0.0, 1.0);
            let theta = t * std::f64::consts::FRAC_PI_2;
            let (from, to) = if x.to_deck == 1 { (0, 1) } else { (1, 0) };
            self.decks[from].gain = theta.cos() as f32;
            self.decks[to].gain = theta.sin() as f32;
            if t >= 1.0 {
                self.decks[from].unload();
                self.decks[from].gain = 0.0;
                self.decks[to].gain = 1.0;
                self.xfade = None;
            }
        }

        let n = out.len();
        if self.scratch_a.len() < n {
            self.scratch_a.resize(n, 0.0);
            self.scratch_b.resize(n, 0.0);
        }
        self.scratch_a[..n].fill(0.0);
        self.scratch_b[..n].fill(0.0);
        self.decks[0].process(&mut self.scratch_a[..n], &self.transport, samples);
        self.decks[1].process(&mut self.scratch_b[..n], &self.transport, samples);

        for (o, (a, b)) in out
            .iter_mut()
            .zip(self.scratch_a.iter().zip(self.scratch_b.iter()))
            .take(n)
        {
            *o = (a + b).clamp(-1.0, 1.0);
        }
        self.transport.advance(n);
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
            song: test_song("c3"),
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
        e.push_command(Command::LoadSong { deck: 0, song });
        // Skip bar 0, play bar 1.
        let mut buf = vec![0f32; 96_000];
        e.process(&mut buf, &bank);
        let mut buf = vec![0f32; 48_000];
        e.process(&mut buf, &bank);
        assert!(buf.iter().any(|s| s.abs() > 0.001), "smoke should sound");
    }
}
