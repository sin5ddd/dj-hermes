//! Integration tests: bundled demo songs + DJ xfade (NullBackend / Engine::process).
//!
//! Device audio is not required. Samples are optional for ambient-only paths.

use std::fs;
use std::path::PathBuf;

use strudel_rs::engine::{Command, Engine};
use strudel_rs::sample::SampleBank;
use strudel_rs::song::parse_song;

const SR: u32 = 48_000;
const BPM: f64 = 126.0;

fn songs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("songs")
}

fn samples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("samples")
}

fn load_song_file(name: &str) -> strudel_rs::song::Song {
    let path = songs_dir().join(name);
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    parse_song(&text, path.to_string_lossy().as_ref())
        .unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn load_bank() -> SampleBank {
    let path = samples_dir();
    if path.is_dir() {
        SampleBank::load_dir(&path, SR)
    } else {
        SampleBank::empty()
    }
}

fn samples_available() -> bool {
    let d = samples_dir();
    d.is_dir()
        && (d.join("bd.wav").is_file()
            || d.join("bd").is_dir()
            || fs::read_dir(&d)
                .map(|mut i| i.next().is_some())
                .unwrap_or(false))
}

fn bar_len(bpm: f64) -> usize {
    // 1 bar = 4 beats; samples = sr * 60 / bpm * 4
    (SR as f64 * 60.0 / bpm * 4.0).round() as usize
}

fn process_n(engine: &mut Engine, bank: &SampleBank, frames: usize) -> Vec<f32> {
    // Engine expects interleaved stereo: len = frames * 2.
    let mut buf = vec![0f32; frames * 2];
    engine.process(&mut buf, bank);
    buf
}

fn process_bars(engine: &mut Engine, bank: &SampleBank, bars: usize, bpm: f64) -> Vec<f32> {
    let mut all = Vec::new();
    let bl = bar_len(bpm);
    for _ in 0..bars {
        all.extend(process_n(engine, bank, bl));
    }
    all
}

fn assert_finite_bounded(buf: &[f32], label: &str) {
    for (i, &s) in buf.iter().enumerate() {
        assert!(s.is_finite(), "{label}: non-finite sample at {i}: {s}");
        assert!(s.abs() <= 1.0 + 1e-5, "{label}: |sample| > 1 at {i}: {s}");
    }
}

fn peak(buf: &[f32]) -> f32 {
    buf.iter().fold(0f32, |m, s| m.max(s.abs()))
}

fn has_energy(buf: &[f32], thr: f32) -> bool {
    buf.iter().any(|s| s.abs() > thr)
}

/// Fraction of samples that sit at the hard clip rail (|x| ≈ 1).
fn clip_rail_ratio(buf: &[f32]) -> f32 {
    if buf.is_empty() {
        return 0.0;
    }
    let n = buf.iter().filter(|s| s.abs() >= 0.999).count();
    n as f32 / buf.len() as f32
}

#[test]
fn all_bundled_songs_parse() {
    let dir = songs_dir();
    assert!(dir.is_dir(), "songs/ missing at {}", dir.display());
    let mut count = 0;
    for entry in fs::read_dir(&dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("strudel") {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        let song = parse_song(&text, path.to_string_lossy().as_ref())
            .unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));
        assert!(!song.tracks.is_empty(), "{} has no tracks", path.display());
        count += 1;
    }
    assert!(
        count >= 2,
        "expected at least techno1 + ambient1, got {count}"
    );
}

#[test]
fn showcase_songs_use_task23_features() {
    let techno = fs::read_to_string(songs_dir().join("techno1.strudel")).unwrap();
    assert!(techno.contains("fm(") || techno.contains(".fm("));
    assert!(techno.contains("orbit(") || techno.contains("duckorbit("));
    assert!(techno.contains("compressor("));

    let ambient = fs::read_to_string(songs_dir().join("ambient1.strudel")).unwrap();
    assert!(ambient.contains("wt_"));
    assert!(ambient.contains("vib(") || ambient.contains(".vib("));
    // Prefer scale + relative integer degrees over chord-quality note tags.
    assert!(
        ambient.contains(".scale(") || ambient.contains("scale("),
        "ambient1 should use .scale(...) with degree patterns"
    );
}

#[test]
fn ambient1_sounds_without_samples() {
    let song = load_song_file("ambient1.strudel");
    assert_eq!(song.title, "ambient1");
    let bank = SampleBank::empty();
    let mut e = Engine::new(SR, BPM);
    e.push_command(Command::LoadSong {
        deck: 0,
        song: Box::new(song),
    });

    // Bar 0 → apply load at next bar head; then play into bar 1.
    let _ = process_bars(&mut e, &bank, 1, BPM);
    let buf = process_bars(&mut e, &bank, 2, BPM);
    assert_finite_bounded(&buf, "ambient1");
    assert!(
        has_energy(&buf, 0.001),
        "ambient1 should sound without SampleBank, peak={}",
        peak(&buf)
    );
    assert!(
        clip_rail_ratio(&buf) < 0.05,
        "ambient1 heavily at clip rail: {}",
        clip_rail_ratio(&buf)
    );
}

#[test]
fn skill_four_on_the_floor_sounds_with_samples() {
    if !samples_available() {
        eprintln!("skip skill_four_on_the_floor: samples/ not found");
        return;
    }
    let song = load_song_file("skill-four-on-the-floor.strudel");
    assert_eq!(song.title, "skill-four-on-the-floor");
    assert_eq!(song.tracks.len(), 1);
    assert!((song.bpm.unwrap() - 124.0).abs() < 1e-6);
    let bank = load_bank();
    let bpm = 124.0;
    let mut e = Engine::new(SR, bpm);
    e.push_command(Command::LoadSong {
        deck: 0,
        song: Box::new(song),
    });
    let _ = process_bars(&mut e, &bank, 1, bpm);
    let buf = process_bars(&mut e, &bank, 2, bpm);
    assert_finite_bounded(&buf, "skill-four-on-the-floor");
    assert!(
        has_energy(&buf, 0.001),
        "four-on-the-floor should sound, peak={}",
        peak(&buf)
    );
    assert!(
        clip_rail_ratio(&buf) < 0.05,
        "four-on-the-floor clip rail: {}",
        clip_rail_ratio(&buf)
    );
}

#[test]
fn skill_minor_scale_loop_sounds_without_samples() {
    let song = load_song_file("skill-minor-scale-loop.strudel");
    assert_eq!(song.title, "skill-minor-scale-loop");
    assert_eq!(song.tracks.len(), 2);
    assert!((song.bpm.unwrap() - 100.0).abs() < 1e-6);
    let bank = SampleBank::empty();
    let bpm = 100.0;
    let mut e = Engine::new(SR, bpm);
    e.push_command(Command::LoadSong {
        deck: 0,
        song: Box::new(song),
    });
    let _ = process_bars(&mut e, &bank, 1, bpm);
    let buf = process_bars(&mut e, &bank, 2, bpm);
    assert_finite_bounded(&buf, "skill-minor-scale-loop");
    assert!(
        has_energy(&buf, 0.001),
        "minor-scale-loop should sound without SampleBank, peak={}",
        peak(&buf)
    );
    assert!(
        clip_rail_ratio(&buf) < 0.05,
        "minor-scale-loop clip rail: {}",
        clip_rail_ratio(&buf)
    );
}

#[test]
fn techno1_sounds_with_samples() {
    if !samples_available() {
        eprintln!("skip techno1_sounds_with_samples: samples/ not found");
        return;
    }
    let song = load_song_file("techno1.strudel");
    assert_eq!(song.title, "techno1");
    let bank = load_bank();
    let mut e = Engine::new(SR, BPM);
    e.push_command(Command::LoadSong {
        deck: 0,
        song: Box::new(song),
    });
    let _ = process_bars(&mut e, &bank, 1, BPM);
    let buf = process_bars(&mut e, &bank, 2, BPM);
    assert_finite_bounded(&buf, "techno1");
    assert!(
        has_energy(&buf, 0.001),
        "techno1 should sound with samples, peak={}",
        peak(&buf)
    );
}

#[test]
fn dj_xfade_techno_to_ambient() {
    let bank = if samples_available() {
        load_bank()
    } else {
        // techno1 may be quiet without samples; still exercise xfade state machine.
        SampleBank::empty()
    };

    let techno = load_song_file("techno1.strudel");
    let ambient = load_song_file("ambient1.strudel");
    let mut e = Engine::new(SR, BPM);
    let bl = bar_len(BPM);

    e.push_command(Command::LoadSong {
        deck: 0,
        song: Box::new(techno),
    });
    // Reach bar 1 and apply load A.
    let mut all = Vec::new();
    all.extend(process_n(&mut e, &bank, bl));
    let buf = process_n(&mut e, &bank, bl);
    assert_finite_bounded(&buf, "after load A");
    all.extend(&buf);
    assert_eq!(e.decks[0].song_title(), Some("techno1"));
    assert!((e.mixer.gain_a - 1.0).abs() < 1e-5);

    e.push_command(Command::LoadSong {
        deck: 1,
        song: Box::new(ambient),
    });
    e.push_command(Command::XFade {
        to_deck: 1,
        bars: 4,
    });

    // Process until pending targets apply (next bar head).
    let buf = process_n(&mut e, &bank, bl);
    assert_finite_bounded(&buf, "pending apply");
    all.extend(&buf);
    if e.mixer.xfade().is_none() && e.decks[1].song_title().is_none() {
        let buf = process_n(&mut e, &bank, bl);
        assert_finite_bounded(&buf, "extra bar for pending");
        all.extend(&buf);
    }
    assert_eq!(
        e.decks[1].song_title(),
        Some("ambient1"),
        "deck B should be loaded"
    );
    assert!(
        e.mixer.xfade().is_some() || e.mixer.gain_b > 0.0,
        "xfade should have started or completed, a={} b={}",
        e.mixer.gain_a,
        e.mixer.gain_b
    );

    // Run enough bars for a 4-bar xfade to finish (+ margin).
    for i in 0..6 {
        let buf = process_n(&mut e, &bank, bl);
        assert_finite_bounded(&buf, &format!("xfade bar {i}"));
        all.extend(&buf);
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
        Some("techno1"),
        "source deck should stay loaded after xfade"
    );
    assert_eq!(e.decks[1].song_title(), Some("ambient1"));

    // After xfade, ambient should still produce energy (synth-only path).
    let buf = process_bars(&mut e, &bank, 2, BPM);
    assert_finite_bounded(&buf, "post-xfade ambient");
    assert!(
        has_energy(&buf, 0.001),
        "ambient on B should sound after xfade, peak={}",
        peak(&buf)
    );
    assert!(
        clip_rail_ratio(&all) < 0.08,
        "session heavily at clip rail: {}",
        clip_rail_ratio(&all)
    );
}
