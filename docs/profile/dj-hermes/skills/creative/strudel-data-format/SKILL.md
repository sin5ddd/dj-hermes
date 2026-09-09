---
name: strudel-data-format
description: "Use when writing .strudel files for dj-hermes apply/save (MCP): 7–8 $: tracks, 4-bar phrases."
version: 5.0.2
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [dj-hermes, music, file-format, metadata, live-coding]
    related_skills:
      - strudel-composition
      - strudel-sound-design
---

# dj-hermes 曲ファイル形式

## Overview

dj-hermes は `.strudel` テキストをパースして再生する。鳴らすのは MCP **`dj_hermes_apply_song`**（無書き込み）。ディスク保存は **`dj_hermes_save_song`** のみ（file ツール不可、明示指示までしない）。書き込み先は `~/.config/dj-hermes/songs/<name>.strudel` のみ。

**曲の長さの目安**: トラック **7–8 本**、4 小節フレーズ（`.scale("<…>")` / 4 子の `<>`）。16 小節 `cat` は非既定。本数・長さのジャンル例外（Future Bass / Kawaii 9 本・16 小節、Ambient 16 本・16 小節、Minimal 14–16 本）は **strudel-composition** / 各 **strudel-genre-***。

## 必須の形（コピー用・8 スロット）

スロット名と正本テンプレは **strudel-composition**。形だけ:

```
// @title My Song
// @by booth
// @genre house
setcpm(124/4)
// drums
$: s("bd*4, [~ cp]*2, [~ hh]*4, <~ ~ ~ [~@3 bd ~@4]>").gain(0.55)
// bass
$: note("0 0 4 <0 2 4 0>").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("bs:hf").gain(0.42)
// lead
$: note("~ 7 6 <4 9 3 7>").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("ld:ss").gain(0.16).cut(1)
// hook
$: note("4 ~ 7 4  2 0 ~ -1").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("plk:lp").gain(0.22).cut(1)
// arp
$: note("0 4 7 4").scale("<C5:minor C5:minor G5:dorian C5:minor>")
  .s("plk:hd").gain(0.14).cut(1)
// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("ep:ky").gain(0.26)
// pad
$: note("0").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("pf:ff").gain(0.16).room(0.3).orbit(2)
// perc
$: s("<~ ~ ~ perc:cm>").gain(0.18)
```

1. 任意: `// @title` / `@by` / `@genre` などのコメントタグ  
2. **必須**: `setcpm(N)` または `setcpm(BPM/4)`（1 cycle = 1 bar = 4 beats → エンジン BPM = N×4）。`setcps(x)` も可  
3. **必須**: 1 本以上の **`$:` 行**（トラック）。直前の `// name` がトラック名  
4. ドラムは原則 **1 本の `s(...)`**。新規は **7–8 本**（ジャンル例外は strudel-composition）  

## dj_hermes_apply_song

| 引数 | 意味 |
| --- | --- |
| `content` | 上の全文 |
| `deck` | `A` / `B` — 次小節でロード。ディスクに書かない |

## dj_hermes_save_song

| 引数 | 意味 |
| --- | --- |
| `name` | ベース名のみ（例 `visitor-house`）。パス禁止 |
| `content` | 任意。省略時は `deck` の現行 source |
| `deck` | 任意 `A` / `B` — content 省略時のスナップショット元。ロードしない |
| `overwrite` | 既定 true |

**ライブ時**: 鳴らすのは apply。同じファイルへ残すのは来場者が残してと言ったときだけ。save は演奏を変えない。保存済みを鳴らすのは `dj_hermes_load_song(path=<basename>, deck)`（`songs/` プレフィックス無し。ユーザーライブラリ → 同梱 `songs/` の順）。

## dj_hermes_load_song

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

1. チャットにコードを書いて終わり → 必ず `dj_hermes_apply_song`
2. `name` に日本語や `/` → ASCII の basename のみ  
3. content に `stack` を入れる → パース失敗  
4. 16 小節 `cat` を毎回書く → 4 小節は `.scale("<…>")` と `<>`。`cat` は最大 8 引数  

## Checklist

- [ ] `setcpm` がある  
- [ ] 各トラックが `$:` で始まる  
- [ ] 7–8 本（drums / bass 1–2 / lead / hook / arp / chords / pad。8 本目は perc または **vox**）。ジャンル例外は composition  
- [ ] 4 小節フレーズ。ドラムは統合記法  
- [ ] `dj_hermes_apply_song(content, deck)` を実行した。save は「残して」と言われたときだけ
