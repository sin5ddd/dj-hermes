---
name: strudel-genre-dubstep
description: "Use when writing short Dubstep live loops for strudel-rs."
version: 3.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, dubstep, live-coding]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × ダブステップ（短いライブループ）

## Overview
ハーフタイム感・重いサブ・スペース。BPM 目安 140（ハーフ感じ）。2–3 トラック。

## コピー用フル例

```
// @title visitor-dubstep
// @genre dubstep
setcpm(140/4)
// drums (half-time snare)
$: s("bd ~ ~ ~, [~ ~ sd ~], [~ hh]*4").gain(0.55)
// sub wobble-ish (manual: change lpf live)
$: note("0 ~ 0 ~").scale("C1:minor").s("sawtooth").lpf(180).lpq(8).gain(0.65)
  .attack(0.01).decay(0.2).sustain(0.4).release(0.1)
// noise hit (optional)
$: s("~ ~ [oh ~] ~").gain(0.25).hpf(3000)
```

## ライブで変えると効く箇所

1. sub `.lpf` / `.lpq`（ウォブル感の代用）  
2. snare 位置  
3. 休符を増やす  

## Pitfalls

1. 4 つ打ち固定でハーフタイム感が消える  
2. `stack` / `.cpm` / `.lfo`  
3. 長尺 `cat`  

## Checklist

- [ ] 短い half-time 骨格  
- [ ] `setcpm` + `$:`  
- [ ] `strudel_apply_song`  
