# Task 1–9: 基盤（CPAL / Transport / ミニ記法 / Synth / Samples）

> 正本: [../2026-07-27_020000-strudel-rs-final.md](../2026-07-27_020000-strudel-rs-final.md)

状態: **完了**（正本 Progress の [x] から切り出し）

---

### Task 1: cargo プロジェクト初期化 + CPAL 疎通

**Objective:** 音が出ることを最速確認。

**Files:**
- Create: `strudel-rs/Cargo.toml`, `strudel-rs/src/main.rs`

**Step 1: コード**

`Cargo.toml`:
```toml
[package]
name = "strudel-rs"
version = "0.1.0"
edition = "2021"

[dependencies]
cpal = "0.15"
crossbeam = "0.8"

[profile.release]
opt-level = 3
lto = true
strip = true
```

`src/main.rs`:
```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device");
    let config = device.default_output_config().unwrap();
    let sr = config.sample_rate().0 as f32;
    let mut phase = 0f32;
    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _| {
            for s in data.iter_mut() {
                *s = (phase * 2.0 * std::f32::consts::PI).sin() * 0.2;
                phase = (phase + 440.0 / sr) % 1.0;
            }
        },
        |e| eprintln!("{e}"),
        None,
    ).unwrap();
    stream.play().unwrap();
    std::thread::sleep(std::time::Duration::from_secs(2));
}
```

**Step 2: 検証**

Run: `cd strudel-rs && cargo run`
Expected: 440Hz サイン波 2 秒。ALSA エラー時: `sudo apt install libasound2-dev`。

**Step 3: Commit**

```bash
git init && git add -A && git commit -m "task 1: cpal sine smoke test"
```

---

---

### Task 2: AudioBackend 抽象化 + NullBackend

**Files:**
- Create: `strudel-rs/src/backend.rs`

**Step 1: コード**

```rust
pub trait AudioBackend {
    fn start(&mut self, sample_rate: u32, cb: Box<dyn FnMut(&mut [f32]) + Send>);
    fn stop(&mut self);
}

pub struct NullBackend {
    cb: Option<Box<dyn FnMut(&mut [f32]) + Send>>,
}

impl NullBackend {
    pub fn new() -> Self { Self { cb: None } }
    pub fn render(&mut self, frames: usize) -> Vec<f32> {
        let mut buf = vec![0.0; frames];
        if let Some(cb) = &mut self.cb { cb(&mut buf); }
        buf
    }
}

impl AudioBackend for NullBackend {
    fn start(&mut self, _sr: u32, cb: Box<dyn FnMut(&mut [f32]) + Send>) { self.cb = Some(cb); }
    fn stop(&mut self) { self.cb = None; }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn null_backend_renders_silence() {
        let mut b = NullBackend::new();
        b.start(48000, Box::new(|_d| {}));
        assert_eq!(b.render(128), vec![0.0; 128]);
    }
}
```

**Step 2: 検証**

Run: `cargo test backend`
Expected: `1 passed`

**Step 3: Commit**

```bash
git add -A && git commit -m "task 2: AudioBackend + NullBackend"
```

---

---

### Task 3: Transport（BPM/バー計算・共有時計）

**Objective:** 両デッキが参照する唯一の時計。バー = 4 拍固定。

**Files:**
- Create: `strudel-rs/src/transport.rs`

**Step 1: コード**

```rust
/// 共有時計。両デッキが同じ global_sample を参照するので同期が保証される。
/// 1 bar = 4 beats。BPM 120 → 1 bar = 2.0s。
pub struct Transport {
    pub sample_rate: u32,
    pub bpm: f64,
    pub global_sample: u64,
}

impl Transport {
    pub fn new(sample_rate: u32, bpm: f64) -> Self {
        Self { sample_rate, bpm, global_sample: 0 }
    }
    pub fn samples_per_beat(&self) -> f64 { self.sample_rate as f64 * 60.0 / self.bpm }
    pub fn samples_per_bar(&self) -> f64 { self.samples_per_beat() * 4.0 }
    /// 現在のバー番号
    pub fn bar_index(&self) -> u64 {
        (self.global_sample as f64 / self.samples_per_bar()) as u64
    }
    /// バー内位置 [0,1)
    pub fn bar_pos(&self) -> f64 {
        (self.global_sample as f64 % self.samples_per_bar()) / self.samples_per_bar()
    }
    /// n バー先のバー頭の絶対サンプル位置 (n=1 で次のバー頭)
    pub fn bar_boundary(&self, n: u64) -> u64 {
        ((self.bar_index() + n) as f64 * self.samples_per_bar()) as u64
    }
    pub fn advance(&mut self, frames: usize) { self.global_sample += frames as u64; }
    pub fn set_bpm(&mut self, bpm: f64) { self.bpm = bpm; }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bar_math() {
        let mut t = Transport::new(48000, 120.0); // 1 bar = 96000 samples
        assert_eq!(t.bar_index(), 0);
        assert_eq!(t.bar_boundary(1), 96000);
        t.advance(96000);
        assert_eq!(t.bar_index(), 1);
        assert_eq!(t.bar_boundary(1), 192000);
        t.advance(48000); // bar 1 の真ん中
        assert!((t.bar_pos() - 0.5).abs() < 1e-9);
        assert_eq!(t.bar_boundary(1), 192000); // 次のバー頭は不変
    }
}
```

**Step 2: 検証**

Run: `cargo test transport`
Expected: `1 passed`

**Step 3: Commit**

```bash
git add -A && git commit -m "task 3: shared transport with bar math"
```

---

---

### Task 4: ミニ記法トークナイザ

**Files:**
- Create: `strudel-rs/src/mini.rs`

**Step 1: コード**

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
    Rest,                          // ~
    OpenBracket, CloseBracket,     // [ ]
    OpenAngle, CloseAngle,         // < >
    Star, Slash,                   // * /
    Number(f64),
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut out = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\n' | ',' => { chars.next(); }
            '~' => { out.push(Token::Rest); chars.next(); }
            '[' => { out.push(Token::OpenBracket); chars.next(); }
            ']' => { out.push(Token::CloseBracket); chars.next(); }
            '<' => { out.push(Token::OpenAngle); chars.next(); }
            '>' => { out.push(Token::CloseAngle); chars.next(); }
            '*' => { out.push(Token::Star); chars.next(); }
            '/' => { out.push(Token::Slash); chars.next(); }
            c if c.is_ascii_alphanumeric() || matches!(c, '.' | '#' | '-' | '_') => {
                let mut w = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_alphanumeric() || matches!(c, '.' | '#' | '-' | '_') {
                        w.push(c); chars.next();
                    } else { break; }
                }
                if let Ok(n) = w.parse::<f64>() { out.push(Token::Number(n)); }
                else { out.push(Token::Word(w)); }
            }
            other => return Err(format!("unexpected char: {other}")),
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tokenizes_basic() {
        let t = tokenize("c3 [e3 g3]*2 ~ <a3 b3> bd_cp").unwrap();
        assert_eq!(t, vec![
            Token::Word("c3".into()),
            Token::OpenBracket, Token::Word("e3".into()), Token::Word("g3".into()),
            Token::CloseBracket, Token::Star, Token::Number(2.0),
            Token::Rest,
            Token::OpenAngle, Token::Word("a3".into()), Token::Word("b3".into()), Token::CloseAngle,
            Token::Word("bd_cp".into()),
        ]);
    }
}
```

**Step 2: 検証**

Run: `cargo test mini`
Expected: `1 passed`

**Step 3: Commit**

```bash
git add -A && git commit -m "task 4: mini-notation tokenizer"
```

---

---

### Task 5: ミニ記法 AST パーサ

**Files:**
- Modify: `strudel-rs/src/mini.rs`

**Step 1: コード（追記）**

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Atom(String),
    Rest,
    Seq(Vec<Node>),        // [a b] / トップレベル: 1サイクル等分
    Stack(Vec<Node>),      // <a b>: サイクルごとに1つ選択
    Fast(Box<Node>, f64),  // node * n
    Slow(Box<Node>, f64),  // node / n
}

pub fn parse(input: &str) -> Result<Node, String> {
    let toks = tokenize(input)?;
    if toks.is_empty() { return Ok(Node::Rest); }
    let mut pos = 0;
    let node = parse_seq(&toks, &mut pos, None)?;
    if pos != toks.len() { return Err(format!("trailing tokens at {pos}")); }
    Ok(node)
}

fn parse_seq(t: &[Token], pos: &mut usize, closing: Option<Token>) -> Result<Node, String> {
    let mut items = Vec::new();
    while *pos < t.len() {
        if Some(&t[*pos]) == closing.as_ref() { *pos += 1; break; }
        let mut item = parse_item(t, pos)?;
        loop {
            if *pos + 1 < t.len() && matches!(t[*pos], Token::Star) {
                if let Token::Number(n) = t[*pos + 1] {
                    item = Node::Fast(Box::new(item), n); *pos += 2; continue;
                }
            }
            if *pos + 1 < t.len() && matches!(t[*pos], Token::Slash) {
                if let Token::Number(n) = t[*pos + 1] {
                    item = Node::Slow(Box::new(item), n); *pos += 2; continue;
                }
            }
            break;
        }
        items.push(item);
    }
    Ok(Node::Seq(items))
}

fn parse_item(t: &[Token], pos: &mut usize) -> Result<Node, String> {
    match t.get(*pos) {
        Some(Token::Word(w)) => { *pos += 1; Ok(Node::Atom(w.clone())) }
        Some(Token::Rest) => { *pos += 1; Ok(Node::Rest) }
        Some(Token::Number(n)) => { *pos += 1; Ok(Node::Atom(n.to_string())) }
        Some(Token::OpenBracket) => { *pos += 1; parse_seq(t, pos, Some(Token::CloseBracket)) }
        Some(Token::OpenAngle) => {
            *pos += 1;
            let s = parse_seq(t, pos, Some(Token::CloseAngle))?;
            if let Node::Seq(v) = s { Ok(Node::Stack(v)) } else { unreachable!() }
        }
        other => Err(format!("unexpected token: {other:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_nested() {
        let n = parse("c3 [e3 g3]*2 ~").unwrap();
        assert!(matches!(n, Node::Seq(ref v) if v.len() == 3));
        let n = parse("c3/2").unwrap();
        assert!(matches!(n, Node::Seq(ref v) if matches!(v[0], Node::Slow(_, 2.0))));
        let n = parse("<c3 e3>").unwrap();
        assert!(matches!(n, Node::Stack(_)));
    }
}
```

**Step 2: 検証**

Run: `cargo test mini`
Expected: 2 passed

**Step 3: Commit**

```bash
git add -A && git commit -m "task 5: mini-notation AST parser"
```

---

---

### Task 6: パターン評価器

**Objective:** AST + サイクル番号 → 時刻付きイベント列。1 サイクル = 1 バー。

**Files:**
- Modify: `strudel-rs/src/mini.rs`

**Step 1: コード（追記）**

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub start: f64,   // サイクル(=バー)内位置 [0,1)
    pub dur: f64,     // バー単位の長さ
    pub value: String,
}

pub fn events(node: &Node, cycle: u64) -> Vec<Event> {
    let mut out = Vec::new();
    emit(node, 0.0, 1.0, cycle, &mut out);
    out
}

fn emit(node: &Node, start: f64, span: f64, cycle: u64, out: &mut Vec<Event>) {
    match node {
        Node::Atom(v) => out.push(Event { start, dur: span, value: v.clone() }),
        Node::Rest => {}
        Node::Seq(items) => {
            if items.is_empty() { return; }
            let each = span / items.len() as f64;
            for (i, it) in items.iter().enumerate() {
                emit(it, start + i as f64 * each, each, cycle, out);
            }
        }
        Node::Stack(items) => {
            if items.is_empty() { return; }
            let sel = (cycle as usize) % items.len();
            emit(&items[sel], start, span, cycle, out);
        }
        Node::Fast(inner, n) => {
            let sub = span / n;
            for k in 0..(*n as usize) {
                emit(inner, start + k as f64 * sub, sub, cycle, out);
            }
        }
        Node::Slow(inner, n) => {
            if cycle % (*n as u64) == 0 { emit(inner, start, span, cycle, out); }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn evaluates_events() {
        let n = parse("c3 e3").unwrap();
        let ev = events(&n, 0);
        assert_eq!(ev.len(), 2);
        assert!((ev[1].start - 0.5).abs() < 1e-9);

        let n = parse("<c3 e3>").unwrap();
        assert_eq!(events(&n, 0)[0].value, "c3");
        assert_eq!(events(&n, 1)[0].value, "e3");

        let n = parse("c3*4 ~").unwrap();
        assert_eq!(events(&n, 0).len(), 4);
    }
}
```

**Step 2: 検証**

Run: `cargo test mini`
Expected: 3 passed

**Step 3: Commit**

```bash
git add -A && git commit -m "task 6: pattern evaluator"
```

---

---

### Task 7: メソッドチェーンパーサ + note_to_hz + 和音展開

**Files:**
- Create: `strudel-rs/src/code.rs`

**Objective:** メソッドチェーンをパースし、音名→Hz に加え **和音記法（`c3'maj` 等）を構成音へ展開**する（Risks #3 確定）。

**Step 1: コード**

```rust
use crate::mini::{self, Node};

#[derive(Debug, Clone)]
pub struct PatternCode {
    pub pattern: Node,
    pub sound: String,        // "sawtooth" または "bd" 等のサンプル名
    pub gain: f32,
    pub lpf: Option<f32>,
    pub speed: f64,           // slow/fast の積算
    pub is_note: bool,        // note()/n() なら音名→Hz
    pub raw: String,          // 表示用の元コード
}

pub fn parse_code(input: &str) -> Result<PatternCode, String> {
    let mut pc = PatternCode {
        pattern: Node::Rest, sound: "sine".into(), gain: 0.5,
        lpf: None, speed: 1.0, is_note: false, raw: input.trim().to_string(),
    };
    let input = input.trim();
    let open = input.find('(').ok_or("missing (")?;
    let head = input[..open].trim();
    if !matches!(head, "note" | "n" | "s" | "sound") {
        return Err(format!("unknown head: {head} (use note/s)"));
    }
    let first_str = extract_first_string(&input[open..])?;
    pc.pattern = mini::parse(&first_str)?;
    pc.is_note = matches!(head, "note" | "n");
    if matches!(head, "s" | "sound") { pc.sound = first_word(&first_str); }

    // .name(args) チェーン走査
    let mut cur = &input[open..];
    while let Some(dot) = cur.find('.') {
        let after = &cur[dot + 1..];
        let paren = after.find('(').ok_or("missing ( in method")?;
        let name = after[..paren].trim();
        let mut depth = 0i32; let mut end = paren;
        for (i, ch) in after[paren..].char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => { depth -= 1; if depth == 0 { end = paren + i; break; } }
                _ => {}
            }
        }
        if depth != 0 { return Err("unbalanced parens".into()); }
        let args = &after[paren + 1..end];
        apply_method(&mut pc, name, args)?;
        cur = &after[end + 1..];
    }
    Ok(pc)
}

fn first_word(s: &str) -> String {
    s.split_whitespace().next().unwrap_or("sine").to_string()
}

fn extract_first_string(s: &str) -> Result<String, String> {
    let start = s.find('"').ok_or("missing \"")?;
    let end = s[start + 1..].find('"').ok_or("missing closing \"")? + start + 1;
    Ok(s[start + 1..end].to_string())
}

fn apply_method(pc: &mut PatternCode, name: &str, args: &str) -> Result<(), String> {
    let num = |a: &str| a.trim().trim_matches('"').parse::<f64>()
        .map_err(|_| format!("bad number: {a}"));
    match name {
        // sound 名は文字列保持のみ。再生時に resolve_sound（波形リスト → サンプル）
        "s" | "sound" => { pc.sound = args.trim().trim_matches('"').to_string(); Ok(()) }
        "gain" => { pc.gain = num(args)? as f32; Ok(()) }
        "velocity" | "vel" => Ok(()), // gain に乗算するフィールドへ
        "lpf" | "cutoff" | "lp" | "ctf" => { pc.lpf = Some(num(args)? as f32); Ok(()) }
        "attack" | "att" | "decay" | "dec" | "sustain" | "sus" | "release" | "rel" | "adsr" => {
            Ok(()) // PatternCode に ADSR を持たせ Task 8 で Voice へ
        }
        "begin" | "end" | "speed" => Ok(()), // サンプル再生（Task 9）
        "slow" => { pc.speed /= num(args)?; Ok(()) }
        "fast" => { pc.speed *= num(args)?; Ok(()) }
        "note" | "n" => Ok(()), // n がサンプル index の文脈は Deck 側で解釈
        other => Err(format!("unsupported method: {other}")),
    }
}

// 再生時: 波形リスト → SampleBank（Risks #2 確定）
// fn resolve_sound(name: &str, bank: &SampleBank) -> Result<SoundSource, String>

/// 単音名 → Hz。和音トークンは expand_chord 経由で使う。
pub fn note_to_hz(note: &str) -> Result<f32, String> {
    let (name, oct) = note.split_at(note.len().saturating_sub(1));
    let oct: i32 = oct.parse().map_err(|_| format!("bad octave in {note}"))?;
    let semis = match name.to_ascii_lowercase().as_str() {
        "c" => 0, "c#" | "db" => 1, "d" => 2, "d#" | "eb" => 3, "e" => 4, "f" => 5,
        "f#" | "gb" => 6, "g" => 7, "g#" | "ab" => 8, "a" => 9, "a#" | "bb" => 10, "b" => 11,
        other => return Err(format!("bad note: {other}")),
    };
    let midi = (oct + 1) * 12 + semis;
    Ok(440.0 * 2f32.powf((midi - 69) as f32 / 12.0))
}

/// `c3'maj` / `c3'min7` 等 → 構成音名の列。単音なら1要素。
/// 初期対応: maj, min, maj7, min7, dim, aug, sus2, sus4（未対応は Err）
pub fn expand_chord(token: &str) -> Result<Vec<String>, String> {
    // 実装時: root + optional accidental + octave + optional 'quality をパースし
    // 半音オフセットを root に足して音名へ戻す。同時発音は Deck 側で Voice 複数起動。
    let _ = token;
    todo!("expand_chord: Risks #3 確定 — 実装時に表を固定")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_full_code() {
        let pc = parse_code(r#"note("c3 e3 [g3 ~]").s("sawtooth").lpf(800).gain(0.4).slow(2)"#).unwrap();
        assert!(pc.is_note);
        assert_eq!(pc.sound, "sawtooth");
        assert_eq!(pc.lpf, Some(800.0));
        assert!((pc.speed - 0.5).abs() < 1e-9);
    }
    #[test]
    fn sample_head() {
        let pc = parse_code(r#"s("bd sd hh*2").gain(0.8)"#).unwrap();
        assert!(!pc.is_note);
        assert_eq!(pc.sound, "bd");
    }
    #[test]
    fn note_freqs() {
        assert!((note_to_hz("a4").unwrap() - 440.0).abs() < 0.01);
        assert!((note_to_hz("c3").unwrap() - 130.81).abs() < 0.01);
    }
    #[test]
    fn rejects_bad() {
        assert!(parse_code("stack(\"a\")").is_err());
        assert!(parse_code("note(\"c3\"").is_err());
    }
}
```

**Step 2: 検証**

Run: `cargo test code`
Expected: 4 passed

**Step 3: Commit**

```bash
git add -A && git commit -m "task 7: method-chain parser"
```

---

---

### Task 8: ソフトシンセ Voice（ティアA）

**Objective:** 基本波形 + ノイズ + ADSR + パーボイス LPF。詳細は「Strudel 音源・エフェクト対応方針」ティアA。

**Files:**
- Create: `strudel-rs/src/synth.rs`
- Create: `strudel-rs/src/sound.rs`（`resolve_sound` の骨格でも可。Task 9 で Sample と接続）

**Step 1: コード**

```rust
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Wave { Sine, Saw, Square, Triangle }

impl Wave {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "sine" => Some(Wave::Sine),
            "sawtooth" | "saw" => Some(Wave::Saw),
            "square" => Some(Wave::Square),
            "triangle" | "tri" => Some(Wave::Triangle),
            _ => None,   // サンプル名の可能性
        }
    }
    fn sample(&self, phase: f32) -> f32 {
        match self {
            Wave::Sine => (phase * 2.0 * std::f32::consts::PI).sin(),
            Wave::Saw => 2.0 * phase - 1.0,
            Wave::Square => if phase < 0.5 { 1.0 } else { -1.0 },
            Wave::Triangle => 4.0 * (phase - 0.5).abs() - 1.0,
        }
    }
}

pub struct Voice {
    pub wave: Wave,
    pub freq: f32,
    pub gain: f32,
    phase: f32,
    pos: u64,
    len: u64,
    lpf: Option<f32>,
    lpf_y: f32,
}

impl Voice {
    pub fn new(wave: Wave, freq: f32, gain: f32, len: u64, lpf: Option<f32>) -> Self {
        Self { wave, freq, gain, phase: 0.0, pos: 0, len, lpf, lpf_y: 0.0 }
    }
    pub fn next_sample(&mut self, sr: f32) -> Option<f32> {
        if self.pos >= self.len { return None; }
        let a = (0.005 * sr) as u64;
        let r = ((self.len as f32 * 0.2) as u64).max(1);
        let env = if self.pos < a { self.pos as f32 / a as f32 }
                  else if self.pos > self.len.saturating_sub(r) {
                      (self.len - self.pos) as f32 / r as f32
                  } else { 1.0 };
        let mut x = self.wave.sample(self.phase) * env * self.gain;
        self.phase = (self.phase + self.freq / sr) % 1.0;
        self.pos += 1;
        if let Some(cut) = self.lpf {
            let k = 1.0 - (-2.0 * std::f32::consts::PI * cut / sr).exp();
            self.lpf_y += k * (x - self.lpf_y);
            x = self.lpf_y;
        }
        Some(x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn voice_produces_sound_and_ends() {
        let mut v = Voice::new(Wave::Sine, 440.0, 0.5, 4800, None);
        let mut energy = 0f32;
        for _ in 0..4800 { energy += v.next_sample(48000.0).unwrap().abs(); }
        assert!(energy > 10.0);
        assert!(v.next_sample(48000.0).is_none());
    }
    #[test]
    fn lpf_reduces_energy() {
        let render = |lpf| {
            let mut v = Voice::new(Wave::Square, 2000.0, 1.0, 4800, lpf);
            (0..4800).map(|_| v.next_sample(48000.0).unwrap().abs()).sum::<f32>()
        };
        assert!(render(Some(200.0)) < render(None));
    }
}
```

**Step 2: 検証**

Run: `cargo test synth`
Expected: 2 passed

**Step 3: Commit**

```bash
git add -A && git commit -m "task 8: soft synth voice"
```

---

---

### Task 9: サンプルプレイヤ + sound 解決（ティアA）

**Objective:** `s("bd sd hh*2")` と `note("c3").s("bd")` をローカル WAV で再生。`resolve_sound` は波形リスト → SampleBank。展示のリズム隊はこちらが主役。`begin`/`end`/`speed`/`n` をティアA範囲で。

**Files:**
- Create: `strudel-rs/src/sample.rs`
- Create: `strudel-rs/samples/`（bd/sd/hh/oh を配置。**Sonic Pi CC0 を WAV 変換して vendoring**。Dirt-Samples は使わない）
- Modify: `strudel-rs/src/synth.rs`（Voice を enum 化してサンプルボイスを追加）

**Step 1: コード**

`src/sample.rs`:
```rust
use std::collections::HashMap;
use std::path::Path;

/// 16bit PCM mono WAV のみ対応（ローダを最小化するため）。
/// 44.1kHz 以外は線形補間で再生側がピッチシフトする形を取らず、
/// ロード時に 48000Hz へ最近傍リサンプルする。
pub struct SampleBank {
    samples: HashMap<String, Vec<f32>>,
}

impl SampleBank {
    pub fn load_dir(dir: &Path, target_sr: u32) -> Self {
        let mut samples = HashMap::new();
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().and_then(|s| s.to_str()) == Some("wav") {
                    if let Ok(data) = load_wav(&p, target_sr) {
                        let name = p.file_stem().unwrap().to_string_lossy().to_string();
                        samples.insert(name, data);
                    }
                }
            }
        }
        Self { samples }
    }
    pub fn get(&self, name: &str) -> Option<&Vec<f32>> { self.samples.get(name) }
    pub fn names(&self) -> Vec<String> { self.samples.keys().cloned().collect() }
}

fn load_wav(path: &Path, target_sr: u32) -> Result<Vec<f32>, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    if bytes.len() < 44 || &bytes[0..4] != b"RIFF" { return Err("not wav".into()); }
    let channels = u16::from_le_bytes([bytes[22], bytes[23]]) as usize;
    let src_sr = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
    let bits = u16::from_le_bytes([bytes[34], bytes[35]]);
    if bits != 16 { return Err("only 16bit wav".into()); }
    // data チャンク探索
    let mut i = 12;
    let data = loop {
        if i + 8 > bytes.len() { return Err("no data chunk".into()); }
        let id = &bytes[i..i + 4];
        let size = u32::from_le_bytes([bytes[i+4], bytes[i+5], bytes[i+6], bytes[i+7]]) as usize;
        if id == b"data" { break &bytes[i + 8..(i + 8 + size).min(bytes.len())]; }
        i += 8 + size + (size % 2);
    };
    // 16bit → f32、多ch は先頭 ch のみ
    let mut out: Vec<f32> = data.chunks_exact(2 * channels)
        .map(|c| i16::from_le_bytes([c[0], c[1]]) as f32 / 32768.0)
        .collect();
    // リサンプル（最近傍）
    if src_sr != target_sr && !out.is_empty() {
        let ratio = src_sr as f64 / target_sr as f64;
        let n = (out.len() as f64 / ratio) as usize;
        out = (0..n).map(|k| out[(k as f64 * ratio) as usize % out.len()]).collect();
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_test_wav(path: &Path, sr: u32) {
        // 100ms の 440Hz サイン、16bit mono
        let n = (sr as f32 * 0.1) as usize;
        let data: Vec<u8> = (0..n)
            .flat_map(|i| {
                let v = ((i as f32 / sr as f32 * 440.0 * 2.0 * std::f32::consts::PI).sin()
                    * 30000.0) as i16;
                v.to_le_bytes()
            }).collect();
        let mut f = Vec::new();
        f.write_all(b"RIFF").unwrap();
        f.write_all(&(36u32 + data.len() as u32).to_le_bytes()).unwrap();
        f.write_all(b"WAVEfmt ").unwrap();
        f.write_all(&16u32.to_le_bytes()).unwrap();
        f.write_all(&1u16.to_le_bytes()).unwrap();          // PCM
        f.write_all(&1u16.to_le_bytes()).unwrap();          // mono
        f.write_all(&sr.to_le_bytes()).unwrap();
        f.write_all(&(sr * 2).to_le_bytes()).unwrap();
        f.write_all(&2u16.to_le_bytes()).unwrap();
        f.write_all(&16u16.to_le_bytes()).unwrap();
        f.write_all(b"data").unwrap();
        f.write_all(&(data.len() as u32).to_le_bytes()).unwrap();
        f.write_all(&data).unwrap();
        std::fs::write(path, f).unwrap();
    }

    #[test]
    fn loads_wav() {
        let dir = std::env::temp_dir().join("strudel_test_samples");
        std::fs::create_dir_all(&dir).unwrap();
        write_test_wav(&dir.join("bd.wav"), 48000);
        let bank = SampleBank::load_dir(&dir, 48000);
        let s = bank.get("bd").expect("bd missing");
        assert!(s.len() > 4000);
        assert!(s.iter().any(|x| x.abs() > 0.5));
        std::fs::remove_dir_all(&dir).ok();
    }
}
```

`synth.rs` の Voice を enum 化:
```rust
pub enum VoiceKind {
    Synth(SynthVoice),     // 旧 Voice
    Sample(SampleVoice),
}

pub struct SampleVoice {
    data: std::sync::Arc<Vec<f32>>,
    pos: usize,
    gain: f32,
}

impl SampleVoice {
    pub fn new(data: std::sync::Arc<Vec<f32>>, gain: f32) -> Self {
        Self { data, pos: 0, gain }
    }
    pub fn next_sample(&mut self) -> Option<f32> {
        let x = *self.data.get(self.pos)? * self.gain;
        self.pos += 1;
        Some(x)
    }
}
// 旧 Voice は SynthVoice にリネーム（既存コード追随）
```

**Step 2: 検証**

Run: `cargo test sample`
Expected: `1 passed`（`loads_wav`）

**Step 3: Commit**

```bash
git add -A && git commit -m "task 9: wav sample player"
```

---

---
