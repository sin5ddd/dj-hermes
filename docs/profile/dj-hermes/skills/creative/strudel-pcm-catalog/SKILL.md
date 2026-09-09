---
name: strudel-pcm-catalog
description: >-
  Use when choosing a rust-fm-synthe PCM one-shot for dj-hermes
  (bd:8b, hh:cl, bs:ht, vc:pa, and other part:slug keys). Not for live 2-op .fm.
version: 1.3.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [dj-hermes, music, samples, pcm, rust-fm-synthe]
    related_skills:
      - strudel-sound-design
      - strudel-composition
      - strudel-data-format
      - strudel-genre-house
      - strudel-genre-dnb
---

# dj-hermes PCM catalog（rust-fm-synthe）

## Overview

`samples/<part>/<slug>.wav` を `s("<part>:<slug>")` で鳴らす。slug は **2〜3 字**（**`vc:yeah` だけ 4 字**）。意味（name / description）は **[INDEX.md](./INDEX.md)** が正本。曲を書く前に INDEX で slug を確認する。

同梱デフォルト（Sonic Pi）は残してある。`s("bd")` は今までどおり `samples/bd/00.wav`。

## When to Use

- キック／スネア／ハット／ベース／プラック／EP／短い FX を **PCM のキャラ付き**で選びたい時
- ボーカルチョップ（`vc:pa` / `vc:na` / `vc:ra` / `vc:tu` / `vc:ya` / `vc:yeah`）をグリッドに置きたい時
- 長尺のドローン／リード／パッド（`dr:` / `ld:` / `pf:` / `ps:`）をワンショットで置きたい時
- `s("bd*4")` のまま音色だけ変えたい時（`bd:hf` など）

Don't use for: ライブ 2-op `.fm`（→ strudel-sound-design）、記法そのもの（→ composition）。長尺は毎小節撃たない（`dr` / `pf` / `ps` は約 16–17 秒、ライザー `fx:fr` / `nr` / `rf` / `rp` / `rw` / `up` は約 15 秒、`ld` と `plk:fp` / `plk:sp` は約 8.2 秒、`vc:` は約 3.2 秒なので 16 分連打は `.cut(1)`）。

## 呼び出し

| 形 | 意味 |
| --- | --- |
| `s("bd")` | part の n=0（同梱 `00.wav`） |
| `s("bd:8b")` | `samples/bd/8b.wav` |
| `s("bd:1")` | 整数 → `.n(1)` と同じ（ソート順） |
| `s("bd").n(1)` | トラック全体の変種。atom の `:` の方が優先 |
| `note("0 2").scale("C4:minor").s("bs:ht")` | 音程。`SAMPLE_ROOT_HZ` は C4 |
| `s("<fx:fr ~ ~ ~ ~ ~ ~ ~>")` | ライザー（`fr` / `nr` / `rf` / `rp` / `rw` / `up`）は 8 小節に 1 回（約 15 秒） |

`kit:bd` は今も不可（bank 名が左に来る書き方）。キット切替は `.bank("tr808-hard")`。FM カタログとは混ぜない。

```
setcpm(124/4)
$: s("bd:hf*4, [~ sd:8s]*2, [~ hh:cl]*4").gain(0.6)
$: note("0 0 4 0").scale("C4:minor").s("bs:ht").gain(0.45)
$: s("<fx:up ~ ~ ~ ~ ~ ~ ~>").gain(0.3)
```

Apply the inline recipe with `dj_hermes_apply_song`. `songs/house/01.strudel` uses `bd:hf` / `bs:su` / `plk:lp` as a live example.

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

### FX `fx:`（`note()` なし）

| slug | 向き |
| --- | --- |
| `up` | アップリフター（約15秒、8小節。`<>` で間引く） |
| `fr` | FMライザー（約15秒、8小節。`<>` で間引く） |
| `nr` | ノイズライザー（約15秒、8小節） |
| `rf` | フィルタライザー（約15秒、8小節） |
| `rp` | ピッチライザー（約15秒、8小節） |
| `rw` | スーパーソーライザー（約15秒、8小節） |
| `id` | DnB インパクト |
| `sd` | サブドロップ |

### ドローン `dr:` / リード `ld:` / パッド `pf:` `ps:`

約 8–17 秒。`s("<dr:ss ~ ~ ~>")` のように `<>` で間引く。詳細は INDEX。

| call | 向き |
| --- | --- |
| `dr:ss` | 正弦サブの床 |
| `dr:fh` | 中空5度の低ドローン |
| `ld:ss` | スーパーソー |
| `ld:hf` | 中空5度リード |
| `pf:ff` | fifth pad（約 8.2 秒） |
| `pf:fo` | 開いた5度（約 16 秒） |
| `ps:hp` | 奇数倍音のキラキラ |

無いキーは無音（演奏は継続）。`plk:fp` / `plk:sp` も約 8.2 秒なので毎小節撃たない。

### ボーカルチョップ `vc:`（C4、約 3.2 秒）

| call | 向き |
| --- | --- |
| `vc:pa` | 破裂の pa。グリッドの頭 |
| `vc:na` | 弱い「な」。柔らかい粒 |
| `vc:ra` | 巻き舌のら→あー |
| `vc:tu` | 破裂の tu。pa より暗い |
| `vc:ya` | や行の滑り。フック |
| `vc:yeah` | yeah（i→e、1 Hz トレモロ） |

リズムは `s("vc:pa vc:na vc:tu")`。移調は `note("0").scale("C4:minor").s("vc:pa")`。連打は `.cut(1)`。録音は C4 なので `C4:` が native C4（C3 録音の `plk:` とは聞こえるオクターブが違う）。全ジャンルで使ってよい。スロットと本数は **strudel-composition** のボーカルチョップ。ジャンルの芯（303 / Reese / `[~ cp]*2`）は奪わない。`vc:yeah` はドロップの掛け声で毎小節撃たない。

## Rules

1. 新規 PCM は `part:slug`。フルネームでリズムを埋めない
2. ドラムは `note()` なし。音程 PCM は `C4:…`。FX は bare `s("…")`。`vc:` はリズムなら bare、移調するなら `note()` + `C4:`
3. 長いワンショットを毎小節撃たない
4. `.bank` と `part:slug` を同じ `$:` で混ぜない
5. `in_bank=no`（未作成）を content に書かない
6. slug の意味が分からなければ INDEX を読む（2〜3 字だけでは足りない）

## Do not

- `bd:808-boom` のような長い slug（ディスクは `8b.wav`）。`vc:yeah` 以外の 4 字 slug を invent する
- 純数字 slug を新しく足す（`bd:2` は index）
- 同梱 `bd/00.wav` を上書きする
- `stack()` / `.cpm()` / ライブ `.fm` で 4-op プリセットを再現しようとする
