---
name: strudel-genre-dnb
description: "Use when writing short Drum and Bass live loops for strudel-rs."
version: 3.2.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, dnb, live-coding]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × ドラムンベース（短いライブループ）

## Overview
高速ブレイク + 重いサブ。**`setcpm(174/4)`（174 BPM）**。ドラム gain はサブより上。Reese は **square サブ + saw ミッド**（スクエア一本にしない）。`db` はサンプルに無い（その拍は無音）→ 必ず `bd`。

正本の理由: リポジトリ `docs/skills/dnb/SKILL.md`。

## コピー用フル例

```
// @title visitor-dnb
// @genre drum-and-bass
setcpm(174/4)
// drums — break in front of the sub
$: s("bd <~ sd> ~ sd ~ <bd ~> <bd sd> <bd ~>, hh*4, [~@5 oh ~@2]").fast(2).gain(0.6).lpf(4000)
// sub
$: note("0 3 0 <0 -1>").scale("C2:minor").s("square").lpf(120).gain(0.45)
  .attack(0.01).decay(0.5).release(0.4)
// mid reese
$: note("0 3 0 <0 -1>").scale("C2:minor").s("sawtooth").lpf(1000).gain(0.32)
  .attack(0.01).decay(0.4).release(0.3)
```

## ライブで変えると効く箇所

1. break の snare 位置（スペース / `<>`）  
2. sub の次数 `0` ↔ `-1`  
3. mid `.lpf` を 800–1200 の中で動かす  

## Pitfalls

1. `bd*4` のまま → 速いハウスになる  
2. `stack(...).cpm(170)` → 保存 400  
3. ドラム `.gain` がサブ以下 → ブレイクが沈む  
4. パターンに `db` → その拍は無音  
5. Reese を square 1 本にする  

## Checklist

- [ ] `setcpm(174/4)`  
- [ ] ドラム gain > サブ gain  
- [ ] square サブ + saw ミッド  
- [ ] `db` なし  
- [ ] `strudel_apply_song`  
