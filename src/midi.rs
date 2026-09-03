//! MIDI output for `strudel-rs play` (notes now; CC / Bank+PC later).
//!
//! Audio thread only `try_send`s. A dedicated thread talks to `midir`.
//! Linux BLE MIDI is an ALSA sequencer port the OS already exposed — this
//! crate does not speak GATT / D-Bus.

use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam::channel::{bounded, Receiver, Sender, TrySendError};
use midir::{MidiOutput, MidiOutputConnection};

use crate::synth::{NoiseKind, Wave};

/// SEQTRAK drum hits use this MIDI note (module mode; not GM kit mapping).
pub const DRUM_NOTE: u8 = 60;

const QUEUE_CAP: usize = 1024;

/// Events the MIDI thread can send. CC / Program Change are added later.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MidiEvent {
    NoteOn { ch: u8, note: u8, vel: u8 },
    NoteOff { ch: u8, note: u8 },
}

impl MidiEvent {
    /// Channel voice bytes (status + data). `ch` is 0..=15.
    pub fn to_bytes(self) -> [u8; 3] {
        match self {
            MidiEvent::NoteOn { ch, note, vel } => [0x90 | (ch & 0x0F), note, vel],
            MidiEvent::NoteOff { ch, note } => [0x80 | (ch & 0x0F), note, 0],
        }
    }
}

impl std::fmt::Display for MidiEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            MidiEvent::NoteOn { ch, note, vel } => {
                write!(f, "NoteOn ch={} note={note} vel={vel}", ch + 1)
            }
            MidiEvent::NoteOff { ch, note } => {
                write!(f, "NoteOff ch={} note={note}", ch + 1)
            }
        }
    }
}

/// Test sink: records events in order.
#[derive(Clone, Default)]
pub struct NullMidi {
    pub events: Arc<Mutex<Vec<MidiEvent>>>,
}

impl NullMidi {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> Vec<MidiEvent> {
        self.events.lock().map(|g| g.clone()).unwrap_or_default()
    }
}

/// Something that can emit MIDI bytes (hardware or the test log).
pub trait MidiSink: Send {
    fn emit(&mut self, ev: MidiEvent);
}

impl MidiSink for NullMidi {
    fn emit(&mut self, ev: MidiEvent) {
        if let Ok(mut g) = self.events.lock() {
            g.push(ev);
        }
    }
}

struct MidirSink {
    conn: MidiOutputConnection,
    sent: u64,
    errors: u64,
}

impl MidiSink for MidirSink {
    fn emit(&mut self, ev: MidiEvent) {
        match self.conn.send(&ev.to_bytes()) {
            Ok(()) => {
                self.sent = self.sent.saturating_add(1);
                if self.sent == 1 {
                    eprintln!("midi: first {ev}");
                }
            }
            Err(e) => {
                self.errors = self.errors.saturating_add(1);
                if self.errors == 1 {
                    eprintln!("warning: midi send failed: {e}");
                }
            }
        }
    }
}

impl Drop for MidirSink {
    fn drop(&mut self) {
        eprintln!(
            "midi: sent {} message(s) ({} error(s))",
            self.sent, self.errors
        );
    }
}

/// Live handle: clone the sender onto the engine; keep this so the thread stays up.
pub struct MidiHandle {
    tx: Option<Sender<MidiEvent>>,
    join: Option<JoinHandle<()>>,
}

impl std::fmt::Debug for MidiHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MidiHandle").finish_non_exhaustive()
    }
}

impl MidiHandle {
    pub fn sender(&self) -> Sender<MidiEvent> {
        self.tx.as_ref().expect("midi handle sender").clone()
    }
}

impl Drop for MidiHandle {
    fn drop(&mut self) {
        self.tx.take();
        let Some(j) = self.join.take() else {
            return;
        };
        // WinMM `midiOutShortMsg` can spin on NOTREADY; API may still hold a
        // Sender clone. Do not block process exit on that join.
        let (done_tx, done_rx) = bounded::<()>(1);
        let spawn = thread::Builder::new()
            .name("strudel-midi-join".into())
            .spawn(move || {
                let _ = j.join();
                let _ = done_tx.send(());
            });
        if spawn.is_err() {
            return;
        }
        if done_rx.recv_timeout(Duration::from_millis(400)).is_err() {
            eprintln!("warning: midi thread did not stop (driver may be stuck); continuing");
        }
    }
}

/// Bounded SPSC from the audio thread. Full queue drops the event (no wait).
pub fn try_push(tx: &Sender<MidiEvent>, ev: MidiEvent) {
    match tx.try_send(ev) {
        Ok(()) | Err(TrySendError::Disconnected(_)) => {}
        Err(TrySendError::Full(_)) => {}
    }
}

/// Spawn a worker that drains `rx` into `sink` until the sender is dropped.
pub fn spawn_sink(sink: Box<dyn MidiSink>) -> MidiHandle {
    let (tx, rx) = bounded(QUEUE_CAP);
    let join = thread::Builder::new()
        .name("strudel-midi".into())
        .spawn(move || midi_worker(rx, sink))
        .ok();
    MidiHandle { tx: Some(tx), join }
}

fn midi_worker(rx: Receiver<MidiEvent>, mut sink: Box<dyn MidiSink>) {
    while let Ok(ev) = rx.recv() {
        sink.emit(ev);
    }
}

/// Names of current MIDI output ports (index matches `--midi-port`).
pub fn list_output_ports() -> Result<Vec<String>, String> {
    let out = MidiOutput::new("strudel-rs").map_err(|e| format!("midi: {e}"))?;
    let mut names = Vec::new();
    for p in out.ports() {
        names.push(out.port_name(&p).unwrap_or_else(|_| "(unnamed)".into()));
    }
    Ok(names)
}

/// Open a port and start the send thread.
///
/// `spec` is a 0-based index or a case-insensitive substring of the port name.
/// `None` prefers a name containing `SEQTRAK`, else the first port.
///
/// The WinMM / ALSA handle is opened, used, and closed on the worker thread.
pub fn connect(spec: Option<&str>) -> Result<MidiHandle, String> {
    let spec = spec.map(str::to_string);
    let (tx, rx) = bounded(QUEUE_CAP);
    let (ready_tx, ready_rx) = bounded::<Result<(), String>>(1);
    let join = thread::Builder::new()
        .name("strudel-midi".into())
        .spawn(move || match open_connection(spec.as_deref()) {
            Ok(conn) => {
                let _ = ready_tx.send(Ok(()));
                midi_worker(
                    rx,
                    Box::new(MidirSink {
                        conn,
                        sent: 0,
                        errors: 0,
                    }),
                );
            }
            Err(e) => {
                let _ = ready_tx.send(Err(e));
            }
        })
        .map_err(|e| format!("midi thread: {e}"))?;
    match ready_rx.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(())) => Ok(MidiHandle {
            tx: Some(tx),
            join: Some(join),
        }),
        Ok(Err(e)) => {
            drop(tx);
            let _ = join.join();
            Err(e)
        }
        Err(_) => {
            drop(tx);
            Err("midi connect timed out".into())
        }
    }
}

fn open_connection(spec: Option<&str>) -> Result<MidiOutputConnection, String> {
    let out = MidiOutput::new("strudel-rs").map_err(|e| format!("midi: {e}"))?;
    let ports = out.ports();
    if ports.is_empty() {
        return Err("no MIDI output ports".into());
    }
    let idx = match spec.map(str::trim).filter(|s| !s.is_empty()) {
        None => ports
            .iter()
            .position(|p| {
                out.port_name(p)
                    .map(|n| n.to_ascii_uppercase().contains("SEQTRAK"))
                    .unwrap_or(false)
            })
            .unwrap_or(0),
        Some(s) => {
            if let Ok(i) = s.parse::<usize>() {
                if i < ports.len() {
                    i
                } else {
                    return Err(format!(
                        "MIDI port index {i} out of range (0..{})",
                        ports.len()
                    ));
                }
            } else {
                let needle = s.to_ascii_lowercase();
                ports
                    .iter()
                    .position(|p| {
                        out.port_name(p)
                            .map(|n| n.to_ascii_lowercase().contains(&needle))
                            .unwrap_or(false)
                    })
                    .ok_or_else(|| format!("no MIDI port matching `{s}`"))?
            }
        }
    };
    let name = out
        .port_name(&ports[idx])
        .unwrap_or_else(|_| format!("port {idx}"));
    eprintln!("midi: connected {name}");
    out.connect(&ports[idx], "strudel-rs")
        .map_err(|e| format!("midi connect {name}: {e}"))
}

/// Hz → nearest MIDI note 0..=127 (A4 = 440 Hz = 69).
pub fn hz_to_nearest_midi(freq: f32) -> u8 {
    if !(freq.is_finite() && freq > 0.0) {
        return DRUM_NOTE;
    }
    let n = 69.0 + 12.0 * (freq.max(1.0) / 440.0).log2();
    n.round().clamp(0.0, 127.0) as u8
}

fn base_sound(sound: &str) -> &str {
    sound
        .split_once(':')
        .map(|(a, _)| a)
        .unwrap_or(sound)
        .trim()
}

fn is_synth_name(name: &str) -> bool {
    let n = name.trim().to_ascii_lowercase();
    Wave::parse(&n).is_some() || NoiseKind::parse(&n).is_some() || n.starts_with("wt_")
}

/// True when this `$:` is a pitched/synth track (for SYNTH 1 vs SYNTH 2).
pub fn track_is_synth(sound: &str, is_note: bool, has_fm: bool) -> bool {
    has_fm || is_note || is_synth_name(sound)
}

/// Map a hit to MIDI channel 0..=15 (SEQTRAK 1..=11 in the user table).
///
/// `track_ch` is the 1-based `// @midi ch=N` override.
pub fn map_channel(sound: &str, has_fm: bool, track_ch: Option<u8>, synth_ordinal: u8) -> u8 {
    if let Some(ch) = track_ch {
        return ch.saturating_sub(1).min(15);
    }
    if has_fm {
        return 9; // DX = ch 10
    }
    let base = base_sound(sound).to_ascii_lowercase();
    match base.as_str() {
        "bd" | "kick" => 0,
        "sd" | "snare" => 1,
        "cp" | "clap" => 2,
        "hh" | "hat" => 3,
        "oh" => 4,
        "lt" | "mt" | "ht" | "tom" | "perc" | "rim" | "cb" | "rs" => 5,
        "rd" | "cr" | "cy" | "ride" | "crash" => 6,
        _ if is_synth_name(&base) => {
            if synth_ordinal == 1 {
                8
            } else {
                7
            }
        }
        _ if is_short_drum(&base) => 5,
        _ => 10, // sampler
    }
}

fn is_short_drum(name: &str) -> bool {
    let n = name.len();
    (2..=4).contains(&n) && name.bytes().all(|b| b.is_ascii_alphanumeric()) && !name.contains('-')
}

/// Channel + note + velocity for one hit. `gain` is 0..=1.
pub fn map_hit(
    sound: &str,
    freq: f32,
    is_note: bool,
    has_fm: bool,
    track_ch: Option<u8>,
    synth_ordinal: u8,
    gain: f32,
) -> (u8, u8, u8) {
    let ch = map_channel(sound, has_fm, track_ch, synth_ordinal);
    let note = if is_note && freq > 1.0 {
        hz_to_nearest_midi(freq)
    } else {
        DRUM_NOTE
    };
    let vel = (gain.clamp(0.0, 1.0) * 127.0).round().clamp(1.0, 127.0) as u8;
    (ch, note, vel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_note_on_off() {
        let on = MidiEvent::NoteOn {
            ch: 0,
            note: 60,
            vel: 100,
        };
        assert_eq!(on.to_bytes(), [0x90, 60, 100]);
        let off = MidiEvent::NoteOff { ch: 1, note: 60 };
        assert_eq!(off.to_bytes(), [0x81, 60, 0]);
    }

    #[test]
    fn hz_a4_is_69() {
        assert_eq!(hz_to_nearest_midi(440.0), 69);
        assert_eq!(hz_to_nearest_midi(261.6256), 60);
    }

    #[test]
    fn drums_split_channels() {
        assert_eq!(map_channel("bd", false, None, 0), 0);
        assert_eq!(map_channel("sd", false, None, 0), 1);
        assert_eq!(map_channel("cp", false, None, 0), 2);
        assert_eq!(map_channel("hh", false, None, 0), 3);
        assert_eq!(map_channel("oh", false, None, 0), 4);
        assert_eq!(map_channel("bd:hf", false, None, 0), 0);
        assert_eq!(map_channel("lt", false, None, 0), 5);
        assert_eq!(map_channel("perc", false, None, 0), 5);
        assert_eq!(map_channel("rd", false, None, 0), 6);
        assert_eq!(map_channel("crash", false, None, 0), 6);
    }

    #[test]
    fn synth_fm_sampler_and_override() {
        assert_eq!(map_channel("sawtooth", false, None, 0), 7);
        assert_eq!(map_channel("sawtooth", false, None, 1), 8);
        assert_eq!(map_channel("sawtooth", false, None, 2), 7);
        assert_eq!(map_channel("sawtooth", true, None, 0), 9);
        assert_eq!(map_channel("pad-ambient_drone01", false, None, 0), 10);
        assert_eq!(map_channel("bd", false, Some(9), 0), 8);
    }

    #[test]
    fn null_midi_records() {
        let mut n = NullMidi::new();
        n.emit(MidiEvent::NoteOn {
            ch: 0,
            note: 60,
            vel: 80,
        });
        n.emit(MidiEvent::NoteOff { ch: 0, note: 60 });
        let log = n.snapshot();
        assert_eq!(log.len(), 2);
        assert!(matches!(log[0], MidiEvent::NoteOn { ch: 0, .. }));
        assert!(matches!(log[1], MidiEvent::NoteOff { ch: 0, note: 60 }));
    }

    #[test]
    fn spawn_sink_note_off_after_on() {
        let null = NullMidi::new();
        let log = Arc::clone(&null.events);
        let h = spawn_sink(Box::new(null));
        try_push(
            &h.sender(),
            MidiEvent::NoteOn {
                ch: 0,
                note: 60,
                vel: 90,
            },
        );
        try_push(&h.sender(), MidiEvent::NoteOff { ch: 0, note: 60 });
        drop(h);
        // Worker exits when sender is dropped; give it a tick.
        thread::sleep(std::time::Duration::from_millis(30));
        let ev = log.lock().unwrap().clone();
        assert_eq!(ev.len(), 2);
        assert!(matches!(ev[0], MidiEvent::NoteOn { .. }));
        assert!(matches!(ev[1], MidiEvent::NoteOff { .. }));
    }

    #[test]
    fn drop_does_not_hang_if_sender_clone_lives() {
        let h = spawn_sink(Box::new(NullMidi::new()));
        let tx = h.sender();
        let start = std::time::Instant::now();
        drop(h);
        assert!(
            start.elapsed() < Duration::from_secs(2),
            "MidiHandle::drop must not join forever while a sender clone lives"
        );
        drop(tx);
    }

    #[test]
    fn display_uses_1based_channel() {
        let on = MidiEvent::NoteOn {
            ch: 0,
            note: 60,
            vel: 90,
        };
        assert_eq!(on.to_string(), "NoteOn ch=1 note=60 vel=90");
    }
}
