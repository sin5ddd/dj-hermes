---
name: strudel-genre-dnb
description: "Use when writing short Drum and Bass live loops for strudel-rs."
version: 3.0.0
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
高速ブレイク + 重いサブ。体感 160–180 BPM。**1 サイクルの短い break 骨格**で十分。16 小節 `cat` は不要。

## コピー用フル例

```
// @title visitor-dnb
// @genre drum-and-bass
setcpm(170/4)
// drums (break + hats; .fast for feel)
$: s("bd ~ ~ sd ~ bd bd ~, hh*16, [~@5 oh ~@2]").fast(2).gain(0.55)
// sub
$: note("0 ~ ~ ~").scale("C1:minor").s("sine").lpf(120).gain(0.7)
  .attack(0.01).release(0.4)
// stab (optional)
$: note("~ 4 ~ <7 9>").scale("C3:minor").s("square").lpf(2000).gain(0.15)
```

## ライブで変えると効く箇所

1. break の snare 位置（スペース区切り）  
2. sub の次数 `0` ↔ `-1`  
3. `hh*16` ↔ `hh*8`  

## Pitfalls

1. `bd*4` のまま → 速いハウスになる  
2. `stack(...).cpm(170)` → 保存 400  
3. サブが高すぎる → `C1` 付近  

## Checklist

- [ ] 短い break + sub  
- [ ] `setcpm` + `$:`  
- [ ] `strudel_save_song`  
