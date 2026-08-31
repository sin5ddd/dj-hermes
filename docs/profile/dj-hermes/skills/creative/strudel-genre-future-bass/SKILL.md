---
name: strudel-genre-future-bass
description: "Use when writing Future Bass for strudel-rs."
version: 3.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, future-bass]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × Future Bass

## Overview
明るいコード + サイドチェイン風の隙間 + 中速 BPM（目安 140–150 相当だがシンプル配置で可）。

## コピー用フル例

```
// @title visitor-future-bass
// @genre future-bass
setcpm(140/4)
// kick
$: s("bd ~ bd ~").gain(0.85)
// snare
$: s("~ sd ~ sd").gain(0.6)
// hat
$: s("hh*8").gain(0.22)
// chords
$: note("[c3,e3,g3] ~ [e3,g3,b3] ~").s("sawtooth").lpf(1600).attack(0.02).gain(0.35).room(0.3)
// bass
$: note("c2 ~ g1 ~").s("sine").lpf(220).gain(0.5)
```

## レシピ

1. コードを前面、キックは間引き可  
2. 明るい lpf  
3. room 薄め  

鳴らすのは `strudel_apply_song(content, deck)`（次小節、無書き込み）。`strudel_save_song` は残す指示のときだけ（演奏は変えない）。

## Pitfalls

1. 複雑なチョップ記法の多用（未対応が多い）  
2. `stack` / `.cpm`  
3. コード gain 過大でクリップ  
4. `note("c3'maj")` は root 単音。和音は `[c3,e3,g3]` または次数 `[0,2,4]`

## Checklist

- [ ] コード + リズム  
- [ ] `setcpm` + `$:`  
- [ ] `strudel_apply_song(content, deck)`（save は残す指示のときだけ）  
