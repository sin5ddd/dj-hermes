---
name: strudel-genre-future-bass
description: "Use when writing short Future Bass live loops for strudel-rs."
version: 3.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, future-bass, live-coding]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × フューチャーベース（短いライブループ）

## Overview
コード感 + 明るいリード + 4 つ打ち。BPM 目安 140–150。短い 3–4 トラック。

## コピー用フル例

```
// @title visitor-future-bass
// @genre future-bass
setcpm(145/4)
// drums
$: s("bd*4, [~ sd]*2, [~ hh]*4, [~ oh]*2").gain(0.5)
// chords (parallel degrees)
$: note("[0,2,4] ~ [0,3,5] ~").scale("C3:minor").s("sawtooth").lpf(1400).gain(0.3)
  .attack(0.02).decay(0.2).sustain(0.4).release(0.15)
// lead
$: note("7 9 <11 12> 9").scale("C4:minor").s("square").lpf(3000).gain(0.18)
// sub
$: note("0 ~ 0 ~").scale("C1:minor").s("sine").lpf(150).gain(0.55)
```

## ライブで変えると効く箇所

1. chord の並列次数  
2. lead `<>`  
3. chord `.lpf`  

## Pitfalls

1. レイヤー過多でクリップ  
2. `stack` / `.cpm` / `.lfo`  
3. 長尺 `cat`  

## Checklist

- [ ] 短い chords + lead  
- [ ] `setcpm` + `$:`  
- [ ] `strudel_save_song`  
