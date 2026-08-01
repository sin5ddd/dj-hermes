---
name: strudel-genre-lofi-hiphop
description: "Use when writing lo-fi hip hop for strudel-rs."
version: 2.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, lofi, hiphop]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × ローファイ・ヒップホップ

## Overview
遅め BPM、乾いたドラム、柔らかいキー。BPM 目安 75–90。

## コピー用フル例

```
// @title visitor-lofi
// @genre lofi-hiphop
setcpm(84/4)
// kick
$: s("bd ~ ~ bd ~ ~ bd ~").gain(0.75)
// snare
$: s("~ ~ sd ~").gain(0.55)
// hat
$: s("hh*8").gain(0.15)
// keys
$: note("c3 e3 g3 a3").s("triangle").lpf(1100).gain(0.32).room(0.35)
// bass
$: note("c2 ~ a1 ~").s("sine").lpf(200).gain(0.4)
```

## レシピ

1. キックを間引く  
2. キーは triangle + 低め lpf  
3. room で空間を薄く  

## Pitfalls

1. 高速ハットでハウス化  
2. `stack` / `.cpm`  
3. ノイズレイヤー過多  

## Checklist

- [ ] 遅め・乾いた感じ  
- [ ] `setcpm` + `$:`  
- [ ] `strudel_save_song`  
