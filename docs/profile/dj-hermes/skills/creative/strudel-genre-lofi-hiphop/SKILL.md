---
name: strudel-genre-lofi-hiphop
description: >-
  Use when writing lo-fi hip hop for dj-hermes: 75–90 BPM, dusty
  bd:lf, keys, slow hats. Not house [~ cp]*2.
version: 5.2.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [dj-hermes, music, genre, lofi, hiphop]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# dj-hermes × ローファイ・ヒップホップ

## Overview

遅め BPM、乾いたドラム、柔らかいキー。BPM 目安 75–90。フェンスは **84**（`setcpm(84/4)`）。
キックは `bd:lf`。ハットは遅い。ハウスの clap グリッドは使わない。
`songs/lofi-hiphop/01.strudel` はこのフェンスと同じ（7 本）。

## When

- 来場者がローファイ、ヒップホップ、ダスト、EP キーを求めるとき
- ハウス `[~ cp]*2`、シティポップ下降のチルポップ、パッド主のアンビエントではないとき
- 新規 apply / プリセット（2–5 本では足りない）

## Pattern

```
// @title visitor-lofi
// @genre lofi-hiphop
setcpm(84/4)
// drums
$: s("bd:lf ~ ~ sd, [~ hh]*4, <~ ~ ~ [bd:lf sd bd:lf sd]>").gain(0.44)
// bass
$: note("0 ~ 2 <0 0 3 2>").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("bs:su").gain(0.38)
// lead
$: note("~ 4 7 <5 4 0 2>").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("plk:lf").adsr("0.01:0.3:0.7:0.2").cut(1).gain(0.18)
// hook
$: note("[0,2,4] ~ [0,3,5] ~").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("ep:rs").adsr("0.05:0.3:0.5:0.3").gain(0.22)
// arp
$: note("~ 0 4 7  ~ 4 0 2").scale("<C5:minor C5:minor F5:dorian C5:minor>")
  .s("plk:ny").gain(0.12).cut(1)
// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("ep:mt").gain(0.2).room(0.3).orbit(1)
// pad
$: note("0").scale("<C2:minor C2:minor F2:dorian C2:minor>")
  .s("pf:ff").adsr("0.2:0.4:0.5:0.4").cut(1).gain(0.14).room(0.35).orbit(1)
```

ハウス `[~ cp]*2` は使わない。ハットは `[~ hh]*4` のまま（`hh*8` にしない）。

## 音色パレット（新規 apply はここから選ぶ）

Pattern はグリッド・次数・スロットの見本。新規曲は下表からスロットごとに 1 つ選び、このフェンスの `.s()` を毎回コピーしない。同一曲の pitched 2 本に同じ `.s()` を使わない。slug の意味は strudel-pcm-catalog の INDEX。lead / pad の PCM は `.s()` の直後に `.adsr`、そのあと `.cut(1)`。FX ライザーだけ `<>` 間引き。

| スロット | 芯 | 代替 | 禁止 |
| --- | --- | --- | --- |
| drums | `bd:lf` + 遅い `[~ hh]*4` | `sd:br`、`hh:dk` | `hh*8`、`[~ cp]*2` |
| bass | `bs:su` 1 本 | （重ねない） | `bs:hf` 重ね、wobble |
| lead | ダスト | `plk:lf`、`ld:ny` | `ld:ss`、スーパーソー |
| hook | キー | `ep:rs`、`ep:wr` | 303、`plk:ss` |
| arp | | `plk:ny`、`plk:lf` | `plk:dt` |
| chords | `[0,2,4]` | `ep:mt` | `triangle` |
| pad | **C2** + `.adsr` | `pf:cl`、`dr:th` を `<>`、`pf:ff`+`note("0")` | C4、ADSR なし、gabber、`ld:an` |
| perc（任意） | | `fx:ck` を 4 小節に 1 | 毎小節のクラックル |
| vox | perc の代わりの 8 本目。疎、`.cut(1)` | `vc:na` at `C4:`（`.lpf` 可） | 16 分 EDM チョップ、毎小節 `vc:yeah`、自前 WAV を invent |

## レシピ

- トラックは 7 本: `// drums` `// bass` `// lead` `// hook` `// arp` `// chords` `// pad`（任意で perc **または** vox）
- ドラムは 1 本の `$:`。キック／スネア／ハットに分けない。`bd:lf` + 遅いハット
- ピッチトラックは 4 小節 `.scale`（bass / lead / hook / chords は C4、arp は C5、**pad は C2**）
- PCM フロアは `C4:`。シンセサブは `C2:`。**pad は C2** + `.adsr`。フロアは `bs:su` だけ（`bs:hf` と重ねない）
- フックは `ep:rs`（C3 録音 → native は C4 スケール）。リードは `plk:lf` + `.adsr`
- コードは 3 音まで。パッドは音色パレット（`pf:ff` なら `note("0")`）
- メロ／コード／パッドに `triangle` / `sine` を使わない。arp はメロディ楽器（`perc:` ではない）
- 長い PCM を ADSR なしで毎小節撃たない。lead / pad は `.adsr` + `.cut(1)` ならグリッド可。`plk:*` / `ep:*` / `bs:*` / `vc:*`、波形、`wt_*`、ライブ `.fm` も使える

## Pitfalls

1. 高速ハットでハウス化する（`hh*8` や `[~ cp]*2`）
2. `stack` / `.cpm` / `.lfo`
3. `note("c3'maj")` は root 単音。和音は `[0,2,4]`
4. `bs:su` の上に `bs:hf` や別のサブを重ねる
5. `ep:rs` / `plk:lf` に `.scale("C3:…")` を付ける（1 オクターブ下がる）。pad を C4 に置く／lead・pad から `.adsr` を外す
6. 新規 apply を 3 本のまま出す
7. 84 BPM を 124 ハウスと DJ ペアにする（Transport は 1 つ）
8. 新規 apply でフェンスの `.s()` を全コピーする。スーパーソーや 303 を載せる

## Checklist

- [ ] 7–8 本（drums / bass / lead / hook / arp / chords / pad。任意 perc または vox）
- [ ] 4 小節 `.scale("<…>")`。**pad は C2** + `.adsr`。lead PCM は `.adsr`。arp はメロディ楽器。ドラムは 1 本。`.s()` は音色パレット
- [ ] `dj_hermes_apply_song(content, deck)`。save は残す指示のときだけ
