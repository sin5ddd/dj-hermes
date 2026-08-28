---
name: strudel-genre-house
description: "Use when writing short House live loops for strudel-rs."
version: 3.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, house, live-coding]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × ハウス（短いライブループ）

## Overview
4 つ打ち + オフビート + 短いベース。BPM 目安 120–128。**この程度で十分。** あとはライブで 1 箇所ずつ変える。

## コピー用フル例

```
// @title visitor-house
// @genre house
setcpm(124/4)
// drums
$: s("bd*4, [~ sd]*2, [~ hh]*4, [~ oh]*2").gain(0.55)
// bass
$: note("0 0 2 4 0 <2 3> 4 0").scale("C2:minor")
  .s("sawtooth").lpf(450).gain(0.5)
  .attack(0.005).decay(0.1).sustain(0.3).release(0.06)
// stab (optional)
$: note("~ ~ 4 ~").scale("C3:minor").s("square").lpf(2200).gain(0.18)
```

## ライブで変えると効く箇所

1. `hh` / `oh` 密度  
2. bass 次数の `<>`  
3. bass `.lpf`  

## Pitfalls

1. キック無しでコードだけ  
2. `stack` / `.cpm`  
3. 16 小節 `cat` を毎回書く  

## Checklist

- [ ] 4 つ打ちの短いループ  
- [ ] `setcpm` + `$:`  
- [ ] `strudel_apply_song`  
