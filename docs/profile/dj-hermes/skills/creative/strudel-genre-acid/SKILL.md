---
name: strudel-genre-acid
description: "Use when writing Acid tracks for strudel-rs."
version: 2.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, acid]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × アシッド

## Overview
303 風ソー/スクエア + 高い resonance（lpq）+ 4 つ打ち。BPM 目安 120–135。

## コピー用フル例

```
// @title visitor-acid
// @genre acid
setcpm(128/4)
// kick
$: s("bd*4").gain(0.9)
// hat
$: s("hh*8").gain(0.28).hpf(8000)
// acid
$: note("c2 eb2 f2 g2 eb2 f2 c2 bb1").s("sawtooth").lpf(800).lpq(16).decay(0.15).sustain(0.05).gain(0.5)
```

## レシピ

1. ベースラインは短い decay  
2. `lpf` + `lpq` を高め（12–20）  
3. キックは邪魔しない  

## Pitfalls

1. lpq 過大で耳が痛い → 16 前後から  
2. 動的 `.lpf("<...>")` → 非対応（スカラーのみ）  
3. `stack` / `.cpm`  

## Checklist

- [ ] saw/square + lpf/lpq  
- [ ] `setcpm` + `$:`  
- [ ] `strudel_save_song`  
