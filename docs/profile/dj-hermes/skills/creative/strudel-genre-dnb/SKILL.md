---
name: strudel-genre-dnb
description: "Use when writing Drum and Bass for strudel-rs."
version: 2.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, dnb]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × ドラムンベース

## Overview
高速ブレイクビーツ + 重いサブ。体感 160–180 BPM 相当。`setcpm(BPM/4)` と `$:` のみ。

## コピー用フル例（そのまま save）

```
// @title visitor-dnb
// @genre drum-and-bass
setcpm(170/4)
// drums
$: s("bd ~ ~ sd ~ bd bd ~ ~ sd ~ ~").fast(2).gain(0.9)
// hat
$: s("hh*16").fast(2).gain(0.22).hpf(9000)
// bass
$: note("c1").s("sine").lpf(120).attack(0.01).release(0.4).gain(0.7)
```

`strudel_save_song(name="visitor-dnb", content=..., deck="B")` のように保存。

## レシピ

1. ドラムは 4 つ打ちではなくブレイク配置 + `.fast(2)`  
2. サブは `c1` 付近 + `sine` + 低い `lpf`  
3. ハットは `hh*16` で高域  

## Pitfalls

1. `bd*4` のまま → 速いハウスになる  
2. `stack(...).cpm(170)` → 保存 400  
3. サブが高すぎる → `c1`–`f1` 付近  

## Checklist

- [ ] `setcpm` + `$:` のみ  
- [ ] ブレイク配置  
- [ ] `strudel_save_song` 実行  
