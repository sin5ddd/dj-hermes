---
name: strudel-genre-lofi-hiphop
description: "Use when writing short Lo-fi Hip Hop live loops for strudel-rs."
version: 3.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, lofi-hiphop, live-coding]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × ローファイ・ヒップホップ（短いライブループ）

## Overview
遅め BPM・柔らかいコード・控えめドラム。BPM 目安 70–90。

## コピー用フル例

```
// @title visitor-lofi
// @genre lofi-hiphop
setcpm(80/4)
// drums
$: s("bd ~ ~ sd, [~ hh]*4").gain(0.45)
// chords
$: note("[0,2,4] ~ [0,3,5] ~").scale("C3:minor").s("triangle").lpf(1200).gain(0.35)
  .attack(0.05).decay(0.3).sustain(0.5).release(0.3).room(0.3).orbit(1)
// bass
$: note("0 ~ 2 ~").scale("C2:minor").s("sine").lpf(300).gain(0.45)
```

## ライブで変えると効く箇所

1. chord 次数  
2. `.room` / `.lpf`  
3. snare の有無  

## Pitfalls

1. ハットを詰めすぎる  
2. `stack` / `.cpm`  
3. 長尺 `cat`  

## Checklist

- [ ] 短い 2–3 トラック  
- [ ] `setcpm` + `$:`  
- [ ] `strudel_save_song`  
