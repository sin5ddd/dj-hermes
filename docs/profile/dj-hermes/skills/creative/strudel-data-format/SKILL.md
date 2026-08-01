---
name: strudel-data-format
description: "Use when writing short .strudel live-loop files for strudel-rs save/load."
version: 3.1.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, file-format, metadata, live-coding]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-live-edit
---

# strudel-rs 曲ファイル形式

## Overview

strudel-rs は `.strudel` テキストをパースして再生する。保存は MCP **`strudel_save_song`** のみ（file ツール不可）。書き込み先は `~/.config/strudel-rs/songs/<name>.strudel` のみ。

**曲の長さの目安**: トラック **2–5 本**、1 サイクル骨格 + `<>`。長尺 `cat` は非既定（→ strudel-composition）。

## 必須の形（コピー用・ライブ短いループ）

```
// @title My Song
// @by booth
// @genre house
setcpm(120/4)
// drums
$: s("bd*4, [~ sd]*2, [~ hh]*4").gain(0.55)
// bass
$: note("0 0 2 4").scale("C2:minor").s("sawtooth").lpf(400).gain(0.55)
```

正本寄りの 4 トラック例:

```
// @title live-basic
setcpm(128/4)
$: note("0 2 0 3 0 <2 4> <4 2>").scale("C2:minor")
  .s("sawtooth").lpf(500).gain(0.7)
  .attack(0.001).decay(0.06).sustain(0.15).release(0.04)
$: s("bd*4, [~ <sd oh>]*2, [~ hh]*4").gain(0.5)
$: note("7 6 <4 9> <3 [4 2]>").scale("C2:minor")
  .s("square").lpf(3200).gain(0.16)
$: note("0 2 4 [6,8] 0 2 4 [7,9]").scale("C4:minor")
  .s("sawtooth").gain(0.35).bpf(1500)
```

1. 任意: `// @title` / `@by` / `@genre` などのコメントタグ  
2. **必須**: `setcpm(N)` または `setcpm(BPM/4)`（1 cycle = 1 bar = 4 beats → エンジン BPM = N×4）。`setcps(x)` も可  
3. **必須**: 1 本以上の **`$:` 行**（トラック）。直前の `// name` がトラック名  
4. ドラムは原則 **1 本の `s(...)`**。詳細は strudel-composition  

## strudel_save_song

| 引数 | 意味 |
| --- | --- |
| `name` | ベース名のみ（例 `visitor-house`）。パス禁止 |
| `content` | 上の全文 |
| `deck` | 任意 `A` / `B` — 保存後にロード |
| `overwrite` | 既定 true |

**ライブ時**: 同じ `name` + `deck` で content を少し変えて上書きする（毎回新しい名前を作らない）。

## 禁止（保存すると 400 または再生失敗）

- `stack(...)` / `).cpm(...)` / 裸の `s("bd")` 行（`$:` 無し）  
- 未実装: `.lfo(...)`（メソッド名）、未同梱 `cp`、`vib("<…>")` など一部の動的引数  
- 可: `.scale("<…>")` 進行、`.lpf("<400 1200>")` / `.lpf(sine.rangex(500,4000))`、`.add` / `.sub` / `.ply`

- `sine.range(...)` などの本家 JS ヘルパ  

## Pitfalls

1. チャットにコードを書いて終わり → 必ず `strudel_save_song`  
2. `name` に日本語や `/` → ASCII の basename のみ  
3. content に `stack` を入れる → パース失敗  
4. 16 小節 `cat` を毎回書く → ライブ向きでない（短いループにする）  

## Checklist

- [ ] `setcpm` がある  
- [ ] 各トラックが `$:` で始まる  
- [ ] 2–5 本程度で短い  
- [ ] ドラムが統合記法になっている  
- [ ] `strudel_save_song(name, content, deck?)` を実行した  
