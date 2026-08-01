---
name: strudel-composition
description: "Use when writing strudel-rs patterns (mini-notation + $: tracks)."
version: 2.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, composition, mini-notation]
    related_skills:
      - strudel-data-format
      - strudel-sound-design
---

# strudel-rs 作曲（Composition）

## Overview
strudel-rs は **曲ファイル**として `setcpm` + **`$:` トラック**だけを受け付ける。mini-notation は文字列の中（`s("...")` / `note("...")`）で使う。保存は必ず `strudel_save_song`。

## 保存テンプレ（そのまま content に）

```
// @title demo
setcpm(120/4)
// kick
$: s("bd*4").gain(0.9)
// snare
$: s("~ sd ~ sd").gain(0.7)
// hat
$: s("hh*8").gain(0.25)
// bass
$: note("c2 c2 eb2 g2").s("sawtooth").lpf(500).gain(0.55)
```

## Mini-notation（`$:` の文字列内だけ）

| 記法 | 意味 |
| --- | --- |
| `bd sd hh` | 順にイベント |
| `bd*4` | 1 サイクルに 4 回 |
| `~` / `-` | 休符 |
| `[a b]` | 細分化 |
| `<a b c>` | サイクルまたぎ列 |
| `a,b` | 同時（ポリ） |
| `bd(3,8)` | ユークリッド |

```
$: s("bd*4").gain(0.9)
$: s("hh*8").gain(0.3)
$: note("c3 e3 g3").s("sawtooth").lpf(800).gain(0.4)
$: s("bd(3,8)").gain(0.85)
```

## チェーン（スカラー引数のみ）

```
$: note("c2 eb2 f2 g2").s("sawtooth").lpf(600).lpq(8).gain(0.5)
$: s("hh*16").hpf(8000).gain(0.2)
```

不可: `.lpf("<400 1200>")`、`.vib("<1 4>")`、`stack(...)`、`.cpm(120)`。

## テンポ

- `setcpm(30)` → BPM 120（30 cycles/min × 4 beats）  
- `setcpm(120/4)` → BPM 120 と同じ書き方  
- 体感テンポを上げるのは主に mini の `*2` / `.fast(2)`  

## 禁止

- 本家 JS: `stack`, `cat`, `arrange`, `.cpm()`  
- `$:` 無しの裸パターン行  

## Pitfalls

1. チャットにコードだけ書いて保存しない  
2. `stack(...).cpm(170)` を content に入れる → 400  
3. 引数にミニ記法パターンを入れる → 非対応  

## Checklist

- [ ] `setcpm` + 複数 `$:`  
- [ ] mini は引用符の中だけ  
- [ ] `strudel_save_song` で保存（必要なら `deck`）  
