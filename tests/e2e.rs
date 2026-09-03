//! Integration tests: bundled genre songs + DJ xfade (NullBackend / Engine::process).
//!
//! Device audio is not required. Samples are optional for synth-only paths.

use std::fs;
use std::path::PathBuf;

use strudel_rs::engine::{Command, Engine};
use strudel_rs::mixer::{FillKind, MixAction, MixCommand, MixGrid};
use strudel_rs::sample::{SampleBank, SAMPLE_ROOT_HZ};
use strudel_rs::song::parse_song;

const SR: u32 = 48_000;
/// Shared clock for the exhibit DJ pair (house-01 + four-on-the-floor-01).
const DJ_BPM: f64 = 124.0;

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
    // 1 bar = 4 beats; samples = sr * 60 / bpm * 4.
    // Ceil so a process() call always crosses the bar head (124 BPM is
    // 92903.23 samples/bar; rounding down leaves LoadSong pending).
    (SR as f64 * 60.0 / bpm * 4.0).ceil() as usize
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

fn render_song(name: &str, bpm: f64, bank: &SampleBank, label: &str) -> Vec<f32> {
    let song = load_song_file(name);
    let mut e = Engine::new(SR, bpm);
    e.push_command(Command::LoadSong {
        deck: 0,
        song: Box::new(song),
    });
    let _ = process_bars(&mut e, bank, 1, bpm);
    let buf = process_bars(&mut e, bank, 2, bpm);
    assert_finite_bounded(&buf, label);
    buf
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
        "expected at least house-01 + four-on-the-floor-01, got {count}"
    );
}

#[test]
fn showcase_songs_use_task23_features() {
    let techno = fs::read_to_string(songs_dir().join("techno-duck-01.strudel")).unwrap();
    assert!(techno.contains("fm(") || techno.contains(".fm("));
    assert!(techno.contains("duckorbit("));
    assert!(
        techno.contains("orbit(2)"),
        "bass and pad must share the ducked orbit"
    );
    assert!(
        !techno.contains("compressor("),
        "techno-duck-01 must not set master compressor from a track"
    );

    let house = fs::read_to_string(songs_dir().join("house-01.strudel")).unwrap();
    assert!(
        house.contains(".scale(") || house.contains("scale("),
        "house-01 should use .scale(...) with degree patterns"
    );
}

#[test]
fn four_on_the_floor_sounds_without_samples() {
    let song = load_song_file("four-on-the-floor-01.strudel");
    assert_eq!(song.title, "sine-pulse");
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
    assert_finite_bounded(&buf, "four-on-the-floor-01");
    assert!(
        has_energy(&buf, 0.001),
        "four-on-the-floor-01 should sound without SampleBank, peak={}",
        peak(&buf)
    );
    assert!(
        clip_rail_ratio(&buf) < 0.05,
        "four-on-the-floor-01 heavily at clip rail: {}",
        clip_rail_ratio(&buf)
    );
}

#[test]
fn four_on_the_floor_01_grid() {
    let song = load_song_file("four-on-the-floor-01.strudel");
    assert_eq!(song.title, "sine-pulse");
    assert!((song.bpm.unwrap() - 124.0).abs() < 1e-6);
    let drums = track(&song, "drums");
    let drums_src = &drums.code.mini_src;
    assert!(drums_src.contains("bd*4"), "{drums_src}");
    assert!(drums_src.contains("[~ hh]*4"), "{drums_src}");
    assert!(
        !drums_src.contains("sd"),
        "techno four-on-the-floor must not use a house snare backbeat: {drums_src}"
    );
    if !samples_available() {
        eprintln!("skip four_on_the_floor_01 render: samples/ not found");
        return;
    }
    let buf = render_song(
        "four-on-the-floor-01.strudel",
        124.0,
        &load_bank(),
        "four-on-the-floor-01",
    );
    assert!(
        has_energy(&buf, 0.001),
        "four-on-the-floor-01 should sound, peak={}",
        peak(&buf)
    );
    assert!(
        clip_rail_ratio(&buf) < 0.05,
        "four-on-the-floor-01 clip rail: {}",
        clip_rail_ratio(&buf)
    );
}

#[test]
fn dnb_01_mix_rules() {
    let song = load_song_file("dnb-01.strudel");
    assert_eq!(song.title, "dnb-01");
    assert!((song.bpm.unwrap() - 174.0).abs() < 1e-6);
    let drums = track(&song, "drums");
    let bass = track(&song, "bass");
    let lead = track(&song, "lead");
    assert!(
        drums.code.gain > bass.code.gain,
        "drums {} must sit above bass {}",
        drums.code.gain,
        bass.code.gain
    );
    assert_eq!(bass.code.sound, "square");
    assert_eq!(lead.code.sound, "sawtooth");
    let mid_lpf = lead.code.filter.lpf.expect("saw mid needs lpf");
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
    let evs = strudel_rs::mini::events(&drums.code.pattern, 0);
    assert!(
        evs.iter().any(|e| e.start >= 0.5),
        "break should occupy the second half of the bar, starts={:?}",
        evs.iter().map(|e| e.start).collect::<Vec<_>>()
    );
}

#[test]
fn dnb_01_sounds_with_samples() {
    if !samples_available() {
        eprintln!("skip dnb_01: samples/ not found");
        return;
    }
    let bank = load_bank();
    assert!(
        !bank.has("db"),
        "db must not resolve — that atom is silence"
    );
    let buf = render_song("dnb-01.strudel", 174.0, &bank, "dnb-01");
    assert!(has_energy(&buf, 0.001), "dnb-01 peak={}", peak(&buf));
    assert!(clip_rail_ratio(&buf) < 0.05, "dnb-01 clip");
}

#[test]
fn techno_duck_01_mix_rules() {
    let song = load_song_file("techno-duck-01.strudel");
    assert_eq!(song.title, "pump-core");
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
        "techno duck must not use a house backbeat: {}",
        hats.code.mini_src
    );
    assert!(
        hats.code.mini_src.contains("hh"),
        "techno hats should be [~ hh]*4"
    );
    assert!(song.tracks.iter().all(|t| t.code.compressor.is_none()));
}

#[test]
fn techno_duck_01_sounds_with_samples() {
    if !samples_available() {
        eprintln!("skip techno_duck_01: samples/ not found");
        return;
    }
    let buf = render_song(
        "techno-duck-01.strudel",
        126.0,
        &load_bank(),
        "techno-duck-01",
    );
    assert!(
        has_energy(&buf, 0.001),
        "techno-duck-01 peak={}",
        peak(&buf)
    );
    assert!(clip_rail_ratio(&buf) < 0.08, "techno-duck-01 clip");
}

#[test]
fn dj_xfade_house_to_four_on_the_floor() {
    let bank = if samples_available() {
        load_bank()
    } else {
        SampleBank::empty()
    };

    let house = load_song_file("house-01.strudel");
    let four = load_song_file("four-on-the-floor-01.strudel");
    assert!(
        (house.bpm.unwrap() - four.bpm.unwrap()).abs() < 1e-6,
        "DJ pair must share one BPM, got {:?} vs {:?}",
        house.bpm,
        four.bpm
    );
    let mut e = Engine::new(SR, DJ_BPM);
    let bl = bar_len(DJ_BPM);

    e.push_command(Command::LoadSong {
        deck: 0,
        song: Box::new(house),
    });
    let mut all = Vec::new();
    all.extend(process_n(&mut e, &bank, bl));
    let buf = process_n(&mut e, &bank, bl);
    assert_finite_bounded(&buf, "after load A");
    all.extend(&buf);
    if e.decks[0].song_title().is_none() {
        let buf = process_n(&mut e, &bank, bl);
        assert_finite_bounded(&buf, "extra bar for load A");
        all.extend(&buf);
    }
    assert_eq!(e.decks[0].song_title(), Some("warehouse-intro"));
    assert!((e.mixer.gain_a - 1.0).abs() < 1e-5);

    e.push_command(Command::LoadSong {
        deck: 1,
        song: Box::new(four),
    });
    e.push_command(Command::XFade {
        to_deck: 1,
        bars: 4,
    });

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
        Some("sine-pulse"),
        "deck B should be loaded"
    );
    assert!(
        e.mixer.xfade().is_some() || e.mixer.gain_b > 0.0,
        "xfade should have started or completed, a={} b={}",
        e.mixer.gain_a,
        e.mixer.gain_b
    );

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
        Some("warehouse-intro"),
        "source deck should stay loaded after xfade"
    );
    assert_eq!(e.decks[1].song_title(), Some("sine-pulse"));

    let buf = process_bars(&mut e, &bank, 2, DJ_BPM);
    assert_finite_bounded(&buf, "post-xfade four-on-the-floor");
    assert!(
        has_energy(&buf, 0.001),
        "four-on-the-floor on B should sound after xfade, peak={}",
        peak(&buf)
    );
    assert!(
        clip_rail_ratio(&all) < 0.08,
        "session heavily at clip rail: {}",
        clip_rail_ratio(&all)
    );
}

#[test]
fn dj_echo_fill_house_to_four_on_the_floor() {
    let bank = if samples_available() {
        load_bank()
    } else {
        SampleBank::empty()
    };

    let house = load_song_file("house-01.strudel");
    let four = load_song_file("four-on-the-floor-01.strudel");
    let mut e = Engine::new(SR, DJ_BPM);
    let bl = bar_len(DJ_BPM);

    e.push_command(Command::LoadSong {
        deck: 0,
        song: Box::new(house),
    });
    let mut all = Vec::new();
    all.extend(process_n(&mut e, &bank, bl));
    let buf = process_n(&mut e, &bank, bl);
    assert_finite_bounded(&buf, "after load A");
    all.extend(&buf);
    if e.decks[0].song_title().is_none() {
        let buf = process_n(&mut e, &bank, bl);
        assert_finite_bounded(&buf, "extra bar for load A");
        all.extend(&buf);
    }
    assert_eq!(e.decks[0].song_title(), Some("warehouse-intro"));

    e.push_command(Command::LoadSong {
        deck: 1,
        song: Box::new(four),
    });
    e.push_command(Command::Mix(MixCommand {
        action: MixAction::Fill,
        to_deck: 1,
        bars: 1,
        eq: true,
        reset_eq: true,
        fill: Some(FillKind::Echo),
        grid: MixGrid::Eighth,
        mute_track: None,
        phrase: 1,
    }));

    let buf = process_n(&mut e, &bank, bl);
    assert_finite_bounded(&buf, "pending apply");
    all.extend(&buf);
    if e.decks[1].song_title().is_none() {
        let buf = process_n(&mut e, &bank, bl);
        assert_finite_bounded(&buf, "extra bar for B title");
        all.extend(&buf);
    }
    assert_eq!(e.decks[1].song_title(), Some("sine-pulse"));
    if !e.mixer.has_mix_job() {
        let buf = process_n(&mut e, &bank, bl);
        assert_finite_bounded(&buf, "extra bar for fill start");
        all.extend(&buf);
    }

    for i in 0..4 {
        let buf = process_n(&mut e, &bank, bl);
        assert_finite_bounded(&buf, &format!("echo fill bar {i}"));
        all.extend(&buf);
    }

    assert!(
        (e.mixer.gain_b - 1.0).abs() < 1e-2,
        "to gain should be 1, got {}",
        e.mixer.gain_b
    );

    let buf = process_n(&mut e, &bank, bl);
    assert_finite_bounded(&buf, "post-echo four-on-the-floor");
    all.extend(&buf);
    assert!(
        has_energy(&buf, 0.001),
        "four-on-the-floor on B should sound after echo fill, peak={}",
        peak(&buf)
    );
    assert!(
        clip_rail_ratio(&all) < 0.08,
        "session heavily at clip rail: {}",
        clip_rail_ratio(&all)
    );
}

#[test]
fn acid_01_filter_envelope() {
    let path = songs_dir().join("acid-01.strudel");
    let text = fs::read_to_string(&path).unwrap();
    assert!(
        text.contains("lpenv(3)") && !text.contains("lpenv(3.5)"),
        "acid-01 must use lpenv(3), not the 10 kHz 3.5 ceiling"
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

    let song = load_song_file("acid-01.strudel");
    assert_eq!(song.title, "saw-303");
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
    let buf = render_song("acid-01.strudel", bpm, &bank, "acid-01");
    assert!(
        has_energy(&buf, 0.001),
        "acid-01 should sound, peak={}",
        peak(&buf)
    );
    assert!(
        clip_rail_ratio(&buf) < 0.05,
        "acid-01 heavily at clip rail: {}",
        clip_rail_ratio(&buf)
    );
}

#[test]
fn house_01_clap_backbeat() {
    let path = songs_dir().join("house-01.strudel");
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
        text.contains("plk:lp"),
        "pluck key is plk:lp (underscore): {text}"
    );
    assert!(
        text.contains("C4:minor") && !text.contains("C3:minor"),
        "C3 sample must use C4:minor, not C3:minor: {text}"
    );
    assert!(
        text.contains("cut(1)"),
        "pluck one-shot needs cut(1): {text}"
    );
    assert!(
        !text.contains("duckorbit") && !text.contains("compressor("),
        "no duck / track compressor in this recipe: {text}"
    );

    let song = load_song_file("house-01.strudel");
    assert_eq!(song.title, "warehouse-intro");
    assert!(
        song.bpm.is_some() && (song.bpm.unwrap() - 124.0).abs() < 1e-6,
        "expected 124 BPM, got {:?}",
        song.bpm
    );

    let drums = track(&song, "drums");
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

    let lead = track(&song, "lead");
    assert_eq!(lead.code.sound, "plk:lp");
    assert_eq!(lead.code.cut, Some(1));
    assert!(lead.code.compressor.is_none());
    let scale = lead
        .code
        .scale
        .as_ref()
        .expect("lead needs .scale")
        .at_cycle(0);
    assert_eq!(scale.root_midi, 60, "C4");
    assert_eq!(scale.intervals, vec![0, 2, 3, 5, 7, 8, 10]);

    if !samples_available() {
        eprintln!("skip house_01 render: samples/ not found");
        return;
    }
    let bank = load_bank();
    if bank.get_stem("bd", "hf").is_none() || bank.get_stem("plk", "lp").is_none() {
        eprintln!("skip house_01 render: factory stems not loaded (LFS?)");
        return;
    }
    let buf = render_song("house-01.strudel", 124.0, &bank, "house-01");
    assert!(
        has_energy(&buf, 0.001),
        "house-01 should sound, peak={}",
        peak(&buf)
    );
    assert!(
        clip_rail_ratio(&buf) < 0.05,
        "house-01 clip rail: {}",
        clip_rail_ratio(&buf)
    );
}

#[test]
fn dnb_reese_01_mid_glue() {
    let path = songs_dir().join("dnb-reese-01.strudel");
    let text = fs::read_to_string(&path).unwrap();
    assert!(
        !text.contains(".fast("),
        "method .fast squeezes the break into the first half: {text}"
    );
    assert!(
        text.contains(r#"scale("C2:minor").s("square")"#) || text.contains(r#".s("square")"#),
        "square sub stays C2:minor: {text}"
    );
    assert!(
        text.contains(r#"scale("C4:minor").s("bs:rm")"#) || text.contains(r#".s("bs:rm")"#),
        "bs:rm must stay C4:minor: {text}"
    );
    assert!(
        !text.contains("duckorbit") && !text.contains("compressor("),
        "no duck / track compressor in this recipe: {text}"
    );
    assert!(
        !text.contains(" cp") && !text.contains("\"cp") && !text.contains("[~ cp]"),
        "no house clap on this grid: {text}"
    );

    let song = load_song_file("dnb-reese-01.strudel");
    assert_eq!(song.title, "dnb-reese-01");
    assert!(
        song.bpm.is_some() && (song.bpm.unwrap() - 174.0).abs() < 1e-6,
        "expected 174 BPM, got {:?}",
        song.bpm
    );

    let drums = track(&song, "drums");
    let bass = track(&song, "bass");
    let lead = track(&song, "lead");
    assert!(
        drums.code.gain > bass.code.gain,
        "drums {} must sit above bass {}",
        drums.code.gain,
        bass.code.gain
    );
    assert_eq!(bass.code.sound, "square");
    let sub_lpf = bass.code.filter.lpf.expect("square sub needs lpf");
    assert!((sub_lpf - 120.0).abs() < 1e-3);
    let sub_scale = bass
        .code
        .scale
        .as_ref()
        .expect("sub needs .scale")
        .at_cycle(0);
    assert_eq!(sub_scale.root_midi, 36, "C2");

    assert_eq!(lead.code.sound, "bs:rm");
    let mid_scale = lead
        .code
        .scale
        .as_ref()
        .expect("mid needs .scale")
        .at_cycle(0);
    assert_eq!(mid_scale.root_midi, 60, "C4 — C2 dumps the 800–1200 band");

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

    let evs = strudel_rs::mini::events(&drums.code.pattern, 0);
    assert!(
        evs.iter().any(|e| e.start >= 0.5),
        "break should occupy the second half of the bar, starts={:?}",
        evs.iter().map(|e| e.start).collect::<Vec<_>>()
    );

    if !samples_available() {
        eprintln!("skip dnb_reese_01 render: samples/ not found");
        return;
    }
    let buf = render_song("dnb-reese-01.strudel", 174.0, &load_bank(), "dnb-reese-01");
    assert!(
        has_energy(&buf, 0.001),
        "dnb-reese-01 should sound, peak={}",
        peak(&buf)
    );
    assert!(
        clip_rail_ratio(&buf) < 0.05,
        "dnb-reese-01 clip rail: {}",
        clip_rail_ratio(&buf)
    );
}

#[test]
fn sample_root_hz_is_c4_not_recorded_octave() {
    // SAMPLE_ROOT_HZ comment says C3; the value is C4. Ratio 1.0 = native wav pitch.
    assert!((SAMPLE_ROOT_HZ - 261.6256).abs() < 1e-4);
    assert!(
        (130.8128 / SAMPLE_ROOT_HZ - 0.5).abs() < 1e-3,
        "C3 dumps an octave"
    );
    assert!(
        (65.4064 / SAMPLE_ROOT_HZ - 0.25).abs() < 1e-3,
        "C2 dumps two octaves"
    );
}

fn factory_pcm_bank_ready(bank: &SampleBank) -> bool {
    bank.has("bd")
        && bank.has("cp")
        && bank.has("hh")
        && bank.get_stem("bs", "hf").is_some()
        && bank.get_stem("pf", "ff").is_some()
        && bank.get_stem("bs", "dk").is_some()
        && bank.get_stem("fx", "up").is_some()
        && bank.get_stem("ld", "ss").is_some()
}

#[test]
fn factory_pcm_flat_stems_resolve_exactly() {
    if !samples_available() {
        eprintln!("skip factory_pcm stems: samples/ not found");
        return;
    }
    let bank = load_bank();
    if !factory_pcm_bank_ready(&bank) {
        eprintln!("skip factory_pcm stems: factory wavs not loaded (LFS?)");
        return;
    }
    assert!(bank.get_stem("bs", "hf").is_some());
    assert!(bank.get_stem("bs", "su").is_some());
    assert!(bank.get_stem("bs", "dk").is_some());
    assert!(bank.get_stem("pf", "ff").is_some());
    assert!(bank.get_stem("ld", "ss").is_some());
    assert!(bank.get_stem("fx", "up").is_some());
    assert!(bank.get_stem("fx", "nr").is_some());
    assert!(bank.get_stem("fx", "id").is_some());
    assert!(bank.get_stem("fx", "sd").is_some());
    assert!(bank.has("bs"), "folder key is the part, not part:slug");
    assert!(!bank.has("bass-fm_house"), "old flat stem is gone");
    assert!(
        !bank.has("bass-fm-house"),
        "old hyphen variant never resolved"
    );
    assert!(!bank.has("reese-dark"));
    assert!(!bank.has("reese_dark"));
    assert!(!bank.has("pad-fm-fifth"));
    assert!(!bank.has("fx-riser-noise"));
    assert!(!bank.has("lead_supersaw"));
}

#[test]
fn catalog_sketches_parse_and_render() {
    const FAMILIES: &[&str] = &["night-market", "glass-garden", "clockwork", "tide-lantern"];
    let dir = songs_dir();
    let mut files: Vec<(String, String)> = Vec::new();
    for family in FAMILIES {
        let mut n = 0usize;
        for entry in fs::read_dir(&dir).unwrap() {
            let name = entry.unwrap().file_name().to_string_lossy().into_owned();
            if name.starts_with(family) && name.ends_with(".strudel") {
                n += 1;
                files.push((name, family.to_string()));
            }
        }
        assert!(n >= 6, "{family} expected at least 6 songs, got {n}");
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    for (file, family) in &files {
        let song = load_song_file(file);
        assert!(
            !song.title.ends_with(".strudel"),
            "{file} needs @title, got {}",
            song.title
        );
        assert!(
            (song.bpm.unwrap() - 96.0).abs() < 1e-6,
            "{file} should share 96 BPM for dj pairing, got {:?}",
            song.bpm
        );
        assert!(
            song.tracks.len() >= 3 && song.tracks.len() <= 5,
            "{file} track count {}",
            song.tracks.len()
        );
        if file == &format!("{family}-01.strudel") {
            assert_eq!(song.title, *family, "{file}");
        }
    }

    if !samples_available() {
        eprintln!("skip catalog_sketches render: samples/ not found");
        return;
    }
    let bank = load_bank();
    if bank.get_stem("perc", "cv").is_none()
        || bank.get_stem("perc", "tk").is_none()
        || bank.get_stem("tom", "lo").is_none()
    {
        eprintln!("skip catalog_sketches render: perc/tom wavs not loaded (LFS?)");
        return;
    }
    for (file, _) in &files {
        let buf = render_song(file, 96.0, &bank, file);
        assert!(
            has_energy(&buf, 0.001),
            "{file} should sound, peak={}",
            peak(&buf)
        );
        assert!(
            clip_rail_ratio(&buf) < 0.05,
            "{file} clip rail: {}",
            clip_rail_ratio(&buf)
        );
    }
}
