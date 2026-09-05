---
name: strudel-genre-ambient
description: >-
  Use when writing ambient for strudel-rs: thin or no kick, pad as
  primary, rest-heavy melody, low gain, around 70 BPM. Not chill
  drums and not house clap.
version: 5.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, ambient]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × アンビエント

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
$: note("[0,2,4] ~ ~ ~").scale("<C3:minor C3:minor G3:dorian C3:minor>")
  .s("triangle").lpf(800).gain(0.18)
  .attack(0.2).release(0.8)
// pad
$: note("[0,4]").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("pf:ff").gain(0.24)
  .attack(0.3).decay(0.4).sustain(0.7).release(0.6).room(0.5).orbit(1)
```

`pf:ff` は録音済みの 5 度（C+G）。`[0,2,4]` で鳴らさない。パッドは `[0,4]` の移調だけ。

## レシピ

- トラックは 7 本: `// drums` `// bass` `// lead` `// hook` `// arp` `// chords` `// pad`（任意で perc。このフェンスでは arp が perc）
- ドラムは 1 本の `$:`。キック／スネア／ハットに分けない
- ピッチトラックは 4 小節 `.scale("<C4:minor C4:minor G4:dorian C4:minor>")`（コードは C3、パッドは C4）
- PCM は `C4:`。シンセサブは `C2:`。`bs:su` と `bs:hf` は重ねない
- キックは `bd:lf` を 1 拍だけ、gain 0.18。無しでもよい
- パッドが主。`pf:ff` は `[0,4]`。コードは `[0,2,4]` を 3 音まで
- メロ（lead / hook）は休符多め、gain 0.10–0.16
- `in_bank=no` の長い PCM は書かない。使えるのは `ld:ss` `pf:ff` `plk:*` `ep:*` `perc:*` `bs:*`、波形、`wt_*`、ライブ `.fm`

## Pitfalls

1. 連打キックでアンビエントが崩れる
2. `stack` / `.cpm` / `.lfo`
3. gain 過大
4. `note("c3'maj")` は root 単音。和音は `[0,2,4]`
5. `pf:ff` を `[0,2,4]` で鳴らす（中身は 5 度のまま三重になる）
6. `bs:su` の上に `bs:hf` や別のサブを重ねる
7. 新規 apply を 3 本のまま出す
8. 70 BPM を 126 テクノと DJ ペアにする（Transport は 1 つ）

## Checklist

- [ ] 7–8 本（drums / bass / lead / hook / arp / chords / pad。任意 perc）
- [ ] 4 小節 `.scale("<…>")`。ドラムは 1 本
- [ ] `strudel_apply_song(content, deck)`。save は残す指示のときだけ
