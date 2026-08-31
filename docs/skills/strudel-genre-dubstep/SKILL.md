---
name: strudel-genre-dubstep
description: "Use when writing Dubstep for strudel-rs."
version: 3.0.0
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
$: s("bd ~ ~ ~ bd ~ sd ~").gain(0.9)
// hat
$: s("hh*8").gain(0.2).hpf(9000)
// wobble bass — cutoff LFO via sine.rangex (method name lfo is unimplemented)
$: note("c1 c1 eb1 c1").s("sawtooth").lpf(sine.rangex(80, 600)).lpq(6).gain(0.55)
```

## レシピ

1. スネアを後ろめに置いてハーフ感  
2. ベースは低音 + `.lpf(sine.rangex(min, max))` の wobble（`.lfo(...)` メソッドは未実装）  
3. キックとサブの同時打に注意  

鳴らすのは `strudel_apply_song(content, deck)`（次小節、無書き込み）。`strudel_save_song` は残す指示のときだけ（演奏は変えない）。

## Pitfalls

1. `.lfo(...)` メソッドは未実装。wobble は `.lpf(sine.rangex(…))`。`.vib("<…>")` は不可  
2. `stack` / `.cpm`  
3. ハイハットだらけで低域が埋もれる  

## Checklist

- [ ] ハーフ感  
- [ ] `setcpm` + `$:`  
- [ ] `strudel_apply_song(content, deck)`（save は残す指示のときだけ）  
