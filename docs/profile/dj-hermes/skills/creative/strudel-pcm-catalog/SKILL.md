---
name: strudel-pcm-catalog
description: >-
  Use when choosing a rust-fm-synthe PCM one-shot for strudel-rs
  (bd:8b, hh:cl, bs:ht, and other part:slug keys). Not for live 2-op .fm.
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, samples, pcm, rust-fm-synthe]
    related_skills:
      - strudel-sound-design
      - strudel-composition
      - strudel-data-format
      - strudel-genre-house
      - strudel-genre-dnb
---

# strudel-rs PCM catalog（rust-fm-synthe）

## Overview

`samples/<part>/<slug>.wav` を `s("<part>:<slug>")` で鳴らす。slug は **2〜3 字**。意味（name / description）は **[INDEX.md](./INDEX.md)** が正本。曲を書く前に INDEX で slug を確認する。

同梱デフォルト（Sonic Pi）は残してある。`s("bd")` は今までどおり `samples/bd/00.wav`。

## When to Use

- キック／スネア／ハット／ベース／プラック／EP／短い FX を **PCM のキャラ付き**で選びたい時
- `s("bd*4")` のまま音色だけ変えたい時（`bd:hf` など）

Don't use for: ライブ 2-op `.fm`（→ strudel-sound-design）、記法そのもの（→ composition）。長尺 `ld:` / `dr:` / `pf:` / `ps:` は INDEX のほとんどが `in_bank=no`。例外の同梱は `ld:ss` と `pf:ff`。

## 呼び出し

| 形 | 意味 |
| --- | --- |
| `s("bd")` | part の n=0（同梱 `00.wav`） |
| `s("bd:8b")` | `samples/bd/8b.wav` |
| `s("bd:1")` | 整数 → `.n(1)` と同じ（ソート順） |
| `s("bd").n(1)` | トラック全体の変種。atom の `:` の方が優先 |
| `note("0 2").scale("C4:minor").s("bs:ht")` | 音程。`SAMPLE_ROOT_HZ` は C4 |
| `s("<fx:up ~ ~ ~>")` | 長い FX は 4 小節に 1 回 |

`kit:bd` は今も不可（bank 名が左に来る書き方）。キット切替は `.bank("tr808-hard")`。FM カタログとは混ぜない。

```
setcpm(124/4)
$: s("bd:hf*4, [~ sd:8s]*2, [~ hh:cl]*4").gain(0.6)
$: note("0 0 4 0").scale("C4:minor").s("bs:ht").gain(0.45)
$: s("<fx:up ~ ~ ~>").gain(0.3)
```

Copy: `songs/skill-pcm-catalog.strudel`.

## 役割 → まずこれを試す

詳細は INDEX。ここは常駐セットの早見。

### キック `bd:`

| slug | 向き |
| --- | --- |
| `hf` | ハウス 4つ打ち |
| `8b` | 808 ブーム／サブ寄り |
| `8d` | 歪み 808 |
| `9p` | 909 パンチ |
| `tc` | テクノの重い胴 |
| `dn` | DnB タイト |
| `gb` / `fc` / `hs` | ガバ／フレンチコア／ハードスタイル |
| `lf` | ローファイ |

### スネア `sd:`

| slug | 向き |
| --- | --- |
| `8s` | 808 |
| `9s` | 909 |
| `hd` | ハウス／ディスコ |
| `dn` | DnB |
| `rm` | リム |

### ハット

| call | 向き |
| --- | --- |
| `hh:cl` | クローズ定番 |
| `hh:hs` | ハウス 16 分 |
| `hh:tt` | 極短い |
| `oh:op` | オープン |

### クラップ `cp:`

`s("cp")` が同梱ハウス 2/4。追加は `cp:dr` / `cp:gt` / `cp:rm`。テクノキック前には載せない。

### ベース `bs:`（`C4:…` で native）

| slug | 向き |
| --- | --- |
| `hf` | 同梱ハウスフロア（キックの下）。`ht` はカタログの別テイク |
| `ht` | カタログのタイトハウス |
| `su` | 同梱サインサブ。他のサブと重ねない |
| `8s` | 808 サブ |
| `ac` | 303 寄り |
| `dk` | 同梱ダーク Reese（サブあり）。`rd` はカタログ別ハッシュ |
| `rd` | カタログの暗い Reese（サブあり。他のサブと重ねない） |
| `rm` | ミッド Reese 糊（サブなし。別途サブが要る） |
| `wb` | ウォブル |

### プラック `plk:` / EP `ep:`（`C4:…`）

| call | 向き |
| --- | --- |
| `plk:lp` | 同梱 FM プラック |
| `plk:s5` / `plk:s3` | 中空5度 / 長三和音スタブ |
| `plk:hd` / `plk:hb` | ハウス |
| `plk:ny` | ナイロンポップ |
| `plk:dt` | DnB タイト |
| `ep:ky` | 同梱 EP ワンショット（ライブ lead ではない） |
| `ep:rs` | 柔らかい Rhodes |
| `pf:ff` | 同梱 fifth pad（既定で in_bank） |
| `ld:ss` | 同梱 supersaw（既定で in_bank） |

### FX `fx:`（`note()` なし）

| slug | 向き |
| --- | --- |
| `up` | アップリフター（長い → `<>`） |
| `nr` | ノイズライザー |
| `id` | DnB インパクト |
| `sd` | サブドロップ |

長尺 `ld:` / `dr:` / `pf:` / `ps:` の残りは INDEX の `in_bank=no`（`samples/` にファイルが無い）。無いキーは無音（演奏は継続）。 `ld:ss` と `pf:ff` は同梱済み。

## Rules

1. 新規 PCM は `part:slug`。フルネームでリズムを埋めない
2. ドラムは `note()` なし。音程は `C4:…`。FX は bare `s("…")`
3. 長いワンショットを毎小節撃たない
4. `.bank` と `part:slug` を同じ `$:` で混ぜない
5. `in_bank=no` を content に書かない
6. slug の意味が分からなければ INDEX を読む（2〜3 字だけでは足りない）

## Do not

- `bd:808-boom` のような長い slug（ディスクは `8b.wav`）
- 純数字 slug を新しく足す（`bd:2` は index）
- 同梱 `bd/00.wav` を上書きする
- `stack()` / `.cpm()` / ライブ `.fm` で 4-op プリセットを再現しようとする
