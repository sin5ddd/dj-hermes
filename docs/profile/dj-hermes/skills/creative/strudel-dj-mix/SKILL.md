---
name: strudel-dj-mix
description: "Use when mixing two decks, long mix, cut-in, fill-in, switch/transformer chops, crossfade hold, つなげる, カットイン, フィル, スイッチ, 次の曲へ."
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, dj, mixer, xfade, fill, switch]
    related_skills:
      - strudel-composition
      - strudel-live-edit
      - strudel-sound-design
---

# strudel-rs DJ ミックス（1 コマンド）

## Overview

2 デッキのつなぎは **`strudel_mix` を 1 回**。EQ を 4 回呼ばない。曲ソースを `apply_song` / `patch_track` しない。

ドラムの `<>` フィルは **strudel-live-edit**。こちらは Mixer のフィル。

## 手順

1. **`strudel_status`** — どちらが主電源か、両デッキに曲があるか、`mix` が動いていないか
2. 下の表で `move` / `kind` / `to` を決める
3. **`strudel_mix` を 1 回だけ**呼ぶ
4. 新しい mix は前のジョブをキャンセルする。ライザー中に long を重ねない

`strudel_hush` は呼ばない（オペレータの `/hush`）。

## ルーティング

| 言い方 | 呼び出し |
| --- | --- |
| ロングでつないで / ゆっくり B へ | `strudel_mix(move="long", to="B")`（既定 8 小節、帯域分け EQ） |
| カットイン / いきなり A | `strudel_mix(move="cut", to="A")`（次の小節、EQ を flat に戻す） |
| フェーダーを途中で止めて | `strudel_mix(move="hold")`（即時。pos は動かさない） |
| ディレイのフィルから B | `strudel_mix(move="fill", kind="delay", to="B")` |
| ローパスで絞ってカット | `strudel_mix(move="fill", kind="lpf", to="B")` |
| 点滅してカット | `strudel_mix(move="fill", kind="flash", to="B")` |
| ライザー入れてカット | `strudel_mix(move="fill", kind="riser", to="B")` |
| B から 8 分でスイッチして A | `strudel_mix(move="fill", kind="switch", to="A", grid="8n")` |
| 4 分でスイッチ | `grid="4n"` |

`to` は **着地先**。スイッチの最初のマスは着地の反対（B から始めて A へ）。

flash は outgoing だけ消す。switch は AB を 100:0 ↔ 0:100 で交互。取り違えない。

任意: `bars`（long 既定 8、fill 既定 1、riser 2）、`phrase` 1/4/8、`eq=false`（long で EQ しない）、`reset_eq=false`、`mute_track`（outgoing のトラック名）。

## やってはいけないこと

- `strudel_mixer_eq` を 4 回積んでロングを再現する
- ミックスのために `strudel_apply_song` / `strudel_save_song` / `strudel_patch_track`
- 174 DnB と 124 house をつなぐ（BPM は共有。creative README と同じ）
- 片デッキに曲がないのに long / switch

両デッキに曲が無いときは先に `strudel_load_song`。フレーズ頭から出したいときは `strudel_head` してから mix。
