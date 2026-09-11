---
name: strudel-genre-progressive-house
description: >-
  Use when writing Progressive House for dj-hermes: 128 BPM, clap on 2
  and 4 ([~ cp]*2), long pad and arp, C-minor / F-dorian 4-bar phrase.
  Not kick-front techno.
version: 5.2.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [dj-hermes, music, genre, progressive-house]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# dj-hermes × プログレッシブハウス

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
  .s("ld:ss").adsr("0.01:0.3:0.7:0.2").cut(1).gain(0.16)
// hook
$: note("~ 7 4 <9 7 4 2>").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("plk:hb").gain(0.18).cut(1)
// arp
$: note("0 2 4 7  4 2 0 ~").scale("<C5:minor C5:minor F5:dorian C5:minor>")
  .s("plk:hd").gain(0.12).cut(1)
// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("ep:ky").gain(0.24)
// pad
$: note("0").scale("<C2:minor C2:minor F2:dorian C2:minor>")
  .s("pf:ff").adsr("0.2:0.4:0.5:0.4").cut(1).gain(0.18).room(0.4).orbit(2)
```

同梱 `songs/progressive-house/01.strudel` はこのフェンスと同じ（グリッドと進行）。新規 apply の `.s()` は下のパレットから選ぶ。

## 音色パレット（新規 apply はここから選ぶ）

Pattern はグリッド・次数・スロットの見本。新規曲は下表からスロットごとに 1 つ選び、このフェンスの `.s()` を毎回コピーしない。同一曲の pitched 2 本に同じ `.s()` を使わない。slug の意味は strudel-pcm-catalog の INDEX。lead / pad の PCM は `.s()` の直後に `.adsr`、そのあと `.cut(1)`。FX ライザーだけ `<>` 間引き。

| スロット | 芯 | 代替 | 禁止 |
| --- | --- | --- | --- |
| drums | `bd*4` + `[~ cp]*2` | `bd:hf`、`cp:rm`、`hh:hs` / `hh*8` | `[~ sd]*2`、キック先行のみ、`bd:gb` |
| bass | `bs:hf` at `C4:` | `bs:ht`、`bs:sw` | `bs:su` 重ね、`bs:wb` |
| lead | 長いノート | `ld:an`、`ld:tg`、`ld:us`、`ld:ss`+`.cut(1)` | `ld:gb` / `gr` / `wb` |
| hook | | `plk:hb`、`plk:tg`、`plk:ss` | `plk:s5`、303 `lpenv` |
| arp | メロディ楽器 | `plk:hd`、`plk:aj` | `perc:`、ADSR なしの 16s パッドを arp に置く |
| chords | `[0,2,4]` | `ep:ky`、`ep:wr`、`plk:sm` | `triangle` |
| pad | **C2** + `.adsr`。長い上物 | `pf:hz`、`pf:wm`、`pf:ju`、`ld:fp` を `<>`、`pf:ff`+`note("0")` | C4、ADSR なし、`dr:hr`、gabber、wobble |
| vox | 任意 8 本目 `// vox`。`.cut(1)` | `vc:pa`、`vc:ya` at `C4:` | 16 分埋め、毎小節 `vc:yeah`、自前 WAV を invent |

## Why

`[~ cp]*2` は 2 拍目と 4 拍目のクラップ（ハウスのバックビート）。パッドと arp が 4 小節の長さを担う。

## レシピ

1. キックは固定。クラップは `cp`（`sd` と重ねない）
2. ベースは PCM `bs:hf`。スケールは **C4**（ネイティブ C2 を C4 で書く）
3. pad は **C2** + `.adsr`、`.room` + 別 `orbit`。lead PCM は `.s(…).adsr(…)`
4. 進行は 4 小節 C minor → C minor → F dorian → C minor
5. 上物を増やしすぎない（リード / フック / arp は掛け合い。arp はメロディ楽器）

鳴らすのは `dj_hermes_apply_song(content, deck)`（次小節、無書き込み）。`dj_hermes_save_song` は残す指示のときだけ（演奏は変えない）。

## Pitfalls

1. `stack()` / `.cpm()` / `.lfo()` → apply / save とも 400
2. ドラムを kick / clap / hat の 3 `$:` に分ける
3. 長い PCM（`ld:` / `dr:` / `pf:` / `ps:`）を **ADSR なし**で毎小節撃つ。lead / pad は `.adsr` + `.cut(1)` ならグリッド可。pad を C4 に置かない。arp を `perc:` にしない
4. `bs:hf` を `C2:` で書く（PCM フロアは `C4:`）
5. `note("c3'maj")` は root 単音。和音は `[0,2,4]`
6. テンポを DnB 域にする。`sd` を 2/4 のクラップに代用する
7. 新規 apply でフェンスの `.s()` を全コピーする。同一曲で pitched の `.s()` を重複させる

## Checklist

- [ ] 7 本（// drums // bass // lead // hook // arp // chords // pad）。任意 8 本目は `// vox`
- [ ] 4 小節フレーズ（`.scale("<…>")` が 4 個）
- [ ] ドラムは 1 本。`[~ cp]*2` がある（`sd` と重ねていない）
- [ ] 4 つ打ち + パッド。PCM ベースは `C4:`。**pad は C2** + `.adsr`。lead PCM は `.adsr`。arp はメロディ楽器。`.s()` は音色パレット
- [ ] `dj_hermes_apply_song(content, deck)`（save は残す指示のときだけ）
