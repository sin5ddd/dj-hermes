---
name: strudel-sound-design
description: "Use when designing synths, samples, banks, or effects for short live loops in strudel-rs."
version: 3.2.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, sound-design, synthesis, effects, live-coding]
    related_skills:
      - strudel-data-format
      - strudel-composition
      - strudel-live-edit
      - strudel-genre-acid
      - strudel-genre-electro
      - strudel-genre-minimal-techno
      - strudel-genre-house
      - strudel-genre-dnb
      - strudel-genre-ambient
      - strudel-genre-chill
      - strudel-genre-dubstep
      - strudel-genre-progressive-house
      - strudel-genre-future-bass
      - strudel-genre-lofi-hiphop
      - strudel-genre-chill-pop
---

# strudel-rs サウンドメイク（Sound Design）

## Overview

この Skill は **strudel-rs**（Rust 自前 DSP）向け。WebAudio 版 Strudel REPL の全機能は持たない。  
音源・メソッドは実装済みのものだけ。パターンの長さ・ライブ差分は **strudel-composition**（短いループが既定）。

**サンプル配置の正本（disk）:** リポジトリ `samples/LAYOUT.md`（bank キー・フルネーム・gitignore）。

**二系統:**

| 系統 | パターン内の名前 | bank | 例 |
| --- | --- | --- | --- |
| **ドラム** | 短い `bd` `sd` `hh` `oh` `cp` … | **キット用に付ける** | `s("bd*4, [~ sd]*2").bank("tr808-hard")` |
| **音程 / pad / lead / piano / FX** | **フルネーム** | 付けない | `s("pad-ambient_drone01")` / `s("piano-acoustic_soft")` |

**ライブで触るとわかりやすいツマミ**: `.lpf` / `.lpq` / `.hpf` / `.bpf` / `.gain` / ADSR / `.room` / `.delay` / `.fm` / `.vib`  
スカラーを 1 つ変えて同じ曲名で save する。

**本家にあって未実装の一覧:** `docs/strudel-gap-synths-fx.md`。例に **`.lfo` は書かない**。  
**明暗（キーの印象）は scale モードを優先**（→ strudel-live-edit）。`.lpf` は音色の副次。

## When to Use

- 短い `$:` ループの音色・FX を決める / ライブで 1 パラメータ変える時
- 波形 / ノイズ / 内蔵 wavetable / サンプル + 対応エフェクトを選ぶ時

Don't use for: パターン記法の詳細（→ strudel-composition）、曲ファイル形式（→ strudel-data-format）、**本家 Strudel 専用**のシンセ（ZZFX・partials・phaser 等）。

---

## 重要: 本家 Strudel との差

| 項目 | strudel-rs |
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

- `s("bd")` = その sound の **n=0**（先頭 WAV）。**`bd:00` は書けない**（mini に `:` 不可）
- 未同梱 `cp` はデフォルトでは使わない（ユーザーキットで `…_cp` を置いたときだけ）

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
例: `pad-ambient_bright01`, `piano-acoustic_soft`, `piano-electric_rhodes`, `reese-dark`, `atmo-noise`。

#### 役割レシピ（シンセでも可・サンプルがあれば優先）

| 役割 | サンプルがあるとき | 無いとき（同梱のみ） |
| --- | --- | --- |
| **Pad** | `pad-ambient_*` 等フル名 + 長め ADSR / room | `wt_organ` / `sawtooth` + 長い attack/release + room |
| **Bass** | 短い hit ならフル名。持続はシンセでも可 | `sine`+FM または `sawtooth`+lpf、`C2:` |
| **Lead** | `lead-*` フル名 | `square` / `triangle` / `wt_bright` + 低 gain |
| **Piano / EP** | `piano-acoustic_*` / `piano-electric_*` フル名 + `note`+`.scale` | 代替弱め: `triangle`/`wt_sine` + 短 attack・中 release（本物のピアノ感はサンプル推奨） |
| **FX** | `fx-*` / `atmo-*` フル名 | `white`/`pink` + 短 ADSR + hpf |
| **Drums** | 短い part + `.bank("…")` | `bd` `sd` `hh` `oh` のみ |

ドラムは **1 本の `s(...)` に統合**（スペース=順、カンマ=同時）。詳細は strudel-composition。

duck 付きキックだけは別トラックにしてよい（hat に duckorbit を付けない）:

```
$: s("bd*4").gain(0.9).duckorbit(2).duckattack(0.12).duckdepth(0.85)
$: s("hh*8, ~ sd ~ sd").gain(0.35)
```

---

## 使えるモジュレーション / 音色パラメータ

引数は **数値または `"a:b"` 形式**。パターン文字列は不可。

### ビブラート

```
.vib(4)              // Hz
.vib("4:12")         // Hz : 深度(半音)
.vibmod(12)           // 深度。第2値があれば Hz
```

### FM（1オペ・簡易）

```
.fm(3)               // 変調指数
.fmh(1.5)            // キャリアに対するモジュレータ比
.fmattack(0.01)
.fmdecay(0.1)
.fmsustain(0.3)
```

```
$: note("c2 c2 eb2 g2").s("sine").fm(3).fmh(1.5).lpf(500).gain(0.55)
```

非対応: `fmenv` / 多オペ / `fmh2` 系の番号付き個別オペ。

### ピッチ / フィルタ・エンベロープ（簡易）

```
.penv(12)            // ピッチ env 深度
.pattack(0.01)
.pdecay(0.1)
.lpenv(0.5)          // LPF env 量
.lpattack(0.01)
.lpdecay(0.2)
.lpsustain(0.3)
.lprelease(0.1)
```

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
- **`.scale` だけ** `<…>` 進行が使える（`.lpf("<…>")` 等は不可）
- 詳細は strudel-composition / 明暗は strudel-live-edit

```
$: note("0 2 0 3 0 <2 4>").scale("C2:minor").s("sawtooth").lpf(500).gain(0.5)
$: note("0 2 4 0").scale("<A2:minor D:dorian G:mixolydian C:major>").s("sawtooth").lpf(600).gain(0.5)
```

### add / sub（移調・スカラー）

```
.add(2)              // scale あり次数 → +2 度; 音名 → +2 半音
.sub(1)              // .add(-1) と同義。累積可
```

```
$: note("0 2 4").scale("C2:minor").add(2).s("sawtooth").lpf(500).gain(0.5)
$: note("c4 e4").add(12).s("sine").gain(0.3)
```

パターン引数（`.add("<1 2>")`）は不可。

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
.cut(1)              // 同一 cut グループで steal
```

---

## 時間スケール（パターン）

```
.fast(2)
.slow(2)
```

（メソッドチェーン上の speed 係数。サンプルの `.speed` と混同しない。）

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
$: s("bd*4").gain(0.9).duckorbit(2).duckattack(0.12).duckdepth(0.85)
// パッドは orbit 2
$: note("c3 e3 g3 c4").s("sawtooth").orbit(2).gain(0.35).lpf(900)
```

- `duckorbit` / `duck`: 対象 orbit id（コロンで最大4）
- `duckattack` / `duckdepth`: 秒 / 0..1

### compressor（マスター、パターンから last-write）

```
.compressor("-18:3:6:.003:.12")
```

（threshold 等のコロン列。詳細は `CompressorParams::parse`。）

---

## 使えるメソッド一覧（クイック）

**音源・音色:** `s` `sound` `note` `n` `scale` `add` `sub` `gain` `velocity`/`vel` `noise` `vib`/`vibrato`/`v` `vibmod` `fm` `fmh` `fmattack` `fmdecay` `fmsustain` `penv` `pattack` `pdecay` `lpenv` `lpattack` `lpdecay` `lpsustain` `lprelease` `attack` `decay` `sustain` `release` `adsr`

**フィルタ:** `lpf` `lpq` `hpf` `hpq` `bpf` `bpq`（および表の別名）

**サンプル:** `begin` `end` `speed` `bank` `clip` `legato` `cut`

**時間:** `fast` `slow` `ply`

**バス / FX:** `orbit`/`o` `duckorbit`/`duck` `duckattack` `duckdepth` `delay` `delaytime` `delayfeedback` `room` `roomsize`/`size` `compressor`

これ以外のメソッド名は **unsupported method**。

---

## 明示的に使わない（本家にあって strudel-rs に無い）

| カテゴリ | 例 |
| --- | --- |
| ZZFX | `z_sawtooth` `zmod` `zcrush` … |
| 加算合成 | `partials` `phases` `sound("user")` |
| 可視化 | `_scope` `_spectrum` `_punchcard`（曲内関数） |
| フィルタ拡張 | `vowel` `ftype` `fanchor` |
| 歪み・特殊 | `distort` `crush` `coarse` `shape` |
| モジュレーション | `tremolo` `phaser` `pan` `detune`（メソッド） |
| 外部サンプル DSL | `samples('github:...')` `loopBegin`/`loopEnd` |
| 引数のミニ記法 | `.lpf("<200 800>")` `.vib("<1 4>")` |
| その他 | `beat` `seg` `supersaw` `crackle` `density` `stretch` |

---

## 信号のイメージ（strudel-rs）

1. 音源（波形 / ノイズ / wt / サンプル）
2. パーボイス: FM・vib・noise mix・ADSR・biquad lpf/hpf/bpf・penv/lpenv
3. Deck 内 orbit 合算 → orbit delay / room（wet）
4. Mixer: チャンネル EQ・フェーダー・マスター LPF/HPF・compressor

---

## 実例（短いライブループ・そのまま save 可能）

```
// @title sound-demo
setcpm(126/4)
// duck 付きキックだけ分離
$: s("bd*4").gain(0.9).duckorbit(2).duckattack(0.12).duckdepth(0.85)
$: s("[~ hh]*4, [~ sd]*2").gain(0.35)
// FM ベース（次数）
$: note("0 0 2 4").scale("C2:minor").s("sine").fm(3).fmh(1.5).lpf(500).gain(0.55)
  .attack(0.005).decay(0.1).sustain(0.3).release(0.08)
// duck されるパッド
$: note("0 2 4 7").scale("C3:minor").s("sawtooth").lpf(900).orbit(2).gain(0.35)
  .attack(0.05).decay(0.2).sustain(0.6).release(0.2)
```

ライブ差分例: パッドの `.lpf(900)` → `600`、または bass `.fm(3)` → `5` だけ変えて同名 save。

---

## Common Pitfalls

1. **本家コピペの動的引数**（`"<...>"` をメソッドに渡す）→ 数値パース失敗。固定値にする。
2. **未対応メソッド**（`phaser` `vowel` `distort` `partials` `lfo` …）→ そのパターン行がエラー。
3. **同じ orbit で delay/room を複数トラックから書く** → last-write で上書き。役割ごとに orbit を分ける。
4. **`c3'maj` で和音が鳴ると思わない** → root のみ。和音は `note("0 2 4")` / `[6,8]` 等で書く。
5. **ZZFX / supersaw / 外部 wt** は使えない。波形・wt_sine/bright/organ・サンプルに寄せる（ユーザー `lead-supersaw_*` WAV があればフル名で可）。
6. 音色のために 16 小節 `cat` を書かない → 短いループのままスカラーを触る。
7. **`bd:00` / `kit:bd`** → mini に `:` 不可。`s("bd")` / `.n(0)` / `.bank("kit")`。
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
