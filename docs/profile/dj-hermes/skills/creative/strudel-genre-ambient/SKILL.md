---
name: strudel-genre-ambient
description: "Use when writing short Ambient live loops for strudel-rs."
version: 3.1.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, ambient, live-coding]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × アンビエント（短いライブループ）

## Overview
遅いテンポ・長い attack/release・room/delay。BPM 目安 60–90。2–3 トラックで十分。  
ユーザー WAV があれば pad/tone/piano は **フルネーム**（`pad-ambient_drone01` / `piano-acoustic_soft` 等、bank なし）。無ければ `wt_organ` / `wt_bright`（→ sound-design / `samples/LAYOUT.md`）。

## コピー用フル例

```
// @title visitor-ambient
// @genre ambient
setcpm(70/4)
// pad — ユーザー kit があれば s("pad-ambient_drone01") 等に差し替え
$: note("<0 2 4 7>/2").scale("C3:minor").s("wt_organ").lpf(1200).gain(0.4)
  .attack(0.2).decay(0.3).sustain(0.7).release(0.5).room(0.45).orbit(1)
// shimmer
$: note("<0 2 4 6>/2").scale("C4:minor").s("wt_bright").vib("5:8").gain(0.18)
  .attack(0.1).release(0.4).delay(0.25).orbit(2)
// soft pulse (optional)
$: note("0 ~ 4 ~").scale("C2:minor").s("sine").lpf(400).gain(0.2)
```

## ライブで変えると効く箇所

1. pad の `.lpf` / `.room`  
2. `<>` で和音次数をゆっくり切替  
3. shimmer gain  

## Pitfalls

1. ドラムを詰めすぎる  
2. `stack` / `.cpm`  
3. 長尺 `cat`  

## Checklist

- [ ] 短い pad + 空間系  
- [ ] `setcpm` + `$:`  
- [ ] `strudel_save_song`  
