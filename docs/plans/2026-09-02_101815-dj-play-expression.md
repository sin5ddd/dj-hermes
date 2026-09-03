# 展示DJプレイ表現 — Mixer fill 拡張 Implementation Plan

> **For Hermes:** Load the plan-execution-loop skill and implement this plan task-by-task.
> 正本 Task 番号は増やさない。Issue #24（`strudel_mix`）の延長。曲ソースは書き換えない。
> **git:** ユーザーが明示するまで commit / stage / push しない（AGENTS.md）。各 Task の「Commit」はスキップ。

**Goal:** 展示ブースで来場者が「エコーで消して」「ロールして」「ドロップ」と言えるよう、Mixer の fill を 4 種足し、TUI に進行中のミックスを出す。

**Architecture:** 新しい MCP ツールは作らない。全部 `FillKind` + 既存 `Command::Mix` / `POST /mix` / `strudel_mix`。DSP は `src/mixer.rs` の preallocated `MixJob` のみ（audio スレッドで Vec 確保しない）。バー量子化・5ms ゲインランプ・バッファ非スプリットは維持。

**Tech Stack:** 既存のみ（cpal / crossbeam / axum）。依存追加なし。

---

## Related Context (from sqlew)

### Past Decisions

> No related decisions found for: dj/mixer, xfade, live-fx

### Applicable Constraints

> No constraints found for: audio thread mixer dj transition bar quantize

（sqlew 再検索は不要。コード上の確定制約は下記 🚫 に書いた。）

---

## Progress

- [ ] Task 1: FillKind に echo / hpf / roll / drop を足す（パースのみ）
- [ ] Task 2: MixJob::Echo — wet/feedback ランプのあと cut-in
- [ ] Task 3: MixJob::Hpf — HPF スイープのあと cut-in
- [ ] Task 4: MixJob::Drop — impact PCM のあと cut-in
- [ ] Task 5: MixJob::Roll — ビートリピートのあと cut-in
- [ ] Task 6: Engine 経由で新 fill がバー頭から動く
- [ ] Task 7: TUI クロスフェーダー行に mix ステータスを出す
- [ ] Task 8: REPL / 補完 / MCP 説明文を通す
- [ ] Task 9: dj-hermes skill と SOUL を更新する
- [ ] Task 10: e2e で fill echo が有限・クリップしない
- [ ] Task 11: fmt / clippy / 単体+e2e を通す

---

## Current context / assumptions

展示は Hermes（profile `dj-hermes`）が自然文 → `strudel_mix` 1 回。いま動くムーブ:

| move | 中身 |
| --- | --- |
| `long` | Lo isolator + 等パワー xfade（既定 8 小節） |
| `cut` | 次バーで 100% 着地、EQ flat |
| `fill delay\|lpf\|flash\|riser\|switch` | 1〜2 小節の演出のあと cut-in |
| `hold` | xfade を即時凍結 |

足りないのは「DJ が手でやる見せ技」で、来場者が見て分かるもの。曲の `$:` をいじるフィルは **strudel-live-edit** のまま（混ぜない）。

制約（触らない）:

- 1 bar = 4 beats。マスター BPM 共有。デッキ独立テンポなし
- 遷移はバー頭（hold だけ即時）。バッファ内スプリットしない
- Mixer が混ぜ方、Deck が中身。ミックスのために `apply_song` / `patch_track` しない
- audio スレッドでアロケーションしない。`Mixer::new` でバッファ確保済み
- 新しい MCP ツール名を増やさない（ローカル小モデルが迷う）

### 📌 Decision: dj/mix/fill-kinds-v2

- **Value:** 新表現は `MixAction` を増やさず `FillKind` に `echo` / `hpf` / `roll` / `drop` を足す。入口は従来どおり `strudel_mix(move="fill", kind=…)`
- **Layer:** business
- **Tags:** dj, mixer, fill, exhibit
- **Rationale:** 展示の LLM は `strudel_mix` 1 ツールに既に慣れている。ツールを増やすと誤呼び出しが増える。long/cut/hold の意味は変えない
- **Alternatives:** 新 MCP `strudel_fx`、Vinyl 系 Command、曲ソースにワンショット `$:` を挿入
- **Tradeoffs:** FillKind の match が伸びる。ツール面は単純なまま

### 📌 Decision: dj/mix/defer-vinyl

- **Value:** バックスピン / スクラッチ / デッキ独立ピッチ / レゾナントフィルタ / ヘッドホン cue は今やらない
- **Layer:** business
- **Tags:** dj, yagni, exhibit
- **Rationale:** クリックと CPU が増えるわりに、自然文 1 発のブースでは再現しにくい。ロールとエコーアウトの方が見て分かる
- **Alternatives:** 逆再生バッファ、3 デッキ、MIDI
- **Tradeoffs:** 「本格 DJ」感は足りない。ブースの分かりやすさを優先

### 🚫 Constraint: architecture

- **Rule:** ミックスマクロは Mixer の MixJob に閉じる。曲ソースを書き換えない。audio コールバックで Vec 確保しない
- **Priority:** critical
- **Reason:** AGENTS.md の 3 層と Issue #24。パース失敗で演奏を止めない方針と両立させる
- **Tags:** mixer, audio-thread

### 🚫 Constraint: performance

- **Rule:** クリック防止は既存 5ms ゲインランプのみ。バッファまたぎスプリットを入れない
- **Priority:** high
- **Reason:** Risks #1 確定。展示は安全性優先
- **Tags:** transport, click

---

## 提案カタログ（実装するのは Adopt のみ）

| ID | 来場者の言い方 | 実装 | 判定 |
| --- | --- | --- | --- |
| A | エコーで消して / 残響でつないで | `fill echo` — delay wet+FB を上げてから cut-in | **Adopt** |
| B | 薄くして落とす / ハイパスで切って | `fill hpf` — HPF 40→2k スイープのあと cut-in | **Adopt** |
| C | ロールして / ループして切って | `fill roll` — 直近 8n/4n を繰り返して cut-in | **Adopt** |
| D | ドロップ / インパクト入れて | `fill drop` — `fx/id`（なければ `fx/sd`）ワンショットのあと cut-in | **Adopt** |
| E | （無言でも分かる） | TUI の XF 行に `MIX echo→B 1` | **Adopt** |
| F | 両方全開 | `move=double` | Defer（fader 2 つは LLM が long と混同する） |
| G | バックスピン | 逆再生 | Defer（クリック） |
| H | スクラッチ | グラニュラ | Defer（ティア C） |
| I | キックだけ残して | 既存 `mute_track` + long の Lo kill | 既存で足りる |
| J | ドラムのフィル | `strudel-live-edit` の `<>` | 既存。Mixer に入れない |

既存 `fill delay` は wet 固定 0.5・FB 0。`echo` は **outgoing を残しつつ wet/FB を上げる** ので別物。取り違えない。

既存 `fill lpf` はマスター LPF 12k→200。`hpf` はその逆（薄くする）。

既存 `fill riser` は `fx/up` or `fx/nr`。`drop` は短い impact（`fx/id` / `fx/sd`）。PCM が空でも cut-in はする（無音ワンショット）。

---

## Step-by-step tasks

### Task 1: FillKind に echo / hpf / roll / drop を足す（パースのみ）

**Objective:** 文字列だけ先に通す。DSP はまだ動かさない。

**Files:**
- Modify: `src/mixer.rs`（`FillKind` の parse / as_str / default_bars）
- Test: 同ファイル `#[cfg(test)]`

**Step 1: Write failing test**

`src/mixer.rs` の tests 末尾に追加:

```rust
#[test]
fn fill_kind_parse_v2() {
    assert_eq!(FillKind::parse("echo").unwrap(), FillKind::Echo);
    assert_eq!(FillKind::parse("hpf").unwrap(), FillKind::Hpf);
    assert_eq!(FillKind::parse("roll").unwrap(), FillKind::Roll);
    assert_eq!(FillKind::parse("drop").unwrap(), FillKind::Drop);
    assert_eq!(FillKind::parse("impact").unwrap(), FillKind::Drop);
    assert_eq!(FillKind::Echo.as_str(), "echo");
    assert_eq!(FillKind::Hpf.as_str(), "hpf");
    assert_eq!(FillKind::Roll.as_str(), "roll");
    assert_eq!(FillKind::Drop.as_str(), "drop");
    assert_eq!(FillKind::Echo.default_bars(), 1);
    assert_eq!(FillKind::Hpf.default_bars(), 1);
    assert_eq!(FillKind::Roll.default_bars(), 1);
    assert_eq!(FillKind::Drop.default_bars(), 1);
    assert!(FillKind::parse("scratch").is_err());
}
```

**Step 2: Run test to verify failure**

```bash
cargo test --lib fill_kind_parse_v2 -- --nocapture
```

Expected: FAIL — `no variant named Echo`（または compile error）

**Step 3: Write minimal implementation**

`FillKind` に 4 variant を追加。`parse`:

```rust
"echo" | "echoout" | "echo-out" => Ok(Self::Echo),
"hpf" => Ok(Self::Hpf),
"roll" | "loop" | "repeat" => Ok(Self::Roll),
"drop" | "impact" => Ok(Self::Drop),
```

エラー文を `delay, lpf, flash, riser, switch, echo, hpf, roll, drop` に更新。

`as_str` は canonical 名のみ（`echo` `hpf` `roll` `drop`）。

`default_bars`: 4 つとも `1`（riser の 2 は変えない）。

`start_fill` の `match kind` はまだ `_ => {}` で早期 return してよい（Task 2 以降で埋める）。**コンパイルを通すため**、既存 5 臂のあと:

```rust
FillKind::Echo | FillKind::Hpf | FillKind::Roll | FillKind::Drop => {
    // Task 2–5: job はまだ組まない
}
```

既存 `tick_job` / `MixJob` は触らない。

**Step 4: Run test to verify pass**

```bash
cargo test --lib fill_kind_parse_v2 -- --nocapture
```

Expected: PASS

**Step 5: Commit** — スキップ

---

### Task 2: MixJob::Echo — wet/feedback ランプのあと cut-in

**Objective:** 1 小節で delay wet 0.25→0.7、feedback 0.2→0.8。終了で既存 `finish_job`（cut-in + EQ reset）。

**Files:**
- Modify: `src/mixer.rs`（`MixJob`, `start_fill`, `tick_job`, `mix`, `clear_job`）
- Test: 同ファイル

**定数（ファイル先頭、既存 `DELAY_WET` の近く）:**

```rust
const ECHO_WET_START: f32 = 0.25;
const ECHO_WET_END: f32 = 0.70;
const ECHO_FB_START: f32 = 0.20;
const ECHO_FB_END: f32 = 0.80;
```

**Mixer にフィールド追加**（`new()` で 0.0）:

```rust
delay_fb: f32,
```

既存 `mix()` の delay 呼び出しを `self.delay_l.process(l, self.delay_fb)` に変える（Delay フィルは fb=0 のまま。`start_fill` Delay 臂で `self.delay_fb = 0.0` を明示）。

**MixJob 臂:**

```rust
Echo {
    to_deck: usize,
    reset_eq: bool,
    start_sample: u64,
    end_sample: u64,
    samples_per_bar: u64,
},
```

`start_fill` の Echo:

- 8 分 delay time（既存 Delay と同じ `60.0 / bpm / 2.0`）
- `delay_wet = ECHO_WET_START`, `delay_fb = ECHO_FB_START`
- `set_mix_status("fill", Some("echo"), …)`

`tick_job`: Delay と同様 `end_sample` で `finish_job`。途中は `t` で wet/fb を線形補間。`clear_job` / `finish_job` で `delay_fb = 0.0`。

`tick_job` の巨大 match は **既存の copy-out `enum Step` に `Echo { start, end, spb }` を足す**。新しい抽象は作らない。

**Step 1: Write failing test**

```rust
#[test]
fn fill_echo_ramps_then_cuts() {
    let mut m = Mixer::new();
    m.gain_a = 1.0;
    m.gain_b = 0.0;
    m.start_fill(
        FillKind::Echo,
        1,
        1,
        true,
        MixGrid::Eighth,
        0,
        1000.0,
        120.0,
        48_000.0,
    );
    assert!(m.has_mix_job());
    let _ = m.tick_job(0, 48_000.0);
    let w0 = m.delay_wet;
    let _ = m.tick_job(500, 48_000.0);
    assert!(m.delay_wet > w0, "wet should rise");
    let done = m.tick_job(1000, 48_000.0);
    assert!(done);
    assert!(!m.has_mix_job());
    assert!(m.gain_a.abs() < 1e-5);
    assert!((m.gain_b - 1.0).abs() < 1e-5);
    assert!(m.delay_wet.abs() < 1e-6);
}
```

`delay_wet` は private。テストから読むなら `#[cfg(test)] pub(crate) fn delay_wet(&self) -> f32 { self.delay_wet }` を Mixer に足す（既存 Delay テストは `m.delay_wet > 0.0` とフィールド直読みしているので、**同じモジュールの tests なら private のまま読める**。フィールド直読みでよい）。

**Step 2:**

```bash
cargo test --lib fill_echo_ramps_then_cuts -- --nocapture
```

Expected: FAIL（Echo が job を組まない）

**Step 3:** 上記 DSP。`mix()` の `process(l, 0.0)` を `process(l, self.delay_fb)` に。Delay フィル開始時 `delay_fb = 0.0`。

**Step 4:** 同テスト PASS。既存 `fill_delay_wet_then_cut` も通す。

```bash
cargo test --lib fill_delay_wet_then_cut fill_echo_ramps_then_cuts -- --nocapture
```

Expected: 2 passed

**Step 5: Commit** — スキップ

---

### Task 3: MixJob::Hpf — HPF スイープのあと cut-in

**Objective:** マスター HPF を 40 Hz → 2000 Hz に上げてから cut-in。既存 Lpf job の鏡。

**定数:**

```rust
const HPF_SWEEP_START_HZ: f32 = 40.0;
const HPF_SWEEP_END_HZ: f32 = 2_000.0;
```

**MixJob:**

```rust
Hpf {
    to_deck: usize,
    reset_eq: bool,
    start_sample: u64,
    end_sample: u64,
    start_hz: f32,
    end_hz: f32,
    samples_per_bar: u64,
},
```

`finish_job` / `clear_job` で `self.hpf_hz = None`（LPF と同様。ジョブ以外でユーザーが HPF を掛けていた場合は消える。YAGNI: fill 中のマスターフィルタはジョブが所有する、と割り切る）。

**Step 1: failing test** — `fill_lpf_sweeps_down` をコピーして `FillKind::Hpf`、`h0 < h1`（上昇）、終了後 `hpf_hz.is_none()`。

**Step 2:**

```bash
cargo test --lib fill_hpf_sweeps_up -- --nocapture
```

Expected: FAIL

**Step 3:** `start_fill` / `tick_job` に Lpf と同じ線形補間（`hpf_hz`）。

**Step 4:** PASS。`fill_lpf_sweeps_down` も残す。

**Step 5: Commit** — スキップ

---

### Task 4: MixJob::Drop — impact PCM のあと cut-in

**Objective:** riser と同じ再生経路。PCM キーだけ変える。

**Files:**
- Modify: `src/mixer.rs`（`MixJob::Drop` は `Riser` と同形でよい）
- Modify: `src/engine.rs` の `riser_needs_pcm` 呼び出し側 — **Mixer に `drop_needs_pcm()` を足すか、`fx_oneshot_needs_pcm()` にまとめる**

**推奨（DRY）:** 既存

```rust
pub fn riser_needs_pcm(&self) -> bool {
    matches!(self.job, Some(MixJob::Riser { .. })) && self.riser_pcm.is_none()
}
```

を

```rust
pub fn oneshot_needs_pcm(&self) -> Option<&'static str> {
    match self.job {
        Some(MixJob::Riser { .. }) if self.riser_pcm.is_none() => Some("riser"),
        Some(MixJob::Drop { .. }) if self.riser_pcm.is_none() => Some("drop"),
        _ => None,
    }
}
```

`src/engine.rs` `process` の該当箇所を置換:

```rust
if let Some(kind) = self.mixer.oneshot_needs_pcm() {
    let pcm = match kind {
        "drop" => samples
            .get_stem("fx", "id")
            .or_else(|| samples.get_stem("fx", "sd")),
        _ => samples
            .get_stem("fx", "up")
            .or_else(|| samples.get_stem("fx", "nr")),
    };
    self.mixer.set_riser_pcm(pcm);
}
```

PCM なしでも job は時間どおり `finish_job`（riser と同じ）。

`mix()` の riser 加算は既存のまま（同じ `riser_pcm` / `riser_idx` / `RISER_GAIN`）。drop は短いので途中で PCM が尽きたら無音。定数はそのまま。

**Step 1: failing test**（Mixer 単体、PCM なし）:

```rust
#[test]
fn fill_drop_cuts_without_pcm() {
    let mut m = Mixer::new();
    m.gain_a = 1.0;
    m.gain_b = 0.0;
    m.start_fill(
        FillKind::Drop,
        1,
        1,
        true,
        MixGrid::Eighth,
        0,
        8.0,
        120.0,
        48_000.0,
    );
    assert!(m.has_mix_job());
    assert!(m.oneshot_needs_pcm() == Some("drop") || m.riser_needs_pcm());
    let done = m.tick_job(8, 48_000.0);
    assert!(done);
    assert!((m.gain_b - 1.0).abs() < 1e-5);
}
```

`riser_needs_pcm` を残すならテストはそちらでもよい。リネームするなら呼び出しを全部置換（`engine.rs` 1 箇所）。

**Step 2:** `cargo test --lib fill_drop_cuts_without_pcm` → FAIL

**Step 3:** MixJob::Drop、start_fill、tick_job の end 判定に Drop を含める（Riser と `|` でまとめてよい）。

**Step 4:** PASS。既存 riser 経路を壊していないこと（`riser_needs_pcm` を残すなら engine の or_else チェーンは現状維持でも可。その場合 drop は PCM 無しでも cut するだけで展示は成立する。**PCM を鳴らすなら engine 側の置換は必須**）。

**Step 5: Commit** — スキップ

---

### Task 5: MixJob::Roll — ビートリピートのあと cut-in

**Objective:** ジョブ開始から `period` サンプルをマスターに録音し、残り時間それをループ。終了で cut-in。

**やってはいけないこと:** `process` 内で `Vec::new` / `resize`。`Mixer::new` で delay と同じ長さを確保する。

**Mixer フィールド（`new()` で delay_n と同じ長さ）:**

```rust
roll_l: Vec<f32>,
roll_r: Vec<f32>,
roll_cap: usize,   // 録音済み長（0 = キャプチャ中）
roll_i: usize,     // 書き/読みインデックス
roll_target: usize, // period サンプル数
```

`new()`:

```rust
roll_l: vec![0.0; delay_n],
roll_r: vec![0.0; delay_n],
roll_cap: 0,
roll_i: 0,
roll_target: 0,
```

**MixJob:**

```rust
Roll {
    to_deck: usize,
    reset_eq: bool,
    start_sample: u64,
    end_sample: u64,
    period: u64,
    samples_per_bar: u64,
},
```

`start_fill` Roll:

- `period = (spb / grid.divisions_per_bar()).max(1)`（flash/switch と同じ）
- `roll_target = period.min(delay_n as u64) as usize`
- `roll_cap = 0`, `roll_i = 0`、バッファは `fill(0.0)`（new 時確保済み。clear は fill のみ）
- `set_mix_status("fill", Some("roll"), …)`

`mix()` の **マスター加算のあと、HPF/LPF/comp の前** に:

```rust
if self.roll_target > 0 {
    if self.roll_cap < self.roll_target {
        let i = self.roll_i;
        if i < self.roll_l.len() {
            self.roll_l[i] = l;
            self.roll_r[i] = r;
        }
        self.roll_i += 1;
        if self.roll_i >= self.roll_target {
            self.roll_cap = self.roll_target;
            self.roll_i = 0;
        }
    } else {
        let i = self.roll_i % self.roll_cap.max(1);
        l = self.roll_l[i];
        r = self.roll_r[i];
        self.roll_i = i + 1;
    }
}
```

`clear_job` / `finish_job`: `roll_target = 0`, `roll_cap = 0`, `roll_i = 0`（Vec は落とさない）。

**Step 1: failing test**

```rust
#[test]
fn fill_roll_repeats_then_cuts() {
    let mut m = Mixer::new();
    m.gain_a = 1.0;
    m.gain_b = 0.0;
    // 1 bar = 8 samples, 8n period = 1 sample（既存 switch テストと同じ玩具）
    m.start_fill(
        FillKind::Roll,
        1,
        1,
        true,
        MixGrid::Eighth,
        0,
        8.0,
        120.0,
        48_000.0,
    );
    assert!(m.has_mix_job());
    let a = vec![0.4f32; 8];
    let b = vec![0.0f32; 8];
    let mut out_l = vec![0.0f32; 8];
    let mut out_r = vec![0.0f32; 8];
    m.mix(&mut out_l, &mut out_r, &a, &a, &b, &b, 48_000.0);
    assert!(out_l.iter().any(|x| x.abs() > 0.01));
    let done = m.tick_job(8, 48_000.0);
    assert!(done);
    assert!(!m.has_mix_job());
    assert!((m.gain_b - 1.0).abs() < 1e-5);
}
```

**Step 2:** `cargo test --lib fill_roll_repeats_then_cuts` → FAIL

**Step 3:** 実装。`tick_job` の finished 判定に Roll の `end_sample` を含める。

**Step 4:** PASS。既存 delay/lpf/flash/switch テストをまとめて:

```bash
cargo test --lib -- mixer::tests --nocapture
```

Expected: 全部 PASS（この時点の mixer tests）

**Step 5: Commit** — スキップ

---

### Task 6: Engine 経由で新 fill がバー頭から動く

**Objective:** `Command::Mix` がバーを待ってから `start_fill` する既存経路に、新 kind が乗ること。

**Files:**
- Modify: `src/engine.rs`（oneshot PCM 分岐。Task 4 で未了ならここで）
- Test: `src/engine.rs` tests。既存 `mix_fill` ヘルパを使う

**Step 1: failing test**（`mix_switch_then_lands_on_to` の隣）:

```rust
#[test]
fn mix_echo_waits_for_bar_then_cuts() {
    let mut e = Engine::new(48_000, 120.0);
    let bank = SampleBank::empty();
    e.load_song_immediate(0, test_song("c3"));
    e.load_song_immediate(1, test_song("g3"));
    e.mixer.set_crossfader(0.0);
    e.push_command(Command::Mix(mix_fill(FillKind::Echo, 1)));
    let mut buf = stereo_buf(4_800);
    e.process(&mut buf, &bank);
    assert!(!e.mixer.has_mix_job(), "must not start mid-bar");
    process_until_next_bar_applied(&mut e, &bank);
    assert!(e.mixer.has_mix_job());
    let mut buf = stereo_buf(96_000);
    e.process(&mut buf, &bank);
    if e.mixer.has_mix_job() {
        let mut buf = stereo_buf(96_000);
        e.process(&mut buf, &bank);
    }
    assert!(!e.mixer.has_mix_job());
    assert!(e.mixer.gain_b > 0.9);
}
```

`mix_fill` は `bars: 1` 固定でそのまま使える。

同様に `FillKind::Hpf` / `Roll` / `Drop` を **1 テストにまとめてもよい**（ループ）。失敗時のメッセージに kind を含める。

**Step 2:** `cargo test --lib mix_echo_waits_for_bar_then_cuts` → FAIL（job 未配線なら）

**Step 3:** `apply_mix` の Fill 臂は kind をそのまま `start_fill` に渡している。Task 2–5 が正しければ Engine は無変更で通る。通らなければ `start_fill` の漏れ。

**Step 4:**

```bash
cargo test --lib mix_echo_waits_for_bar_then_cuts mix_switch_then_lands_on_to hush_clears_mix_job -- --nocapture
```

Expected: PASS

**Step 5: Commit** — スキップ

---

### Task 7: TUI クロスフェーダー行に mix ステータスを出す

**Objective:** 来場者が「いまエコーしてる」と画面で分かる。audio は playhead 以外出さない（既存どおり UI が mixer を try_lock で読む）。

**Files:**
- Modify: `src/live_ui.rs`（`LiveState`, `sync_models_from_engine`, `format_crossfader_line`, `draw_frame`）
- Test: 同ファイル tests の `crossfader_track_capped_at_10`

**LiveState に:**

```rust
mix_label: Option<String>,
```

初期化 `None`。

`sync_models_from_engine` の末尾近く:

```rust
state.mix_label = eng.mixer.mix_status().map(|s| {
    match &s.kind {
        Some(k) => format!("MIX {}/{}→{} {}", s.move_name, k, s.to, s.bars_left),
        None => format!("MIX {}→{} {}", s.move_name, s.to, s.bars_left),
    }
});
```

`format_crossfader_line` の署名を変える:

```rust
fn format_crossfader_line(
    pos: f32,
    cols: usize,
    row: u16,
    mix_label: Option<&str>,
) -> (String, SliderHit)
```

`pct` のあとに mix があれば dim で付ける。トラック幅計算は **ラベルを fixed に入れない**（スライダーが消えやすい）。ラベルは右側に余白があれば出す:

```rust
let extra = mix_label
    .filter(|s| !s.is_empty())
    .map(|s| format!(" \x1b[2m{s}\x1b[0m"))
    .unwrap_or_default();
```

`content` のあと、`cols` に収まるときだけ `raw` に連結。ヒット領域（スライダー）は変えない。

呼び出し `draw_frame` の XF 行:

```rust
let (xf_line, hit) =
    format_crossfader_line(state.xfade_pos, cols, xf_row as u16, state.mix_label.as_deref());
```

**Step 1: failing test**

既存 `crossfader_track_capped_at_10` の呼び出しが壊れる（引数追加）。直したうえで:

```rust
#[test]
fn crossfader_line_shows_mix_label() {
    let (line, hit) = format_crossfader_line(0.5, 120, 5, Some("MIX fill/echo→B 1"));
    assert!(hit.cols <= MAX_SLIDER_TRACK as u16);
    assert!(line.contains("echo"), "{line}");
    assert!(line.contains("XF"));
}
```

**Step 2:** compile fail → 署名修正でテスト RED（ラベル未使用なら contains 失敗）

**Step 3:** ラベル描画

**Step 4:**

```bash
cargo test --lib crossfader_track_capped_at_10 crossfader_line_shows_mix_label -- --nocapture
```

Expected: PASS

**Step 5: Commit** — スキップ

---

### Task 8: REPL / 補完 / MCP 説明文を通す

**Objective:** 人間の `/mix fill echo B` と Hermes の schema が新しい kind を知っている。

**Files:**
- Modify: `src/cmd.rs` — HELP の fill 行、`exec_mix` の usage 文字列
- Modify: `src/complete.rs` — `MIX_KINDS`
- Modify: `src/mcp.rs` — `strudel_mix` の description と kind の description
- Modify: `src/api.rs` — fill の kind エラー文字列（`mix_command_from_req` は `FillKind::parse` 依存なのでパースは自動）
- Test: `src/api.rs` の `mix_fill_switch_queues` の隣に echo を 1 本。`src/mcp.rs` の tools_list 件数 **17 のまま**（ツールは増やさない）

HELP 行（`src/cmd.rs`）:

```
mix fill <kind> A|B [8n|4n]  delay|lpf|flash|riser|switch|echo|hpf|roll|drop then cut-in
```

`complete.rs`:

```rust
const MIX_KINDS: &[&str] = &[
    "delay", "lpf", "flash", "riser", "switch", "echo", "hpf", "roll", "drop",
];
```

MCP description の kind 行:

```
fill only: delay | lpf | flash | riser | switch | echo | hpf | roll | drop
```

`strudel_mix` の長い description に 1 文足す:

```
echo=delay wet/fb ramp then cut. hpf=high-pass sweep then cut. roll=beat-repeat then cut. drop=impact one-shot then cut.
```

API テスト:

```rust
#[tokio::test]
async fn mix_fill_echo_queues() {
    // mix_fill_switch_queues と同じく両デッキに load_song_immediate
    // body: {"move":"fill","kind":"echo","to":"B"}
    // Command::Mix の fill == Some(FillKind::Echo), to_deck == 1
}
```

**Step 1:** テストを先に書く（echo キュー）。RED。

**Step 2:** `cargo test --lib mix_fill_echo_queues tools_list_mixer_deck_transport_no_set_code -- --nocapture`

Expected: echo テスト FAIL or 17 tools は PASS のまま

**Step 3:** 文字列と MIX_KINDS。API ハンドラのロジック変更は不要（parse が通れば ACCEPTED）。

**Step 4:** 上記 PASS。`assert_eq!(tools.len(), 17)` を壊さない。

**Step 5: Commit** — スキップ

---

### Task 9: dj-hermes skill と SOUL を更新する

**Objective:** 展示モデルが新しい kind を 1 回で呼ぶ。EQ 4 連打や apply_song でミックスしない。

**Files:**
- Modify: `docs/profile/dj-hermes/skills/creative/strudel-dj-mix/SKILL.md`
- Modify: `docs/profile/dj-hermes/SOUL.md`（mix の 1 行だけ。長文にしない）
- Create: `docs/plans/done/2026-09-02-dj-play-expression.md` は **実装完了後**。今はプラン本体だけ。

ルーティング表に行を足す（既存表のスタイルを守る）:

| 言い方 | 呼び出し |
| --- | --- |
| エコーで消して B へ | `strudel_mix(move="fill", kind="echo", to="B")` |
| ハイパスで薄くしてカット | `strudel_mix(move="fill", kind="hpf", to="B")` |
| ロールしてから A | `strudel_mix(move="fill", kind="roll", to="A", grid="8n")` |
| インパクト入れてカット | `strudel_mix(move="fill", kind="drop", to="B")` |

やってはいけないことに 1 行: ロールのために `apply_song` で同じヒットを並べない。

SOUL の DJ mix 行に kind 名を列挙（長くしない）。

**検証:** テストなし。レビューで表の `kind=` がコードの `as_str()` と一致すること。

ライブ profile へのコピーは展示セットアップ（`docs/exhibit/README.md`）の既存手順。この Task ではリポジトリ内の見本だけ直す。

**Commit** — スキップ

---

### Task 10: e2e で fill echo が有限・クリップしない

**Objective:** NullBackend で house → four-on-the-floor を echo fill しても NaN/クリップしない。

**Files:**
- Modify: `tests/e2e.rs`（既存 `dj_xfade_house_to_four_on_the_floor` の下）

`use` に `MixAction, MixCommand, MixGrid, FillKind`（`strudel_rs::mixer`）。

```rust
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
    let mut all = process_n(&mut e, &bank, bl);
    all.extend(process_n(&mut e, &bank, bl));
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
    for i in 0..4 {
        let buf = process_n(&mut e, &bank, bl);
        assert_finite_bounded(&buf, &format!("echo bar {i}"));
        all.extend(&buf);
    }
    assert!(
        (e.mixer.gain_b - 1.0).abs() < 1e-2,
        "should land on B, b={}",
        e.mixer.gain_b
    );
    assert!(has_energy(&process_n(&mut e, &bank, bl), 0.001));
    assert!(clip_rail_ratio(&all) < 0.08);
}
```

**Step 1:** テスト追加。未実装なら land on B で FAIL。

**Step 2:**

```bash
cargo test --test e2e dj_echo_fill_house_to_four_on_the_floor -- --nocapture
```

**Step 3:** 必要なら wet 上限を下げる（クリップしたら `ECHO_WET_END` を 0.55 に。テストを緩くしない）。

**Step 4:** PASS

**Step 5: Commit** — スキップ

---

### Task 11: fmt / clippy / 単体+e2e を通す

**Objective:** CI 相当。

```bash
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo test --test e2e
```

Expected: fmt check exit 0、clippy 0 warnings、test 全 PASS。

`start_fill` の `too_many_arguments` は既存 allow を維持。新しい allow を増やさない。

match 非網羅は許さない（FillKind を足したので compiler が導く）。

**Commit** — スキップ

---

## Files likely to change

| ファイル | 内容 |
| --- | --- |
| `src/mixer.rs` | FillKind / MixJob / mix() delay_fb / roll バッファ |
| `src/engine.rs` | drop PCM 解決、Engine テスト |
| `src/live_ui.rs` | XF 行の MIX ラベル |
| `src/cmd.rs` | HELP / usage |
| `src/complete.rs` | MIX_KINDS |
| `src/mcp.rs` | strudel_mix 説明（ツール数 17 固定） |
| `src/api.rs` | エラー文字、HTTP テスト |
| `tests/e2e.rs` | echo fill シナリオ |
| `docs/profile/dj-hermes/skills/creative/strudel-dj-mix/SKILL.md` | ルーティング表 |
| `docs/profile/dj-hermes/SOUL.md` | kind 列挙 |

触らない: `src/deck.rs` のスケジューラ、`src/mini.rs`、曲ファイル、新しい crate。

---

## Tests / validation

TDD は Task 1–6, 7, 8, 10。最終:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo test --test e2e
```

手動（実装後、オペレータ）:

```bash
cargo run -- dj songs/house-01.strudel songs/four-on-the-floor-01.strudel
```

TUI で `/mix fill echo B` → XF 行に `MIX fill/echo→B`、1 小節後に B。Hermes 自然文「エコーで B につないで」は skill 更新後。

---

## Risks, tradeoffs, and open questions

1. **ロールの位相:** キャプチャ開始がバー頭なので 8n はキックに揃いやすい。バッファ途中開始はしない（量子化方針）。
2. **echo と既存 delay:** 同時に 1 MixJob だけ。新しい mix は `clear_job` で前を消す（現行どおり）。
3. **HPF fill がユーザー HPF を消す:** 展示ではマスター HPF を手で置くことは稀。リセットしてよい。
4. **drop PCM 欠落:** Windows で LFS 未 smudge でも cut-in はする。無音インパクトを許容。
5. **TUI ラベル幅:** 狭い端末ではラベルを落とす（スライダー優先）。
6. **正本 Task 番号:** 増やさない。完了メモは実装後に `docs/plans/done/2026-09-02-dj-play-expression.md`（今回の実装ターンで書く。プラン段階では作らない）。

Open: 展示当日に 4 kind 全部を Hermes に覚えさせるか。skill 表は 4 行とも載せる。モデルが `delay` と `echo` を取り違えても事故ではない。
