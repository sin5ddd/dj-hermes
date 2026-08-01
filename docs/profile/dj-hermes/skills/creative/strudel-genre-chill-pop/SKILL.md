---
name: strudel-genre-chill-pop
description: "Use when writing Chill Pop for strudel-rs."
version: 2.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, chill-pop]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × チルポップ

## Overview
ポップなコード感 + 軽いビート。BPM 目安 95–110。

## コピー用フル例

```
// @title visitor-chill-pop
// @genre chill-pop
setcpm(100/4)
// kick
$: s("bd ~ bd ~").gain(0.8)
// snare
$: s("~ sd ~ sd").gain(0.55)
// hat
$: s("hh*8").gain(0.2)
// chords
$: note("c3'maj ~ f3'maj ~").s("triangle").lpf(1400).gain(0.35).room(0.25)
// bass
$: note("c2 ~ f2 ~").s("sine").lpf(250).gain(0.45)
```

## レシピ

1. シンプルな 2–4 コード循環  
2. キックはハーフっぽくても可  
3. メロ/コードは gain 控えめ  

## Pitfalls

1. コードに歪みを載せすぎ  
2. `stack` / `.cpm`  
3. 和音記法を知らない場合は単音アルペジオで代替  

## Checklist

- [ ] ポップな進行  
- [ ] `setcpm` + `$:`  
- [ ] `strudel_save_song`  
