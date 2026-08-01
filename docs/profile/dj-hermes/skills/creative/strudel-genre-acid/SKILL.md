---
name: strudel-genre-acid
description: "Use when writing short Acid live loops for strudel-rs."
version: 3.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, acid, live-coding]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × アシッド（短いライブループ）

## Overview
303 風 saw/square + 高い `lpq` + 4 つ打ち。BPM 目安 120–135。**短い 2–3 トラック**で十分。長尺 `cat` は書かない。

## コピー用フル例

```
// @title visitor-acid
// @genre acid
setcpm(128/4)
// drums
$: s("bd*4, [~ hh]*4, [~ sd]*2").gain(0.5)
// acid line (0=c 2=eb 3=f 4=g; −1 below root)
$: note("0 2 3 4 2 3 0 <-1 2>").scale("C2:minor")
  .s("sawtooth").lpf(800).lpq(16).gain(0.5)
  .attack(0.001).decay(0.15).sustain(0.05).release(0.05)
```

## ライブで変えると効く箇所

1. `.lpf` / `.lpq`（暗い ↔ 鼻にかかる）  
2. 次数列の末尾 `<>`  
3. drums の `hh*8` 化  

## Pitfalls

1. lpq 過大で耳が痛い → 12–16 前後  
2. 動的 `.lpf("<...>")` / `.lfo` → 非対応  
3. `stack` / `.cpm` / 16 小節 `cat`  

## Checklist

- [ ] saw/square + lpf/lpq、短いループ  
- [ ] `setcpm` + `$:`  
- [ ] `strudel_save_song`（同名上書きでライブ差分）  
