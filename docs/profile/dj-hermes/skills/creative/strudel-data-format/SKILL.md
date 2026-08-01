---
name: strudel-data-format
description: "Use when writing .strudel files for strudel-rs save/load."
version: 2.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, file-format, metadata]
    related_skills:
      - strudel-composition
      - strudel-sound-design
---

# strudel-rs 曲ファイル形式

## Overview
strudel-rs は `.strudel` テキストをパースして再生する。保存は MCP **`strudel_save_song`** のみ（file ツール不可）。書き込み先は `~/.config/strudel-rs/songs/<name>.strudel` のみ。

## 必須の形（コピー用）

```
// @title My Song
// @by booth
// @genre house
setcpm(120/4)
// kick
$: s("bd*4").gain(0.9)
// hat
$: s("hh*8").gain(0.3)
// bass
$: note("c2 c2 eb2 g2").s("sawtooth").lpf(400).gain(0.55)
```

1. 任意: `// @title` / `@by` / `@genre` などのコメントタグ  
2. **必須**: `setcpm(N)` または `setcpm(BPM/4)`（1 cycle = 1 bar = 4 beats → エンジン BPM = N×4）。`setcps(x)` も可  
3. **必須**: 1 本以上の **`$:` 行**（トラック）。直前の `// name` がトラック名  

## strudel_save_song

| 引数 | 意味 |
| --- | --- |
| `name` | ベース名のみ（例 `visitor-house`）。パス禁止 |
| `content` | 上の全文 |
| `deck` | 任意 `A` / `B` — 保存後にロード |
| `overwrite` | 既定 true |

## 禁止（保存すると 400）

- `stack(...)` / `).cpm(...)` / 裸の `s("bd")` 行（`$:` 無し）  
- メソッド引数の動的ミニ記法: `.lpf("<200 800>")` など  
- `sine.range(...)` などの本家 JS ヘルパ  

## Pitfalls

1. チャットにコードを書いて終わり → 必ず `strudel_save_song` を呼ぶ  
2. `name` に日本語や `/` → ASCII の basename のみ  
3. content に `stack` を入れる → パース失敗  

## Checklist

- [ ] `setcpm` がある  
- [ ] 各トラックが `$:` で始まる  
- [ ] `strudel_save_song(name, content, deck?)` を実行した  
