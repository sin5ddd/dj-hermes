---
name: strudel-genre-progressive-house
description: >-
  Use when writing Progressive House for strudel-rs: 128 BPM, clap on 2
  and 4 ([~ cp]*2), long pad and arp, C-minor / F-dorian 4-bar phrase.
  Not kick-front techno.
version: 5.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, progressive-house]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × プログレッシブハウス

## Overview

4 つ打ちにクラップ、長いパッドとアルペジオ。この Skill のテンポは **128 BPM**（目安 124–130）。キック先行のテクノグリッドではない。

## When

- 依頼が **プログレッシブハウス**（4 つ打ち、2/4 クラップ、長い上物）のとき
- ハウスのクラップ `[~ cp]*2` を使い、スネアを重ねないとき
- キック先行テクノ（`bd*4` + オフビートハットのみ）ではないとき

## Pattern

```
// @title visitor-prog-house
// @genre progressive-house
setcpm(128/4)
// drums
$: s("bd*4, [~ cp]*2, hh*8, <~ ~ ~ oh>").gain(0.58)
// bass
$: note("0 0 4 <0 2 4 0>").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("bs:hf").gain(0.42)
// lead
$: note("4@2 7 9@3 ~").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("ld:ss").gain(0.16).cut(1)
// hook
$: note("~ 7 4 <9 7 4 2>").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("plk:hb").gain(0.18).cut(1)
// arp
$: note("0 2 4 7  4 2 0 ~").scale("<C5:minor C5:minor F5:dorian C5:minor>")
  .s("triangle").lpf(2800).gain(0.12)
// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("<C3:minor C3:minor F3:dorian C3:minor>")
  .s("sawtooth").lpf(1200).gain(0.24)
  .attack(0.04).release(0.25)
// pad
$: note("[0,4]").scale("<C3:minor C3:minor F3:dorian C3:minor>")
  .s("sawtooth").lpf(900).attack(0.1).release(0.4).gain(0.18).room(0.4).orbit(2)
```

同梱 `songs/progressive-house-01.strudel` はまだ 3 本前後の薄いデモ。新規の apply はこのフェンスを正本にする。

## Why

`[~ cp]*2` は 2 拍目と 4 拍目のクラップ（ハウスのバックビート）。パッドと arp が 4 小節の長さを担う。

## レシピ

1. キックは固定。クラップは `cp`（`sd` と重ねない）
2. ベースは PCM `bs:hf`。スケールは **C4**（ネイティブ C2 を C4 で書く）
3. pad は attack を少し長く、`.room` + 別 `orbit`
4. 進行は 4 小節 C minor → C minor → F dorian → C minor
5. 上物を増やしすぎない（リード / フック / arp は掛け合い）

鳴らすのは `strudel_apply_song(content, deck)`（次小節、無書き込み）。`strudel_save_song` は残す指示のときだけ（演奏は変えない）。

## Pitfalls

1. `stack()` / `.cpm()` / `.lfo()` → apply / save とも 400
2. ドラムを kick / clap / hat の 3 `$:` に分ける
3. `in_bank=no` の `ld:ac` / `pf:al` / `dr:*` / `ps:*` → 無音。長いリードは `ld:ss`
4. `bs:hf` を `C2:` で書く（PCM フロアは `C4:`）
5. `note("c3'maj")` は root 単音。和音は `[0,2,4]`
6. テンポを DnB 域にする。`sd` を 2/4 のクラップに代用する

## Checklist

- [ ] 7 本（// drums // bass // lead // hook // arp // chords // pad）
- [ ] 4 小節フレーズ（`.scale("<…>")` が 4 個）
- [ ] ドラムは 1 本。`[~ cp]*2` がある（`sd` と重ねていない）
- [ ] 4 つ打ち + パッド。PCM ベースは `C4:`
- [ ] `strudel_apply_song(content, deck)`（save は残す指示のときだけ）
