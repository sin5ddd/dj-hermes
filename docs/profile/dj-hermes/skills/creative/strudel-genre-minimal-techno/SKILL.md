---
name: strudel-genre-minimal-techno
description: "Use when writing Minimal Techno for strudel-rs."
version: 2.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, minimal-techno]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × ミニマルテクノ

## Overview
要素少なめ・反復・細かい変化。BPM 目安 124–130。音を足しすぎない。

## コピー用フル例

```
// @title visitor-minimal
// @genre minimal-techno
setcpm(126/4)
// drums
$: s("bd*4, hh*8, ~ sd ~ ~").gain(0.5)
// bass
$: note("c2 ~ eb2 ~").s("sawtooth").lpf(350).gain(0.45)
```

## レシピ

1. キックは 4 つ打ち、他は隙間多め  
2. 変化は gain / lpf のスカラー調整で  
3. トラック数 3–5 本まで  

## Pitfalls

1. レイヤー過多 → ミニマルが崩れる  
2. `stack` / `.cpm` → 保存不可  
3. メロディを盛りすぎる  

## Checklist

- [ ] `setcpm` + `$:`  
- [ ] 要素が少ない  
- [ ] `strudel_save_song`  
