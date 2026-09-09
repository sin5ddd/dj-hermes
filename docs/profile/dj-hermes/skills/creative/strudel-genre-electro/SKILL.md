---
name: strudel-genre-electro
description: >-
  Use when writing Electro for dj-hermes: 126 BPM, mechanical
  four-on-the-floor, short square bass, supersaw hook, pitched parts
  two octaves below typical C4 PCM. Not house clap-front, not a thin
  zap hook, and not sparse minimal.
version: 6.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [dj-hermes, music, genre, electro]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# dj-hermes × エレクトロ

## Overview

機械的なキックとスネア、短い square ベース、太いスーパーソーのフック。この Skill のテンポは **126 BPM**（目安 120–130）。ハウスのクラップ先行でも、休符だらけのミニマルでもない。

帯域は他ジャンルの PCM `C4:` より **だいたい 2 オクターブ下**。ベースは `C2:` が床。lead / hook / chords / pad も `C2:`。arp は `C3:`。PCM も `C4:` に戻さない（native C4 を 2 オクターブ下げて鳴らす）。**例外: ボーカルチョップ `vc:` だけは `C4:`**（録音が C4。C2 に落とすと声が床になる）。

## When

- 依頼が **エレクトロ**（機械的な 4 つ打ち、短いシンセ、スーパーソーのフック）のとき
- キック / スネア / ハットを **1 本のドラム** にまとめるとき
- フックを細い zap や `square`+`penv` のヒョロヒョロにしないとき
- ハウスの `[~ cp]*2` や、隙間を主にしたミニマルではないとき

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
$: note("~ 7 4 <9 7 12 7>").scale("<C2:minor C2:minor G2:phrygian C2:minor>")
  .s("ld:pu").gain(0.14).cut(1)
// hook
$: note("12 ~ 7 <12 15 12 7>").scale("<C2:minor C2:minor G2:phrygian C2:minor>")
  .s("ld:ss").gain(0.16).cut(1)
// arp
$: note("~ 0 3 7  3 0 ~ 5").scale("<C3:minor C3:minor G3:phrygian C3:minor>")
  .s("plk:cv").gain(0.14).cut(1)
// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("<C2:minor C2:minor G2:phrygian C2:minor>")
  .s("plk:sp").gain(0.18)
// pad
$: note("0").scale("<C2:minor C2:minor G2:phrygian C2:minor>")
  .s("pf:ff").gain(0.14).room(0.25).orbit(2)
```

同梱 `songs/electro/01.strudel` はこのフェンスと同じ（グリッド・次数・帯域）。新規 apply の `.s()` は下のパレットから選ぶ。**フックはスーパーソー。リードに `ld:ss` を重ねない。**

フック次数 12 は `C2:` 上で C3。C5 の zap にはしない。

## 音色パレット（新規 apply はここから選ぶ）

Pattern はグリッド・次数・スロットの見本。新規曲は下表からスロットごとに 1 つ選び、このフェンスの `.s()` を毎回コピーしない。同一曲の pitched 2 本に同じ `.s()` を使わない。slug の意味は strudel-pcm-catalog の INDEX。長尺（`ld:` / `dr:` / `pf:` / `ps:`、`plk:fp` / `plk:sp`）は `.cut(1)` か `s("<x ~ ~ ~>")`。

短い square ベースは **1 役だけ**。フックの芯は `ld:ss`。zap（`ld:zp`）は下の Variations だけ。

| スロット | 芯 | 代替 | 禁止 |
| --- | --- | --- | --- |
| drums | 機械的 4 つ打ち + 2/4 `sd` | `bd:ez` / `bd:9p`、`sd:rm`、`hh:ch` | `[~ cp]*2`、`bd:gb` |
| bass | 短い `square`+`lpf(500)` at `C2:` | `bs:dq` at `C2:`（square と同時に使わない。`C4:` に上げない） | `bs:su` 重ね、長い pad をベースに、`C0:` |
| lead | `C2:` | `ld:pu`、`ld:ch`、`ld:dp` | `ld:ss`（フックの役）、`ep:rs`、`C4:` |
| hook | `ld:ss` at `C2:` | `ld:st`、`ld:us` | `ld:zp` を既定、bass と同じ `square`、`plk:mx`、`C4:` |
| arp | `C3:` | `plk:cv`、`perc:zp` | ナイロン `plk:ny`、`C5:` |
| chords | `C2:` | `plk:sf`、`plk:s5`、`plk:sp` を `<>` | `ep:rs`、`triangle`、`C4:` |
| pad | `C2:` | `pf:pu`、`ld:hf` を `<>`、`pf:ff`+`note("0")` | `pf:al`、オルゴール、Rhodes、`C4:` |
| vox | **`C4:` のみ**（他 pitched の C2 ルール対象外）。任意 8 本目。`.cut(1)` | `vc:tu`、`vc:pa` | `C2:` / `C3:` に落とす、`C5:`、16 分埋め、自前 WAV を invent |

## Why

グリッドは `bd*4` に 2/4 の `sd` と 16 分 `hh`。4 小節目だけ `[bd sd bd sd]` のフィルで反復を崩す。

フックを `ld:zp` や高い `square`+`penv` にすると、倍音が細く C4〜C5 に寄る。`ld:ss` を `C2:` に置くとミッドに厚みが出る。PCM を他ジャンルどおり `C4:` にすると、`bs:dq` も含めて低音が空く。

## レシピ

1. キック / スネア / ハットはカンマで 1 本
2. ベースは短い square（シンセサブ）**または** `bs:dq` 1 つ。どちらも **`C2:`**。ADSR を短くする
3. フックはスーパーソー（`ld:ss` / `ld:st` / `ld:us`）。長い `ld:` は `.cut(1)`
4. ピッチトラックは 4 小節 `.scale`。オクターブは **bass/lead/hook/chords/pad = C2、arp = C3**（他ジャンルの C4/C5 から 2 オクターブ下）。**`vc:` だけ `C4:`**
5. ハットは乾いたまま（長い room をドラムに載せない）

鳴らすのは `dj_hermes_apply_song(content, deck)`（次小節、無書き込み）。`dj_hermes_save_song` は残す指示のときだけ（演奏は変えない）。

## Variations（同じ文法）

| 目的 | 変更 |
| --- | --- |
| フックを zap に | `ld:ss` を `ld:zp`（既定にはしない。帯域は `C2:` のまま） |
| ハットを細かく | `hh*8` を `hh*16` |
| チョップ | 8 本目 `// vox` を **`C4:`**（他 pitched は C2/C3 のまま） |

## Pitfalls

1. `stack()` / `.cpm()` / `.lfo()` → apply / save とも 400
2. ドラムを kick / snare / hat の 3 `$:` に分ける
3. 長い PCM（`ld:` / `dr:` / `pf:` / `ps:`）を毎小節撃たない
4. アンビエント寄りの長い release
5. square サブの上に `bs:su` を重ねる
6. フックを `ld:zp` や `square`+`penv` の既定にする。lead と hook の両方を `ld:ss` にする
7. pitched を `C4:` / `C5:` に書く（他ジャンルの PCM native ルールをそのまま使う）。ベースを `C0:` にする。**`vc:` を `C2:` に落とす**

## Checklist

- [ ] 7 本（// drums // bass // lead // hook // arp // chords // pad）。任意 8 本目は `// vox`（`C4:`）
- [ ] 4 小節フレーズ（`.scale("<…>")` が 4 個）
- [ ] ドラムは 1 本のカンマ層
- [ ] 機械的な 4 つ打ち + 2/4 スネア（クラップ先行にしない）
- [ ] フックはスーパーソー（`ld:ss` / `ld:st` / `ld:us`）。lead と重ねない
- [ ] 帯域は bass/lead/hook/chords/pad `C2:`、arp `C3:`（PCM も `C4:` に上げない）。`vc:` だけ `C4:`
- [ ] `.s()` は音色パレット
- [ ] `dj_hermes_apply_song(content, deck)`（save は残す指示のときだけ）
