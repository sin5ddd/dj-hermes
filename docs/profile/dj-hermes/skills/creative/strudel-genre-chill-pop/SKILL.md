---
name: strudel-genre-chill-pop
description: "Use when writing short Chill Pop live loops for strudel-rs."
version: 3.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, chill-pop, live-coding]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × チルポップ（短いライブループ）

## Overview
明るいメジャー寄り・シンプルなフック。BPM 目安 100–120。

## コピー用フル例

```
// @title visitor-chill-pop
// @genre chill-pop
setcpm(110/4)
// drums
$: s("bd*4, [~ sd]*2, [~ hh]*4").gain(0.5)
// bass
$: note("0 0 4 2").scale("C2:major").s("sawtooth").lpf(450).gain(0.5)
// hook
$: note("0 2 4 <7 9>").scale("C4:major").s("square").lpf(2500).gain(0.18)
```

## ライブで変えると効く箇所

1. hook の `<>`  
2. scale major ↔ minor  
3. hook gain / lpf  

## Pitfalls

1. レイヤー過多  
2. `stack` / `.cpm`  
3. 長尺 `cat`  

## Checklist

- [ ] 短い 2–3 トラック  
- [ ] `setcpm` + `$:`  
- [ ] `strudel_apply_song`  
