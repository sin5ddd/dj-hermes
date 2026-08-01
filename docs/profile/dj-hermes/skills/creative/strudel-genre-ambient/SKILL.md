---
name: strudel-genre-ambient
description: "Use when writing Ambient for strudel-rs."
version: 2.0.0
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
ビート控えめ・パッド長め・低めの gain。BPM 目安 60–90 相当（遅め setcpm）。

## コピー用フル例

```
// @title visitor-ambient
// @genre ambient
setcpm(70/4)
// pad
$: note("c3'maj ~ eb3'min ~").s("sawtooth").lpf(600).attack(0.2).release(0.8).gain(0.3).room(0.5)
// soft bass
$: note("c2 ~ ~ ~").s("sine").lpf(200).gain(0.35)
// air
$: s("hh*4").gain(0.08).hpf(10000)
```

## レシピ

1. 長い attack/release  
2. キックは無し or ごく薄い  
3. room は薄〜中  

## Pitfalls

1. 連打キックでアンビエントが崩れる  
2. `stack` / `.cpm`  
3. gain 過大  

## Checklist

- [ ] ゆったり  
- [ ] `setcpm` + `$:`  
- [ ] `strudel_save_song`  
