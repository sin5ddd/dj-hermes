# Task 10–16: Song / Deck / Engine / Mixer / Watcher / REPL

> 正本: [../2026-07-27_020000-strudel-rs-final.md](../2026-07-27_020000-strudel-rs-final.md)

状態: **完了**（正本 Progress の [x] から切り出し）

---

### Task 10: 曲ファイルフォーマット `song.rs`

**Objective:** 複数トラック + bpm ヘッダの曲ファイルをパース。

**Files:**
- Create: `strudel-rs/src/song.rs`

**Step 1: コード**

```rust
use crate::code::{parse_code, PatternCode};

#[derive(Debug, Clone)]
pub struct Track {
    pub name: String,
    pub code: PatternCode,
    pub muted: bool,
}

#[derive(Debug, Clone)]
pub struct Song {
    pub title: String,
    pub bpm: Option<f64>,
    pub tracks: Vec<Track>,
    pub path: String,          // ホットリロード用
}

/// フォーマット:
///   // コメント
///   bpm: 126
///   title: My Song        (省略可, 省略時はファイル名)
///   ---
///   kick: s("bd*4").gain(0.9)
///   bass: note("c2 eb2 g2").s("sawtooth").lpf(400)
pub fn parse_song(text: &str, path: &str) -> Result<Song, String> {
    let mut bpm = None;
    let mut title: Option<String> = None;
    let mut tracks = Vec::new();
    let mut in_header = true;

    for (lineno, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") || line.starts_with('#') { continue; }
        if in_header {
            if line == "---" { in_header = false; continue; }
            if let Some(v) = line.strip_prefix("bpm:") {
                bpm = Some(v.trim().parse().map_err(|_| format!("line {}: bad bpm", lineno + 1))?);
                continue;
            }
            if let Some(v) = line.strip_prefix("title:") {
                title = Some(v.trim().to_string());
                continue;
            }
            // ヘッダ以外の行が来たらトラック開始とみなす（--- 省略可）
            in_header = false;
        }
        let (name, code_str) = line.split_once(':')
            .ok_or_else(|| format!("line {}: expected 'name: code'", lineno + 1))?;
        let name = name.trim().to_string();
        if name.is_empty() { return Err(format!("line {}: empty track name", lineno + 1)); }
        let code = parse_code(code_str.trim())
            .map_err(|e| format!("line {} ({}): {}", lineno + 1, name, e))?;
        tracks.push(Track { name, code, muted: false });
    }
    if tracks.is_empty() { return Err("no tracks".into()); }
    Ok(Song {
        title: title.unwrap_or_else(|| path.rsplit('/').next().unwrap_or(path).to_string()),
        bpm, tracks, path: path.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_song() {
        let text = r#"
// demo
bpm: 126
title: Demo
---
kick: s("bd*4").gain(0.9)
bass: note("c2 eb2 g2 bb2").s("sawtooth").lpf(400).gain(0.7)
"#;
        let s = parse_song(text, "songs/demo.strudel").unwrap();
        assert_eq!(s.title, "Demo");
        assert_eq!(s.bpm, Some(126.0));
        assert_eq!(s.tracks.len(), 2);
        assert_eq!(s.tracks[0].name, "kick");
    }
    #[test]
    fn reports_error_line() {
        let err = parse_song("---\nbad: nope(\"x\")", "t").unwrap_err();
        assert!(err.contains("line 2"), "{err}");
    }
}
```

**Step 2: 検証**

Run: `cargo test song`
Expected: 2 passed

**Step 3: Commit**

```bash
git add -A && git commit -m "task 10: song file format parser"
```

---

---

### Task 11: Deck（トラック集合のスケジューリング + デッキゲイン）

**Objective:** 1 デッキ = Song の全トラックを鳴らすユニット。ボイスプール + デッキゲイン（フェード用）。

**Files:**
- Create: `strudel-rs/src/deck.rs`

**Step 1: コード**

```rust
use crate::code::note_to_hz;
use crate::mini;
use crate::sample::SampleBank;
use crate::song::Song;
use crate::synth::{SampleVoice, SynthVoice, VoiceKind, Wave};
use crate::transport::Transport;

pub const MAX_VOICES: usize = 32;

/// 1イベント分の発音指示
struct ScheduledHit {
    at_sample: u64,
    len_samples: u64,
    sound: String,
    freq: f32,
    gain: f32,
    lpf: Option<f32>,
}

pub struct Deck {
    pub name: String,           // "A" / "B"
    song: Option<Song>,
    pub gain: f32,              // クロスフェード用 0..1
    voices: Vec<Option<VoiceKind>>,
    scheduled: Vec<ScheduledHit>,
    next_event: usize,
    scheduled_bar: u64,
}

impl Deck {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(), song: None, gain: 1.0,
            voices: vec![None; MAX_VOICES],
            scheduled: Vec::with_capacity(512),
            next_event: 0, scheduled_bar: u64::MAX,
        }
    }

    pub fn load(&mut self, song: Song) {
        self.song = Some(song);
        self.scheduled_bar = u64::MAX;   // 強制再スケジュール
    }
    pub fn unload(&mut self) {
        self.song = None;
        self.voices.iter_mut().for_each(|v| *v = None);
    }
    pub fn song_title(&self) -> Option<&str> { self.song.as_ref().map(|s| s.title.as_str()) }

    /// 指定バーのイベントを前計算（1 bar = 1 cycle）
    fn schedule_bar(&mut self, bar: u64, transport: &Transport) {
        self.scheduled.clear();
        self.next_event = 0;
        if let Some(song) = &self.song {
            let spb = transport.samples_per_bar();
            let bar_start = (bar as f64 * spb) as u64;
            for track in &song.tracks {
                if track.muted { continue; }
                let pc = &track.code;
                for ev in mini::events(&pc.pattern, bar) {
                    let at = bar_start + (ev.start * spb / pc.speed.max(1e-6)) as u64;
                    let len = ((ev.dur * spb / pc.speed.max(1e-6)) as u64).max(64);
                    let freq = if pc.is_note {
                        match note_to_hz(&ev.value) { Ok(h) => h, Err(_) => continue }
                    } else { 0.0 };
                    self.scheduled.push(ScheduledHit {
                        at_sample: at, len_samples: len,
                        sound: if pc.is_note { pc.sound.clone() } else { ev.value.clone() },
                        freq, gain: pc.gain, lpf: pc.lpf,
                    });
                }
            }
            self.scheduled.sort_by_key(|e| e.at_sample);
        }
        self.scheduled_bar = bar;
    }

    /// オーディオコールバック本体。samples バンクは発音時に参照。
    pub fn process(&mut self, out: &mut [f32], transport: &Transport, samples: &SampleBank) {
        let sr = transport.sample_rate as f32;
        let bar = transport.bar_index();
        if bar != self.scheduled_bar { self.schedule_bar(bar, transport); }

        for (i, frame) in out.iter_mut().enumerate() {
            let now = transport.global_sample + i as u64;
            while self.next_event < self.scheduled.len()
                && self.scheduled[self.next_event].at_sample <= now
            {
                let hit = &self.scheduled[self.next_event];
                self.next_event += 1;
                if let Some(slot) = self.voices.iter_mut().find(|v| v.is_none()) {
                    *slot = if let Some(wave) = Wave::from_str(&hit.sound) {
                        Some(VoiceKind::Synth(SynthVoice::new(wave, hit.freq, hit.gain, hit.len_samples, hit.lpf)))
                    } else if let Some(data) = samples.get(&hit.sound) {
                        Some(VoiceKind::Sample(SampleVoice::new(
                            std::sync::Arc::new(data.clone()), hit.gain)))
                    } else {
                        None   // 未知の音名/サンプル名は捨てる
                    };
                }
            }
            let mut mix = 0f32;
            for v in self.voices.iter_mut() {
                if let Some(voice) = v {
                    let s = match voice {
                        VoiceKind::Synth(sv) => sv.next_sample(sr),
                        VoiceKind::Sample(pv) => pv.next_sample(),
                    };
                    match s { Some(x) => mix += x, None => *v = None }
                }
            }
            *frame = mix * self.gain;
        }
    }
}
```

**Step 2: 検証**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::song::parse_song;
    use std::path::Path;

    #[test]
    fn deck_renders_song() {
        let song = parse_song("---\nbass: note(\"c3 e3 g3\").s(\"sawtooth\")", "t").unwrap();
        let mut d = Deck::new("A");
        d.load(song);
        let t = Transport::new(48000, 120.0);
        let bank = SampleBank::load_dir(Path::new("samples"), 48000);
        let mut buf = vec![0f32; 48000];
        d.process(&mut buf, &t, &bank);
        assert!(buf.iter().any(|s| s.abs() > 0.01));
    }
}
```

Run: `cargo test deck`
Expected: 1 passed

**Step 3: Commit**

```bash
git add -A && git commit -m "task 11: deck with track scheduling"
```

---

---

### Task 12: Engine 統合（Command + バー境界 pending 適用）

**Objective:** 全入力を Command として受け、バー境界で遷移を適用する中核。まず単一デッキで完結させる。

**Files:**
- Create: `strudel-rs/src/engine.rs`

**Step 1: コード**

```rust
use crate::deck::Deck;
use crate::sample::SampleBank;
use crate::song::Song;
use crate::transport::Transport;

pub enum Command {
    LoadSong { deck: usize, song: Song },        // バー境界で適用
    UnloadDeck { deck: usize },
    SetBpm(f64),                                  // 次バー頭から適用
    Hush,                                         // 即時（全デッキ unload）
    XFade { to_deck: usize, bars: u32 },          // 次バー頭から bars バーかけてフェード
    SetTrackMute { deck: usize, track: String, muted: bool }, // 次バー頭から
}

/// バー境界で適用待ちの遷移
enum Pending {
    LoadSong { deck: usize, song: Song },
    SetBpm(f64),
    XFade { to_deck: usize, bars: u32 },
    TrackMute { deck: usize, track: String, muted: bool },
}

pub struct Engine {
    pub transport: Transport,
    pub decks: [Deck; 2],
    pending: Vec<Pending>,
    xfade: Option<XFadeState>,
}

struct XFadeState {
    to_deck: usize,
    start_sample: u64,
    end_sample: u64,
}

impl Engine {
    pub fn new(sample_rate: u32, bpm: f64) -> Self {
        Self {
            transport: Transport::new(sample_rate, bpm),
            decks: [Deck::new("A"), Deck::new("B")],
            pending: Vec::new(),
            xfade: None,
        }
    }

    pub fn push_command(&mut self, cmd: Command) {
        match cmd {
            Command::Hush => {                       // hush だけ即時
                self.decks[0].unload();
                self.decks[1].unload();
                self.xfade = None;
            }
            Command::UnloadDeck { deck } => self.decks[deck].unload(),
            Command::LoadSong { deck, song } => self.pending.push(Pending::LoadSong { deck, song }),
            Command::SetBpm(b) => self.pending.push(Pending::SetBpm(b)),
            Command::XFade { to_deck, bars } => self.pending.push(Pending::XFade { to_deck, bars }),
            Command::SetTrackMute { deck, track, muted } =>
                self.pending.push(Pending::TrackMute { deck, track, muted }),
        }
    }

    /// バー境界をまたいだか判定し、pending を適用する。
    /// audio バッファ処理の先頭で呼ぶ（厳密にはバッファ内で境界をまたぐ場合も
    /// バッファ先頭で適用する。バッファ 256 samples ≒ 5ms なので体感問題なし）。
    fn apply_pending_at_bar_boundary(&mut self) {
        let bar_now = self.transport.bar_index();
        // 前回呼び出し時とバーが変わっていたら境界通過
        // （engine は last_bar を内部保持）
        if self.last_bar() == bar_now { return; }
        self.set_last_bar(bar_now);

        let pending = std::mem::take(&mut self.pending);
        for p in pending {
            match p {
                Pending::LoadSong { deck, song } => self.decks[deck].load(song),
                Pending::SetBpm(b) => self.transport.set_bpm(b),
                Pending::TrackMute { deck, track, muted } => {
                    if let Some(song) = self.decks[deck].song_mut() {
                        for t in &mut song.tracks {
                            if t.name == track { t.muted = muted; }
                        }
                    }
                }
                Pending::XFade { to_deck, bars } => {
                    let start = self.transport.global_sample;
                    let len = (bars as f64 * self.transport.samples_per_bar()) as u64;
                    self.xfade = Some(XFadeState { to_deck, start_sample: start, end_sample: start + len });
                }
            }
        }
    }

    // last_bar の保持（簡潔さのためフィールドとして定義すること。実装時に追加）
    fn last_bar(&self) -> u64 { self.last_bar }
    fn set_last_bar(&mut self, b: u64) { self.last_bar = b; }

    pub fn process(&mut self, out: &mut [f32], samples: &SampleBank) {
        self.apply_pending_at_bar_boundary();

        // クロスフェードゲイン計算（バッファ先頭の値で代表）
        if let Some(x) = &self.xfade {
            let t = ((self.transport.global_sample.saturating_sub(x.start_sample)) as f64
                / (x.end_sample - x.start_sample) as f64).clamp(0.0, 1.0);
            let theta = t * std::f64::consts::FRAC_PI_2;
            let (from, to) = if x.to_deck == 1 { (0, 1) } else { (1, 0) };
            self.decks[from].gain = theta.cos() as f32;
            self.decks[to].gain = theta.sin() as f32;
            if t >= 1.0 {
                self.decks[from].unload();   // フェード完了で旧デッキ停止
                self.decks[from].gain = 0.0;
                self.decks[to].gain = 1.0;
                self.xfade = None;
            }
        }

        let mut a = vec![0f32; out.len()];
        let mut b = vec![0f32; out.len()];
        self.decks[0].process(&mut a, &self.transport, samples);
        self.decks[1].process(&mut b, &self.transport, samples);
        for i in 0..out.len() { out[i] = (a[i] + b[i]).clamp(-1.0, 1.0); }
        self.transport.advance(out.len());
    }
}
```

※ `last_bar: u64` フィールドと `Deck::song_mut()` を Deck に追加すること（実装時の追随変更）。

**Step 2: 検証**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::song::parse_song;

    fn test_song(n: &str) -> Song {
        parse_song(&format!("---\nb: note(\"{n}\").s(\"sawtooth\")"), "t").unwrap()
    }

    #[test]
    fn song_loads_and_plays_after_bar_boundary() {
        let mut e = Engine::new(48000, 120.0);
        let bank = SampleBank::load_dir(std::path::Path::new("samples"), 48000);
        e.push_command(Command::LoadSong { deck: 0, song: test_song("c3") });
        // バー境界前: まだ無音
        let mut buf = vec![0f32; 9600];
        e.process(&mut buf, &bank);
        assert!(buf.iter().all(|s| s.abs() < 1e-6), "should be silent before bar boundary");
        // バー境界(96000 samples)をまたぐ
        let mut buf = vec![0f32; 96000];
        e.process(&mut buf, &bank);
        assert!(buf.iter().any(|s| s.abs() > 0.001), "should sound after bar boundary");
    }
}
```

Run: `cargo test engine`
Expected: 1 passed

**Step 3: Commit**

```bash
git add -A && git commit -m "task 12: engine with bar-quantized pending transitions"
```

---

---

### Task 13: バークオンタイズ検証

**Objective:** 変更が必ずバー頭で反映されることを自動テストで保証。

**Files:**
- Modify: `strudel-rs/src/engine.rs`（テスト追加）

**Step 1: テスト追加**

```rust
#[test]
fn bpm_change_waits_for_bar() {
    let mut e = Engine::new(48000, 120.0);
    let bank = SampleBank::load_dir(std::path::Path::new("samples"), 48000);
    e.push_command(Command::SetBpm(60.0));
    let mut buf = vec![0f32; 4800]; // バー境界前
    e.process(&mut buf, &bank);
    assert!((e.transport.bpm - 120.0).abs() < 1e-9, "bpm must not change mid-bar");
    let mut buf = vec![0f32; 96000]; // 境界をまたぐ
    e.process(&mut buf, &bank);
    assert!((e.transport.bpm - 60.0).abs() < 1e-9, "bpm must change at bar boundary");
}

#[test]
fn track_mute_waits_for_bar() {
    let mut e = Engine::new(48000, 120.0);
    let bank = SampleBank::load_dir(std::path::Path::new("samples"), 48000);
    let song = parse_song("---\nb: note(\"c3\").s(\"sawtooth\")", "t").unwrap();
    e.push_command(Command::LoadSong { deck: 0, song });
    let mut buf = vec![0f32; 96000]; // 1バー鳴らす
    e.process(&mut buf, &bank);
    e.push_command(Command::SetTrackMute { deck: 0, track: "b".into(), muted: true });
    let mut buf = vec![0f32; 4800];  // 境界前: まだ鳴っている
    e.process(&mut buf, &bank);
    assert!(buf.iter().any(|s| s.abs() > 0.0));
    let mut buf = vec![0f32; 96000]; // 境界後: 無音
    e.process(&mut buf, &bank);
    assert!(buf.iter().all(|s| s.abs() < 1e-6));
}
```

**Step 2: 検証**

Run: `cargo test engine`
Expected: 3 passed（Task 12 のテスト含む）

**Step 3: Commit**

```bash
git add -A && git commit -m "task 13: bar-quantize verification tests"
```

---

---

### Task 14: デュアルデッキ + Mixer 層（フェーダー / EQ・フィルター / xfade）

**Objective:** 曲A・曲Bを同時に鳴らし、**ミキサー層**で音量フェーダー・簡易 EQ/フィルター・N バー等パワー xfade / 切替を行う。

**Files:**
- Create: `strudel-rs/src/mixer.rs`
- Modify: `strudel-rs/src/engine.rs`（Mixer 組み込み・テスト追加）

**責務分担:**

| コンポーネント | 担当 |
| --- | --- |
| Deck A/B | Song ロード、パターン→イベント、Voice 生成 |
| Mixer | `gain_a` / `gain_b`、EQ/フィルター、`xfade` 状態、最終ステレオ/モノ合成 |

**Step 1: テスト追加（xfade は Mixer 経由）**

```rust
#[test]
fn xfade_transitions_between_decks() {
    let mut e = Engine::new(48000, 120.0); // 1 bar = 96000 samples
    let bank = SampleBank::load_dir(std::path::Path::new("samples"), 48000);
    e.push_command(Command::LoadSong { deck: 0, song: test_song("c3") });
    let mut buf = vec![0f32; 96000];
    e.process(&mut buf, &bank);   // bar 1: deck A 鳴動開始

    e.push_command(Command::LoadSong { deck: 1, song: test_song("g3") });
    e.push_command(Command::XFade { to_deck: 1, bars: 2 });
    e.process(&mut buf, &bank);   // bar 2: フェード開始 (両方鳴る)
    assert!(e.decks[0].gain < 1.0 && e.decks[1].gain > 0.0 || e.decks[0].gain == 1.0);
    e.process(&mut buf, &bank);   // bar 3
    e.process(&mut buf, &bank);   // bar 4: フェード完了
    assert_eq!(e.decks[0].gain, 0.0);
    assert_eq!(e.decks[1].gain, 1.0);
    assert!(e.decks[0].song_title().is_none(), "old deck unloaded after xfade");
    assert_eq!(e.decks[1].song_title(), Some("t"));
}

#[test]
fn decks_share_transport() {
    // 両デッキが同じ global_sample を見ることを、
    // 同じコードを両デッキに載せて位相が一致することで検証
    let mut e = Engine::new(48000, 120.0);
    let bank = SampleBank::load_dir(std::path::Path::new("samples"), 48000);
    e.push_command(Command::LoadSong { deck: 0, song: test_song("c3") });
    e.push_command(Command::LoadSong { deck: 1, song: test_song("c3") });
    let mut buf = vec![0f32; 96000];
    e.process(&mut buf, &bank);
    // 両デッキ同一内容 → 合成振幅が片デッキの約2倍になる区間がある
    let peak = buf.iter().fold(0f32, |m, s| m.max(s.abs()));
    assert!(peak > 0.3, "expected constructive interference, peak={peak}");
}
```

**Step 2: 検証**

Run: `cargo test engine`
Expected: 5 passed

**Step 3: Commit**

```bash
git add -A && git commit -m "task 14: dual deck + xfade verified"
```

---

---

### Task 15: 曲ファイルウォッチャ

**Objective:** `songs/*.strudel` の保存を検知し、ロード済みデッキの曲をバー境界で差し替える。

**Files:**
- Modify: `strudel-rs/Cargo.toml`（notify）
- Create: `strudel-rs/src/watcher.rs`

**Step 1: コード**

`Cargo.toml` 追加: `notify = "6"`

`src/watcher.rs`:
```rust
use crate::engine::Command;
use crate::song::parse_song;
use crossbeam::channel::Sender;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// 監視対象: パス → どのデッキにロードされているか
pub type DeckPaths = Arc<Mutex<[Option<PathBuf>; 2]>>;

pub fn watch_songs(
    dir: &Path,
    tx: Sender<Command>,
    deck_paths: DeckPaths,
) -> notify::Result<RecommendedWatcher> {
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        let Ok(ev) = res else { return };
        if !matches!(ev.kind, notify::EventKind::Modify(_) | notify::EventKind::Create(_)) {
            return;
        }
        for path in ev.paths {
            if path.extension().and_then(|s| s.to_str()) != Some("strudel") { continue; }
            let deck = {
                let dp = deck_paths.lock().unwrap();
                dp.iter().position(|p| p.as_ref() == Some(&path))
            };
            let Some(deck) = deck else { continue };  // ロードされていないファイルは無視
            match std::fs::read_to_string(&path) {
                Ok(text) => match parse_song(&text, &path.to_string_lossy()) {
                    Ok(song) => {
                        println!("↻ {} → deck {} (次の小節から反映)", song.title, deck);
                        tx.send(Command::LoadSong { deck, song }).ok();
                    }
                    Err(e) => eprintln!("⚠ {}: parse error (現行の曲を継続): {e}", path.display()),
                },
                Err(e) => eprintln!("⚠ read error: {e}"),
            }
        }
    })?;
    watcher.watch(dir, RecursiveMode::NonRecursive)?;
    Ok(watcher)
}
```

**Step 2: 検証（手動）**

1. `songs/demo.strudel` を作成し REPL/API からデッキ A にロード
2. エディタで `demo.strudel` のコードを変更して保存
3. ターミナルに `↻ Demo → deck 0 (次の小節から反映)` と表示され、**バー頭で**新コードに切り替わる（音が途切れない）
4. わざと文法エラーを入れて保存 → `⚠ parse error (現行の曲を継続)` と表示され、現行の曲が鳴り続ける

**Step 3: Commit**

```bash
git add -A && git commit -m "task 15: song file watcher with bar-quantized reload"
```

---

---

### Task 16: REPL

**Objective:** 人間用の対話操作一式。

**Files:**
- Modify: `strudel-rs/Cargo.toml`（rustyline）
- Create: `strudel-rs/src/repl.rs`

**Step 1: コード**

`Cargo.toml` 追加: `rustyline = "14"`

`src/repl.rs`:
```rust
use crate::engine::Command;
use crate::song::parse_song;
use crossbeam::channel::Sender;
use rustyline::DefaultEditor;
use std::path::PathBuf;
use crate::watcher::DeckPaths;

pub fn run(tx: Sender<Command>, deck_paths: DeckPaths) {
    let mut rl = DefaultEditor::new().unwrap();
    println!(r#"strudel-rs
  :load <A|B> <file.strudel>   曲をデッキにロード（次の小節から）
  :xfade <A|B> [bars=4]        N小節かけてデッキ切替
  :mute <A|B> <track> / :unmute <A|B> <track>
  :bpm <N>                      次の小節からBPM変更
  :hush                         全停止（即時）
  :status                       状態表示
  :quit"#);
    loop {
        match rl.readline("» ") {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() { continue; }
                rl.add_history_entry(line).ok();
                let args: Vec<&str> = line.split_whitespace().collect();
                match args[0] {
                    ":quit" | ":q" => { tx.send(Command::Hush).ok(); break; }
                    ":hush" => { tx.send(Command::Hush).ok(); println!("(hushed)"); }
                    ":bpm" if args.len() == 2 => {
                        if let Ok(b) = args[1].parse::<f64>() {
                            tx.send(Command::SetBpm(b)).ok();
                            println!("BPM → {b} (次の小節から)");
                        }
                    }
                    ":load" if args.len() == 3 => {
                        let deck = match args[1] { "A" | "a" => 0, "B" | "b" => 1, _ => { eprintln!("deck は A か B"); continue; } };
                        let path = PathBuf::from(args[2]);
                        match std::fs::read_to_string(&path) {
                            Ok(text) => match parse_song(&text, &path.to_string_lossy()) {
                                Ok(song) => {
                                    deck_paths.lock().unwrap()[deck] = Some(path);
                                    tx.send(Command::LoadSong { deck, song }).ok();
                                    println!("loaded (次の小節から)");
                                }
                                Err(e) => eprintln!("parse error: {e}"),
                            },
                            Err(e) => eprintln!("read error: {e}"),
                        }
                    }
                    ":xfade" if args.len() >= 2 => {
                        let to = match args[1] { "A" | "a" => 0, "B" | "b" => 1, _ => continue };
                        let bars = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(4);
                        tx.send(Command::XFade { to_deck: to, bars }).ok();
                        println!("xfade → {} ({} bars, 次の小節から)", args[1], bars);
                    }
                    ":mute" | ":unmute" if args.len() == 3 => {
                        let deck = match args[1] { "A" | "a" => 0, "B" | "b" => 1, _ => continue };
                        tx.send(Command::SetTrackMute {
                            deck, track: args[2].to_string(),
                            muted: args[0] == ":mute",
                        }).ok();
                    }
                    ":status" => {
                        let dp = deck_paths.lock().unwrap();
                        println!("deck A: {:?} / deck B: {:?}", dp[0], dp[1]);
                    }
                    _ => eprintln!("unknown command"),
                }
            }
            Err(_) => break,
        }
    }
}
```

**Step 2: 検証（手動）**

`:load A songs/techno1.strudel` → 次小節から再生。`:load B songs/ambient1.strudel` → `:xfade B 4` → 4 小節かけて移行。`:mute A kick` → 次小節から kick 停止。

**Step 3: Commit**

```bash
git add -A && git commit -m "task 16: repl with deck commands"
```

---

---
