---
name: strudel-genre-progressive-house
description: "Use when writing short Progressive House live loops for strudel-rs."
version: 3.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, progressive-house, live-coding]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × プログレッシブハウス（短いライブループ）

## Overview
4 つ打ち + じわっと動くベース/パッド。BPM 目安 122–128。構成変化は **ライブで gain/lpf を触る**（16 小節 cat ではない）。

## コピー用フル例

```
// @title visitor-prog-house
// @genre progressive-house
setcpm(124/4)
// drums
$: s("bd*4, [~ sd]*2, [~ hh]*4, [~ oh]*2").gain(0.55)
// bass
$: note("0 0 0 2 0 0 <3 4> 0").scale("C2:minor")
  .s("sawtooth").lpf(400).gain(0.5)
// pad
$: note("0 2 4 7").scale("C3:minor").s("wt_organ").lpf(1000).gain(0.25)
  .attack(0.1).sustain(0.6).release(0.3).orbit(1)
```

## ライブで変えると効く箇所

1. pad `.lpf` / gain をゆっくり上げる（疑似ビルド）  
2. bass 次数  
3. oh の有無  

## Pitfalls

1. 最初から全部盛り  
2. `stack` / `.cpm`  
3. 16 小節 `cat` を標準にする  

## Checklist

- [ ] 短いループ + ライブでフィルタ  
- [ ] `setcpm` + `$:`  
- [ ] `strudel_save_song`  
