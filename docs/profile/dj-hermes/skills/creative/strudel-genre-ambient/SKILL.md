---
name: strudel-genre-ambient
description: >-
  Use when writing ambient for dj-hermes: thin or no kick, pad as
  primary, rest-heavy melody, low gain, around 70 BPM. Not chill
  drums and not house clap.
version: 5.1.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [dj-hermes, music, genre, ambient]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# dj-hermes × アンビエント

## Overview

キックは薄い、または無し。パッドが主。メロは休符多め。gain は低め。
BPM 目安 60–90。フェンスは **70**（`setcpm(70/4)`）。
ハウスやチルより疎い。`songs/ambient/01.strudel` はこのフェンスと同じ（7 本）。

## When

- 来場者がアンビエント、ドローン、空間、静かなパッドを求めるとき
- キック前のテクノ、ハウスの clap、チルの 2/4 スネアではないとき
- 新規 apply / プリセット（2–5 本では足りない）

## Pattern

```
// @title visitor-ambient
// @genre ambient
setcpm(70/4)
// drums
$: s("bd:lf ~ ~ ~").gain(0.18)
// bass
$: note("0 ~ ~ <0 0 ~ 0>").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("bs:su").gain(0.28)
// lead
$: note("~ 7 ~ <9 7 4 11>").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("plk:bl").gain(0.1).cut(1)
// hook
$: note("0@2 ~ 4@2 ~").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("plk:am").gain(0.16).cut(1)
// arp
$: s("<~ perc:cm ~ perc:tg>").gain(0.1)
// chords
$: note("[0,2,4] ~ ~ ~").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("ep:mt").gain(0.18)
// pad
$: note("0").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("pf:ff").gain(0.24).room(0.5).orbit(1)
```

`pf:ff` は録音済みの 5 度（C+G）。`[0,2,4]` でも `[0,4]` でも鳴らさない。パッドは `note("0")` の移調だけ。コードは `ep:mt`。

## 音色パレット（新規 apply はここから選ぶ）

Pattern はグリッド・次数・スロットの見本。新規曲は下表からスロットごとに 1 つ選び、このフェンスの `.s()` を毎回コピーしない。同一曲の pitched 2 本に同じ `.s()` を使わない。slug の意味は strudel-pcm-catalog の INDEX。長尺はパッド役で `<>`（gain 低）。キック連打にしない。

| スロット | 芯 | 代替 | 禁止 |
| --- | --- | --- | --- |
| drums | `bd:lf` 1 打または無し | `bd:lf`、無し | `bd*4`、house clap、zap |
| bass | 疎なサブ 1 本 | `bs:su` | `bs:hf` 重ね、wobble |
| lead | 休符多め | `ld:et`、`ld:fl`、`plk:bl`、`ld:si` | `ld:ss` アンセム、`ld:zp` |
| hook | 長いノート | `plk:am`、`ld:cr` | gabber、`plk:ss` |
| arp | perc 疎 | `perc:cm`、`perc:tg` | 16 分埋め |
| chords | 疎 | `ep:mt`、薄い `ld:fp` を `<>` | `triangle`、`[0,2,4]` 連打 |
| pad | **主** | `dr:ad`、`dr:fg`、`pf:cl`、`pf:wa`、`ps:sh`、`dr:uw`（`<>`） | スーパーソー、`bd*4` の上に載せるだけ |

## レシピ

- トラックは 7 本: `// drums` `// bass` `// lead` `// hook` `// arp` `// chords` `// pad`（任意で perc。このフェンスでは arp が perc）
- ドラムは 1 本の `$:`。キック／スネア／ハットに分けない
- ピッチトラックは 4 小節 `.scale("<C4:minor C4:minor G4:dorian C4:minor>")`（コードは C3、パッドは C4）
- PCM は `C4:`。シンセサブは `C2:`。`bs:su` と `bs:hf` は重ねない
- キックは `bd:lf` を 1 拍だけ、gain 0.18。無しでもよい
- パッドが主。音色パレットの pad（`pf:ff` なら `note("0")`）。コードは 3 音まで
- メロ（lead / hook）は休符多め、gain 0.10–0.16
- 長い PCM（`ld:` / `dr:` / `pf:` / `ps:`）は毎小節撃たない。`plk:*` / `ep:*` / `perc:*` / `bs:*`、波形、`wt_*`、ライブ `.fm` も使える

## Pitfalls

1. 連打キックでアンビエントが崩れる
2. `stack` / `.cpm` / `.lfo`
3. gain 過大
4. `note("c3'maj")` は root 単音。和音は `[0,2,4]`
5. `pf:ff` を `[0,2,4]` で鳴らす（中身は 5 度のまま三重になる）
6. `bs:su` の上に `bs:hf` や別のサブを重ねる
7. 新規 apply を 3 本のまま出す
8. 70 BPM を 126 テクノと DJ ペアにする（Transport は 1 つ）
9. 新規 apply でフェンスの `.s()` を全コピーする。パッド以外をスーパーソーや zap にする

## Checklist

- [ ] 7–8 本（drums / bass / lead / hook / arp / chords / pad。任意 perc）
- [ ] 4 小節 `.scale("<…>")`。ドラムは 1 本。pad が主。`.s()` は音色パレット
- [ ] `dj_hermes_apply_song(content, deck)`。save は残す指示のときだけ
