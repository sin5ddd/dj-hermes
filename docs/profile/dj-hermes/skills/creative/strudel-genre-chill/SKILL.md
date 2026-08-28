---
name: strudel-genre-chill
description: "Use when writing short Chill live loops for strudel-rs."
version: 3.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, chill, live-coding]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × チル（短いライブループ）

## Overview
ゆったり BPM・控えめドラム・柔らかいベース。BPM 目安 90–110。

## コピー用フル例

```
// @title visitor-chill
// @genre chill
setcpm(100/4)
// drums (sparse)
$: s("bd ~ ~ ~, [~ sd]*2, [~ hh]*4").gain(0.45)
// bass
$: note("0 ~ 2 ~ 0 <3 4>").scale("C2:minor").s("sine").lpf(350).gain(0.5)
// soft lead
$: note("~ 4 ~ 7 ~ <6 9>").scale("C3:minor").s("triangle").lpf(1800).gain(0.2)
```

## ライブで変えると効く箇所

1. drums の密度（`hh` を増減）  
2. bass の `<>`  
3. lead `.lpf` / gain  

## Pitfalls

1. 4 つ打ちを詰めすぎてチルが崩れる  
2. `stack` / `.cpm`  
3. 長尺 `cat`  

## Checklist

- [ ] 隙間のある短いループ  
- [ ] `setcpm` + `$:`  
- [ ] `strudel_apply_song`  
