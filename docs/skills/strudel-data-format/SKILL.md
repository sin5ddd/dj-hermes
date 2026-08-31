---
name: strudel-data-format
description: "Use when writing short .strudel live-loop files for strudel-rs apply/save (MCP)."
version: 4.1.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, file-format, metadata, live-coding]
    related_skills:
      - strudel-composition
      - strudel-sound-design
---

# strudel-rs 曲ファイル形式

## Overview

strudel-rs は `.strudel` テキストをパースして再生する。鳴らすのは MCP **`strudel_apply_song`**（無書き込み）。ディスク保存は **`strudel_save_song`** のみ（file ツール不可、明示指示までしない）。書き込み先は `~/.config/strudel-rs/songs/<name>.strudel` のみ。

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

## strudel_apply_song

| 引数 | 意味 |
| --- | --- |
| `content` | 上の全文 |
| `deck` | `A` / `B` — 次小節でロード。ディスクに書かない |

## strudel_save_song

| 引数 | 意味 |
| --- | --- |
| `name` | ベース名のみ（例 `visitor-house`）。パス禁止 |
| `content` | 任意。省略時は `deck` の現行 source |
| `deck` | 任意 `A` / `B` — content 省略時のスナップショット元。ロードしない |
| `overwrite` | 既定 true |

**ライブ時**: 鳴らすのは apply。同じファイルへ残すのは来場者が残してと言ったときだけ。save は演奏を変えない。保存済みを鳴らすのは `strudel_load_song(path=<basename>, deck)`（`songs/` プレフィックス無し。ユーザーライブラリ → 同梱 `songs/` の順）。

## strudel_load_song

| 引数 | 意味 |
| --- | --- |
| `path` | ベース名のみ（例 `visitor-house`）。`songs/` を付けない |
| `deck` | `A` / `B` — 次小節でロード |

## 禁止（apply / save すると 400 または再生失敗）

- `stack(...)` / `).cpm(...)` / 裸の `s("bd")` 行（`$:` 無し）  
- 未実装: `.lfo(...)`（メソッド名）、`vib("<…>")`、mini の `bd(3,8)`（`(` は unexpected char）
- 可: `.scale("<…>")` 進行、`.lpf("<400 1200>")` / `.lpf(sine.rangex(500,4000))` / `.lpf(sine.range(200,2000).slow(4))`、`.add` / `.sub`（スカラーまたは mini。LFO は不可） / `.ply`（整数スカラー） / `.pan(0)`
- `cp` は同梱（`samples/cp/00.wav`）。ハウス 2/4 用。テクノキック前には載せない  

## Pitfalls

1. チャットにコードを書いて終わり → 必ず `strudel_apply_song`
2. `name` に日本語や `/` → ASCII の basename のみ  
3. content に `stack` を入れる → パース失敗  
4. 16 小節 `cat` を毎回書く → ライブ向きでない（短いループにする）  

## Checklist

- [ ] `setcpm` がある  
- [ ] 各トラックが `$:` で始まる  
- [ ] 2–5 本程度で短い  
- [ ] ドラムが統合記法になっている  
- [ ] `strudel_apply_song(content, deck)` を実行した。save は「残して」と言われたときだけ
