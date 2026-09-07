---
name: strudel-genre-minimal-techno
description: >-
  Use when writing Minimal Techno for strudel-rs: 126 BPM, 8 sparse
  tracks, 16-bar mute map (kick stays; others rest in sections), offbeat
  open hats `[~ oh]*4`, dark FX one-shots. Not a house [~ cp]*2 backbeat;
  not on-beat `hh*8`; not all loops on at once.
version: 7.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, minimal-techno]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-pcm-catalog
---

# strudel-rs × ミニマルテクノ

## Overview

要素は反復、隙間は多い、gain は低め。この Skill のテンポは **126 BPM**（目安 124–130）。トラック数を削るのではなく、**8 本のまま `~` を残す**。

ミニマルテクノの展開は新しいメロではない。**同じ短いループを、16 小節のあいだ出し入れする**。キックは常時。他はセクションで休符。華やかさはダークな FX ワンショット（リバースシンバル、ラジオスタブ、スイープ、サブドロップ）。明るいリードやガバのヒットではない。

和声は 4 小節 `.scale` のまま。16 小節はミュートマップ（`<>` 16 子）。`cat` ではない。このエンジンの `<>` は `@` で小節数を伸ばさない。

## When

- 依頼が **ミニマルテクノ**（4 つ打ち、隙間、ループのオンオフ）のとき
- ハットはキックの裏のオープン（`[~ oh]*4`）。表拍の `hh*8` / `hh*4` ではないとき
- 音を足しすぎず、ハウスの 2/4 クラップでもないとき
- 4 小節に 1 回の `cp` は色付けで、バックビートではないとき
- 全トラックを 16 小節ずっと鳴らしたままにしないとき

## ミュートマップ（16 小節）

1 サイクル＝1 小節。オンの小節はフェンスの 1 小節ループ。オフは `~`。キックは 16 小節とも `bd*4`。

| ブロック | 1–4 strip | 5–8 groove | 9–12 body | 13–16 turn |
| --- | --- | --- | --- | --- |
| kick | on | on | on | on |
| hats | on | on | on | 13–14 off、15–16 on |
| bass | off | on | on | on |
| lead | off | off | on | off |
| hook | off | on | on | 13–14 off、15–16 on |
| arp | on（4 小節に 2 クリック） | on | on | 13–14 off、16 に 1 クリック |
| chords | off | off | on | 13–14 on、15–16 off |
| pad | off | off | 9 だけ | off |
| fx | 4 ラジオ | 7 暗いリバースシンバル | 9 クラング | 13 サブドロップ、16 スイープ |

`strudel_mute` はライブで一時的に落とす用。曲のフォームにはしない。

## Pattern

```
// @title visitor-minimal
// @genre minimal-techno
setcpm(126/4)
// drums
$: s("bd*4, <[~ oh]*4 [~ oh]*4 [~ oh]*4 [~ oh]*4 [~ oh]*4 [~ oh]*4 [~ oh]*4 [~ oh]*4 [~ oh]*4 [~ oh]*4 [~ oh]*4 [~ oh]*4 ~ ~ [~ oh]*4 [~ oh]*4>, <~ ~ ~ cp ~ ~ ~ cp ~ ~ ~ cp ~ ~ ~ cp>").gain(0.7)
// bass
$: note("<~ ~ ~ ~ [0 ~ 3 ~] [0 ~ 3 ~] [0 ~ 3 ~] [0 ~ 3 ~] [0 ~ 3 ~] [0 ~ 3 ~] [0 ~ 3 ~] [0 ~ 3 ~] [0 ~ 3 ~] [0 ~ 3 ~] [0 ~ 3 ~] [0 ~ 3 ~]>").scale("<C2:minor C2:minor C2:minor G2:phrygian>")
  .s("sawtooth").lpf(320).gain(0.4)
// lead
$: note("<~ ~ ~ ~ ~ ~ ~ ~ [~ ~ 7 ~] [~ ~ 7 ~] [~ ~ 7 ~] [~ ~ 7 ~] ~ ~ ~ ~>").scale("<C4:minor C4:minor C4:minor G4:phrygian>")
  .s("plk:pk").gain(0.1).cut(1)
// hook
$: note("<~ ~ ~ ~ [~ 4 ~ ~] [~ 4 ~ ~] [~ 4 ~ ~] [~ 4 ~ ~] [~ 4 ~ ~] [~ 4 ~ ~] [~ 4 ~ ~] [~ 4 ~ ~] ~ ~ [~ 4 ~ ~] [~ 4 ~ ~]>").scale("<C4:minor C4:minor C4:minor G4:phrygian>")
  .s("plk:ac").gain(0.12).cut(1)
// arp
$: s("<~ perc:tk ~ perc:st ~ perc:tk ~ perc:st ~ perc:tk ~ perc:st ~ ~ ~ perc:st>").gain(0.12)
// chords
$: note("<~ ~ ~ ~ ~ ~ ~ ~ [~ [0,2,4] ~ ~] [~ [0,2,4] ~ ~] [~ [0,2,4] ~ ~] [~ [0,2,4] ~ ~] [~ [0,2,4] ~ ~] [~ [0,2,4] ~ ~] ~ ~>").scale("<C4:minor C4:minor C4:minor G4:phrygian>")
  .s("ep:mt").gain(0.14)
// pad
$: note("<~ ~ ~ ~ ~ ~ ~ ~ 0 ~ ~ ~ ~ ~ ~ ~>").scale("<C4:minor C4:minor C4:minor G4:phrygian>")
  .s("pf:ff").orbit(2).cut(1).gain(0.1).room(0.25)
// fx
$: s("<~ ~ ~ fx:rd ~ ~ fx:rk ~ fx:cg ~ ~ ~ fx:sd ~ ~ fx:sw>").gain(0.18).cut(1)
```

オンの 1 小節は `[0 ~ 3 ~]` のようにブラケットで 1 子にする。`<>` の空白は子の区切りなので、ブラケット無しの `0 ~ 3 ~` は 4 小節になってしまう。

同梱 `songs/minimal-techno/01.strudel` はこのフェンスと同じ（16 小節ミュートとグリッド）。新規 apply の `.s()` は下のパレットから選ぶ。

## 音色パレット（新規 apply はここから選ぶ）

Pattern はグリッド・次数・ミュートマップの見本。新規曲は下表からスロットごとに 1 つ選び、このフェンスの `.s()` を毎回コピーしない。同一曲の pitched 2 本に同じ `.s()` を使わない。slug の意味は strudel-pcm-catalog の INDEX。長尺（`ld:` / `dr:` / `pf:` / `ps:`、`plk:fp` / `plk:sp`、`fx:rk` / `fx:rl` / `fx:ry`）は `.cut(1)` か 16 子の休符。隙間と低 gain は残す。

126 BPM の 1 小節は約 1.9 秒。`fx:rk` は 3.0 秒、`fx:rl` は 3.8 秒なので、撃った小節の次も余韻が残る。連続した小節に長い FX を置かない。

| スロット | 芯 | 代替 | 禁止 |
| --- | --- | --- | --- |
| drums | `bd*4` + 16 子の裏拍オープン `[~ oh]*4`、4 小節に 1 `cp` | `bd:tc`、`oh:op` / `oh:dn` | `[~ cp]*2`、`bd:gb`、`hh*8` / `hh*4`（表拍）、ハットを 16 小節常時 |
| bass | 低い短いノート。1–4 はオフ | `sawtooth`+`lpf(320)`、`bs:ht` at `C4:` | `bs:su` 重ね、wobble |
| lead | 休符多め。9–12 だけ | `plk:pk`、`plk:ac` | `ld:ss` アンセム、`ld:an`、16 小節常時 |
| hook | 休符多め。strip と 13–14 はオフ | `plk:ac`、`plk:pk` | Rhodes、`plk:mx` |
| arp | perc クリック。13–14 はオフ | `perc:tk`、`perc:st` | 16 分埋め |
| chords | 疎な `[0,2,4]`。9–14 だけ | `ep:mt`、`plk:sf` | `triangle`、スーパーソー |
| pad | 薄い。9 だけ | `pf:pu`、`pf:cs`（低 gain）、`pf:ff`+次数 `0` | `ps:gt`、gabber、毎小節 |
| fx | 16 子のダークワンショット | `fx:rd`、`fx:rk`、`fx:ry`、`fx:rl`、`fx:cg`、`fx:mc`、`fx:sd`、`fx:sw`、`fx:dn`、`fx:nh`、`fx:nb`、`fx:wh`、`fx:wd`、`fx:ha`、`fx:ck`、`fx:im` | `fx:gb`、`fx:fc`、`fx:up`、`fx:rb`、`fx:rs`、毎小節、`note()` を付ける |

## Why

キックは毎拍、ハットは裏拍オープン（`[~ oh]*4` = 1& 2& 3& 4&）。`hh*8` は 8 分の表拍も含む。`hh*4` は全部表拍。どちらもこのジャンルの既定ではない。`cp` は 4 小節に 1 回だけ。`[~ cp]*2` にするとハウスのバックビートになる。クローズの裏拍 `[~ hh]*4` は four-on-the-floor。

同じ 1 小節ループを全部同時に回すと、クラブのミニマルではなくなる。16 子の `<>` でハット・ベース・プラック・コードをブロック単位で落とす。キックだけが残る 2 小節（13–14）がターン。

FX は明るいメロの代わり。ラジオスタブと暗いリバースシンバルとサブドロップで、ダークなままイベントを付ける。`fx:gb` / `fx:up` はガバ／フェス側。

PCM ベースは `C4:`（native）。シンセサブは `C2:`。

## レシピ

1. キックは 4 つ打ち。ハットは裏拍オープン `[~ oh]*4`。スネアの 2/4 は置かない
2. メロとコードは `~` を残す。gain は低め（リード 0.1 前後）
3. 16 小節ミュート（上表）。和声の変化は 4 小節目の G phrygian と `cp`
4. arp は毎小節撃たない perc（`perc:tk` / `perc:st`）
5. 8 本目は `// fx`。長い FX はセクション境界だけ
6. 本数は 8。3–5 本に削らない。全トラック常時オンにもしない

鳴らすのは `strudel_apply_song(content, deck)`（次小節、無書き込み）。`strudel_save_song` は残す指示のときだけ（演奏は変えない）。

## Variations（同じ文法）

| 目的 | 変更 |
| --- | --- |
| ハットを粗く | `[~ oh]*4` を `[~ oh ~ ~]*2`（1& と 3& だけ。表拍の `hh*4` にはしない） |
| ターンを長く | ハット 13–16 を全部 `~` |
| リバースを長く | 7 小節目を `fx:rl`（3.8 秒。8 小節は休符のまま） |
| ラジオをノイズに | 4 小節目を `fx:nh` / `fx:ck` |
| クラングをエアに | 9 小節目を `fx:ha` |
| ベースを早く出す | 1–4 の `~` を `[0 ~ 3 ~]` にする（キック以外を全部オンにはしない） |

## Pitfalls

1. `stack()` / `.cpm()` / `.lfo()` → apply / save とも 400
2. ドラムを kick / hat / perc の 3 `$:` に分ける
3. 長い PCM（`ld:` / `dr:` / `pf:` / `ps:`、`fx:rk` / `fx:rl`）を毎小節撃たない
4. `[~ cp]*2` を書く（4 小節に 1 回の `cp` とは別物）
5. レイヤー過多やメロの埋めすぎで隙間が消える
6. 新規 apply でフェンスの `.s()` を全コピーする。スーパーソーや Rhodes を載せる
7. 7 本を 16 小節ずっとオンにする（展開が無くなる）
8. `strudel_mute` やミキサー mute を曲のフォームにする
9. `fx:gb` / `fx:fc` / `fx:up` / `fx:rb` など明るい／ガバの FX
10. 16 引数の `cat`。`<~@4 [0 ~ 3 ~]@12>` のように `@` でミュート小節を稼ぐ（このエンジンではサイクル選択の長さにならない）
11. `<>` の中で 1 小節ループをブラケット無しにする（空白が 16 子を壊す）
12. `hh*8` や `hh*4` でハットを表拍に置く（既定は裏拍オープン `[~ oh]*4`）

## Checklist

- [ ] **8 本**（// drums // bass // lead // hook // arp // chords // pad // fx）
- [ ] キック常時。他は 16 子のミュートマップ。和声は 4 小節 `.scale`
- [ ] ドラムは 1 本のカンマ層。ハットは裏拍オープン `[~ oh]*4`。`[~ cp]*2` も `hh*8` / `hh*4` も書いていない
- [ ] 隙間と低 gain が残っている。`.s()` は音色パレット。FX はダークワンショット
- [ ] 長い FX は連続小節に置いていない。`note()` は FX に付けていない
- [ ] `strudel_apply_song(content, deck)`（save は残す指示のときだけ）
