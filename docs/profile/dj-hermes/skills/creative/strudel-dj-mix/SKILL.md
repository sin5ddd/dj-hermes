---
name: strudel-dj-mix
description: "Use when mixing two decks, long mix, cut-in, fill-in, switch/transformer chops, crossfade hold, つなげる, カットイン, フィル, スイッチ, 次の曲へ, エコー, ハイパス, ロール, ビニール, echo, hpf, roll, vinyl."
version: 1.1.2
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [dj-hermes, dj, mixer, xfade, fill, switch]
    related_skills:
      - strudel-composition
      - strudel-dj-hype
      - strudel-live-edit
      - strudel-sound-design
---

# dj-hermes DJ ミックス（1 コマンド）

## Overview

2 デッキのつなぎは **`dj_hermes_mix` を 1 回**。EQ を 4 回呼ばない。曲ソースを `apply_song` / `patch_track` しない。

ドラムの `<>` フィルは **strudel-live-edit**。こちらは Mixer のフィル。  
**フロアを沸かせて / 盛り上げて / ドロップ** は **strudel-dj-hype**（状況で kind と `to` を選ぶ。ライザー → B 固定にしない）。

## 手順

1. **`dj_hermes_status`** — どちらが主電源か、両デッキに曲があるか、`mix` が動いていないか
2. 下の表で `move` / `kind` / `to` を決める
3. **`dj_hermes_mix` を 1 回だけ**呼ぶ
4. 新しい mix は前のジョブをキャンセルする。ライザー中に long を重ねない

`dj_hermes_hush` は呼ばない（オペレータの `/hush`）。

## ルーティング

| 言い方 | 呼び出し |
| --- | --- |
| ロングでつないで / ゆっくり B へ | `dj_hermes_mix(move="long", to="B")`（既定 8 小節、帯域分け EQ） |
| カットイン / いきなり A | `dj_hermes_mix(move="cut", to="A")`（次の小節、EQ を flat に戻す） |
| フェーダーを途中で止めて | `dj_hermes_mix(move="hold")`（即時。pos は動かさない） |
| ディレイのフィルから B | `dj_hermes_mix(move="fill", kind="delay", to="B")` |
| ローパスで絞ってカット | `dj_hermes_mix(move="fill", kind="lpf", to="B")` |
| 点滅してカット | `dj_hermes_mix(move="fill", kind="flash", to="B")` |
| ライザー入れてカット | `dj_hermes_mix(move="fill", kind="riser", to="B")`（既定 4 小節。`mixes/riser.strudel`） |
| B から 8 分でスイッチして A | `dj_hermes_mix(move="fill", kind="switch", to="A", grid="8n")` |
| エコーで消して B へ | `dj_hermes_mix(move="fill", kind="echo", to="B")` |
| ハイパスで薄くしてカット | `dj_hermes_mix(move="fill", kind="hpf", to="B")` |
| ロールしてから A | `dj_hermes_mix(move="fill", kind="roll", to="A", grid="8n")` |
| タイムリピート（四分/八分/16分/32分音符、1 小節で Off） | `dj_hermes_mixer_repeat(div="16n")`（相対再生位置。fill roll の PCM カットインではない） |
| テープストップ（全音符 1 小節） | `dj_hermes_mixer_tape(on=true, len="1n")`（即時。途中で止めるなら `on=false`） |
| 四分音符のテープを 2 連 | `dj_hermes_mixer_tape(on=true, len="4n", reps=2)` |
| 速い曲をテープで落として遅い曲へ | 主電源で `mixer_tape(on=true, len="1n")` → 途中で `on=false` → `dj_hermes_mix(move="cut", to=着地)`。BPM は共有のまま |
| インパクト入れてカット | `dj_hermes_mix(move="fill", kind="drop", to="B")` |
| ビニール（かすれ＋音程揺れ）してカット | `dj_hermes_mix(move="fill", kind="vinyl", to="B")`（既定 8 小節。バンドパスのかすれと wow 振幅がジョブ進行で大きくなり、カット） |
| カウントしてから B | `dj_hermes_mix(move="fill", kind="count", to="B")`（次バーから「いくよー」、次のバーで 4 分のいちにさんし。既定 2 バー後にカット。`mixes/count.strudel`） |
| 4 分でスイッチ | `grid="4n"` |

`to` は **着地先**。スイッチの最初のマスは着地の反対（B から始めて A へ）。

flash は outgoing だけ消す。switch は AB を 100:0 ↔ 0:100 で交互。取り違えない。

任意: `bars`（long 既定 8、fill は `mixes/<kind>.strudel` の `@bars`、無ければ 1、riser 4、vinyl 8、count 2）、`phrase` 1/4/8、`eq=false`（long で EQ しない）、`reset_eq=false`、`mute_track`（outgoing のトラック名）。

fill の kind は **`mixes/<kind>.strudel`**。DSP（delay / lpf / …）はファイルの `// @dsp`。`$:` は Mixer 専用レーン（A/B と同じ mini、TUI には出ない）。掛け声を足すときはファイルを足す。新しい DSP アルゴリズムだけ Rust の `FillKind` が要る。曲ソースへ `$:` を `apply_song` / `patch_track` しない。

## やってはいけないこと

- `dj_hermes_mixer_eq` を 4 回積んでロングを再現する
- ミックスのために `dj_hermes_apply_song` / `dj_hermes_save_song` / `dj_hermes_patch_track`
- ロールのために `apply_song` で同じヒットを並べない
- 174 DnB と 124 house をつなぐ（BPM は共有。creative README と同じ）
- 片デッキに曲がないのに long / switch

両デッキに曲が無いときは先に `dj_hermes_load_song`。フレーズ頭から出したいときは `dj_hermes_head` してから mix。
