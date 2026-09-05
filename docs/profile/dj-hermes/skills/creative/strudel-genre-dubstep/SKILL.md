---
name: strudel-genre-dubstep
description: >-
  Use when writing Dubstep for strudel-rs: 140 BPM half-time drums,
  wobble bass via .lpf(sine.rangex(...)). No second sub under the
  wobble. Not four-on-the-floor and not .lfo().
version: 5.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, dubstep]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-future-bass
---

# strudel-rs × ダブステップ

## Overview

ハーフタイムのドラムと、カットオフが動く低域ベース。この Skill のテンポは **140 BPM**（ハーフで体感 70）。wobble 自体が低域なので、その下に `bs:su` は置かない。

## When

- 依頼が **ダブステップ**（140、ハーフタイム、wobble ベース）のとき
- 4 つ打ちハウス / テクノではなく、スネアを後ろめに置くとき
- `.lfo(...)` メソッドではなく `.lpf(sine.rangex(...))` で wobble するとき
- 同じ 140 の 2 ステップに J-pop 王道／小室＋kawaii を載せるのは [strudel-genre-future-bass](../strudel-genre-future-bass/SKILL.md)（wobble を主役にしない）

## Pattern

```
// @title visitor-dubstep
// @genre dubstep
setcpm(140/4)
// drums
$: s("bd ~ ~ ~ bd ~ sd ~, hh*8, <~ ~ ~ [bd sd bd sd]>").gain(0.72)
// bass
$: note("0 0 3 <0 0 3 -1>").scale("<C2:minor C2:minor G2:phrygian C2:minor>")
  .s("sawtooth").lpf(sine.rangex(80, 600)).lpq(6).gain(0.5)
// lead
$: note("~ 7 ~ <10 7 3 7>").scale("<C3:minor C3:minor G3:phrygian C3:minor>")
  .s("sawtooth").fm(4).fmh(1).fmdec(0.25).fmsus(0.2)
  .lpf(800).gain(0.16)
// hook
$: note("~ 4 ~ <7 4 4 7>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("plk:s5").gain(0.18).cut(1)
// arp
$: note("~ ~ 12 ~").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("square").lpf(2400).gain(0.1)
// chords
$: note("[0,4] ~ ~ [0,4]").scale("<C3:minor C3:minor G3:phrygian C3:minor>")
  .s("triangle").lpf(900).gain(0.16)
// pad
$: note("[0,4]").scale("<C3:minor C3:minor G3:phrygian C3:minor>")
  .s("wt_organ").lpf(700).gain(0.12).room(0.25).orbit(2)
  .attack(0.08).release(0.3)
```

同梱 `songs/dubstep/01.strudel` はまだ 3 本前後の薄いデモ。新規の apply はこのフェンスを正本にする。

## Why

キックは 1 拍目と 3 拍目、スネアは 4 拍目付近。ハットは 16 分のまま。これがハーフタイムのグリッド。

## レシピ

1. スネアを後ろめに置いてハーフ感を出す。`bd*4` にしない
2. ベースは C2 の saw + `.lpf(sine.rangex(80, 600))` + `.lpq(6)`。`.lfo(...)` は未実装
3. wobble の下に `bs:su` や 2 本目ベースを足さない（bass-mid も置かない）
4. キックとサブの同時打は gain でキックを前に（drums 0.72 / bass 0.5）
5. ピッチトラックは 4 小節 `.scale("<C:minor C:minor G:phrygian C:minor>")`

鳴らすのは `strudel_apply_song(content, deck)`（次小節、無書き込み）。`strudel_save_song` は残す指示のときだけ（演奏は変えない）。

## Pitfalls

1. `stack()` / `.cpm()` / `.lfo()` → apply / save とも 400。wobble は `.lpf(sine.rangex(…))`。`.vib("<…>")` は不可
2. ドラムを kick / snare / hat の 3 `$:` に分ける
3. `in_bank=no` の `ld:ac` / `pf:al` / `dr:*` / `ps:*` → 無音
4. wobble の下に `bs:su` を重ねる（低域が二重になる）
5. ハイハットだらけで低域が埋もれる。`bd*4` にして 4 つ打ち化する

## Checklist

- [ ] 7 本（// drums // bass // lead // hook // arp // chords // pad）。bass-mid は置かない
- [ ] 4 小節フレーズ（`.scale("<…>")` が 4 個）
- [ ] ドラムは 1 本。ハーフタイム（`bd*4` ではない）
- [ ] wobble は `.lpf(sine.rangex(...))`。サブは 1 本
- [ ] `strudel_apply_song(content, deck)`（save は残す指示のときだけ）
