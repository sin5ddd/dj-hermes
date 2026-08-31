---
name: strudel-genre-chill-pop
description: "Use when writing Chill Pop for strudel-rs."
version: 3.0.0
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
$: note("[c3,e3,g3] ~ [f3,a3,c4] ~").s("triangle").lpf(1400).gain(0.35).room(0.25)
// bass
$: note("c2 ~ f2 ~").s("sine").lpf(250).gain(0.45)
```

## レシピ

1. シンプルな 2–4 コード循環  
2. キックはハーフっぽくても可  
3. メロ/コードは gain 控えめ  

鳴らすのは `strudel_apply_song(content, deck)`（次小節、無書き込み）。`strudel_save_song` は残す指示のときだけ（演奏は変えない）。

## Pitfalls

1. コードに歪みを載せすぎ  
2. `stack` / `.cpm`  
3. `note("c3'maj")` は root 単音。和音は `[c3,e3,g3]` または次数 `[0,2,4]`

## Checklist

- [ ] ポップな進行  
- [ ] `setcpm` + `$:`  
- [ ] `strudel_apply_song(content, deck)`（save は残す指示のときだけ）  
