---
name: strudel-sound-design
description: "Use when designing synths or effects for strudel-rs (not full Strudel REPL)."
version: 2.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, sound-design, synthesis, effects]
    related_skills:
      - strudel-data-format
      - strudel-composition
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
音源・メソッドは `src/code.rs` / `src/sound.rs` / `src/synth.rs` に実装されているものだけを使う。

**本家にあって未実装の一覧:** リポジトリ `docs/strudel-gap-synths-fx.md`（シンセ / FX / サンプルギャップ）。

## When to Use

- ユーザーが **strudel-rs** で音色・FX を設計する時
- 波形 / ノイズ / 内蔵 wavetable / サンプル + 対応エフェクトを選ぶ時

Don't use for: パターン記法の詳細（→ strudel-composition）、曲ファイル形式（→ strudel-data-format）、**本家 Strudel 専用**のシンセ（ZZFX・partials・phaser 等）。

---

## 重要: 本家 Strudel との差

| 項目 | strudel-rs |
| --- | --- |
| メソッド引数 | **スカラー数値のみ**（`parse_num` / `a:b` コロン列）。`vib("<1 2 4>")` のような **ミニ記法の動的引数は不可** |
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

同梱は主に `bd` / `sd` / `hh` / `oh`（`samples/`）。フォルダ名 = sound 名。

```
$: s("bd*4").gain(0.9)
$: s("~ sd ~ sd").gain(0.7)
$: s("hh*8").gain(0.25)
$: s("bd").bank("rolandtr808")   // bank 接頭辞 → rolandtr808_bd（bank 内にあれば）
$: s("bd").n(1)                  // 同一 sound の n 番 WAV（あれば）
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
```

非対応: `ftype` / `fanchor` / `vowel` / ladder 切替。

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

**音源・音色:** `s` `sound` `note` `n` `gain` `velocity`/`vel` `noise` `vib`/`vibrato`/`v` `vibmod` `fm` `fmh` `fmattack` `fmdecay` `fmsustain` `penv` `pattack` `pdecay` `lpenv` `lpattack` `lpdecay` `lpsustain` `lprelease` `attack` `decay` `sustain` `release` `adsr`

**フィルタ:** `lpf` `lpq` `hpf` `hpq` `bpf` `bpq`（および表の別名）

**サンプル:** `begin` `end` `speed` `bank` `clip` `legato` `cut`

**時間:** `fast` `slow`

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
| その他 | `scale` `beat` `seg` `supersaw` `crackle` `density` `stretch` |

---

## 信号のイメージ（strudel-rs）

1. 音源（波形 / ノイズ / wt / サンプル）
2. パーボイス: FM・vib・noise mix・ADSR・biquad lpf/hpf/bpf・penv/lpenv
3. Deck 内 orbit 合算 → orbit delay / room（wet）
4. Mixer: チャンネル EQ・フェーダー・マスター LPF/HPF・compressor

---

## 実例（デモ曲と同系統）

```
// FM ベース
$: note("c2 c2 eb2 g2").s("sine").fm(3).fmh(1.5).lpf(500).gain(0.55)
  .attack(0.005).decay(0.1).sustain(0.3).release(0.08)

// duck されるパッド
$: note("c3 e3 g3 c4").s("sawtooth").lpf(900).orbit(2).gain(0.35)
  .attack(0.05).decay(0.2).sustain(0.6).release(0.2)

// キックが duck
$: s("bd*4").gain(0.9).duckorbit(2).duckattack(0.12).duckdepth(0.85)

// 空間
$: note("c4 e4 g4 b4").s("wt_bright").gain(0.18)
  .delay(0.25).delaytime(0.375).delayfeedback(0.45).orbit(2)
```

---

## Common Pitfalls

1. **本家コピペの動的引数**（`"<...>"` をメソッドに渡す）→ 数値パース失敗。固定値にする。
2. **未対応メソッド**（`phaser` `vowel` `distort` `partials` …）→ そのパターン行がエラー。
3. **同じ orbit で delay/room を複数トラックから書く** → last-write で上書き。役割ごとに orbit を分ける。
4. **`c3'maj` で和音が鳴ると思わない** → root のみ。和音は `note("c3 e3 g3")` 等で書く。
5. **ZZFX / supersaw / 外部 wt** は使えない。波形・wt_sine/bright/organ・サンプルに寄せる。

## Verification Checklist

- [ ] sound は波形 / white|pink|brown / wt_* / ローカル sample のいずれか
- [ ] メソッドは上記一覧のみ
- [ ] メソッド引数は数値または `a:b` のみ（ミニ記法パターンなし）
- [ ] delay/room を共有するトラックは orbit を意識している
- [ ] duck する側に `duckorbit`、される側に同じ `orbit`
