---
name: strudel-sound-design
description: >-
  Use when designing synths, samples, banks, or effects for dj-hermes
  (live 2-op FM, factory PCM stems, not full Strudel REPL).
version: 4.2.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [dj-hermes, music, sound-design, synthesis, effects, fm, samples]
    related_skills:
      - strudel-data-format
      - strudel-composition
      - strudel-pcm-catalog
      - strudel-mood-bright-dark
      - strudel-genre-acid
      - strudel-genre-house
      - strudel-genre-dnb
      - strudel-genre-dnb-reese-mid-stab
      - strudel-genre-four-on-the-floor
      - strudel-genre-techno-duck
      - strudel-genre-electro
      - strudel-genre-minimal
      - strudel-genre-ambient
      - strudel-genre-chill
      - strudel-genre-dubstep
      - strudel-genre-progressive-house
      - strudel-genre-future-bass
      - strudel-genre-kawaii-future-bass
      - strudel-genre-lofi-hiphop
      - strudel-genre-chill-pop
---

# dj-hermes サウンドメイク（Sound Design）

## Overview

この Skill は **dj-hermes**（Rust 自前 DSP）向け。WebAudio 版 Strudel REPL の全機能は持たない。  
音源・メソッドは実装済みのものだけ。パターンの長さ・本数・ライブ差分は **strudel-composition**（新規は 7–8 本・4 小節フレーズ。ライブ差分は 1 トラック）。

**サンプル配置の正本（disk）:** リポジトリ `samples/LAYOUT.md`（bank キー・フルネーム・gitignore）。

**二系統:**

| 系統 | パターン内の名前 | bank | 例 |
| --- | --- | --- | --- |
| **ドラム** | 短い `bd` `sd` `hh` `oh` `cp` … | **キット用に付ける** | `s("bd*4, [~ sd]*2").bank("tr808-hard")` |
| **音程 / pad / lead / piano / FX** | **フルネーム** | 付けない | `s("pad-ambient_drone01")` / `s("piano-acoustic_soft")` |

**ライブで触るとわかりやすいツマミ**: `.lpf` / `.lpq` / `.hpf` / `.bpf` / `.gain` / ADSR / `.room` / `.delay` / `.fm` / `.vib`  
スカラーを 1 つ変えるときは `dj_hermes_edit_method`（または `dj_hermes_apply_song`）。`dj_hermes_save_song` は残す指示のときだけ（演奏は変えない）。

**本家にあって未実装の一覧:** `docs/strudel-gap-synths-fx.md`。例に **`.lfo` は書かない**。  
**明暗（キーの印象）は scale モードを優先**（→ **strudel-mood-bright-dark**）。`.lpf` は音色の副次。

## When to Use

- 短い `$:` ループの音色・FX を決める / ライブで 1 パラメータ変える時
- 波形 / ノイズ / 内蔵 wavetable / サンプル + 対応エフェクトを選ぶ時

Don't use for: パターン記法の詳細（→ strudel-composition）、曲ファイル形式（→ strudel-data-format）、**本家 Strudel 専用**のシンセ（ZZFX・partials・phaser 等）。

---

## 重要: 本家 Strudel との差

| 項目 | dj-hermes |
| --- | --- |
| メソッド引数 | **スカラー**、**ミニ数値パターン**（`.lpf("<400 1200>")`）、**LFO**（`.lpf(sine.rangex(500,4000))`）。`vib` 等はまだスカラーのみ |
| 未知メソッド | パースエラー → その行は落ちる（演奏は継続） |
| 未知 sound | `unknown sound`（波形でもサンプルでもない） |
| orbit | **デッキ内 1..4** のみ。A/B デッキ間で共有しない |
| delay / room | 同一 orbit 内 **last-write**。IR・roomlp 等なし |
| 和音 `c3'maj` | パーサはあるが **再生は root 単音**（多声展開しない） |

---

## 音源の解決順

`s("name")` / `.s("name")` / サンプルイベント値:

1. 基本波形（`sine` / `sawtooth`…）
2. ノイズ音源（`white` / `pink` / `brown`）
3. 内蔵 wavetable（`wt_*`）
4. `samples/` の SampleBank（`bd` 等、任意で `.bank("tr808")` → `tr808_bd`）

`note(...)` で sound 未指定のときデフォルトは **`triangle`**。

---

## 使える音源（シンセ / ノイズ / wt）

### 基本波形

| 名前 | 別名 |
| --- | --- |
| `sine` | |
| `sawtooth` | `saw` |
| `square` | |
| `triangle` | `tri`（note のデフォルト） |

```
$: note("c2 c2 eb2 g2").s("sine").gain(0.55)
$: note("c3 e3 g3").s("sawtooth").lpf(800).gain(0.4)
```

### ノイズ音源（sound 名）

| 名前 | 別名 |
| --- | --- |
| `white` | |
| `pink` | |
| `brown` | `brownian` |

```
$: s("white").gain(0.15).hpf(4000).attack(0.001).decay(0.05).sustain(0).release(0.02)
```

### オシレータへのノイズ混ぜ（`.noise`）

sound の `pink` とは別。波形に pink 系をミックス（0..1）。

```
$: note("c2").s("sawtooth").noise(0.2).lpf(600)
```

### 内蔵 wavetable（手続き 1 周期）

| 名前 | 内容の目安 |
| --- | --- |
| `wt_sine` / `wt_demo0` | サイン |
| `wt_bright` / `wt_demo1` | 倍音多め |
| `wt_organ` / `wt_demo2` | オルガン風倍音 |

```
$: note("<c3 e3 g3>/2").s("wt_organ").lpf(1200).gain(0.4).room(0.4).orbit(1)
$: note("c4 e4 g4").s("wt_bright").vib("5:8").gain(0.2).delay(0.25).orbit(2)
```

ネットから `samples('...')` で wt を取る、外部 `wt_flute` 等は **非対応**（ローカル bank に WAV があれば `wt_` 名で読める場合のみ）。

### サンプル（キット）

配置の詳細は **`samples/LAYOUT.md`**。エンジンは `samples/` を **1 階層だけ**読む。

#### 同梱デフォルト（短いパート名）

| sound | 変種 | パス例 |
| --- | --- | --- |
| `bd` | `.n(0)` / `.n(1)` | `samples/bd/00.wav`, `01.wav` |
| `sd` | 0, 1 | `samples/sd/` |
| `hh` | 0 | `samples/hh/00.wav` |
| `oh` | 0 | `samples/oh/00.wav` |
| `cp` | 0 | `samples/cp/00.wav`（ハウス 2/4。テクノグリッドには載せない） |

- `s("bd")` = その sound の **n=0**（先頭 WAV）
- rust-fm カタログ: `s("bd:8b")` → `samples/bd/8b.wav`（→ **strudel-pcm-catalog**）。整数 `bd:1` は `.n(1)` と同じ
- `s("cp")` は同梱 clap。`[~ cp]*2` がハウスバックビート。`sd` と重ねない
- `note().s("sample")` の再生比は `target_hz / SAMPLE_ROOT_HZ`。`SAMPLE_ROOT_HZ` は **261.63 Hz (C4)**（コメントが C3 でも定数は C4）。C3 録音は **`.scale("C4:…")` で native**

#### ドラム + `.bank`（リズムは短く、キットは bank）

`.bank("X")` + `s("bd")` → キー **`x_bd`**（小文字化、`bank` と part を `_` で連結）。

```
// デフォルトキット（bank なし）
$: s("bd*4, [~ sd]*2, [~ hh]*4").gain(0.55)
// ユーザーキット（disk: samples/tr808-hard_bd.wav 等）
$: s("bd hh [bd,sd] hh").bank("tr808-hard").gain(0.55)
$: s("bd*4, [~ sd]*2, hh*8").bank("accdrum-jazz").gain(0.5)
$: s("bd").n(1)   // 同一キー内の変種（フォルダに 01.wav 等があるとき）
```

- **1 ドラム `$:` につき bank は 1 つ**（チェーン全体に効く）
- bank 名にキット／特性を載せる: `tr808-hard`, `tr808-soft`, `accdrum-jazz`
- ファイルは `samples/{bank}_{part}.wav` または `samples/{bank}_{part}/00.wav`
- 直下の追加 `*.wav` は git 外想定（`.gitignore`）

#### 音程・FX — フルネーム（bank なし）

```
$: note("0 2 4 7").scale("C3:minor").s("pad-ambient_drone01")
  .attack(0.2).release(0.5).room(0.45).orbit(1).gain(0.35)
$: note("7 6 4").scale("C4:minor").s("lead-supersaw_4oct").lpf(3200).gain(0.16)
// piano / EP（disk: piano-acoustic_soft.wav 等。録音 root ≈ C3）
$: note("0 2 4 0").scale("C3:minor").s("piano-acoustic_soft").gain(0.35)
  .attack(0.005).decay(0.3).sustain(0.2).release(0.25)
$: note("<0 2 4 7>/2").scale("C4:major").s("piano-electric_rhodes")
  .room(0.3).orbit(1).gain(0.28)
$: s("fx-riser_short01")
```

命名目安: `{family}-{character}_{detail}`  
例: `pad-ambient_bright01`, `piano-acoustic_soft`, `piano-electric_rhodes`, `atmo-noise`。同梱 FM は `part:slug`（`bs:dk` など）。

#### 役割レシピ（シンセでも可・サンプルがあれば優先）

| 役割 | サンプルがあるとき | 無いとき（同梱のみ） |
| --- | --- | --- |
| **Pad** | `pad-ambient_*` 等フル名 + 長め ADSR / room | `pf:ff` `note("0")` at `C4:`（録音が 5 度。`[0,4]` で重ねない） |
| **Bass** | 短い hit ならフル名。持続はシンセでも可 | PCM `bs:hf` / `bs:su` at `C4:`、または `sawtooth`+lpf のシンセサブ `C2:` |
| **Lead** | `lead-*` フル名 | `ld:ss` / `plk:*` at `C4:`。`triangle` にしない |
| **Piano / EP** | `piano-acoustic_*` / `piano-electric_*` フル名 + `note`+`.scale` | `ep:rs` / `ep:ky` / `ep:mt` at `C4:` |
| **FX** | `fx-*` / `atmo-*` フル名 | `white`/`pink` + 短 ADSR + hpf |
| **Drums** | 短い part + `.bank("…")` | `bd` `sd` `hh` `oh` `cp` |

ドラムは **1 本の `s(...)` に統合**（スペース=順、カンマ=同時）。詳細は strudel-composition。

duck 付きキックだけは別トラックにしてよい（hat に duckorbit を付けない）:

```
$: s("bd*4").gain(0.9).duckorbit(2).duckattack(0.04).duckdepth(0.85)
$: s("[~ hh]*4").gain(0.4)
```

---

## 使えるモジュレーション / 音色パラメータ

`vib` / `fm` / ADSR 等は **スカラーまたは `"a:b"`**。`.lpf` / `.hpf` / `.bpf` は mini 数値パターンと `sine.rangex` も可。

### ビブラート

```
.vib(4)              // Hz
.vib("4:12")         // Hz : 深度(半音)
.vibmod(12)           // 深度。第2値があれば Hz
```

### FM（ライブ 2-op）

キャリア 1 + サイン変調 1。詳細と署名済みレシピは下の **Live 2-op FM**。

```
.fm(3)               // ピーク指数。2–4 が常用。bass に .fm(8) は割れる
.fmh(2)              // モジュレータ比。整数比 = 倍音。ヒット用の 3.5/11 は PCM
.fmatt(0.01) / .fmdec(0.3) / .fmsus(0.25)
```

ライブ `.fm` は **evolving lead / bass のみ**。プラック・ベル・メタルは PCM。
非対応: `fmenv` / `fmh2` / `.fm("3 5")` / サンプルへの 2-op パス。

### ピッチ / フィルタ・エンベロープ（簡易）

```
.penv(12)            // ピッチ env 深度
.pattack(0.01)
.pdecay(0.1)
.lpenv(0.5)          // LPF env 量
.lpattack(0.01)
.lpdecay(0.2)
.lpsustain(0.3)
.lprelease(0.1)        // パースされるが Voice::advance_lp_env は未使用。書かない
```

`lpenv` 深度は **オクターブ**: cutoff = `base * 2^(lpenv * env)`。パーノート。`.lpf(sine.rangex(…))` と併用すると LFO が env に勝つ。303 は **strudel-genre-acid**。

### 振幅 ADSR

```
.attack(0.01) / .decay(0.1) / .sustain(0.5) / .release(0.2)
.adsr("0.01:0.1:0.5:0.2")
```

別名: `att` `dec` `sus` `rel`。

### scale（次数 → 音高 / コード進行）

```
.scale("C2:minor")   // 固定: Root[:octave]:mode — オクターブ省略は 4
.scale("C:major")
.scale("A2:minor:pentatonic")
// コード進行: 1 サイクルごとに scale を切替（次数パターンはそのまま）
.scale("<A2:minor D:dorian G:mixolydian C:major>")
```

- `note("0 2 4")` / head `n("0 2 4")` の **整数**は 0 始まりのスケール次数（**負可**。例: `C2:major` の `-1` → B1）
- 音名 atom（`c2`）はそのまま
- **`.scale("<Root:mode …>")`** はコード進行（1 サイクル 1 スケール）
- `.lpf("<400 1200>")` / `.lpf(sine.rangex(500,4000))` は可。`.vib("<…>")` はまだスカラーのみ
- 詳細は strudel-composition / 明暗は **strudel-mood-bright-dark**

```
$: note("0 2 0 3 0 <2 4>").scale("C2:minor").s("sawtooth").lpf(500).gain(0.5)
$: note("0 2 4 0").scale("<A2:minor D:dorian G:mixolydian C:major>").s("sawtooth").lpf(600).gain(0.5)
```

### add / sub（移調）

```
.add(2)              // scale あり次数 → +2 度; 音名 → +2 半音
.sub(1)              // .add(-1) と同義。累積可
.add("<0 2>")        // mini 数値パターン可（イベントごとに加算）
```

```
$: note("0 2 4").scale("C2:minor").add(2).s("sawtooth").lpf(500).gain(0.5)
$: note("c4 e4").add(12).s("sine").gain(0.3)
$: note("0").scale("C4:major").add("<0 2>").s("sine").gain(0.3)
```

`.add(sine.rangex(…))` など LFO は不可（エラー文どおり mini かスカラー）。`.ply("<2 4>")` も不可（整数スカラーのみ）。

### ply（イベント分割連打・スカラー）

```
.ply(2)              // 各イベントを 2 分割して連打（1..=16、複数回は乗算）
```

```
$: s("bd*4, [~ sd]*2").ply(2).gain(0.55)
```

### ゲイン

```
.gain(0.5)
.velocity(0.8)       // または .vel — gain に乗算
```

### pan（ステレオ、等パワー）

```
.pan(0)              // 左。0.5=中央、1=右
.pan("<0 1>")        // mini 可。LFO も可
```

```
$: s("bd*4").pan(0).gain(0.7)
$: note("0 4").scale("C4:minor").s("square").pan("<0 1>").gain(0.2)
```

デッキ mix で `θ = pan · π/2`、`L=cosθ` `R=sinθ`。

---

## フィルタ（パーボイス）

| メソッド | 別名 | 備考 |
| --- | --- | --- |
| `lpf` | `cutoff` `lp` `ctf` | `"1000:8"` = cutoff:Q |
| `lpq` | `resonance` | |
| `hpf` | `hp` `hcutoff` | 同上コロン |
| `hpq` | `hresonance` | |
| `bpf` | `bp` `bandf` | 同上 |
| `bpq` | `bandq` | |

```
$: note("c3 e3 g3").s("sawtooth").lpf(400).lpq(2).gain(0.4)
$: s("bd*4").hpf(80)
// バーごと / イベントごとに cutoff を変える（mini 数値パターン）
$: s("bd*4, hh*8").lpf("<500 2000 8000 2000>").gain(0.5)
// 連続 LFO（1 バーで 1 周。Hz は rangex が指数マップ）
$: note("0 2 4 7").scale("C3:minor").s("sawtooth")
  .lpf(sine.rangex(500, 4000)).gain(0.35)
// ゆっくりスイープ
$: note("0").scale("C2:minor").s("sawtooth").lpf(sine.range(200, 2000).slow(4)).gain(0.5)
```

対応 LFO 波形: `sine` `cosine` `tri` `saw` `square` + `.range` / `.rangex` + 任意で `.slow(n)` / `.fast(n)`。  
非対応: `ftype` / `fanchor` / `vowel` / ladder 切替 / `perlin` / `rand`（未実装）。

---

## サンプル再生オプション

```
.begin(0.0)          // 再生開始位置 0..1
.end(1.0)
.speed(1.0)          // サンプル再生速度（パターン .fast/.slow とは別）
.n(0)                // サンプル index
.bank("name")        // 小文字化して接頭辞
.clip(0.5)           // イベント長スケール
.legato(0.8)
.cut(1)              // 同じ `$:` の前の音を ADSR release（即殺しない。他トラックは切らない）
```

---

## 時間スケール（パターン）

```
.fast(2)
.slow(2)
```

メソッド `.fast(n)` は `PatternCode.speed` を掛けるだけ。フルバーのブレイクをバー全体に敷くには mini `*2`（`Node::Fast`）を使う。`.fast(2)` は前半に潰れて後半が空になる。サンプルの `.speed` と混同しない。

---

## orbit / delay / room / duck / compressor

### orbit

```
.orbit(2)            // 1..4（デッキ内）。別名 .o(2)
```

- 同 orbit の **delay / room は last-write**
- 複数トラックで同じ room 設定を共有したいときも、パラメータは最後に鳴ったヒットが勝つ

### delay（orbit バス）

```
.delay(0.25)                    // wet 0..1
.delay("0.25:0.375:0.45")       // wet : time(s) : feedback
.delaytime(0.375)
.delayfeedback(0.45)            // DSP 側 max 0.95
```

### room（簡易リバーブ、orbit バス）

```
.room(0.4)
.room("0.4:4")                  // wet : size(0..10)
.roomsize(4)                    // 別名 rsize / sz / size
```

非対応: IR、`rdim`、`rlp`、`roomlp` 等。

### duck（サイドチェイン風）

```
// キックが orbit 2 を squish
$: s("bd*4").gain(0.9).duckorbit(2).duckattack(0.04).duckdepth(0.85)
// パッドは orbit 2
$: note("c3 e3 g3 c4").s("sawtooth").orbit(2).gain(0.35).lpf(900)
```

- `duckorbit` / `duck`: 対象 orbit id（コロンで最大4）
- `duckattack` / `duckdepth`: 秒 / 0..1。 **126 BPM では recover 0.03–0.05**（0.12 は緩い）。 pad **と** bass を同じ orbit に載せる。kick は default orbit 1。 hat に `duckorbit` を付けない（16th が env を retrigger する）
- 詳細レシピ: **strudel-genre-techno-duck**

### compressor（マスター、パターンから last-write）

```
.compressor("-18:3:6:.003:.12")
```

パターンの `.compressor` は **mixer master・last-write**（トラック insert ではない）。キックを潰す。レシピでは省略。

---

## 使えるメソッド一覧（クイック）

**音源・音色:** `s` `sound` `note` `n` `scale` `add` `sub` `gain` `velocity`/`vel` `pan` `noise` `vib`/`vibrato`/`v` `vibmod` `fm` `fmh` `fmattack` `fmdecay` `fmsustain` `penv` `pattack` `pdecay` `lpenv` `lpattack` `lpdecay` `lpsustain` `lprelease` `attack` `decay` `sustain` `release` `adsr`

**フィルタ:** `lpf` `lpq` `hpf` `hpq` `bpf` `bpq`（および表の別名）

**サンプル:** `begin` `end` `speed` `bank` `clip` `legato` `cut`

**時間:** `fast` `slow` `ply`

**バス / FX:** `orbit`/`o` `duckorbit`/`duck` `duckattack` `duckdepth` `delay` `delaytime` `delayfeedback` `room` `roomsize`/`size` `compressor`

これ以外のメソッド名は **unsupported method**。

---

## 明示的に使わない（本家にあって dj-hermes に無い）

| カテゴリ | 例 |
| --- | --- |
| ZZFX | `z_sawtooth` `zmod` `zcrush` … |
| 加算合成 | `partials` `phases` `sound("user")` |
| 可視化 | `_scope` `_spectrum` `_punchcard`（曲内関数） |
| フィルタ拡張 | `vowel` `ftype` `fanchor` |
| 歪み・特殊 | `distort` `crush` `coarse` `shape` |
| モジュレーション | `tremolo` `phaser` `detune`（メソッド）。`pan` は実装済み |
| 外部サンプル DSL | `samples('github:...')` `loopBegin`/`loopEnd` |
| 引数のミニ記法 | `.vib("<1 4>")` など **未対応メソッド**のパターン引数。`.lpf("<400 1200>")` / `.add("<0 2>")` / `.pan("<0 1>")` と `sine.rangex` は可。`.add` の LFO と `.ply("<…>")` は不可 |
| その他 | `beat` `seg` `crackle` `density` `stretch`。`supersaw` 波形名は無い。`ld:ss` は PCM |

---

## 信号のイメージ（dj-hermes）

1. 音源（波形 / ノイズ / wt / サンプル）
2. パーボイス: FM・vib・noise mix・ADSR・biquad lpf/hpf/bpf・penv/lpenv
3. Deck 内 orbit 合算 → orbit delay / room（wet）
4. Mixer: チャンネル EQ・フェーダー・マスター LPF/HPF・compressor

---

## 実例（短いライブループ・そのまま apply 可能）

```
// @title sound-demo
setcpm(126/4)
// duck 付きキックだけ分離（hat に duckorbit を付けない）
$: s("bd*4").gain(0.9).duckorbit(2).duckattack(0.04).duckdepth(0.85)
$: s("[~ hh]*4").gain(0.4)
// FM ベース（次数）— pad と同じ orbit 2
$: note("0 0 2 4").scale("C2:minor").s("sine").fm(3).fmh(1.5).lpf(500).gain(0.55)
  .attack(0.005).decay(0.1).sustain(0.3).release(0.08)
  .orbit(2)
// duck されるパッド
$: note("0 2 4 7").scale("C3:minor").s("sawtooth").lpf(900).orbit(2).gain(0.35)
  .attack(0.05).decay(0.2).sustain(0.6).release(0.2)
```

ライブ差分例: パッドの `.lpf(900)` → `600`、または bass `.fm(3)` → `5` は `dj_hermes_edit_method(op=set)`。残すときだけ `dj_hermes_save_song`。

---

## Common Pitfalls

1. **未対応メソッドへ `"<...>"` を渡す**（`.vib("<1 4>")` 等）→ パース失敗。`.lpf("<…>")` と `sine.rangex` は可。
2. **未対応メソッド**（`phaser` `vowel` `distort` `partials` `lfo` …）→ そのパターン行がエラー。
3. **同じ orbit で delay/room を複数トラックから書く** → last-write で上書き。役割ごとに orbit を分ける。
4. **`c3'maj` で和音が鳴ると思わない** → root のみ。和音は `note("0 2 4")` / `[6,8]` 等で書く。
5. **ZZFX / supersaw / 外部 wt** は使えない。波形・wt_sine/bright/organ・サンプルに寄せる（ユーザー `lead-supersaw_*` WAV があればフル名で可）。
6. 音色のために 16 小節 `cat` を書かない → 4 小節フレーズのままスカラーを触る。
7. **`kit:bd`**（bank が左）は不可。カタログは `bd:8b`。同梱は `s("bd")` / `.n(0)` / `.bank("kit")`。
8. **ドラムをフルネームで埋める**（`s("tr808-hard_bd …")`）→ リズムが読めない。短い part + `.bank`。
9. **bank のファイル名を `{bank}-{part}` にする** → 正は **`{bank}_{part}`**（アンダースコア）。
10. **深いパス** `pad/ambient/x.wav` → 読まれない。フラット or 1 段フォルダ（LAYOUT.md）。

## Verification Checklist

- [ ] sound は波形 / white|pink|brown / wt_* / ローカル sample のいずれか
- [ ] ドラムは短い part。キット差は `.bank`（ディスクは `{bank}_{part}`）
- [ ] pad/lead/piano/FX はフルネーム（bank なし）またはシンセ代替
- [ ] メソッドは上記一覧のみ
- [ ] メソッド引数は数値 / `a:b` / 対応メソッドの mini・LFO のみ
- [ ] delay/room を共有するトラックは orbit を意識している
- [ ] duck する側に `duckorbit`、される側に同じ `orbit`

---

## Live 2-op FM

Live `.fm` / `.fmh` is **only** for **time-varying lead and bass synths**. Drums and one-shots are **PCM** (`s("bd")`, `s("cp")`, `s("plk:lp")`, `s("plk:s5")`, …). Do not build pluck / bell / metal-hit as live 2-op one-shots.

Apply the inline FM recipe with `dj_hermes_apply_song` (`setcpm(124/4)`). How to trigger issue #21 batch 1 factory stems (`C4:…` / unpitched FX): [Factory PCM batch 1](#factory-pcm-batch-1).

## When to use

- You need an **evolving** lead / pad / growl bass that this engine synthesizes.
- You need to know **how the 2-op path runs** (`synth.rs`) so the signed-off numbers make sense.
- You are **not** writing a house / techno / DnB grid recipe (those skills own the drums).
- You are **not** recreating a `rust-fm-synthe` TOML patch with `.fm(4)`. Factory files are **samples**.
- You are **not** replacing the lead with the retired live EP recipe, or with PCM `ep:ky` (tine harmonics landed in #37; that wav is a one-shot, not this lead).

## Two FM worlds (keep them distinct)

| World | What it is | How you trigger it here |
| --- | --- | --- |
| **Live 2-op** | One **carrier** + one **sine modulator**. Index, ratio, ADS envelope on the index. | `.s("sine")` or `.s("sawtooth")` + `.fm` / `.fmh` / `.fmdec` / `.fmsus` — **lead and bass only** |
| **Offline 4-op factory** | [sin5ddd/rust-fm-synthe](https://github.com/sin5ddd/rust-fm-synthe) renders WAV one-shots | `s("plk:lp")`, `s("plk:s5")`, `s("plk:s3")`, `s("cp")`, … The engine **samples** the wav |

`.fm(4)` does **not** reproduce a 4-op TOML patch. There is no algorithm graph, no `fmh2`…`fmh8`, no second modulator.

## How live FM actually works

`Voice::osc_sample_modulated` + `Voice::fm_env_level` in `synth.rs`. Methods are `parse_num` **scalars** (`code.rs`).

```
fm_idx    = fm * fm_env_level()
mod_sig   = sin(2π · mod_phase)          // modulator is always a sine
mod_phase += (fmh * freq) / sr           // fmh clamped to min 0.01
inst_freq = max(freq + fm_idx * freq * mod_sig, 0.1)
carrier   += inst_freq / sr
```

| Knob | Engine meaning | Musical use |
| --- | --- | --- |
| `.fm(index)` | Peak deviation (Hz) = `fm_idx * carrier_hz` at env 1 | **Index 2–4** is everyday. **8+ breaks up fast.** Do not put `.fm(8)` on bass. |
| `.fmh(ratio)` | Modulator advances at `fmh * freq`. Default **1.0** | **Signed-off recipes use integer `fmh`** (`1` growl/pad, `2` lead) = harmonics. `3.5` / `11` = bell / metal **timbre** — those hits are **PCM**, not live FM one-shots. Do **not** copy `techno-duck-01`'s `.fmh(1.5)` into this rule. |
| `.fmdec` / `.fmsus` | Index env: attack → decay toward sustain. **No FM release.** Defaults 0.001 / 0.1 / **0.0** | `fmsus(0)` is percussive (do not use live FM for that). **Leave sustain** on evolving leads / pads. |
| `.noise(0..1)` | Pink **mix** into the osc | Not a second operator |

Carrier is `.s("sine"|"triangle"|"sawtooth"|"square"|wt_*)`. The modulator waveform is not selectable. `.s("pink")` is a noise *source* and does not use `inst_freq` as a carrier — do not FM that.

Amp ADSR is a **different** envelope (defaults 0.01 / 0.1 / 0.7 / 0.1). It scales the voice after FM.

| Idiom | Works? |
| --- | --- |
| Scalar `.fm` / `.fmh` / `.fmattack` / `.fmdecay` / `.fmsustain` (`fmatt` / `fmdec` / `fmsus`) | **Yes** |
| `.fm("3 5")` / patterned `fmh` / `fmh2` / `fmenv` | **No** |
| Live FM on a sample (`s("plk:lp").fm(4)`) | Plays the **wav**; SampleVoice does not run this 2-op path |

## Signed-off live recipes (synthesist + Beatmaker)

Carrier is **sine** unless noted. Put these numbers **as-is**. Mix: the growl already fills the mids — **do not** stack a square sub, `bs:rm`, pluck, or stab under it.

### Lead — C4 and above, evolving (not a one-shot)

```
$: note("4 2 0 2").scale("C4:minor")
  .s("sine").fm(3).fmh(2).fmatt(0.01).fmdec(0.3).fmsus(0.25)
  .lpf(1800).lpenv(2).gain(0.3)
```

Degrees `4 2 0 2` in C minor = **5–♭3–1–♭3** (G–Eb–C–Eb). Integer `fmh(2)` = harmonics. `.fm(3)` + `.fmatt(0.01)` + `.fmdec(0.3)` + `.fmsus(0.25)` keeps the index moving through the note (lead, not a hit). `.lpf(1800).lpenv(2)` is the per-note filter sweep on that parked base.

The live EP recipe (`.fm(2).fmh(1).fmdec(0.6).fmsus(0.15)` on `0 ~ 4 2`) is **retired**. PCM `ep:ky` is a tine one-shot (#37), not this lead.

### Pad — C4 and above; leave `fmsus`, add `.room`

```
$: note("[0,4]").scale("C4:minor")
  .s("sine").fm(1.2).fmh(1).fmdec(0.8).fmsus(0.4)
  .room(0.3).orbit(2)
  .gain(0.16)
```

`.room` is per-orbit, last-write (`deck.rs`). Pad on **orbit 2** so the bass / lead stay dry. Do not drop `.fmsus(0.4)` — sustain is what makes it a pad.

### Growl bass — do not stack on square / `bs:rm`

```
$: note("0 0 3 0").scale("C2:minor")
  .s("sawtooth").fm(4).fmh(1).fmdec(0.25).fmsus(0.2)
  .lpf(400).lpenv(3)
  .gain(0.38)
```

`.fm(4).fmh(1)` + `lpf(400)` already fills the mids. `.lpenv(3)` is the per-note filter sweep on that parked base (same env law as the 303 skill: `cutoff = base * 2^(lpenv * level)`). **Do not** put `.fm(8)` on bass (it breaks up). **Do not** add a square sub or `bs:rm` on another `$:` — that is a different mix ([strudel-genre-dnb-reese-mid-stab](../strudel-genre-dnb-reese-mid-stab/SKILL.md)).

`songs/techno-duck/01.strudel` has another live FM bass (`.s("sine").fm(3).fmh(1.5).lpf(500)` at 126). That is this 2-op world, **not** a signed-off recipe here. Do not fold `.fmh(1.5)` into the integer-ratio rule above.

## PCM one-shots (not live FM)

Pluck, bell, and metal-hit were dropped from live 2-op. Trigger the factory / kit wavs. `SAMPLE_ROOT_HZ` is **261.63 Hz (C4)** (the comment in `sample.rs` says C3). Bundled FM wavs are **C3** recordings — write **`C4:…`**.

This demo does **not** stack those one-shots on the growl (mids fill up). Keep them PCM when you need them elsewhere.

| Use | Sound key | How to write | Do not |
| --- | --- | --- | --- |
| Drums / clap | `bd` `sd` `hh` `oh` `cp` | `s("bd*4")` etc. | Live `.fm` on a kick |
| プラック | `plk:lp` | `note("…").scale("C4:minor").s("plk:lp")`. `.cut(1)` **only if monophonic** | `lead-fm-pluck`; `C3:minor`; live `.fm` pluck; stacking it on this demo |
| ベル / メタルヒット | `plk:s5` / factory bell-metal wavs | Factory/PCM, **low gain**, not chords | Live `.fmh(3.5)` / `.fmh(11)` hits |
| Hollow fifth | `plk:s5` | Transposes C–G (no third). Sparse degrees, low gain. Dark-side stab if you must — **prefer none** on this mix | Inventing a third; `stab-fm-fifth` |
| Major stab | `plk:s3` | Already a **C–E–G** triad wav: `note("0 ~ 0 ~").s("plk:s3")` | On this **minor** demo (E vs Eb); `note("[0,2,4]")` **triples** it |
| EP one-shot | `ep:ky` | PCM tine (#37). Not the live lead | Using it as the evolving lead; live EP recipe |

`expand_chord("c3'maj")` is not called by the deck. `note("c3'maj")` is the **root only**.

Inharmonic `fmh` (3.5, 11) is why those factory wavs sound like bell / metal. That is **lore**, not a live one-shot recipe.

## Playable song

Minor growl bass + evolving lead + pad only. Drums are PCM. **No** pluck, **no** `plk:s3`. One clock: **124 BPM**.

```
// @title skill-fm-sound-design
// @details live FM growl+lead+pad; drums are PCM
setcpm(124/4)
// drums
$: s("bd*4").gain(0.3)
// bass
$: note("0 0 3 0").scale("C2:minor")
  .s("sawtooth").fm(4).fmh(1).fmdec(0.25).fmsus(0.2)
  .lpf(400).lpenv(3)
  .gain(0.38)
// lead
$: note("4 2 0 2").scale("C4:minor")
  .s("sine").fm(3).fmh(2).fmatt(0.01).fmdec(0.3).fmsus(0.25)
  .lpf(1800).lpenv(2).gain(0.3)
// pad
$: note("[0,4]").scale("C4:minor")
  .s("sine").fm(1.2).fmh(1).fmdec(0.8).fmsus(0.4)
  .room(0.3).orbit(2)
  .gain(0.16)
```

`bd*4` is a **pulse**, not a signed-off house/techno grid (no `[~ cp]*2`, no `[~ hh]*4`, no 174 break).

This file is **124** — same clock as house / mood / minor-scale. A leftover **120** file is isolated; **do not DJ-pair 120 with 124** (shared Transport discards the other tempo). Do not pair with 126 techno or 174 DnB.

No `.compressor` (mixer master, last-write). No `.duckorbit`. No square sub. No `bs:rm`. No pluck. No stab.

## Try it in this app

```bash
# Apply the inline recipe with dj_hermes_apply_song.
# Existing 124 pair:
dj-hermes dj songs/house/01.strudel songs/four-on-the-floor/01.strudel
```

No device: `cargo test --test e2e house_01 -- --nocapture`.

Live TUI: apply the inline recipe with `dj_hermes_apply_song`.

## Variations (still this syntax)

| Goal | Change |
| --- | --- |
| Lead only | mute or drop the pad `$:` |
| Darker pad | keep `.fmsus(0.4)` and `.room`; do not zero sustain |
| House bed | do **not** invent a grid — [strudel-genre-house](../strudel-genre-house/SKILL.md) is the **124** house grid. This pulse stays `bd*4` |
| Dark stab | prefer **none**. If you must, `plk:s5` (hollow fifth, no third) — never `plk:s3` on this minor demo |

Do not “vary” by turning the lead into a live FM pluck (`fmsus(0)` + short decay) or a live bell (`.fmh(3.5)`). Those are PCM. Do not bring back the retired live EP.

## Rules (do not skip)

1. Live `.fm` / `.fmh` = **evolving lead and bass only**. Hits are PCM.
2. Signed-off numbers stay as written. Integer `fmh` on those recipes (`1` growl/pad, `2` lead). Index 2–4 everyday; **no `.fm(8)` on bass**. Do not copy `techno-duck-01`'s `.fmh(1.5)` into that rule.
3. Growl fills the mids — **no** square sub, **no** `bs:rm`, **no** pluck, **no** stab on this demo.
4. Factory wavs: `C4` on C3 recordings. `plk:s3` is `note("0 ~ 0 ~")` only — and **not** on this minor file. `.cut(1)` on pluck only if monophonic.
5. This file is **124**. Leftover **120** files stay isolated. Do not DJ-pair 120 with 124.

## Do not

- Teach pluck / bell / metal-hit as live 2-op one-shots.
- Invent 4-operator algorithms, `fmh2`, or “this `.fm(4)` is the factory pluck”.
- Stack square, `bs:rm`, pluck, or stab under the growl.
- Put `.fm(8)` on bass.
- Put `plk:s3` on this C-minor demo (baked E vs the lead's Eb).
- Play `plk:s3` as `note("[0,2,4]")` (triples the baked triad).
- Revive the live EP recipe, or substitute PCM `ep:ky` for the evolving lead.
- Copy `techno-duck-01`'s `.fmh(1.5)` into these integer-ratio recipes.
- Write `C3:minor` on `plk:lp` / `plk:s5` / `plk:s3` (`SAMPLE_ROOT_HZ` is C4).
- Write `lead-fm-pluck` or `stab-fm-fifth` (wrong stem).
- Put `.compressor` or `.duckorbit` on these recipes.
- Borrow a house/techno/DnB drum string and change it. Pulse here is `bd*4` only.
- Pair a leftover 120 file with this 124 file (shared clock). Do not pair 126 / 174 either.
- `stack()` / `.cpm(124)` / `.fm("3 5")`.

---

## Factory PCM batch 1

This skill is **how each factory one-shot is triggered** — keys, register, and
what the wav already contains. It is **not** a house / techno / DnB loop recipe.
Those skills own the drum grids. Playable files are **separate beds** at the
same 124 clock: floor (drums + house bass + pad), dark Reese, supersaw lead.

Existing stems stay on their own skills. Do not rewrite those recipes:

| Stem | Skill |
| --- | --- |
| `bs:rm` | [strudel-genre-dnb-reese-mid-stab](../strudel-genre-dnb-reese-mid-stab/SKILL.md) |
| `plk:lp` | [strudel-genre-house](../strudel-genre-house/SKILL.md) |
| Live 2-op `.fm` | [Live 2-op FM](#live-2-op-fm) (PCM hits are not that path) |

This batch adds **no** `bd/` bank and **no** third-party drum kit (issue #21
license). Drums stay the bundled folder keys `bd` / `cp` / `hh` / `sd` / `oh`.

## When to use

- You need the signed-off `.s(...)` / `.scale(...)` for a **batch 1** factory wav.
- You need to know **why C4 plays native pitch**, **why FX have no `note()`**,
  **why `pf:ff` cannot brighten**, **why `bs:dk` ≠ `bs:rm`**.
- You are **not** inventing a genre grid. Do not rewrite signed-off hook degrees. Extra tracks follow **strudel-composition**.

## Engine: `SAMPLE_ROOT_HZ` is C4

`note().s("sample")` sets playback rate to `target_hz / SAMPLE_ROOT_HZ`
(`deck.rs`). `SAMPLE_ROOT_HZ` is **261.63 Hz (C4)** even though the constant
comment in `sample.rs` says C3.

Ratio **1.0** plays the recording at its **native** pitch. All pitched one-shots
in this batch were recorded at **C3 or C2**. Writing `.scale("C4:…")` on
degree 0 targets C4, so the ratio is 1.0 and you hear C2 or C3 as recorded.
Writing `C3:…` targets C3 (~130.8 Hz) → ratio **0.5** → an **extra octave down**.

| Written scale (degree 0) | Target Hz | Ratio | Heard (C3 wav) | Heard (C2 wav) |
| --- | --- | --- | --- | --- |
| `.scale("C4:minor")` | 261.63 (C4) | 1.0 | native C3 | native C2 |
| `.scale("C3:minor")` | ~130.8 (C3) | 0.5 | C2 (dumped) | C1 (dumped) |
| `.scale("C2:minor")` | ~65.4 (C2) | 0.25 | C1 | C0 |

Always write **`C4:…`** for native pitch on this batch. Do not “match” the
recorded octave in the scale string.

Bare `s("name")` (no `note()`) sets `is_note = false` (`code.rs`). Then
`pitch_ratio` is **1.0** (`deck.rs`) — the wav plays as recorded, unpitched.

## Pitched stems (always `C4:…` for native pitch)

These batch-1 files sit at `samples/<part>/<slug>.wav`. Call them as
`s("part:slug")`. Old flat stems (`bass-fm_house`, `reese-dark`, …) are gone.

### `bs:hf` — recorded C2, house floor under the kick

```
setcpm(124/4)
$: note("0 0 4 0").scale("C4:minor").s("bs:hf").gain(0.45)
```

Native **C2** (~65 Hz). Tight house floor. `C4:minor` yields that C2. Do **not**
put it at `C3:…` — musically that would be a mid-bass **on top of the kick**,
and the engine would dump an extra octave anyway (you hear C1, not C3).
Key is `bs:hf` (`samples/bs/hf.wav`). Not `bass-fm_house` or `bass-fm-house`.

### `bs:su` — recorded C2 (~65 Hz), floor only

```
setcpm(124/4)
$: note("0 0 4 0").scale("C4:minor").s("bs:su").gain(0.5)
```

Clean sine sub. `C4:…` yields native C2. **Floor only.** Do **not** stack with
`bs:dk` or a square sub (double basement). Not in the house-floor song.

### `bs:dk` — dark Reese **with** ~65 Hz sub

```
setcpm(124/4)
$: note("0 3 0 <0 -1>").scale("C4:minor").s("bs:dk").gain(0.35)
```

Full-range dark Reese (sub + mid). **Not** a band-swap for `bs:rm`.
`bs:rm` is 800–1200 Hz glue with **no** sub and still needs a square C2
([strudel-genre-dnb-reese-mid-stab](../strudel-genre-dnb-reese-mid-stab/SKILL.md)). `bs:dk` already
owns ~65 Hz. Do **not** stack with square sub, `bs:su`, or
`bs:hf` (all have sub). This is a **different bed** from the house
floor — own file, not the pad + `bs:hf` song.
Key is `bs:dk` (`samples/bs/dk.wav`). Not `reese-dark` or `reese_dark`. Catalog `bs:rd` is a different take.

### `pf:ff` — hollow C3+G3, **no third**

```
setcpm(124/4)
$: note("0 ~ 0 ~").scale("C4:minor").s("pf:ff").gain(0.25)
```

Sustained fifth pad (C and G only). `note()` only **transposes** that recording.
It cannot invent a major (or minor) third. Do **not** use this to brighten.
Do **not** play it as `note("[0,2,4]")` — that stacks three hollow fifths
(root / third / fifth), still no E or Eb inside the wav.
Key is `pf:ff` (`samples/pf/ff.wav`). Not `pad-fm_fifth` or `pad-fm-fifth`.
Not a swap for `plk:s5` (that one is a short stab).

### `ld:ss` — recorded C3, melody at `C4:minor`

Same register rule as `plk:lp`. Do **not** stack this on the floor pad
or the dark-Reese bed (it fills the mids). Separate song at the same 124 clock.

```
setcpm(124/4)
$: note("4 ~ 7 4").scale("C4:minor").s("ld:ss").gain(0.28).cut(1)
```

The wav is a long hold (~8 s). `.cut(1)` steals the previous shot so the
melody stays monophonic. Key is `ld:ss` (`samples/ld/ss.wav`).
Not `lead-supersaw` or `lead_supersaw`. Do not rewrite the [strudel-genre-house](../strudel-genre-house/SKILL.md)
pluck line to this stem.

## Unpitched FX — `s("name")` only, **never `note()`**

These are gestures, not pitched instruments. Bare `s("…")`. No `.scale`.

| Disk | Sound key | Role |
| --- | --- | --- |
| `samples/fx/up.wav` | `fx:up` | Uplifter (pitch + filter open, ~15 s / 8 bars at 130 BPM) |
| `samples/fx/nr.wav` | `fx:nr` | Noise riser (~15 s / 8 bars at 130 BPM) |
| `samples/fx/rf.wav` | `fx:rf` | Filter-open riser (~15 s / 8 bars at 130 BPM) |
| `samples/fx/rp.wav` | `fx:rp` | Pitch riser (~15 s / 8 bars at 130 BPM) |
| `samples/fx/rw.wav` | `fx:rw` | Supersaw riser (~15 s / 8 bars at 130 BPM) |
| `samples/fx/fr.wav` | `fx:fr` | FM riser (~15 s / 8 bars at 130 BPM) |
| `samples/fx/id.wav` | `fx:id` | DnB impact (~0.5 s) |
| `samples/fx/sd.wav` | `fx:sd` | Sub drop (~50 Hz, ~1.1 s) |

Long one-shots must **not** fire every bar. The six risers are ~15 s; a
bar at 124 BPM is ~1.94 s. `s("fx:up")` overlaps itself. Mini `<>` picks
one child **per cycle** (`mini.rs` `Node::Stack`):

```
setcpm(124/4)
$: s("<fx:up ~ ~ ~ ~ ~ ~ ~>").gain(0.3)
```

That is once per **8 bars**. Same idea for `fx:nr` / `fx:rf` / `fx:rp` /
`fx:rw` / `fx:fr`. Short hits (`fx:id` ~0.5 s) can sit on a denser grid.

```
setcpm(124/4)
$: s("<fx:nr ~ ~ ~ ~ ~ ~ ~>").gain(0.28)
$: s("fx:id").gain(0.35)
$: s("<fx:sd ~ ~ ~>").gain(0.35)
```

`fx:sd` is already ~50 Hz. It still gets **no** `note()`. Kick-lead-in,
not a bass note. Wrong keys (`fx-riser-noise`, `fx_uplifter`) fail resolve;
the current performance continues.

## Why C4 vs recorded C2 / C3

The engine does not know the recording key. It always divides by **C4**.

A C2 house bass recorded at ~65 Hz is **already** the floor. Degree 0 at
`C4:minor` leaves the rate at 1.0, so you hear that C2 under the kick.
Writing `C3:minor` (or `C2:minor`) is “I want the written octave” — the
ratio drops to 0.5 or 0.25 and the floor disappears an octave (or two).

A C3 pad / Reese / supersaw works the same way: `C4:…` = native C3.
`C3:…` dumps them into the bass register. That is why `plk:lp` and
`bs:rm` already use `C4:minor` — same constant, same rule.

## Why FX have no `note()`

`s("fx:up")` is not a note head (`code.rs` `is_note = false`).
`deck.rs` then uses `pitch_ratio = 1.0`. The riser / impact / drop already
has its pitch motion **baked into the wav**.

`note("0").scale("C4:minor").s("fx:sd")` would retune a ~50 Hz drop
by `target / C4`. That stretches the gesture and moves the basement. Leave
FX unpitched even when the recording is low.

## Why `pf:ff` cannot brighten

The wav is **C3+G3** (ratios 1 and 3/2). No third in the file.

`note().s("pf:ff")` only changes playback rate. Deck scheduling still
plays **one pitch per mini event**; `expand_chord` is not called.
`note("c3'maj")` is the **root only**.

`note("[0,2,4]")` fires three events (degrees 0, 2, 4). Each event is still
the hollow fifth, transposed. You hear stacked C–G / Eb–Bb / G–D — **not**
a major triad and **not** a brighter pad. Brightening in this engine is a
mode / voicing / sample swap ([strudel-mood-bright-dark](../strudel-mood-bright-dark/SKILL.md)),
not this stem.

## Why `bs:dk` ≠ `bs:rm`

| | `bs:rm` | `bs:dk` |
| --- | --- | --- |
| Band | 800–1200 Hz, **no sub** | Full-range, **with ~65 Hz sub** |
| Scale | `C4:minor` (native C3 mid) | `C4:minor` (native C3 + baked sub) |
| Needs a synth sub? | Yes — square at `C2:minor` | **No** |
| Swap? | Not a dark version of the other | Not a mid-band replacement |

Do not put `bs:dk` on the DnB mid track and drop the square. Do not put
`bs:rm` under the house floor. Do not put `bs:dk` on the same `$:`
list as `bs:hf` (both have sub). Do not stack it with `bs:su`
or `s("square")` + `lpf(120)`.

## Sample keys (`part:slug`)

One-level load only. Folder `samples/<part>/<slug>.wav` → `s("part:slug")`.
Old flat stems at `samples/<stem>.wav` are gone.

| Disk | Sound key | Silent (wrong) |
| --- | --- | --- |
| `samples/bs/hf.wav` | `bs:hf` | `bass-fm_house` / `bass-fm-house` |
| `samples/bs/su.wav` | `bs:su` | `bass-fm_sub` / `bass-fm-sub` |
| `samples/bs/dk.wav` | `bs:dk` | `reese-dark` / `reese_dark` |
| `samples/pf/ff.wav` | `pf:ff` | `pad-fm_fifth` / `pad-fm-fifth` |
| `samples/ld/ss.wav` | `ld:ss` | `lead-supersaw` / `lead_supersaw` |
| `samples/fx/nr.wav` | `fx:nr` | `fx-riser_noise` / `fx-riser-noise` |
| `samples/fx/rf.wav` | `fx:rf` | `fx-riser-filter` / `fx_riser_filter` |
| `samples/fx/rp.wav` | `fx:rp` | `fx-riser-pitch` / `fx_riser_pitch` |
| `samples/fx/rw.wav` | `fx:rw` | `fx-riser-saw` / `fx_riser_saw` |
| `samples/fx/fr.wav` | `fx:fr` | `fm-riser` / `fm_riser` |
| `samples/fx/id.wav` | `fx:id` | `fx-impact_dnb` / `fx-impact-dnb` |
| `samples/fx/sd.wav` | `fx:sd` | `fx-sub_drop` / `fx-sub-drop` |
| `samples/fx/up.wav` | `fx:up` | `fx-uplifter` / `fx_uplifter` |

No `.bank(...)` on these names. This batch did not add a `bd/` kit. User extras
may still use a full stem (`pad-ambient_drone01`).

## Playable songs (do not rewrite **signed-off degrees**)

Track count follows **strudel-composition** (7–8 `$:`). Do not “improve” the signed-off hook degrees or drum grids below. Adding the missing lead/arp/chords/pad slots is the composition skill, not a rewrite of these stems.

**Floor** = drums + `bs:hf` + `pf:ff` only. House bass is
`0 0 4 0` (i and 5). Pad stays `0 ~ 0 ~`. FX is
`<fx:up ~ ~ ~ ~ ~ ~ ~>` — once per 8 bars, not every bar. **No** `bs:dk`
here (both that stem and the house bass have sub).

```
// @title skill-factory-pcm-usage
setcpm(124/4)
$: s("bd*4, [~ cp]*2, [~ hh]*4").gain(0.65)
$: note("0 0 4 0").scale("C4:minor").s("bs:hf").gain(0.45)
$: note("0 ~ 0 ~").scale("C4:minor").s("pf:ff").gain(0.25)
$: s("<fx:up ~ ~ ~ ~ ~ ~ ~>").gain(0.3)
```

Apply the inline recipe with `dj_hermes_apply_song`. `songs/house/01.strudel` is a live factory-PCM house floor.

### Dark Reese (different bed, same 124 clock)

No `bs:hf`, no square sub.

```
// @title skill-factory-pcm-reese
setcpm(124/4)
$: s("bd*4, [~ cp]*2, [~ hh]*4").gain(0.65)
$: note("0 3 0 <0 -1>").scale("C4:minor").s("bs:dk").gain(0.35)
```

Apply the inline recipe with `dj_hermes_apply_song`. Play **solo**. Do not `dj` this
with the floor or the lead (those files already have `bs:hf`).

### Lead only (same clock, not stacked on the floor pad)

Signed-off: degrees `4 ~ 7 4` (C minor 5–1–5). The wav is ~8 s so `.cut(1)`
is required. Do not change this grid.

```
// @title skill-factory-pcm-lead
setcpm(124/4)
$: s("bd*4, [~ cp]*2, [~ hh]*4").gain(0.65)
$: note("0 0 4 0").scale("C4:minor").s("bs:hf").gain(0.45)
$: note("4 ~ 7 4").scale("C4:minor").s("ld:ss").gain(0.28).cut(1)
```

Apply the inline recipe with `dj_hermes_apply_song`. Shared `setcpm(124/4)` so the
floor + lead pair can `dj`. Do not add pad or `bs:dk` on this file.

## Try it in this app

```bash
# Apply the inline factory-PCM recipes with dj_hermes_apply_song.
# Existing 124 pair:
dj-hermes dj songs/house/01.strudel songs/four-on-the-floor/01.strudel
```

No device: `cargo test --test e2e house_01 -- --nocapture`.

Live TUI: apply the inline recipes with `dj_hermes_apply_song`.

## Do not

- Write `C3:…` or `C2:…` on these pitched stems — that dumps octaves.
- Put `bs:hf` at C3 (on top of the kick) or hear it as a mid-bass.
- Put `bs:dk` on the same song as `bs:hf` (both have sub).
- Stack `bs:dk` with `bs:su` or square sub.
- Treat `bs:dk` as a darker `bs:rm` (or the reverse).
- Use `pf:ff` to brighten, or play it as `[0,2,4]`.
- Fire `fx:up` every bar (`s("fx:up")` overlaps; use `<>`).
- Put `note()` / `.scale` on `fx:up`, `fx:nr`, `fx:rf`, `fx:rp`,
  `fx:rw`, `fx:fr`, `fx:id`, or `fx:sd` (including the ~50 Hz drop).
- Stack `ld:ss` on the floor pad or the Reese bed.
- Rewrite the lead degrees `4 ~ 7 4` or drop `.cut(1)`.
- Rewrite `bs:rm` / `plk:lp` / live 2-op recipes to these stems.
- Add a `bd/` bank or a third-party drum kit from this batch.
- Write old flats (`bass-fm_house`, `pad-fm_fifth`, `reese-dark`, `fx-uplifter`) or hyphen swaps (`bass-fm-house`, `pad-fm-fifth`, `reese_dark`, `fx-riser-noise`).
- Put `.compressor` on a `$:` (mixer master, last-write).
- Pair these 124 files with 174 DnB or 126 techno (shared clock).
- `stack()` / `.cpm(124)`.
