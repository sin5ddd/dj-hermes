---
name: strudel-genre-electro
description: "Use when writing Electro for strudel-rs."
version: 2.0.0
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
機械的なキック/スネア + シンセベース。BPM 目安 120–130。

## コピー用フル例

```
// @title visitor-electro
// @genre electro
setcpm(126/4)
// kick
$: s("bd*4").gain(0.9)
// snare
$: s("~ sd ~ sd").gain(0.7)
// hat
$: s("hh*8").gain(0.25)
// bass
$: note("c2 ~ c2 eb2").s("square").lpf(500).gain(0.5)
```

## レシピ

1. タイトな 4 つ打ち  
2. square / saw の短いベース  
3. ハットは乾いたまま  

## Pitfalls

1. アンビエント寄りの長い release  
2. `stack` / `.cpm`  
3. サンプル bank 名の推測書き  

## Checklist

- [ ] 機械的グルーヴ  
- [ ] `setcpm` + `$:`  
- [ ] `strudel_save_song`  
