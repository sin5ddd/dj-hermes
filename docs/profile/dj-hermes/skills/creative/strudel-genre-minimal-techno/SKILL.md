---
name: strudel-genre-minimal-techno
description: "Use when writing short Minimal Techno live loops for strudel-rs."
version: 3.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, minimal-techno, live-coding]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × ミニマルテクノ（短いライブループ）

## Overview
要素少なめ・反復・細かい 1 点変化。BPM 目安 124–130。**トラックを足しすぎない。**

## コピー用フル例

```
// @title visitor-minimal
// @genre minimal-techno
setcpm(126/4)
// drums
$: s("bd*4, [~ hh]*4, [~ sd]*2").gain(0.5)
// bass (sparse)
$: note("0 ~ ~ ~ 0 ~ <2 3> ~").scale("C2:minor").s("sawtooth").lpf(350).gain(0.45)
```

## ライブで変えると効く箇所

1. 1 つの休符を音に変える  
2. `.lpf` を 50 ずつ  
3. たまに `,<~ [~@3 bd ~@4]>`  

## Pitfalls

1. レイヤー過多  
2. `stack` / `.cpm`  
3. メロディ盛り + 長尺 `cat`  

## Checklist

- [ ] 要素が少ない短いループ  
- [ ] `setcpm` + `$:`  
- [ ] `strudel_apply_song`  
