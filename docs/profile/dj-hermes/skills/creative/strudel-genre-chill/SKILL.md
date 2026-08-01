---
name: strudel-genre-chill
description: "Use when writing Chill / downtempo for strudel-rs."
version: 2.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, chill]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × チル / ダウンテンポ

## Overview
遅め〜中庸、柔らかいドラム、控えめメロ。BPM 目安 80–100。

## コピー用フル例

```
// @title visitor-chill
// @genre chill
setcpm(90/4)
// drums
$: s("bd ~ ~ bd ~ ~ bd ~, ~ ~ sd ~, hh*8").gain(0.45)
// keys
$: note("c3 e3 g3 e3").s("triangle").lpf(1200).gain(0.35).room(0.3)
```

## レシピ

1. キックを間引く  
2. メロは triangle / soft saw  
3. room 薄め  

## Pitfalls

1. 高速 DnB 配置を持ってくる  
2. `stack` / `.cpm`  
3. 歪み過多  

## Checklist

- [ ] 落ち着いたテンポ  
- [ ] `setcpm` + `$:`  
- [ ] `strudel_save_song`  
