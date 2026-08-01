---
name: strudel-genre-house
description: "Use when writing House for strudel-rs."
version: 2.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, house]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × ハウス

## Overview
4 つ打ちキック + オープンハット + ベース。BPM 目安 120–128。

## コピー用フル例

```
// @title visitor-house
// @genre house
setcpm(124/4)
// kick
$: s("bd*4").gain(0.9)
// clap
$: s("~ cp ~ cp").gain(0.65)
// hat
$: s("hh*8").gain(0.28)
// bass
$: note("c2 c2 eb2 g2").s("sawtooth").lpf(450).gain(0.5)
```

## レシピ

1. `bd*4` を土台  
2. オフビート clap / snare  
3. ベースは短いノート列  

## Pitfalls

1. キック無しでコードだけ  
2. `stack` / `.cpm`  
3. ベースがキックと同帯域で濁る → lpf を下げる  

## Checklist

- [ ] 4 つ打ち  
- [ ] `setcpm` + `$:`  
- [ ] `strudel_save_song`  
