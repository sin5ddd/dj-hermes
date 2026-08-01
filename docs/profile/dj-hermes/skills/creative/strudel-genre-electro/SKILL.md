---
name: strudel-genre-electro
description: "Use when writing short Electro live loops for strudel-rs."
version: 3.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, electro, live-coding]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × エレクトロ（短いライブループ）

## Overview
シャープなキック・電子感のあるベース。BPM 目安 120–130。

## コピー用フル例

```
// @title visitor-electro
// @genre electro
setcpm(126/4)
// drums
$: s("bd*4, [~ sd]*2, hh*8").gain(0.55)
// bass
$: note("0 0 3 0 <2 5> 0 4 0").scale("C2:minor")
  .s("square").lpf(700).gain(0.5)
  .attack(0.001).decay(0.08).sustain(0.2).release(0.04)
// stab
$: note("~ ~ 7 ~").scale("C3:minor").s("sawtooth").lpf(1800).gain(0.2)
```

## ライブで変えると効く箇所

1. bass 次数と `.lpf`  
2. `hh*8` ↔ `[~ hh]*4`  
3. stab 追加/削除  

## Pitfalls

1. ベースがキックと同帯域で濁る  
2. `stack` / `.cpm`  
3. 長尺 `cat`  

## Checklist

- [ ] 短い 2–3 トラック  
- [ ] `setcpm` + `$:`  
- [ ] `strudel_save_song`  
