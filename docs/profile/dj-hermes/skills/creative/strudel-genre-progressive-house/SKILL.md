---
name: strudel-genre-progressive-house
description: "Use when writing Progressive House for strudel-rs."
version: 2.0.0
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
4 つ打ち + 長いパッド/アルペジオ感。BPM 目安 124–130。

## コピー用フル例

```
// @title visitor-prog-house
// @genre progressive-house
setcpm(128/4)
// drums
$: s("bd*4, hh*8, ~ sd ~ sd").gain(0.55)
// pad
$: note("c3'maj ~ g2'maj ~").s("sawtooth").lpf(900).attack(0.08).release(0.3).gain(0.32).room(0.4)
// bass
$: note("c2 c2 g1 g1").s("sine").lpf(280).gain(0.5)
```

## レシピ

1. キック固定、上物をゆっくり  
2. pad は attack を少し長め  
3. ベースは単純なルート  

## Pitfalls

1. 上物を増やしすぎて濁る  
2. `stack` / `.cpm`  
3. テンポを DnB 域にする  

## Checklist

- [ ] 4 つ打ち + パッド  
- [ ] `setcpm` + `$:`  
- [ ] `strudel_save_song`  
