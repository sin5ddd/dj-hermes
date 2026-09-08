//! MIDI output for `dj-hermes play` (notes, 18.3 CC, Bank Select + Program Change).
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

/// SEQTRAK 18.3 / Data List CCs we transmit.
pub const CC_BANK_MSB: u8 = 0;
pub const CC_BANK_LSB: u8 = 32;
pub const CC_VOLUME: u8 = 7;
pub const CC_PAN: u8 = 10;
pub const CC_RESONANCE: u8 = 71;
pub const CC_ATTACK: u8 = 73;
pub const CC_CUTOFF: u8 = 74;
pub const CC_DECAY: u8 = 75;
pub const CC_REVERB: u8 = 91;
pub const CC_DELAY: u8 = 94;
pub const CC_FM_AMOUNT: u8 = 117;

const MAX_HIT_CCS: usize = 9;

/// Events the MIDI thread can send.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MidiEvent {
    NoteOn { ch: u8, note: u8, vel: u8 },
    NoteOff { ch: u8, note: u8 },
    Cc { ch: u8, cc: u8, val: u8 },
    ProgramChange { ch: u8, program: u8 },
}

impl MidiEvent {
    /// Channel voice bytes (status + data). `ch` is 0..=15.
    /// Program Change occupies only the first two bytes; see [`Self::byte_len`].
    pub fn to_bytes(self) -> [u8; 3] {
        match self {
            MidiEvent::NoteOn { ch, note, vel } => [0x90 | (ch & 0x0F), note, vel],
            MidiEvent::NoteOff { ch, note } => [0x80 | (ch & 0x0F), note, 0],
            MidiEvent::Cc { ch, cc, val } => [0xB0 | (ch & 0x0F), cc, val],
            MidiEvent::ProgramChange { ch, program } => [0xC0 | (ch & 0x0F), program, 0],
        }
    }

    /// Bytes to put on the wire. Program Change is 2 bytes; a trailing 0
    /// would be a second PC under running status (ALSA).
    pub fn byte_len(self) -> usize {
        match self {
            MidiEvent::ProgramChange { .. } => 2,
            _ => 3,
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
            MidiEvent::Cc { ch, cc, val } => {
                write!(f, "CC ch={} cc={cc} val={val}", ch + 1)
            }
            MidiEvent::ProgramChange { ch, program } => {
                write!(f, "PC ch={} program={program}", ch + 1)
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
        let bytes = ev.to_bytes();
        match self.conn.send(&bytes[..ev.byte_len()]) {
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
    let out = MidiOutput::new("dj-hermes").map_err(|e| format!("midi: {e}"))?;
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
    let out = MidiOutput::new("dj-hermes").map_err(|e| format!("midi: {e}"))?;
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
    out.connect(&ports[idx], "dj-hermes")
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

/// 0..=1 → 0..=127.
pub fn unit_to_cc7(x: f32) -> u8 {
    (x.clamp(0.0, 1.0) * 127.0).round() as u8
}

/// 0..=1 → 1..=127 (center 0.5 → 64).
pub fn pan_to_cc(pan: f32) -> u8 {
    ((pan.clamp(0.0, 1.0) * 126.0).round() as u8)
        .saturating_add(1)
        .min(127)
}

/// Hz 20..=8000, log map → 0..=127.
pub fn hz_to_cutoff_cc(hz: f32) -> u8 {
    let hz = hz.clamp(20.0, 8000.0);
    let span = (8000.0_f32 / 20.0).log2();
    let t = (hz / 20.0).log2() / span;
    (t.clamp(0.0, 1.0) * 127.0).round() as u8
}

/// Engine Q 0..=16 → 0..=127.
pub fn q_to_cc(q: f32) -> u8 {
    (q.clamp(0.0, 16.0) / 16.0 * 127.0).round() as u8
}

/// FM index 0..=32 → 0..=127.
pub fn fm_to_cc(fm: f32) -> u8 {
    (fm.clamp(0.0, 32.0) / 32.0 * 127.0).round() as u8
}

/// Snapshot at Note On (LFO is already sampled into `lpf`).
#[derive(Clone, Copy)]
pub struct HitCcInput {
    pub ch: u8,
    pub gain: f32,
    pub pan: f32,
    pub lpf: Option<f32>,
    pub lpq: f32,
    pub attack: f32,
    pub decay: f32,
    pub release: f32,
    pub room: f32,
    pub delay: f32,
    pub fm: f32,
}

/// CCs to send at Note On (not every sample). `ch` is 0..=15.
/// Returns `(pairs, n)` into a stack array — no heap.
pub fn hit_ccs(hit: HitCcInput) -> ([(u8, u8); MAX_HIT_CCS], usize) {
    let mut out = [(0u8, 0u8); MAX_HIT_CCS];
    let mut n = 0usize;
    let mut push = |cc: u8, val: u8| {
        out[n] = (cc, val);
        n += 1;
    };
    push(CC_VOLUME, unit_to_cc7(hit.gain));
    push(CC_PAN, pan_to_cc(hit.pan));
    if let Some(hz) = hit.lpf {
        push(CC_CUTOFF, hz_to_cutoff_cc(hz));
    }
    push(CC_RESONANCE, q_to_cc(hit.lpq));
    push(CC_ATTACK, unit_to_cc7(hit.attack));
    push(CC_DECAY, unit_to_cc7(hit.decay.max(hit.release)));
    push(CC_REVERB, unit_to_cc7(hit.room));
    push(CC_DELAY, unit_to_cc7(hit.delay));
    if hit.ch == 9 && hit.fm.abs() > 1e-6 {
        push(CC_FM_AMOUNT, fm_to_cc(hit.fm));
    }
    (out, n)
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

    #[test]
    fn cc_and_pc_bytes() {
        assert_eq!(
            MidiEvent::Cc {
                ch: 0,
                cc: 7,
                val: 64
            }
            .to_bytes(),
            [0xB0, 7, 64]
        );
        let pc = MidiEvent::ProgramChange { ch: 7, program: 12 };
        assert_eq!(pc.to_bytes(), [0xC7, 12, 0]);
        assert_eq!(pc.byte_len(), 2);
        assert_eq!(&pc.to_bytes()[..pc.byte_len()], &[0xC7, 12]);
    }

    #[test]
    fn pan_center_is_64() {
        assert_eq!(pan_to_cc(0.0), 1);
        assert_eq!(pan_to_cc(0.5), 64);
        assert_eq!(pan_to_cc(1.0), 127);
    }

    fn hit_cc_input(ch: u8, lpf: Option<f32>, fm: f32) -> HitCcInput {
        HitCcInput {
            ch,
            gain: 0.5,
            pan: 0.5,
            lpf,
            lpq: 0.707,
            attack: 0.01,
            decay: 0.1,
            release: 0.1,
            room: 0.0,
            delay: 0.0,
            fm,
        }
    }

    #[test]
    fn hit_ccs_skips_cutoff_without_lpf() {
        let (pairs, n) = hit_ccs(hit_cc_input(0, None, 0.0));
        let ccs: Vec<u8> = pairs[..n].iter().map(|p| p.0).collect();
        assert!(ccs.contains(&CC_VOLUME));
        assert!(!ccs.contains(&CC_CUTOFF));
        assert!(!ccs.contains(&CC_FM_AMOUNT));
    }

    #[test]
    fn hit_ccs_fm_only_on_dx_channel() {
        let (pairs10, n10) = hit_ccs(hit_cc_input(9, None, 4.0));
        assert!(pairs10[..n10].iter().any(|p| p.0 == CC_FM_AMOUNT));
        let (pairs8, n8) = hit_ccs(hit_cc_input(7, None, 4.0));
        assert!(!pairs8[..n8].iter().any(|p| p.0 == CC_FM_AMOUNT));
    }

    #[test]
    fn hit_ccs_includes_cutoff_when_lpf() {
        let (pairs, n) = hit_ccs(hit_cc_input(0, Some(800.0), 0.0));
        assert!(pairs[..n].iter().any(|p| p.0 == CC_CUTOFF));
        assert_eq!(hz_to_cutoff_cc(20.0), 0);
        assert_eq!(hz_to_cutoff_cc(8000.0), 127);
        assert_eq!(unit_to_cc7(0.0), 0);
        assert_eq!(unit_to_cc7(1.0), 127);
    }
}
