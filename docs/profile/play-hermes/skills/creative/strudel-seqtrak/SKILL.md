---
name: strudel-seqtrak
description: >-
  Use when writing dj-hermes for Yamaha SEQTRAK over MIDI (play --midi /
  --midi-only): channel map, // @midi bank/pc, and 18.3 CC knobs.
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [dj-hermes, music, midi, seqtrak, exhibit]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-live-edit
---

# dj-hermes → SEQTRAK（MIDI）

## Overview

`dj-hermes play` がヒットを **MIDI** に出す。本体の音色は SEQTRAK 側。ソフトシンセのレシピ（`.bank`、PCM `bd:hf`、wavetable）は **選ばれない**。

| CLI | ソフトシンセ | MIDI |
| --- | --- | --- |
| （フラグなし） | 鳴る | 出さない |
| `--midi` | 鳴る | 出す |
| `--midi-only` | 出さない | 出す（ポート必須。失敗したら終了） |

`dj` と `play --repl` は MIDI 対象外（デッキは A だけ）。SysEx / MIDI クロックは送らない。

```bash
dj-hermes play --midi-list
dj-hermes play songs/house/01.strudel --midi-port SEQTRAK
dj-hermes play songs/house/01.strudel --midi-only --midi-port SEQTRAK
```

USB は class-compliant。Linux の BLE MIDI は、OS が ALSA シーケンサに出していれば `--midi-list` に並ぶ（アプリは GATT を話さない）。Windows の BLE MIDI は対象外。BLE のペアリングは `bluetoothctl` など OS 側。

## When to Use

- 来場者・オペレータが SEQTRAK / MIDI 本体 / `--midi-only` の話をしている
- チャンネルや SOUND SELECT（Bank + Program Change）を曲に書く
- `.lpf` / `.gain` などを本体のツマミ（CC）に載せたい

Don't use for: `dj-hermes dj` の A/B ミックス、PCM キットの `.bank`、SysEx での音色内部。

## チャンネル（1 始まり）

注釈が無いヒットは sound 名で決める。`// @midi ch=N`（N=1..16）があればそれを使う。

| ch | SEQTRAK | このエンジン |
| --- | --- | --- |
| 1 | KICK | `bd` / `kick`（`bd:hf` もキック） |
| 2 | SNARE | `sd` / `snare` |
| 3 | CLAP | `cp` / `clap` |
| 4 | HAT 1 | `hh` / `hat` |
| 5 | HAT 2 | `oh` |
| 6 | PERC 1 | 短いドラム（`lt` `perc` `rim` など） |
| 7 | PERC 2 | シンバル（`rd` `crash`） |
| 8 | SYNTH 1 | 最初のシンセ `$:`、および 3 本目以降 |
| 9 | SYNTH 2 | 2 本目のシンセ `$:` |
| 10 | DX | `.fm(...)` があるトラック |
| 11 | SAMPLER | ハイフン名 / 長い名前 / `part:slug` |

ドラムのノート番号は **60**（C4）。GM キットのキック番号ではない。ピッチ付き `note()` は周波数から一番近い MIDI ノート。

ノートだけなら `s("bd*4, [~ sd]*2, [~ hh]*4")` の 1 本でもチャンネルは sound ごとに分かれる。**Bank/PC をトラック単位で付けたいとき**は `$:` を分ける。

## `// @midi`（トラック直前）

```
// @midi ch=8 msb=63 lsb=0 pc=12 lead
$: note("0 2 4 0").scale("C2:minor").s("sawtooth").lpf(800).gain(0.5)

// @midi bank=12,3 pc=40
$: s("bd*4").gain(0.7)
```

| キー | 意味 |
| --- | --- |
| `ch=N` | MIDI チャンネル 1..16 |
| `msb=N` / `lsb=N` | Bank Select CC0 / CC32（0..127） |
| `pc=N` | Program Change（0..127） |
| `bank=M,L` | `msb=M lsb=L` の短縮。`bank=M` は LSB=0 |

曲が載ったバーで **CC0 → CC32 → Program Change**（欠けた値は 0）。注釈が無いトラックには Bank/PC を出さない。番号は SEQTRAK Data List（Preset はだいたい MSB 63）。**知らない PC を推測して書かない。** オペレータが書いた値をそのまま使う。

## メソッド → CC（Note On の直前、同じ値は再送しない）

| メソッド | CC | マップ |
| --- | --- | --- |
| `.gain` | 7 Volume | 0..1 → 0..127 |
| `.pan` | 10 Pan | 0..1 → 1..127（中央 0.5 → 64） |
| `.lpf` | 74 Cutoff | 20..8000 Hz を対数で 0..127。LFO は Note On 時点の値 |
| `.lpq` | 71 Resonance | Q 0..16 → 0..127 |
| `.attack` | 73 Attack | 0..1 s → 0..127 |
| `.decay` / `.release` | 75 Decay/Release | 大きい方 |
| `.room` | 91 Reverb send | 0..1 → 0..127 |
| `.delay` | 94 Delay send | 0..1 → 0..127 |
| `.fm` | 117 FM amount | **ch10 のみ**。0..32 → 0..127 |

送らない: Mute/Solo（CC23/24）、Assigned FX（102–115）、EQ Gain（20/21）、Drum Pitch（25）、Arp、SysEx。

## 正本テンプレ（そのまま content に）

```
// @title seqtrak-live
setcpm(120/4)

// @midi ch=1
$: s("bd*4").gain(0.7)

// @midi ch=2
$: s("[~ sd]*2").gain(0.55)

// @midi ch=4
$: s("[~ hh]*4").gain(0.4)

// @midi ch=8 msb=63 lsb=0 pc=0
$: note("0 2 0 3 0 <2 4>").scale("C2:minor")
  .s("sawtooth").lpf(600).gain(0.5)
  .attack(0.001).decay(0.08).sustain(0.2).release(0.05)
```

DX にするなら 2 本目以降のシンセに `.fm(3)` を付ける（ch10）。ライブ差分は **strudel-live-edit**（`get_song` → `edit_method` / `patch_track`）。`@midi` 行を変えるときはその `$:` を `patch_track` する。

## 禁止

- `s("bd:hf")` や `.bank("tr808")` で SEQTRAK の音色が変わると想定する（チャンネルはキックのまま、PCM は本体に届かない）
- 知らない `pc=` をカタログから捏造する
- SysEx、`.lfo(...)` メソッド、`stack()`、`.cpm()`
- デッキ B / mix / xfade（play セッションではない）

## Pitfalls

1. 3 本目のシンセは ch8 に戻る（ch は 8 と 9 の 2 つだけ）
2. `--midi` のままだと PC スピーカと本体が同時に鳴る → 展示は `--midi-only`
3. `--midi-only` でポートが無いと起動しない（`--midi` は警告してソフトシンセ継続）
4. TouchOSC で見るときは note 60 と ch1 付近。GM キック番号を探さない

## Checklist

- [ ] `play --midi` または `--midi-only`（ポートは `--midi-list`）
- [ ] ドラムは短い `bd`/`sd`/`hh`（Bank/PC が要るなら `$:` を分ける）
- [ ] ピッチは次数 + `.scale`。`.fm` は DX 用
- [ ] `@midi` の msb/lsb/pc はオペレータ既知の値だけ
- [ ] 鳴らすのは `dj_hermes_apply_song`。save は残す指示のときだけ
