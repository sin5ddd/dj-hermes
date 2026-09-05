---
name: strudel-genre-electro
description: >-
  Use when writing Electro for strudel-rs: 126 BPM, mechanical
  four-on-the-floor, short square bass, zap hook. Not house clap-front
  and not sparse minimal-techno.
version: 5.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, electro]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × エレクトロ

## Overview

機械的なキックとスネア、短い square ベース。この Skill のテンポは **126 BPM**（目安 120–130）。ハウスのクラップ先行でも、休符だらけのミニマルでもない。

## When

- 依頼が **エレクトロ**（機械的な 4 つ打ち、短いシンセ、zap 風フック）のとき
- キック / スネア / ハットを **1 本のドラム** にまとめるとき
- ハウスの `[~ cp]*2` や、隙間を主にしたミニマルテクノではないとき

## Pattern

```
// @title visitor-electro
// @genre electro
setcpm(126/4)
// drums
$: s("bd*4, ~ sd ~ sd, hh*8, <~ ~ ~ [bd sd bd sd]>").gain(0.62)
// bass
$: note("0 ~ 0 <3 0 0 5>").scale("<C2:minor C2:minor G2:phrygian C2:minor>")
  .s("square").lpf(500).gain(0.46)
  .attack(0.001).decay(0.08).sustain(0.15).release(0.04)
// lead
$: note("~ 7 4 <9 7 12 7>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("wt_bright").lpf(3200).gain(0.14)
// hook
$: note("12 ~ 7 <12 15 12 7>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("square").penv(12).pattack(0.001).pdecay(0.08).lpf(2400).gain(0.16)
// arp
$: note("~ 0 3 7  3 0 ~ 5").scale("<C5:minor C5:minor G5:phrygian C5:minor>")
  .s("plk:cv").gain(0.14).cut(1)
// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("<C3:minor C3:minor G3:phrygian C3:minor>")
  .s("triangle").lpf(1600).gain(0.2)
  .attack(0.001).decay(0.12).sustain(0.1).release(0.06)
// pad
$: note("[0,4]").scale("<C3:minor C3:minor G3:phrygian C3:minor>")
  .s("sawtooth").lpf(900).gain(0.14)
  .attack(0.04).decay(0.15).sustain(0.5).release(0.2)
```

同梱 `songs/electro/01.strudel` はまだ 3 本前後の薄いデモ。新規の apply はこのフェンスを正本にする。

## Why

グリッドは `bd*4` に 2/4 の `sd` と 16 分 `hh`。4 小節目だけ `[bd sd bd sd]` のフィルで反復を崩す。

## レシピ

1. キック / スネア / ハットはカンマで 1 本
2. ベースは C2 の square（シンセサブ）。ADSR を短くする
3. フックは square + `penv` の短い zap。長い release は使わない
4. ピッチトラックは 4 小節 `.scale("<C:minor C:minor G:phrygian C:minor>")`（オクターブは帯域に合わせる）
5. ハットは乾いたまま（長い room をドラムに載せない）

鳴らすのは `strudel_apply_song(content, deck)`（次小節、無書き込み）。`strudel_save_song` は残す指示のときだけ（演奏は変えない）。

## Pitfalls

1. `stack()` / `.cpm()` / `.lfo()` → apply / save とも 400
2. ドラムを kick / snare / hat の 3 `$:` に分ける
3. `in_bank=no` の `ld:ac` / `pf:al` / `dr:*` / `ps:*` → 無音。`ld:ss` / `plk:*` / 波形 / `wt_*` を使う
4. アンビエント寄りの長い release
5. square サブの上に `bs:su` を重ねる

## Checklist

- [ ] 7 本（// drums // bass // lead // hook // arp // chords // pad）
- [ ] 4 小節フレーズ（`.scale("<…>")` が 4 個）
- [ ] ドラムは 1 本のカンマ層
- [ ] 機械的な 4 つ打ち + 2/4 スネア（クラップ先行にしない）
- [ ] `strudel_apply_song(content, deck)`（save は残す指示のときだけ）
