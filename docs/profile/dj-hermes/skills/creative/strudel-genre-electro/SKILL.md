---
name: strudel-genre-electro
description: >-
  Use when writing Electro for strudel-rs: 126 BPM, mechanical
  four-on-the-floor, short square bass, zap hook. Not house clap-front
  and not sparse minimal-techno.
version: 5.1.0
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
  .s("ld:ss").gain(0.14).cut(1)
// hook
$: note("12 ~ 7 <12 15 12 7>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("square").penv(12).pattack(0.001).pdecay(0.08).lpf(2400).gain(0.16)
// arp
$: note("~ 0 3 7  3 0 ~ 5").scale("<C5:minor C5:minor G5:phrygian C5:minor>")
  .s("plk:cv").gain(0.14).cut(1)
// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("plk:sp").gain(0.18)
// pad
$: note("0").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("pf:ff").gain(0.14).room(0.25).orbit(2)
```

同梱 `songs/electro/01.strudel` はこのフェンスと同じ（グリッドと次数）。新規 apply の `.s()` は下のパレットから選ぶ。**ベースとフックの両方を `square` にしない。**

## 音色パレット（新規 apply はここから選ぶ）

Pattern はグリッド・次数・スロットの見本。新規曲は下表からスロットごとに 1 つ選び、このフェンスの `.s()` を毎回コピーしない。同一曲の pitched 2 本に同じ `.s()` を使わない。slug の意味は strudel-pcm-catalog の INDEX。長尺（`ld:` / `dr:` / `pf:` / `ps:`、`plk:fp` / `plk:sp`）は `.cut(1)` か `s("<x ~ ~ ~>")`。

短い square ベースは **1 役だけ**。フックの zap は `ld:zp` 側へ。`square`+`penv` はベースが square でないときだけ。

| スロット | 芯 | 代替 | 禁止 |
| --- | --- | --- | --- |
| drums | 機械的 4 つ打ち + 2/4 `sd` | `bd:ez` / `bd:9p`、`sd:rm`、`hh:ch` | `[~ cp]*2`、`bd:gb` |
| bass | 短い `square`+`lpf(500)` at `C2:` | `bs:dq` at `C4:`（square と同時に使わない） | `bs:su` 重ね、長い pad をベースに |
| lead | | `ld:pu`、`ld:ch`、`ld:dp`、`ld:lz` | `ld:ss` 固定、`ep:rs` |
| hook | zap 1 役 | `ld:zp`、`ld:lz`、`perc:zp`。`square`+`penv` は bass が square でないとき | bass と同じ `square`、`plk:mx` |
| arp | | `plk:cv`、`perc:zp` | ナイロン `plk:ny` |
| chords | | `plk:sf`、`plk:s5`、`plk:sp` を `<>` | `ep:rs`、`triangle` |
| pad | | `pf:pu`、`ld:hf` を `<>`、`pf:ff`+`note("0")` | `pf:al`、オルゴール、Rhodes |

## Why

グリッドは `bd*4` に 2/4 の `sd` と 16 分 `hh`。4 小節目だけ `[bd sd bd sd]` のフィルで反復を崩す。

## レシピ

1. キック / スネア / ハットはカンマで 1 本
2. ベースは短い square（シンセサブ）**または**パレットの別キー 1 つ。ADSR を短くする
3. フックは zap（`ld:zp` 等）。`square`+`penv` はベースが square でないときだけ。長い release は使わない
4. ピッチトラックは 4 小節 `.scale("<C:minor C:minor G:phrygian C:minor>")`（オクターブは帯域に合わせる）
5. ハットは乾いたまま（長い room をドラムに載せない）

鳴らすのは `strudel_apply_song(content, deck)`（次小節、無書き込み）。`strudel_save_song` は残す指示のときだけ（演奏は変えない）。

## Pitfalls

1. `stack()` / `.cpm()` / `.lfo()` → apply / save とも 400
2. ドラムを kick / snare / hat の 3 `$:` に分ける
3. 長い PCM（`ld:` / `dr:` / `pf:` / `ps:`）を毎小節撃たない
4. アンビエント寄りの長い release
5. square サブの上に `bs:su` を重ねる
6. ベースとフックの両方を `square` にする。新規 apply でフェンスの `.s()` を全コピーする

## Checklist

- [ ] 7 本（// drums // bass // lead // hook // arp // chords // pad）
- [ ] 4 小節フレーズ（`.scale("<…>")` が 4 個）
- [ ] ドラムは 1 本のカンマ層
- [ ] 機械的な 4 つ打ち + 2/4 スネア（クラップ先行にしない）
- [ ] `.s()` は音色パレット。bass と hook が両方 `square` ではない
- [ ] `strudel_apply_song(content, deck)`（save は残す指示のときだけ）
