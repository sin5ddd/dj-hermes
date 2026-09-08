---
name: strudel-genre-minimal
description: >-
  Use when writing Minimal (ミニマル) for dj-hermes: 126 BPM, ~16 PCM
  tracks, four-on-the-floor + offbeat open hat + LPF bass always on,
  dark upper synth as the other lead, 16-bar mute of non-rhythm parts
  only, closed-hat 16ths `[hh hh ~ hh]*4` exclusive with OHH, metallic
  uneasy pluck, note() allowed on perc/tom/metal. Not a house [~ cp]*2
  backbeat; not hh*16; not 8 thin tracks; not muting kick/ohh/bass.
version: 8.2.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [dj-hermes, music, genre, minimal]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-pcm-catalog
---

# dj-hermes × ミニマル

## Overview

ミニマルは **4 つ打ち + 裏拍オープンハット + ベースが常に鳴っている** 音楽。主役は **ローパスで絞ったベース** と **ダークな上モノシンセ**。各パートのメロとビートはほぼ固定。展開はリズム帯以外の **ミュート オン／オフ**（16 小節）。PCM を重ねて **14–16 本**。

クローズハットは展開用。16 分で刻むが、OHH の裏拍 `&` には置かない（`[hh hh ~ hh]*4`）。金属の不安なプラックはメロ担当。perc / tom / metal は **`note()` + `.scale` でピッチしてよい**（このジャンルの例外）。

テンポは **126 BPM**（目安 124–130）。和声は 4 小節 `.scale`。16 小節はミュートマップ（`<>` 16 子）。`cat` ではない。このエンジンの `<>` は `@` で小節数を伸ばさない。

同梱 `songs/minimal/` はこのフェンス（14–16 本）。新規 apply の `.s()` は下のパレットから選ぶ。

## When

- 依頼が **ミニマル** のとき（「ミニマルテクノ」もこちら。フォルダと `@genre` は `minimal`）
- 床はキック・裏拍 OHH・ベース。上モノはダークシンセ。展開は CHH / プラック / その他 PCM の出し入れ
- ハウスの 2/4 クラップ、表拍の `hh*8` / `hh*4`、`hh*16`（OHH と重なる）、8 本に削る、キックやベースをセクションで落とす、明るいリード、ではないとき

## スロット（14–16 本）

コメント名は固定（`dj_hermes_patch_track`）。キック / ohh / chh は **別 `$:`**（CHH だけミュートするため。composition のドラム 1 本原則の例外）。

| # | コメント | 常時 | 役割 |
| --- | --- | --- | --- |
| 1 | `// kick` | 常時 | `bd*4`。ミュートしない |
| 2 | `// ohh` | 常時 | 裏拍オープン `[~ oh]*4`。ミュートしない |
| 3 | `// bass` | 常時 | 1 小節オスティナート + `.lpf`。ミュートしない |
| 4 | `// chh` | ミュート | 16 分 `[hh hh ~ hh]*4`（裏拍 `&` は休符） |
| 5 | `// perc` | ミュート | クリック。**`note()` 可** |
| 6 | `// tom` | ミュート | 低いタム。**`note()` 可** |
| 7 | `// metal` | ミュート | 金属ヒット。**`note()` 可** |
| 8 | `// clap` | ミュート | 4 小節に 1 `cp`（バックビートではない） |
| 9 | `// synth` | ミュート | ダークな上モノ（主役）。次数はほぼ固定 |
| 10 | `// pluck` | ミュート | 金属系の不安なプラックメロ |
| 11 | `// stab` | ミュート | 疎なスタブ（5 度や短いコード） |
| 12 | `// chords` | ミュート | 疎な `[0,2,4]` |
| 13 | `// pad` | ミュート | 薄いダークパッド |
| 14 | `// drone` | ミュート | 長い `dr:`。撃つ小節は少ない |
| 15 | `// texture` | ミュート | ノイズ／エアのワンショット |
| 16 | `// fx` | ミュート | ダークな FX ワンショット |

14 本にするときは `// clap` と `// texture` を落とす。13 本以下にしない。キック・ohh・bass を 1 本の `// drums` に戻さない。

## ミュートマップ（16 小節）

1 サイクル＝1 小節。オンの小節はフェンスの 1 小節ループ。オフは `~`。**kick / ohh / bass は 16 子を書かない**（毎小節オン）。

| ブロック | 1–4 strip | 5–8 groove | 9–12 body | 13–16 turn |
| --- | --- | --- | --- | --- |
| kick / ohh / bass | on | on | on | on |
| chh | off | on | on | 13–14 off、15–16 on |
| perc | on | on | on | 13–14 off、15–16 on |
| tom | off | off | on | off |
| metal | off | on | on | 13–14 off |
| clap | 4 だけ | 8 だけ | 12 だけ | 16 だけ |
| synth | off | on | on | 13–14 off、15–16 on |
| pluck | off | off | on | off |
| stab | off | on | on | 13–14 off、15–16 on |
| chords | off | off | on | 13–14 on、15–16 off |
| pad | off | off | 9 だけ | off |
| drone | off | 5 だけ | 9 だけ | off |
| texture | off | 7 だけ | 11 だけ | off |
| fx | 4 ラジオ | 7 暗いリバースシンバル | 9 クラング | 13 サブドロップ、16 スイープ |

ターン（13–14）は **キック + OHH + ベースだけ**。そこへ CHH やシンセを残さない。`dj_hermes_mute` はライブ用。曲のフォームにはしない。

## ハット（OHH と CHH は排他）

16 分グリッド（1 拍）:

| 16 分 | 1 | e | &（裏拍） | a |
| --- | --- | --- | --- | --- |
| OHH `[~ oh]*4` | | | 攻撃 | （尾） |
| CHH `[hh hh ~ hh]*4` | hh | hh | 休符 | hh |

- 常時 OHH は `[~ oh]*4`（1& 2& 3& 4&）
- 展開 CHH は `[hh hh ~ hh]*4`。**`&` に hh を置かない**
- `hh*16` は `&` で OHH と重なるので禁止
- `hh*8` / `hh*4` は表拍クローズ。このジャンルの既定ではない
- 来場者が「ハット細かく」→ OHH は触らず、`// chh` をオンにする

## `note()`（非メロディ）

このジャンルでは perc / tom / metal（短い金属ヒット）を **`note("…").scale("C4:…").s("perc:tm")` のようにピッチしてよい**。次数オスティナートは 1 小節のまま。キーは pitched と同じ 4 小節 `.scale`。PCM は `C4:` が native。

いまも `note()` を付けないもの: kick / ohh / chh、ライザー・サブドロップ・リバースシンバル（`fx:up` / `fx:sd` / `fx:rk` / `fx:rl` / `fx:ry` など。wav にピッチ動作が入っている）。

## Pattern

キック・ohh・bass は 16 子なし。他は 16 子。オンの 1 小節は `[0 ~ ~ ~]` のように **ブラケットで 1 子**。`<>` の空白は子の区切りなので、ブラケット無しの `0 ~ ~ ~` は 4 小節になる。

```
// @title visitor-minimal
// @genre minimal
setcpm(126/4)
// kick
$: s("bd*4").gain(0.7)
// ohh
$: s("[~ oh]*4").gain(0.28)
// bass
$: note("0 ~ 3 ~").scale("<C2:minor C2:minor C2:minor G2:phrygian>")
  .s("sawtooth").lpf(280).lpq(4).gain(0.45)
// chh
$: s("<~ ~ ~ ~ [hh hh ~ hh]*4 [hh hh ~ hh]*4 [hh hh ~ hh]*4 [hh hh ~ hh]*4 [hh hh ~ hh]*4 [hh hh ~ hh]*4 [hh hh ~ hh]*4 [hh hh ~ hh]*4 ~ ~ [hh hh ~ hh]*4 [hh hh ~ hh]*4>").gain(0.2)
// perc
$: note("<[0 ~ ~ ~] [0 ~ ~ ~] [0 ~ ~ ~] [0 ~ ~ ~] [0 ~ ~ ~] [0 ~ ~ ~] [0 ~ ~ ~] [0 ~ ~ ~] [0 ~ ~ ~] [0 ~ ~ ~] [0 ~ ~ ~] [0 ~ ~ ~] ~ ~ [0 ~ ~ ~] [0 ~ ~ ~]>")
  .scale("<C5:minor C5:minor C5:minor G5:phrygian>").s("perc:tm").gain(0.1)
// tom
$: note("<~ ~ ~ ~ ~ ~ ~ ~ [~ 0 ~ ~] [~ 0 ~ ~] [~ 0 ~ ~] [~ 0 ~ ~] ~ ~ ~ ~>")
  .scale("<C4:minor C4:minor C4:minor G4:phrygian>").s("tom:lo").gain(0.16)
// metal
$: note("<~ ~ ~ ~ [~ ~ 7 ~] [~ ~ 7 ~] [~ ~ 7 ~] [~ ~ 7 ~] [~ ~ 7 ~] [~ ~ 7 ~] [~ ~ 7 ~] [~ ~ 7 ~] ~ ~ ~ ~>")
  .scale("<C4:minor C4:minor C4:minor G4:phrygian>").s("perc:mh").gain(0.08)
// clap
$: s("<~ ~ ~ cp ~ ~ ~ cp ~ ~ ~ cp ~ ~ ~ cp>").gain(0.22)
// synth
$: note("<~ ~ ~ ~ [~ ~ 4 ~] [~ ~ 4 ~] [~ ~ 4 ~] [~ ~ 4 ~] [~ ~ 4 ~] [~ ~ 4 ~] [~ ~ 4 ~] [~ ~ 4 ~] ~ ~ [~ ~ 4 ~] [~ ~ 4 ~]>")
  .scale("<C4:minor C4:minor C4:minor G4:phrygian>")
  .s("ld:in").lpf(900).gain(0.16).cut(1)
// pluck
$: note("<~ ~ ~ ~ ~ ~ ~ ~ [~ 7 ~ 1] [~ 7 ~ 1] [~ 7 ~ 1] [~ 7 ~ 1] ~ ~ ~ ~>")
  .scale("<C4:minor C4:minor C4:minor G4:phrygian>")
  .s("plk:nn").gain(0.12).cut(1)
// stab
$: note("<~ ~ ~ ~ [~ [0,4] ~ ~] [~ [0,4] ~ ~] [~ [0,4] ~ ~] [~ [0,4] ~ ~] [~ [0,4] ~ ~] [~ [0,4] ~ ~] [~ [0,4] ~ ~] [~ [0,4] ~ ~] ~ ~ [~ [0,4] ~ ~] [~ [0,4] ~ ~]>")
  .scale("<C4:minor C4:minor C4:minor G4:phrygian>")
  .s("plk:s5").gain(0.12).cut(1)
// chords
$: note("<~ ~ ~ ~ ~ ~ ~ ~ [~ [0,2,4] ~ ~] [~ [0,2,4] ~ ~] [~ [0,2,4] ~ ~] [~ [0,2,4] ~ ~] [~ [0,2,4] ~ ~] [~ [0,2,4] ~ ~] ~ ~>")
  .scale("<C4:minor C4:minor C4:minor G4:phrygian>")
  .s("plk:sf").gain(0.12)
// pad
$: note("<~ ~ ~ ~ ~ ~ ~ ~ 0 ~ ~ ~ ~ ~ ~ ~>").scale("<C4:minor C4:minor C4:minor G4:phrygian>")
  .s("dr:pd").orbit(2).cut(1).gain(0.1).room(0.25)
// drone
$: note("<~ ~ ~ ~ 0 ~ ~ ~ 0 ~ ~ ~ ~ ~ ~ ~>").scale("<C4:minor C4:minor C4:minor G4:phrygian>")
  .s("dr:mb").orbit(2).cut(1).gain(0.08)
// texture
$: s("<~ ~ ~ ~ ~ ~ fx:ha ~ ~ ~ fx:nh ~ ~ ~ ~ ~>").gain(0.12).cut(1)
// fx
$: s("<~ ~ ~ fx:rd ~ ~ fx:rk ~ fx:cg ~ ~ ~ fx:sd ~ ~ fx:sw>").gain(0.18).cut(1)
```

新規 apply の `.s()` は下のパレットから選ぶ。次数とミュートマップはフェンスを毎回コピーしない（1 小節ループは変えてよいが、16 小節でメロを書き換えるのではない）。

## 音色パレット（新規 apply はここから選ぶ）

Pattern はグリッド・次数・ミュートの見本。スロットごとに 1 つ選ぶ。同一曲の pitched 2 本に同じ `.s()` を使わない。slug は strudel-pcm-catalog の INDEX。長尺（`ld:` / `dr:` / `pf:` / `ps:`、`plk:fp` / `plk:sp`、`fx:rk` / `fx:rl` / `fx:ry`）は `.cut(1)` か 16 子の休符。

126 BPM の 1 小節は約 1.9 秒。`fx:rk` は 3.0 秒、`fx:rl` は 3.8 秒。連続した小節に長い FX を置かない。

| スロット | 芯 | 代替 | 禁止 |
| --- | --- | --- | --- |
| kick | `bd*4` 常時 | `bd:tc`、`bd:hf` | `bd:gb`、キックをミュート |
| ohh | `[~ oh]*4` 常時 | `oh:op`、`oh:dn` | 表拍 `hh*4`、OHH をミュート、`hh*8` |
| bass | `0 ~ 3 ~` + `.lpf(240–320)`。シンセは `C2:` | `sawtooth`+lpf、`bs:ht` / `bs:rd` at `C4:` も **`.lpf` を付ける** | 1–4 をオフ、`bs:su` 重ね、wobble、lpf 無しの明るいベース |
| chh | `[hh hh ~ hh]*4` | `hh:dk`、`hh:tt`、`hh:cl` | `hh*16`、`hh*8`、裏拍 `&` に hh |
| perc | ピッチしたティック | `perc:tm`、`perc:tk`、`perc:st`、`perc:ti` | 16 分埋め、毎小節ベロシティだけ変えて次数を書く |
| tom | ピッチしたロータム | `tom:lo`、`tom:md`、`perc:gl` | 2/4 スネア代用、`sd` のバックビート |
| metal | ピッチした金属 | `perc:mh`、`perc:fm`、`perc:cw`、`fx:cg`（短いクラングは `note()` 可） | 明るいベル `perc:gs` をメロ代わり |
| clap | 4 小節に 1 `cp` | `cp:dr`、`perc:rm` | `[~ cp]*2` |
| synth | ダーク上モノ + `.lpf(700–1200)` | `ld:in`、`ld:mt`、`ld:nb`、`ld:gr`、`ld:pu`、`sawtooth`+lpf | `ld:ss` アンセム、`ld:an`、`ld:cy`、`ld:mx`、16 小節常時、lpf 無し |
| pluck | 金属・不安。次数は少なめ（`1` や中空 5 度） | `plk:nn`、`plk:s5`、`plk:gm`、`plk:kl`、`ld:mt` | `plk:mx`、`plk:ch`、`plk:aj`、`plk:sm`、16 小節常時 |
| stab | 疎な `[0,4]` | `plk:s5`、`plk:sf`、`plk:nn` | Rhodes、スーパーソー、`plk:ss` |
| chords | 疎な `[0,2,4]` | `plk:sf`、`ep:mt` | `triangle`、`ld:ss`、毎拍 |
| pad | 薄い。9 だけ | `dr:pd`、`dr:fg`、`pf:ff`+次数 `0` | `ps:gt`、gabber、毎小節 |
| drone | 金属／ホラー床。5 と 9 | `dr:mb`、`dr:hr`、`dr:md`、`dr:id` | `dr:sl` ソー壁、毎小節 |
| texture | 短い砂／エア | `fx:ha`、`fx:nh`、`fx:ck`、`fx:wd` | ライザーを毎 4 小節 |
| fx | 16 子のダークワンショット（`note()` なし） | `fx:rd`、`fx:rk`、`fx:ry`、`fx:rl`、`fx:cg`、`fx:mc`、`fx:sd`、`fx:sw`、`fx:dn`、`fx:nb`、`fx:wh` | `fx:gb`、`fx:fc`、`fx:up`、`fx:rb`、`fx:rs`、毎小節、ライザーに `note()` |

## Why

キックは毎拍、OHH は裏拍、ベースはローパスのオスティナート。この 3 つが床。上モノの主役はダークシンセ（`ld:in` 族 + lpf）。明るいスーパーソーやベルはクラブのミニマルではない。

展開は新しいメロではない。同じ 1 小節を、リズム帯以外で出し入れする。CHH の 16 分はグルーヴを足すためで、OHH の `&` を埋めない。`hh*16` はオープンと同時に閉じる。

perc / tom / metal を `note()` でキーに乗せるのは、このジャンルではメロディ楽器を増やさずに不安感を足す書き方。ライザーとサブドロップは wav 側にピッチがあるので `note()` しない。

`[~ cp]*2` はハウスのバックビート。`cp` は 4 小節に 1 回だけ。

PCM ベースは `C4:`（native）。シンセサブは `C2:`。どちらも `.lpf` を残す。

## レシピ

1. kick `bd*4`、ohh `[~ oh]*4`、bass は lpf 付きで常時。この 3 本をミュートしない
2. **14–16 本**。PCM をスロットに割り当てる。8 本にまとめない
3. CHH は `[hh hh ~ hh]*4`。`hh*16` にしない
4. synth はダーク + lpf。pluck は金属・不安。次数は 16 小節で書き換えない
5. perc / tom / metal は `note()` + 同じ 4 小節 `.scale` でよい
6. 16 小節ミュート（上表）。和声の変化は 4 小節目の G phrygian と `cp`
7. 長い FX はセクション境界だけ。ライザーに `note()` しない

鳴らすのは `dj_hermes_apply_song(content, deck)`（次小節、無書き込み）。`dj_hermes_save_song` は残す指示のときだけ。

## Variations（同じ文法）

| 目的 | 変更 |
| --- | --- |
| ハットを粗く | OHH を `[~ oh ~ ~]*2`（1& と 3&）。CHH はオフのまま |
| CHH を足す | マップどおり 5–12 と 15–16。OHH は触らない |
| ターンを長く | 13–16 をリズム帯だけ（chh / synth の 15–16 も `~`） |
| ベースを暗く | `.lpf(200)`。パターンは変えない |
| シンセを金属に | synth を `ld:mt`（pluck と同じ slug にしない） |
| ラジオをノイズに | 4 小節目を `fx:nh` / `fx:ck` |
| リバースを長く | 7 小節目を `fx:rl`（3.8 秒。8 小節は休符） |
| 14 本 | `// clap` と `// texture` を置かない |

## Pitfalls

1. `stack()` / `.cpm()` / `.lfo()` → apply / save とも 400
2. kick / ohh / chh を 1 本の `// drums` に戻す（CHH がミュートできない）
3. キック・OHH・ベースを 1–4 や 13–14 で落とす
4. `[~ cp]*2`（4 小節に 1 回の `cp` とは別）
5. `hh*16` / `hh*8` / `hh*4`。CHH を裏拍 `&` に置く
6. 8 本のまま出す。新規メロで展開する。16 引数の `cat`
7. `<~@4 [0 ~ 3 ~]@12>` のように `@` でミュート小節を稼ぐ
8. `<>` の中で 1 小節ループをブラケット無しにする
9. 新規 apply でフェンスの `.s()` を全コピーする。スーパーソーや Rhodes、`ld:ss`
10. `dj_hermes_mute` を曲のフォームにする
11. `fx:gb` / `fx:fc` / `fx:up` / `fx:rb`。ライザー・サブドロップに `note()`
12. 長い PCM（`ld:` / `dr:` / `pf:` / `ps:`、`fx:rk` / `fx:rl`）を毎小節撃つ
13. 同一曲の synth と pluck が同じ `.s()`

## Checklist

- [ ] `setcpm(126/4)` + **14–16 本**（kick / ohh / bass は常時。chh / perc / tom / metal / synth / pluck / stab / chords / pad / drone / fx を含む）
- [ ] キック `bd*4`、OHH `[~ oh]*4`、bass は `.lpf` 付きオスティナート。この 3 本に 16 子ミュートを書いていない
- [ ] 他は 16 子のミュートマップ。和声は 4 小節 `.scale`。13–14 はリズム帯だけ
- [ ] CHH は `[hh hh ~ hh]*4`。`hh*16` も `[~ cp]*2` も書いていない
- [ ] synth はダーク + lpf。pluck は金属・不安。次数はほぼ固定
- [ ] perc / tom / metal は `note()` してよい。ライザー / サブドロップ / リバースシンバルには付けていない
- [ ] `.s()` は音色パレット。長い FX は連続小節に置いていない
- [ ] `dj_hermes_apply_song(content, deck)`（save は残す指示のときだけ）
