---
name: strudel-genre-minimal-techno
description: >-
  Use when writing Minimal Techno for strudel-rs: 126 BPM, 7–8 sparse
  tracks with many rests and low gain. Not a house [~ cp]*2 backbeat;
  one cp every 4 bars is perc color only.
version: 5.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, minimal-techno]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × ミニマルテクノ

## Overview

要素は反復、隙間は多い、gain は低め。この Skill のテンポは **126 BPM**（目安 124–130）。トラック数を削るのではなく、**7–8 本のまま `~` を残す**。

## When

- 依頼が **ミニマルテクノ**（4 つ打ち、隙間、細かい変化）のとき
- 音を足しすぎず、ハウスの 2/4 クラップでもないとき
- 4 小節に 1 回の `cp` は色付けで、バックビートではないとき

## Pattern

```
// @title visitor-minimal
// @genre minimal-techno
setcpm(126/4)
// drums
$: s("bd*4, hh*8, <~ ~ ~ cp>").gain(0.7)
// bass
$: note("0 ~ 3 ~").scale("<C2:minor C2:minor C2:minor G2:phrygian>")
  .s("sawtooth").lpf(320).gain(0.4)
// lead
$: note("~ ~ 7 ~").scale("<C4:minor C4:minor C4:minor G4:phrygian>")
  .s("plk:pk").gain(0.1).cut(1)
// hook
$: note("~ 4 ~ ~").scale("<C4:minor C4:minor C4:minor G4:phrygian>")
  .s("plk:ac").gain(0.12).cut(1)
// arp
$: s("<~ perc:tk ~ perc:st>").gain(0.12)
// chords
$: note("~ [0,2,4] ~ ~").scale("<C4:minor C4:minor C4:minor G4:phrygian>")
  .s("ep:mt").gain(0.14)
// pad
$: note("0").scale("<C4:minor C4:minor C4:minor G4:phrygian>")
  .s("pf:ff").orbit(2).gain(0.1).room(0.25)
```

同梱 `songs/minimal-techno/01.strudel` はこのフェンスと同じ。

## Why

キックは毎拍、ハットは 16 分、`cp` は 4 小節に 1 回だけ。`[~ cp]*2` にするとハウスのバックビートになる。

## レシピ

1. キックは 4 つ打ち。スネアの 2/4 は置かない
2. メロとコードは `~` を残す。gain は低め（リード 0.1 前後）
3. 変化は 4 小節目の `cp` と `.scale` の 4 小節目（G phrygian）
4. arp スロットは毎小節撃たない perc（`perc:tk` / `perc:st`）
5. 本数は 7–8 のまま。3–5 本に削らない

鳴らすのは `strudel_apply_song(content, deck)`（次小節、無書き込み）。`strudel_save_song` は残す指示のときだけ（演奏は変えない）。

## Pitfalls

1. `stack()` / `.cpm()` / `.lfo()` → apply / save とも 400
2. ドラムを kick / hat / perc の 3 `$:` に分ける
3. 長い PCM（`ld:` / `dr:` / `pf:` / `ps:`）を毎小節撃たない
4. `[~ cp]*2` を書く（4 小節に 1 回の `cp` とは別物）
5. レイヤー過多やメロの埋めすぎで隙間が消える

## Checklist

- [ ] 7 本（// drums // bass // lead // hook // arp // chords // pad）
- [ ] 4 小節フレーズ（`.scale("<…>")` が 4 個。arp は 4 子の `<>`）
- [ ] ドラムは 1 本のカンマ層。`[~ cp]*2` は書いていない
- [ ] 隙間と低 gain が残っている
- [ ] `strudel_apply_song(content, deck)`（save は残す指示のときだけ）
