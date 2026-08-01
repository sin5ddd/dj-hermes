---
name: strudel-genre-dubstep
description: "Use when writing Dubstep for strudel-rs."
version: 2.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, dubstep]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × ダブステップ

## Overview
ハーフタイム感のドラム + 低域ベース。BPM 目安 140（ハーフで体感 70）。

## コピー用フル例

```
// @title visitor-dubstep
// @genre dubstep
setcpm(140/4)
// drums
$: s("bd ~ ~ ~ bd ~ sd ~, hh*8").gain(0.55)
// wobble-ish bass (static lpf; no dynamic LFO mini-args)
$: note("c1 c1 eb1 c1").s("sawtooth").lpf(280).lpq(6).gain(0.55)
```

## レシピ

1. スネアを後ろめに置いてハーフ感  
2. ベースは低音 + 低め lpf  
3. キックとサブの同時打に注意  

## Pitfalls

1. 本家 wobble の複雑な LFO 記法は非対応  
2. `stack` / `.cpm`  
3. ハイハットだらけで低域が埋もれる  

## Checklist

- [ ] ハーフ感  
- [ ] `setcpm` + `$:`  
- [ ] `strudel_save_song`  
