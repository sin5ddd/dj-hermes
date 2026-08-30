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
    assert!(techno.contains("duckorbit("));
    assert!(
        techno.contains("orbit(2)"),
        "bass and pad must share the ducked orbit"
    );
    // .compressor is mixer master last-write — do not showcase it on the bass.
    assert!(
        !techno.contains("compressor("),
        "techno1 must not set master compressor from a track"
    );

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
    let drums = &song.tracks[0].code.mini_src;
    assert!(drums.contains("bd*4"), "{drums}");
    assert!(drums.contains("[~ hh]*4"), "{drums}");
    assert!(
        !drums.contains("sd"),
        "techno four-on-the-floor must not use a house snare backbeat: {drums}"
    );
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
    assert!((song.bpm.unwrap() - 124.0).abs() < 1e-6);
    let bank = SampleBank::empty();
    let bpm = 124.0;
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

fn track<'a>(song: &'a strudel_rs::song::Song, name: &str) -> &'a strudel_rs::song::Track {
    song.tracks
        .iter()
        .find(|t| t.name == name || t.name.starts_with(name))
        .unwrap_or_else(|| {
            panic!(
                "no track {name:?} in {:?}",
                song.tracks
                    .iter()
                    .map(|t| t.name.as_str())
                    .collect::<Vec<_>>()
            )
        })
}

#[test]
fn skill_dnb_mix_rules() {
    let song = load_song_file("skill-dnb.strudel");
    assert_eq!(song.title, "skill-dnb");
    assert!((song.bpm.unwrap() - 174.0).abs() < 1e-6);
    let drums = track(&song, "drums");
    let sub = track(&song, "sub");
    let mid = track(&song, "mid");
    assert!(
        drums.code.gain > sub.code.gain,
        "drums {} must sit above sub {}",
        drums.code.gain,
        sub.code.gain
    );
    assert_eq!(sub.code.sound, "square");
    assert_eq!(mid.code.sound, "sawtooth");
    let mid_lpf = mid.code.filter.lpf.expect("mid reese needs lpf");
    assert!(
        (800.0..=1200.0).contains(&mid_lpf),
        "mid Reese lpf {mid_lpf} should be 800–1200"
    );
    for t in &song.tracks {
        assert!(
            !t.code.mini_src.contains("db"),
            "db is not a sample (silent): {}",
            t.code.mini_src
        );
    }
}

#[test]
fn dnb16_follows_skill_mix_rules() {
    let song = load_song_file("dnb16.strudel");
    assert!((song.bpm.unwrap() - 174.0).abs() < 1e-6);
    let drums = track(&song, "drums");
    let sub = track(&song, "sub");
    assert!(drums.code.gain > sub.code.gain);
    assert_eq!(sub.code.sound, "square");
    let mid = song
        .tracks
        .iter()
        .find(|t| t.code.is_note && t.code.sound == "sawtooth" && t.name.contains("mid"))
        .expect("dnb16 needs a saw mid Reese track");
    let mid_lpf = mid.code.filter.lpf.expect("mid lpf");
    assert!((800.0..=1200.0).contains(&mid_lpf));
    for t in &song.tracks {
        assert!(!t.code.mini_src.contains("db"), "{}", t.code.mini_src);
    }
}

#[test]
fn skill_dnb_sounds_with_samples() {
    if !samples_available() {
        eprintln!("skip skill_dnb: samples/ not found");
        return;
    }
    let song = load_song_file("skill-dnb.strudel");
    let bank = load_bank();
    assert!(
        !bank.has("db"),
        "db must not resolve — that atom is silence"
    );
    let bpm = 174.0;
    let mut e = Engine::new(SR, bpm);
    e.push_command(Command::LoadSong {
        deck: 0,
        song: Box::new(song),
    });
    let _ = process_bars(&mut e, &bank, 1, bpm);
    let buf = process_bars(&mut e, &bank, 2, bpm);
    assert_finite_bounded(&buf, "skill-dnb");
    assert!(has_energy(&buf, 0.001), "skill-dnb peak={}", peak(&buf));
    assert!(clip_rail_ratio(&buf) < 0.05, "skill-dnb clip");
}

#[test]
fn skill_techno_duck_mix_rules() {
    let song = load_song_file("skill-techno-duck.strudel");
    assert_eq!(song.title, "skill-techno-duck");
    assert!((song.bpm.unwrap() - 126.0).abs() < 1e-6);
    let kick = track(&song, "kick");
    let bass = track(&song, "bass");
    let pad = track(&song, "pad");
    assert_eq!(kick.code.duck.count, 1);
    assert_eq!(kick.code.duck.orbits[0], 2);
    let atk = kick.code.duck.attack[0];
    assert!(
        (0.03..=0.05).contains(&atk),
        "duckattack {atk} must be 0.03–0.05"
    );
    assert_eq!(bass.code.orbit, 2, "bass must be on the ducked orbit");
    assert_eq!(pad.code.orbit, 2, "pad must be on the ducked orbit");
    assert!(
        kick.code.orbit != 2,
        "kick must not sit on the ducked orbit"
    );
    assert!(bass.code.compressor.is_none());
    assert!(pad.code.compressor.is_none());
    assert!(kick.code.compressor.is_none());
}

#[test]
fn techno1_follows_skill_mix_rules() {
    let song = load_song_file("techno1.strudel");
    let kick = track(&song, "kick");
    let bass = track(&song, "bass");
    let pad = track(&song, "pad");
    let atk = kick.code.duck.attack[0];
    assert!((0.03..=0.05).contains(&atk), "techno1 duckattack {atk}");
    assert_eq!(bass.code.orbit, 2);
    assert_eq!(pad.code.orbit, 2);
    assert!(song.tracks.iter().all(|t| t.code.compressor.is_none()));
}

#[test]
fn skill_techno_duck_sounds_with_samples() {
    if !samples_available() {
        eprintln!("skip skill_techno_duck: samples/ not found");
        return;
    }
    let song = load_song_file("skill-techno-duck.strudel");
    let bank = load_bank();
    let bpm = 126.0;
    let mut e = Engine::new(SR, bpm);
    e.push_command(Command::LoadSong {
        deck: 0,
        song: Box::new(song),
    });
    let _ = process_bars(&mut e, &bank, 1, bpm);
    let buf = process_bars(&mut e, &bank, 2, bpm);
    assert_finite_bounded(&buf, "skill-techno-duck");
    assert!(
        has_energy(&buf, 0.001),
        "skill-techno-duck peak={}",
        peak(&buf)
    );
    assert!(clip_rail_ratio(&buf) < 0.08, "skill-techno-duck clip");
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

#[test]
fn skill_drum_and_bass_mix_rules() {
    let song = load_song_file("skill-drum-and-bass.strudel");
    assert_eq!(song.title, "skill-drum-and-bass");
    assert!((song.bpm.unwrap() - 174.0).abs() < 1e-6);
    let drums = track(&song, "drums");
    let sub = track(&song, "sub");
    let mid = track(&song, "mid");
    assert!(
        drums.code.gain > sub.code.gain,
        "drums {} must sit above sub {}",
        drums.code.gain,
        sub.code.gain
    );
    assert_eq!(sub.code.sound, "square");
    assert_eq!(mid.code.sound, "sawtooth");
    let mid_lpf = mid.code.filter.lpf.expect("mid reese needs lpf");
    assert!(
        (800.0..=1200.0).contains(&mid_lpf),
        "mid Reese lpf {mid_lpf} should be 800–1200"
    );
    for t in &song.tracks {
        assert!(
            !t.code.mini_src.contains("db"),
            "db is not a sample (silent): {}",
            t.code.mini_src
        );
    }
    // Mini *2 tiles the break across the bar. Method .fast(2) would squeeze
    // one cycle into [0, 0.5) and leave the second half empty.
    let evs = strudel_rs::mini::events(&drums.code.pattern, 0);
    assert!(
        evs.iter().any(|e| e.start >= 0.5),
        "break should occupy the second half of the bar, starts={:?}",
        evs.iter().map(|e| e.start).collect::<Vec<_>>()
    );
    assert!(evs.iter().any(|e| e.value == "bd"));
    assert!(evs.iter().any(|e| e.value == "sd"));
}

#[test]
fn skill_drum_and_bass_sounds_with_samples() {
    if !samples_available() {
        eprintln!("skip skill_drum_and_bass: samples/ not found");
        return;
    }
    let song = load_song_file("skill-drum-and-bass.strudel");
    let bank = load_bank();
    assert!(
        !bank.has("db"),
        "db must not resolve — that atom is silence"
    );
    let bpm = 174.0;
    let mut e = Engine::new(SR, bpm);
    e.push_command(Command::LoadSong {
        deck: 0,
        song: Box::new(song),
    });
    let _ = process_bars(&mut e, &bank, 1, bpm);
    let buf = process_bars(&mut e, &bank, 2, bpm);
    assert_finite_bounded(&buf, "skill-drum-and-bass");
    assert!(
        has_energy(&buf, 0.001),
        "skill-drum-and-bass peak={}",
        peak(&buf)
    );
    assert!(
        clip_rail_ratio(&buf) < 0.05,
        "skill-drum-and-bass clip rail: {}",
        clip_rail_ratio(&buf)
    );
}

#[test]
fn skill_sidechain_ducking_mix_rules() {
    let song = load_song_file("skill-sidechain-ducking.strudel");
    assert_eq!(song.title, "skill-sidechain-ducking");
    assert!((song.bpm.unwrap() - 126.0).abs() < 1e-6);
    let kick = track(&song, "kick");
    let hats = track(&song, "hats");
    let bass = track(&song, "bass");
    let pad = track(&song, "pad");
    assert_eq!(kick.code.duck.count, 1);
    assert_eq!(kick.code.duck.orbits[0], 2);
    let atk = kick.code.duck.attack[0];
    assert!(
        (0.03..=0.05).contains(&atk),
        "duckattack {atk} must be 0.03–0.05"
    );
    assert_eq!(bass.code.orbit, 2, "bass must be on the ducked orbit");
    assert_eq!(pad.code.orbit, 2, "pad must be on the ducked orbit");
    assert!(
        kick.code.orbit != 2,
        "kick must not sit on the ducked orbit"
    );
    assert!(
        !hats.code.mini_src.contains("sd"),
        "techno duck skill must not use a house backbeat: {}",
        hats.code.mini_src
    );
    assert!(
        hats.code.mini_src.contains("hh"),
        "techno hats should be [~ hh]*4"
    );
    assert!(bass.code.compressor.is_none());
    assert!(pad.code.compressor.is_none());
    assert!(kick.code.compressor.is_none());
    assert!(hats.code.compressor.is_none());
}

#[test]
fn skill_sidechain_ducking_sounds_with_samples() {
    if !samples_available() {
        eprintln!("skip skill_sidechain_ducking: samples/ not found");
        return;
    }
    let song = load_song_file("skill-sidechain-ducking.strudel");
    let bank = load_bank();
    let bpm = 126.0;
    let mut e = Engine::new(SR, bpm);
    e.push_command(Command::LoadSong {
        deck: 0,
        song: Box::new(song),
    });
    let _ = process_bars(&mut e, &bank, 1, bpm);
    let buf = process_bars(&mut e, &bank, 2, bpm);
    assert_finite_bounded(&buf, "skill-sidechain-ducking");
    assert!(
        has_energy(&buf, 0.001),
        "skill-sidechain-ducking peak={}",
        peak(&buf)
    );
    assert!(
        clip_rail_ratio(&buf) < 0.08,
        "skill-sidechain-ducking clip rail: {}",
        clip_rail_ratio(&buf)
    );
}

#[test]
fn skill_acid_303_filter_envelope_sounds() {
    let path = songs_dir().join("skill-acid-303-filter-envelope.strudel");
    let text = fs::read_to_string(&path).unwrap();
    assert!(
        text.contains("lpenv(3)") && !text.contains("lpenv(3.5)"),
        "303 skill song must use lpenv(3), not the 10 kHz 3.5 ceiling"
    );
    assert!(
        !text.contains(" 900") && !text.contains("lpf(900"),
        "accent base must stay in 600–800 Hz, not 900"
    );
    assert!(
        !text.contains("[~ sd]"),
        "acid techno example must not add a house snare unless labeled house"
    );
    assert!(
        !text.contains("duckorbit") && !text.contains("compressor("),
        "this recipe is the filter env, not duck/compressor"
    );

    let song = load_song_file("skill-acid-303-filter-envelope.strudel");
    assert_eq!(song.title, "skill-acid-303-filter-envelope");
    assert!(
        song.bpm.is_some() && (song.bpm.unwrap() - 130.0).abs() < 0.1,
        "expected 130 BPM, got {:?}",
        song.bpm
    );

    let bpm = 130.0;
    let bank = if samples_available() {
        load_bank()
    } else {
        SampleBank::empty()
    };
    let mut e = Engine::new(SR, bpm);
    e.push_command(Command::LoadSong {
        deck: 0,
        song: Box::new(song),
    });
    let _ = process_bars(&mut e, &bank, 1, bpm);
    let buf = process_bars(&mut e, &bank, 2, bpm);
    assert_finite_bounded(&buf, "skill-acid-303");
    assert!(
        has_energy(&buf, 0.001),
        "303 skill song should sound, peak={}",
        peak(&buf)
    );
    assert!(
        clip_rail_ratio(&buf) < 0.05,
        "303 skill song heavily at clip rail: {}",
        clip_rail_ratio(&buf)
    );
}

#[test]
fn skill_house_clap_backbeat_sounds() {
    let path = songs_dir().join("skill-house-clap-backbeat.strudel");
    let text = fs::read_to_string(&path).unwrap();
    assert!(
        text.contains("[~ cp]*2"),
        "house backbeat must be clap on 2/4: {text}"
    );
    assert!(
        !text.contains("[~ sd]") && !text.contains(",sd") && !text.contains("sd*"),
        "do not stack or substitute sd on the clap grid: {text}"
    );
    assert!(
        text.contains("lead-fm_pluck") && !text.contains("lead-fm-pluck"),
        "pluck key is lead-fm_pluck (underscore): {text}"
    );
    assert!(
        text.contains("C4:minor") && !text.contains("C3:minor"),
        "C3 sample must use C4:minor, not C3:minor: {text}"
    );
    assert!(
        text.contains("4 ~ 7 4  2 0 ~ -1"),
        "signed-off degrees must not be rewritten: {text}"
    );
    assert!(
        text.contains("cut(1)"),
        "pluck one-shot needs cut(1): {text}"
    );
    assert!(
        !text.contains("duckorbit") && !text.contains("compressor("),
        "no duck / track compressor in this recipe: {text}"
    );
    assert!(
        !text.contains("stab-fm_fifth") && !text.contains("reese-mid"),
        "fifth/reese samples are not this skill: {text}"
    );

    let song = load_song_file("skill-house-clap-backbeat.strudel");
    assert_eq!(song.title, "skill-house-clap-backbeat");
    assert_eq!(song.tracks.len(), 2);
    assert!(
        song.bpm.is_some() && (song.bpm.unwrap() - 124.0).abs() < 1e-6,
        "expected 124 BPM, got {:?}",
        song.bpm
    );

    let drums = track(&song, "drums");
    assert!(
        drums.code.mini_src.contains("bd*4"),
        "{}",
        drums.code.mini_src
    );
    assert!(
        drums.code.mini_src.contains("[~ cp]*2"),
        "{}",
        drums.code.mini_src
    );
    assert!(
        drums.code.mini_src.contains("[~ hh]*4"),
        "{}",
        drums.code.mini_src
    );
    assert!(
        !drums.code.mini_src.contains("sd"),
        "drums must not include sd: {}",
        drums.code.mini_src
    );

    let pluck = track(&song, "pluck");
    assert_eq!(pluck.code.sound, "lead-fm_pluck");
    assert_eq!(pluck.code.cut, Some(1));
    assert!((pluck.code.gain - 0.4).abs() < 1e-5);
    assert!(pluck.code.compressor.is_none());
    let scale = pluck
        .code
        .scale
        .as_ref()
        .expect("pluck needs .scale")
        .at_cycle(0);
    assert_eq!(scale.root_midi, 60, "C4");
    assert_eq!(scale.intervals, vec![0, 2, 3, 5, 7, 8, 10]);

    if !samples_available() {
        eprintln!("skip skill_house_clap render: samples/ not found");
        return;
    }
    let bank = load_bank();
    let bpm = 124.0;
    let mut e = Engine::new(SR, bpm);
    e.push_command(Command::LoadSong {
        deck: 0,
        song: Box::new(song),
    });
    let _ = process_bars(&mut e, &bank, 1, bpm);
    let buf = process_bars(&mut e, &bank, 2, bpm);
    assert_finite_bounded(&buf, "skill-house-clap-backbeat");
    assert!(
        has_energy(&buf, 0.001),
        "house clap skill should sound, peak={}",
        peak(&buf)
    );
    assert!(
        clip_rail_ratio(&buf) < 0.05,
        "house clap skill clip rail: {}",
        clip_rail_ratio(&buf)
    );
}

#[test]
fn skill_dnb_reese_mid_stab_sounds() {
    let path = songs_dir().join("skill-dnb-reese-mid-stab.strudel");
    let text = fs::read_to_string(&path).unwrap();
    assert!(
        text.contains("[bd <~ sd> ~ sd ~ <bd ~> <bd sd> <bd ~>, hh*4, [~@5 oh ~@2]]*2"),
        "signed-off break + mini *2 must not be rewritten: {text}"
    );
    assert!(
        !text.contains(".fast("),
        "method .fast squeezes the break into the first half: {text}"
    );
    assert!(
        text.contains(r#"note("0 3 0 <0 -1>").scale("C2:minor").s("square")"#),
        "square sub stays C2:minor: {text}"
    );
    assert!(
        text.contains(r#"note("0 3 0 <0 -1>").scale("C4:minor").s("reese-mid")"#),
        "reese-mid must stay C4:minor (hyphen stem): {text}"
    );
    assert!(
        !text.contains("reese_mid") && !text.contains("reese-mid.wav"),
        "sound key is reese-mid, not reese_mid: {text}"
    );
    assert!(
        text.contains(r#"note("~ 4 ~ <7 4>").scale("C4:minor").s("stab-fm_fifth")"#),
        "signed-off stab degrees / underscore stem: {text}"
    );
    assert!(
        !text.contains("stab-fm-fifth"),
        "stab key is stab-fm_fifth (underscore): {text}"
    );
    assert!(
        text.contains("cut(1)"),
        "stab one-shot needs cut(1): {text}"
    );
    assert!(
        !text.contains("duckorbit") && !text.contains("compressor("),
        "no duck / track compressor in this recipe: {text}"
    );
    assert!(
        !text.contains(" cp") && !text.contains("\"cp") && !text.contains("[~ cp]"),
        "no house clap on this grid: {text}"
    );

    let song = load_song_file("skill-dnb-reese-mid-stab.strudel");
    assert_eq!(song.title, "skill-dnb-reese-mid-stab");
    assert_eq!(song.tracks.len(), 4);
    assert!(
        song.bpm.is_some() && (song.bpm.unwrap() - 174.0).abs() < 1e-6,
        "expected 174 BPM, got {:?}",
        song.bpm
    );

    let drums = track(&song, "drums");
    let sub = track(&song, "sub");
    let mid = track(&song, "mid");
    let stab = track(&song, "stab");
    assert!(
        drums.code.gain > sub.code.gain,
        "drums {} must sit above sub {}",
        drums.code.gain,
        sub.code.gain
    );
    assert!((drums.code.gain - 0.7).abs() < 1e-5);
    assert_eq!(sub.code.sound, "square");
    assert!((sub.code.gain - 0.42).abs() < 1e-5);
    let sub_lpf = sub.code.filter.lpf.expect("square sub needs lpf");
    assert!((sub_lpf - 120.0).abs() < 1e-3);
    let sub_scale = sub
        .code
        .scale
        .as_ref()
        .expect("sub needs .scale")
        .at_cycle(0);
    assert_eq!(sub_scale.root_midi, 36, "C2");
    assert_eq!(sub_scale.intervals, vec![0, 2, 3, 5, 7, 8, 10]);

    assert_eq!(mid.code.sound, "reese-mid");
    assert!((mid.code.gain - 0.38).abs() < 1e-5);
    let mid_scale = mid
        .code
        .scale
        .as_ref()
        .expect("mid needs .scale")
        .at_cycle(0);
    assert_eq!(mid_scale.root_midi, 60, "C4 — C2 dumps the 800–1200 band");
    assert_eq!(mid_scale.intervals, vec![0, 2, 3, 5, 7, 8, 10]);

    assert_eq!(stab.code.sound, "stab-fm_fifth");
    assert_eq!(stab.code.cut, Some(1));
    assert!((stab.code.gain - 0.22).abs() < 1e-5);
    let stab_scale = stab
        .code
        .scale
        .as_ref()
        .expect("stab needs .scale")
        .at_cycle(0);
    assert_eq!(stab_scale.root_midi, 60, "C4");
    assert_eq!(stab_scale.intervals, vec![0, 2, 3, 5, 7, 8, 10]);

    for t in &song.tracks {
        assert!(
            t.code.compressor.is_none(),
            "no track compressor: {}",
            t.name
        );
        assert!(
            !t.code.mini_src.contains("db"),
            "db is not a sample (silent): {}",
            t.code.mini_src
        );
    }

    // Mini *2 tiles the break across the bar. Method .fast(2) would squeeze
    // one cycle into [0, 0.5) and leave the second half empty.
    let evs = strudel_rs::mini::events(&drums.code.pattern, 0);
    assert!(
        evs.iter().any(|e| e.start >= 0.5),
        "break should occupy the second half of the bar, starts={:?}",
        evs.iter().map(|e| e.start).collect::<Vec<_>>()
    );
    assert!(evs.iter().any(|e| e.value == "bd"));
    assert!(evs.iter().any(|e| e.value == "sd"));

    if !samples_available() {
        eprintln!("skip skill_dnb_reese_mid_stab render: samples/ not found");
        return;
    }
    let bank = load_bank();
    let bpm = 174.0;
    let mut e = Engine::new(SR, bpm);
    e.push_command(Command::LoadSong {
        deck: 0,
        song: Box::new(song),
    });
    let _ = process_bars(&mut e, &bank, 1, bpm);
    let buf = process_bars(&mut e, &bank, 2, bpm);
    assert_finite_bounded(&buf, "skill-dnb-reese-mid-stab");
    assert!(
        has_energy(&buf, 0.001),
        "dnb reese-mid skill should sound, peak={}",
        peak(&buf)
    );
    assert!(
        clip_rail_ratio(&buf) < 0.05,
        "dnb reese-mid skill clip rail: {}",
        clip_rail_ratio(&buf)
    );
}

fn assert_mood_house_drums(text: &str, song: &strudel_rs::song::Song) {
    assert!(
        text.contains("[~ cp]*2"),
        "mood pair keeps the house 2/4 clap: {text}"
    );
    assert!(
        !text.contains("[~ sd]") && !text.contains(",sd") && !text.contains("sd*"),
        "do not stack or substitute sd on the clap grid: {text}"
    );
    assert!(
        !text.contains("stab-fm_fifth") && !text.contains("'maj") && !text.contains("'min"),
        "no hollow-fifth stab / chord-suffix fake quality: {text}"
    );
    assert!(
        !text.contains("duckorbit") && !text.contains("compressor("),
        "no duck / track compressor in this recipe: {text}"
    );
    assert_eq!(song.tracks.len(), 3);
    assert!(
        song.bpm.is_some() && (song.bpm.unwrap() - 124.0).abs() < 1e-6,
        "expected 124 BPM, got {:?}",
        song.bpm
    );
    let drums = track(song, "drums");
    assert_eq!(drums.code.mini_src, r#"bd*4, [~ cp]*2, [~ hh]*4"#);
    assert!((drums.code.gain - 0.6).abs() < 1e-5);
}

#[test]
fn skill_mood_dark_sounds() {
    let path = songs_dir().join("skill-mood-dark.strudel");
    let text = fs::read_to_string(&path).unwrap();
    assert!(
        text.contains(r#"note("0 2 4 0").scale("C2:minor").s("square")"#),
        "dark bass stays C2:minor square: {text}"
    );
    assert!(
        text.contains(r#"note("[0,2,4] ~ [0,2,4] ~").scale("C4:minor").s("reese-mid")"#),
        "dark close triad must stay C4:minor reese-mid: {text}"
    );
    assert!(
        !text.contains("reese_mid"),
        "sound key is reese-mid, not reese_mid: {text}"
    );
    assert!(
        !text.contains(".scale(\"C2:minor\").s(\"reese-mid\")")
            && !text.contains(".scale(\"C3:minor\").s(\"reese-mid\")"),
        "reese-mid C2/C3 dumps the 800–1200 band: {text}"
    );

    let song = load_song_file("skill-mood-dark.strudel");
    assert_eq!(song.title, "skill-mood-dark");
    assert_mood_house_drums(&text, &song);

    let bass = track(&song, "bass");
    assert_eq!(bass.code.sound, "square");
    let bass_lpf = bass.code.filter.lpf.expect("square sub needs lpf");
    assert!((bass_lpf - 140.0).abs() < 1e-3);
    let bass_scale = bass
        .code
        .scale
        .as_ref()
        .expect("bass needs .scale")
        .at_cycle(0);
    assert_eq!(bass_scale.root_midi, 36, "C2");
    assert_eq!(bass_scale.intervals, vec![0, 2, 3, 5, 7, 8, 10]);

    let chords = track(&song, "chords");
    assert_eq!(chords.code.sound, "reese-mid");
    let chord_scale = chords
        .code
        .scale
        .as_ref()
        .expect("chords need .scale")
        .at_cycle(0);
    assert_eq!(chord_scale.root_midi, 60, "C4 — C2/C3 dumps the mid band");
    assert_eq!(chord_scale.intervals, vec![0, 2, 3, 5, 7, 8, 10]);

    if !samples_available() {
        eprintln!("skip skill_mood_dark render: samples/ not found");
        return;
    }
    let bank = load_bank();
    let bpm = 124.0;
    let mut e = Engine::new(SR, bpm);
    e.push_command(Command::LoadSong {
        deck: 0,
        song: Box::new(song),
    });
    let _ = process_bars(&mut e, &bank, 1, bpm);
    let buf = process_bars(&mut e, &bank, 2, bpm);
    assert_finite_bounded(&buf, "skill-mood-dark");
    assert!(
        has_energy(&buf, 0.001),
        "mood-dark skill should sound, peak={}",
        peak(&buf)
    );
    assert!(
        clip_rail_ratio(&buf) < 0.05,
        "mood-dark skill clip rail: {}",
        clip_rail_ratio(&buf)
    );
}

#[test]
fn skill_mood_bright_sounds() {
    let path = songs_dir().join("skill-mood-bright.strudel");
    let text = fs::read_to_string(&path).unwrap();
    assert!(
        text.contains(r#"note("0 2 4 0").scale("C3:major").s("sawtooth")"#),
        "bright bass is C3:major saw: {text}"
    );
    assert!(
        text.contains(r#"note("[0,4,9] ~ [0,4,9] ~").scale("C4:major").s("lead-fm_pluck")"#),
        "bright spread voicing must stay C4:major pluck: {text}"
    );
    assert!(
        text.contains("lead-fm_pluck") && !text.contains("lead-fm-pluck"),
        "pluck key is lead-fm_pluck (underscore): {text}"
    );
    assert!(
        !text.contains("cut(1)") && !text.contains(".cut("),
        "cut on a parallel chord kills voices one at a time: {text}"
    );
    assert!(
        !text.contains(".scale(\"C3:major\").s(\"lead-fm_pluck\")")
            && !text.contains(".scale(\"C2:major\").s(\"lead-fm_pluck\")"),
        "pluck C2/C3 dumps to bass: {text}"
    );

    let song = load_song_file("skill-mood-bright.strudel");
    assert_eq!(song.title, "skill-mood-bright");
    assert_mood_house_drums(&text, &song);

    let bass = track(&song, "bass");
    assert_eq!(bass.code.sound, "sawtooth");
    let bass_lpf = bass.code.filter.lpf.expect("bright saw needs open lpf");
    assert!((bass_lpf - 1400.0).abs() < 1e-3);
    let bass_scale = bass
        .code
        .scale
        .as_ref()
        .expect("bass needs .scale")
        .at_cycle(0);
    assert_eq!(bass_scale.root_midi, 48, "C3");
    assert_eq!(bass_scale.intervals, vec![0, 2, 4, 5, 7, 9, 11]);

    let chords = track(&song, "chords");
    assert_eq!(chords.code.sound, "lead-fm_pluck");
    assert_eq!(chords.code.cut, None, "no cut on [0,4,9] parallel chord");
    let chord_scale = chords
        .code
        .scale
        .as_ref()
        .expect("chords need .scale")
        .at_cycle(0);
    assert_eq!(chord_scale.root_midi, 60, "C4");
    assert_eq!(chord_scale.intervals, vec![0, 2, 4, 5, 7, 9, 11]);

    if !samples_available() {
        eprintln!("skip skill_mood_bright render: samples/ not found");
        return;
    }
    let bank = load_bank();
    let bpm = 124.0;
    let mut e = Engine::new(SR, bpm);
    e.push_command(Command::LoadSong {
        deck: 0,
        song: Box::new(song),
    });
    let _ = process_bars(&mut e, &bank, 1, bpm);
    let buf = process_bars(&mut e, &bank, 2, bpm);
    assert_finite_bounded(&buf, "skill-mood-bright");
    assert!(
        has_energy(&buf, 0.001),
        "mood-bright skill should sound, peak={}",
        peak(&buf)
    );
    assert!(
        clip_rail_ratio(&buf) < 0.05,
        "mood-bright skill clip rail: {}",
        clip_rail_ratio(&buf)
    );
}

#[test]
fn skill_mood_pair_shares_clock_and_drum_grid() {
    let dark = load_song_file("skill-mood-dark.strudel");
    let bright = load_song_file("skill-mood-bright.strudel");
    assert!(
        (dark.bpm.unwrap() - bright.bpm.unwrap()).abs() < 1e-6,
        "DJ pair must share one BPM, got {:?} vs {:?}",
        dark.bpm,
        bright.bpm
    );
    assert_eq!(
        track(&dark, "drums").code.mini_src,
        track(&bright, "drums").code.mini_src,
        "drum grid must stay stable so contrast is harmonic/timbre"
    );

    if !samples_available() {
        eprintln!("skip skill_mood_pair render: samples/ not found");
        return;
    }
    let bank = load_bank();
    let bpm = 124.0;
    let mut e = Engine::new(SR, bpm);
    e.push_command(Command::LoadSong {
        deck: 0,
        song: Box::new(dark),
    });
    e.push_command(Command::LoadSong {
        deck: 1,
        song: Box::new(bright),
    });
    let _ = process_bars(&mut e, &bank, 1, bpm);
    let buf = process_bars(&mut e, &bank, 2, bpm);
    assert_finite_bounded(&buf, "skill-mood pair");
    assert!(
        has_energy(&buf, 0.001),
        "mood pair should sound on two decks, peak={}",
        peak(&buf)
    );
    assert!(
        clip_rail_ratio(&buf) < 0.05,
        "mood pair clip rail: {}",
        clip_rail_ratio(&buf)
    );
}

#[test]
fn skill_fm_sound_design_sounds() {
    let path = songs_dir().join("skill-fm-sound-design.strudel");
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("setcpm(124/4)"), "FM demo is 124 BPM: {text}");
    assert!(
        !text.contains("setcpm(120"),
        "120 is isolated — this file is 124: {text}"
    );
    assert!(
        !text.contains("duckorbit") && !text.contains("compressor("),
        "no duck / track compressor: {text}"
    );
    assert!(
        !text.contains("[~ cp]") && !text.contains("[~ hh]") && !text.contains("[~ sd]"),
        "drums are bd*4 only — do not borrow a genre grid: {text}"
    );
    assert!(
        !text.contains("fmh2") && !text.contains("fmenv") && !text.contains(".fm(8)"),
        "no 4-op API or .fm(8) on this song: {text}"
    );
    assert!(
        !text.contains("fmh(1.5)"),
        "do not copy techno1 fmh(1.5) into this demo: {text}"
    );
    assert!(
        !text.contains("reese-mid") && !text.contains(r#".s("square")"#),
        "do not stack square sub or reese-mid under the growl: {text}"
    );
    assert!(
        !text.contains("lead-fm_pluck") && !text.contains("keys-fm_ep"),
        "do not stack PCM pluck or keys-fm_ep on this mix: {text}"
    );
    assert!(
        !text.contains("stab-fm_major") && !text.contains("stab-fm_fifth"),
        "no stab on this minor demo (prefer none): {text}"
    );
    assert!(
        !text.contains("// ep") && !text.contains(r#"note("0 ~ 4 2")"#),
        "live EP recipe is retired: {text}"
    );
    assert!(
        !text.contains("C3:"),
        "C3 wavs must be written at C4: {text}"
    );

    let song = load_song_file("skill-fm-sound-design.strudel");
    assert_eq!(song.title, "skill-fm-sound-design");
    assert_eq!(song.tracks.len(), 4);
    assert!(
        song.bpm.is_some() && (song.bpm.unwrap() - 124.0).abs() < 1e-6,
        "expected 124 BPM, got {:?}",
        song.bpm
    );

    let drums = track(&song, "drums");
    assert!(
        drums.code.mini_src.contains("bd*4"),
        "{}",
        drums.code.mini_src
    );
    assert!(drums.code.mod_params.fm.abs() < 1e-6);

    let bass = track(&song, "bass");
    assert_eq!(bass.code.sound, "sawtooth");
    assert!((bass.code.mod_params.fm - 4.0).abs() < 1e-5);
    assert!((bass.code.mod_params.fmh - 1.0).abs() < 1e-5);
    assert!((bass.code.mod_params.fm_decay - 0.25).abs() < 1e-5);
    assert!((bass.code.mod_params.fm_sustain - 0.2).abs() < 1e-5);
    let bass_lpf = bass.code.filter.lpf.expect("bass needs lpf(400)");
    assert!((bass_lpf - 400.0).abs() < 1e-3);
    assert!((bass.code.mod_params.lpenv - 3.0).abs() < 1e-5);
    let bass_scale = bass
        .code
        .scale
        .as_ref()
        .expect("bass needs .scale")
        .at_cycle(0);
    assert_eq!(bass_scale.root_midi, 36, "C2");

    let lead = track(&song, "lead");
    assert_eq!(lead.code.sound, "sine");
    assert!((lead.code.mod_params.fm - 3.0).abs() < 1e-5);
    assert!((lead.code.mod_params.fmh - 2.0).abs() < 1e-5);
    assert!((lead.code.mod_params.fm_attack - 0.01).abs() < 1e-5);
    assert!((lead.code.mod_params.fm_decay - 0.3).abs() < 1e-5);
    assert!((lead.code.mod_params.fm_sustain - 0.25).abs() < 1e-5);
    let lead_lpf = lead.code.filter.lpf.expect("lead needs lpf(1800)");
    assert!((lead_lpf - 1800.0).abs() < 1e-3);
    assert!((lead.code.mod_params.lpenv - 2.0).abs() < 1e-5);
    assert!(
        lead.code.mini_src.contains("4 2 0 2"),
        "lead degrees 4 2 0 2 (5–b3–1–b3): {}",
        lead.code.mini_src
    );
    let lead_scale = lead
        .code
        .scale
        .as_ref()
        .expect("lead needs .scale")
        .at_cycle(0);
    assert_eq!(lead_scale.root_midi, 60, "C4");

    let pad = track(&song, "pad");
    assert_eq!(pad.code.sound, "sine");
    assert!((pad.code.mod_params.fm - 1.2).abs() < 1e-5);
    assert!((pad.code.mod_params.fmh - 1.0).abs() < 1e-5);
    assert!((pad.code.mod_params.fm_decay - 0.8).abs() < 1e-5);
    assert!((pad.code.mod_params.fm_sustain - 0.4).abs() < 1e-5);
    assert!(pad.code.room > 0.0);
    assert_eq!(pad.code.orbit, 2);
    let pad_scale = pad
        .code
        .scale
        .as_ref()
        .expect("pad needs .scale")
        .at_cycle(0);
    assert_eq!(pad_scale.root_midi, 60, "C4");

    for t in &song.tracks {
        assert!(
            t.code.compressor.is_none(),
            "no track compressor: {}",
            t.name
        );
    }

    let bank = if samples_available() {
        load_bank()
    } else {
        SampleBank::empty()
    };
    let bpm = 124.0;
    let mut e = Engine::new(SR, bpm);
    e.push_command(Command::LoadSong {
        deck: 0,
        song: Box::new(song),
    });
    let _ = process_bars(&mut e, &bank, 1, bpm);
    let buf = process_bars(&mut e, &bank, 2, bpm);
    assert_finite_bounded(&buf, "skill-fm-sound-design");
    assert!(
        has_energy(&buf, 0.001),
        "FM skill song should sound, peak={}",
        peak(&buf)
    );
    assert!(
        clip_rail_ratio(&buf) < 0.05,
        "FM skill song clip rail: {}",
        clip_rail_ratio(&buf)
    );
}
