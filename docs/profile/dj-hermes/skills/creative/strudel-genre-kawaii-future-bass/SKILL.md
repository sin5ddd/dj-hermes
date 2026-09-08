---
name: strudel-genre-kawaii-future-bass
description: >-
  Use when writing Kawaii Future Bass for dj-hermes: 140 BPM
  trap-influenced half-time, sparkly pads, J-pop 王道/小室 melody,
  kawaii bells, 16-bar loop, equal-weight <> children, hat rolls only
  at phrase ends. Not supersaw-anthem Future Bass, not four-on-the-floor,
  not hh*16 every 4 bars, not kick-only bass.
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [dj-hermes, music, genre, kawaii-future-bass, kawaii, j-pop]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-future-bass
      - strudel-genre-chill-pop
      - strudel-genre-dubstep
---

# dj-hermes × Kawaii Future Bass（キラキラパッド + J-pop 進行）

## Overview

このリポジトリの **Kawaii Future Bass** は、ハーフタイムの上に **キラキラしたパッドと J-pop 進行のメロ** を載せたもの。派手さはスーパーソーの壁ではなく、**ベル／オルゴール／ガラスプラックと長いパッド**。端的な参照は Snail's House / Ujico* 周辺。来場者が「アニソン」と言ったら **王道進行** を書く。

**Future Bass とは別物。** スーパーソー中心のユーロビート派生アンセムは [strudel-genre-future-bass](../strudel-genre-future-bass/SKILL.md)。「フューチャーベース」だけでスーパーソーのフェスドロップを指しているならそちら。こちらに `ld:ss` の壁とアンセム i–VI–III–VII を載せない。

| 層 | 取るもの | 取らないもの |
| --- | --- | --- |
| ドラム | ハーフタイム（スネアは 3 拍目）。キックは曲ごとに表から選ぶ。ハットは 8 分。ロールは **8 / 16 小節目の末**だけ | 全曲 `bd ~ bd ~`、`bd*4`、ハウス `[~ cp]*2`、**4 小節ごとの `[hh*16]`** |
| ベース | コードのルートを **8 分**で追う。ミッドがある 1 本（`.cut(1)`） | キックにだけ揃う `0 ~ 0 ~`、1 小節ループ、wobble 主役 |
| 上物 | **キラキラパッド**、ベル／オルゴール、リフレインするメロ、**16 小節の王道＋逆転＋クリシェ**。`<>` の子は **同じウェイト** | スーパーソーの壁、アンセム i–VI–III–VII、全部 `triangle`、4 小節王道の使い回し、ウェイト 9 のリフレイン |

テンポ既定は **140**（Future Bass / ダブステップと同じ時計。ペア可）。174 はそのときは **solo**。

同梱 `songs/future-bass/*.strudel` は **旧・未分化**（王道＋ベルだが、2 ステップ固定、4 小節ごと `hh*16`、キック揃えベース、ウェイト 9）。新規の Kawaii はこの Skill。ジャンルフォルダ `songs/kawaii-future-bass/` は未作成。曲ファイルは別作業。

## When

- 依頼が **Kawaii Future Bass / カワイイベース / キラキラ** のとき
- 「アニソン」「王道」「小室」をこの床に載せたいとき
- 「ダブステップに J-pop を載せたい」とき → 進行は下の **16 小節フォーム**。ドラムは下のキック表（ダブステップ Skill を貼らない）
- **使わないとき**: スーパーソーのフェスドロップ、アンセム、ユーロビート的なソーの壁 → **strudel-genre-future-bass**
- シティポップ下降は [strudel-genre-chill-pop](../strudel-genre-chill-pop/SKILL.md)

## グリッドをずらさない（Issue #58）

1 小節 = ウェイト合計で割る。8 分グリッドなら **子の `@` 合計は 8**。`<>` の子ごとに合計が違うと、その小節だけメロがドラムからずれる。

| 書き方 | 合計 | 結果 |
| --- | --- | --- |
| `[~ 4 ~ 7  ~ 9 4 2]` | 8 | 8 分に乗る |
| `[4@2 7 9@2 7 4 2]` | 2+1+2+1+1+1 = **8** | 長い音でもグリッド維持 |
| `[4@2 7 9@2  7 4 2 0]` | 2+1+2+1+1+1+1 = **9** | **禁止。** リフレインだけ遅れて聞こえる |

長い音は `@` で伸ばす。伸ばした分、その子の原子を減らす。`.cut(1)` で同じ次数を 2 回書くとレトリガする。

新規の lead / hook / bass は、書き終わったら **各 `<>` 子のウェイトを数える**。

## 進行（16 小節。Hermes は名前で書く）

親キー C。次数は固定、`.scale("<…>")` の **16 子**が 1 コード／小節。pitched 全部で同じ 16 小節。PCM は `C4:` 帯。arp は C5。

4 小節ブロックを 4 つ並べる。**新規曲を 4 小節王道の繰り返しにしない。**

| ブロック | 名前 | 度数 | C 親キー | 4 子 |
| --- | --- | --- | --- | --- |
| A | **王道**（「アニソン」「J-pop 王道」） | IV–V–iii–vi | F–G–Em–Am | `F4:lydian G4:mixolydian E4:phrygian A4:minor` |
| A' | 王道の繰り返し（リフレイン和声） | 同じ | 同じ | 同じ 4 子 |
| B | **王道の逆転** | vi–iii–V–IV | Am–Em–G–F | `A4:minor E4:phrygian G4:mixolydian F4:lydian` |
| C | **クリシェ**（下行バス） | I–viiø–vi–V | C–Bø–Am–G | `C4:major B4:locrian A4:minor G4:mixolydian` |

既定フォームは **A A' B C**（王道 8 → 逆転 4 → クリシェ 4）。

名前付き差し替え（16 子を組み替える。4 小節に戻さない）:

| 名前 | 16 小節 |
| --- | --- |
| **小室**（「小室」「90s J-pop」） | 小室 8 + 小室逆転 4 + クリシェ 4。小室 4 子: `A4:minor F4:lydian G4:mixolydian C4:major`（vi–IV–V–I）。逆転: `C4:major G4:mixolydian F4:lydian A4:minor`（I–V–IV–vi） |
| **カノン**（来場者が「カノン」「I–V–vi–IV」と言ったときだけ） | カノン 8 + カノン逆転 4 + クリシェ 4。カノン 4 子: `C4:major G4:mixolydian A4:minor F4:lydian`。逆転: `F4:lydian A4:minor G4:mixolydian C4:major` |

**取り違えない。** I–V–vi–IV はカノンであって王道ではない。王道は **IV–V–iii–vi**。Future Bass のアンセム i–VI–III–VII はこちらに使わない。

**クリシェ**は **下行バス**（I のルートから B–A–G）。次数 0 がルートを辿る。半音クロマティック（Bb を挟む）は書かない。

モードは C のダイアトニック（IV=lydian、V=mixolydian、iii=phrygian、vi=minor、I=major、vii=locrian）。`F4:major` にすると Bb が入る。

既定 16 子（arp はオクターブ 5）:

```
<F4:lydian G4:mixolydian E4:phrygian A4:minor F4:lydian G4:mixolydian E4:phrygian A4:minor A4:minor E4:phrygian G4:mixolydian F4:lydian C4:major B4:locrian A4:minor G4:mixolydian>
```

## ドラム（曲ごとに表から 1 行）

スネアは **3 拍目**（`~ ~ sd ~`）。キックは **1 と 3 の 2 ステップを全曲の既定にしない。**

ハットの床は `hh*8`。**32 分ロールは 8 小節目と 16 小節目の最後の 8 分だけ。** 4 小節ごと `[hh*16]` は禁止。

新規曲は下表から **キック 1 行**を選ぶ。前の Kawaii 曲と同じキック文字列を使わない。

| 名 | キック（4 子。16 小節で 4 周） | ハット | いつ |
| --- | --- | --- | --- |
| **skip** | `<[bd ~ ~ ~] [bd [~ bd] ~ ~] [bd ~ ~ bd] [bd ~ ~ ~]>` | `hh*8` + 下のロール層 | **既定** |
| **sparse** | `<[bd ~ ~ ~] [bd ~ ~ ~] [bd [~ bd] ~ ~] [bd ~ ~ ~]>` | `[~ hh]*4` | 空きが多い |
| **bounce** | `<[bd ~ [bd ~] ~] [bd ~ ~ ~] [bd [~ bd] ~ ~] [bd ~ ~ bd]>` | `hh*8`、4 小節目に `oh` 可 | シンコペ |
| **two-step** | `<[bd ~ bd ~] [bd ~ ~ ~] [bd ~ bd ~] [bd ~ ~ bd]>` | `hh*8` + ロール層 | **4 子すべて `bd ~ bd ~` にはしない** |

ロール層:

```
<~ ~ ~ ~ ~ ~ ~ [~ ~ ~ ~ ~ ~ ~ [hh*4]] ~ ~ ~ ~ ~ ~ ~ [~ ~ ~ ~ ~ ~ ~ [hh*4]]>
```

禁止: `<~ ~ ~ [hh*16] ~ ~ ~ [hh*16] ~ ~ ~ [hh*16] ~ ~ ~ [hh*16]>`。

キックは duck 用に **別 `$:`**。ハット＋スネアは 2 本目。ハットに `duckorbit` は付けない。

## ベース（8 分、8 子以上）

ベースは和声の床で、**キックのコピーではない。** ルートを 8 分で追う。1 小節 `0 ~ 0 <0 4 0 2>` は禁止（低音がスカスカ）。

- 次数の `<>` は **8 子以上**。各子ウェイト 8
- PCM は 1 本。8 分なら **`.cut(1)`**。既定は `bs:ht`（ベルの下で鳴るタイト）。`bs:su` はキック位置だけ撃つ書き方にしない
- サブを 2 本にしない。wobble を主役にしない

## Pattern

duck でキックを分離。bass / chords / pad / **strings** を orbit 2。lead / hook / arp はドライ。**9 本**。進行は **王道 A A' B C**。

このフェンスは **skip キック + 8 分ベース + キラキラパッド** の見本。新規曲はキック表・ベース次数・lead 次数を **書き直す**。フェンスをキーだけ変えて量産しない。

`// lead` は **リフレイン**: 1–4 前振り、5–8 と 13–16 が同じ決め（各子ウェイト 8）、9–12 は逆転の上で変化。

```
// @title visitor-kawaii-future-bass
// @genre kawaii-future-bass
setcpm(140/4)
// kick — trap skip (not 2-step on every song)
$: s("<[bd ~ ~ ~] [bd [~ bd] ~ ~] [bd ~ ~ bd] [bd ~ ~ ~]>").gain(0.85).duckorbit(2).duckattack(0.05).duckdepth(0.8)
// hats — snare on 3; 8th hats; 32nd roll on last 8th of bars 8 and 16 only
$: s("~ ~ sd ~, hh*8, <~ ~ ~ ~ ~ ~ ~ [~ ~ ~ ~ ~ ~ ~ [hh*4]] ~ ~ ~ ~ ~ ~ ~ [~ ~ ~ ~ ~ ~ ~ [hh*4]]>").gain(0.42)
// bass — 8ths follow the chord root; 8-bar phrase
$: note("<[0 0 ~ 0  0 ~ 0 4] [0 0 0 ~  0 4 ~ 0] [0 ~ 0 0  4 0 0 ~] [0 0 ~ 4  0 ~ 2 0] [0 0 0 0  ~ 0 4 0] [0 ~ 0 4  0 0 ~ 2] [0 0 ~ 0  4 ~ 0 0] [0 4 0 ~  0 0 2 0]>")
  .scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor F4:lydian G4:mixolydian E4:phrygian A4:minor A4:minor E4:phrygian G4:mixolydian F4:lydian C4:major B4:locrian A4:minor G4:mixolydian>")
  .s("bs:ht").gain(0.4).cut(1).orbit(2)
// lead — refrain; every child weight 8 (not supersaw wall)
$: note("<[~ 4 ~ 7  ~ 9 4 2] [~ 7 4 9  7 ~ 4 2] [4 ~ 9 7  ~ 4 2 0] [~ 4 7 9  4 2 ~ 7] [4@2 7 9@2 7 4 2] [4@2 7 9@2 7 4 0] [4@2 7 9@2 7 2 ~] [4@2 7 9@2 7 4 2] [~ 9 7 4  2 0 ~ 4] [9 ~ 7 4  ~ 2 0 4] [7 4 ~ 2  0 ~ 4 7] [~ 4 2 0  4 7 ~ 9] [4@2 7 9@2 7 4 2] [4@2 7 9@2 7 4 0] [4@2 7 9@2 7 2 ~] [4@2 7 9@2 7 4 2]>")
  .scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor F4:lydian G4:mixolydian E4:phrygian A4:minor A4:minor E4:phrygian G4:mixolydian F4:lydian C4:major B4:locrian A4:minor G4:mixolydian>")
  .s("ld:mx").gain(0.16).cut(1)
// hook — kawaii bell; denser on refrain bars
$: note("<[~ 11 ~ 12  ~ 9 ~ 11] [~ 12 ~ 9  ~ 11 ~ 7] [~ 11 ~ 12  ~ 9 ~ 4] [~ 9 ~ 7  ~ 4 ~ 11] [11 ~ 12 9  ~ 11 12 9] [~ 11 ~ 12  9 ~ 11 7] [11 12 ~ 9  11 ~ 12 9] [11 12 9 11  12 9 11 7] [~ 12 ~ 9  ~ 7 ~ 4] [~ 11 ~ 7  ~ 4 ~ 0] [~ 9 ~ 4  ~ 7 ~ 2] [~ 7 ~ 4  ~ 2 ~ 0] [11 ~ 12 9  ~ 11 12 9] [~ 11 ~ 12  9 ~ 11 7] [11 12 ~ 9  11 ~ 12 9] [11 12 9 11  12 9 11 7]>")
  .scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor F4:lydian G4:mixolydian E4:phrygian A4:minor A4:minor E4:phrygian G4:mixolydian F4:lydian C4:major B4:locrian A4:minor G4:mixolydian>")
  .s("plk:mx").gain(0.18).cut(1)
// arp — glass pluck as vocal-chop stand-in; 8-bar
$: note("<[0 4 ~ 7  4 ~ 9 4] [0 ~ 4 7  ~ 4 12 7] [4 0 7 ~  4 9 ~ 4] [0 7 4 0  ~ 4 7 12] [0 4 7 ~  9 4 ~ 7] [4 ~ 0 7  4 12 ~ 4] [0 4 ~ 9  7 4 0 4] [7 4 0 ~  4 7 12 4]>")
  .scale("<F5:lydian G5:mixolydian E5:phrygian A5:minor F5:lydian G5:mixolydian E5:phrygian A5:minor A5:minor E5:phrygian G5:mixolydian F5:lydian C5:major B5:locrian A5:minor G5:mixolydian>")
  .s("plk:fg").gain(0.14).cut(1)
// chords — add9 [0,4,8] on a short sparkle, not a supersaw wall
$: note("<[[0,4,8] ~ ~ [0,4,8]  [0,4,8] ~ [0,4,8] ~] [[0,2,8] ~ [0,2,8] ~  ~ [0,4,8] ~ [0,4,8]] [[0,4,8] ~ ~ [0,4,8]  [0,2,8] ~ [0,2,8] ~] [[0,4,8] ~ [0,4,8] [0,4,8]  ~ [0,2,8] ~ ~]>")
  .scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor F4:lydian G4:mixolydian E4:phrygian A4:minor A4:minor E4:phrygian G4:mixolydian F4:lydian C4:major B4:locrian A4:minor G4:mixolydian>")
  .s("plk:ch").gain(0.2).orbit(2)
// pad — sparkly hold (the kawaii identity)
$: note("<0 ~ ~ ~ 0 ~ ~ ~ ~ ~ 0 ~ 0 ~ ~ ~>").scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor F4:lydian G4:mixolydian E4:phrygian A4:minor A4:minor E4:phrygian G4:mixolydian F4:lydian C4:major B4:locrian A4:minor G4:mixolydian>")
  .s("ps:mx").gain(0.16).room(0.35).orbit(2)
// strings — choir / wide sparkle, offset from pad
$: note("<~ ~ 0 ~ ~ ~ 0 ~ 0 ~ ~ ~ ~ ~ 0 ~>").scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor F4:lydian G4:mixolydian E4:phrygian A4:minor A4:minor E4:phrygian G4:mixolydian F4:lydian C4:major B4:locrian A4:minor G4:mixolydian>")
  .s("ld:cr").gain(0.12).room(0.45).orbit(2)
```

新規 apply の `.s()` は下のパレットから選ぶ。

## リード動機（フェンスの 16 子を使い回さない）

各子ウェイト 8。5–8 と 13–16 を同じ決めに戻す。

| 名 | 決め（ウェイト 8） | 前振りの味 |
| --- | --- | --- |
| **lift** | `[4@2 7 9@2 7 4 2]` | 休符始まり、4 と 7 と 9 |
| **fall** | `[9@2 7 4@2 2 0 4]` | 高い 9 から下りる |
| **skip** | `[4 7 ~ 9  7@2 4 2]` | 8 分と欠拍 |
| **hold** | `[4@4 7@2 9 7]` | 長い 4 のあと短い飾り（4+2+1+1=8） |

`[4@2 7 9@2  7 4 2 0]`（合計 9）は使わない。

## 音色パレット（新規 apply はここから選ぶ）

同一曲の pitched 2 本に同じ `.s()` を使わない。長尺は `.cut(1)` か間引き。カタログに violin は無い。

| スロット | 芯 | 代替 | 禁止 |
| --- | --- | --- | --- |
| drums | キック表 1 行 + 3 拍目 `sd` + `hh*8` + 8/16 末ロール | `bd:8t`、`sd:tr`、`hh:ch`、`hh:tt` | `bd*4`、`[~ cp]*2`、全曲 `bd ~ bd ~`、4 小節ごと `[hh*16]` |
| bass | 8 分ルート追い。`bs:ht` / `bs:8s`+`.cut(1)` at `C4:` | `bs:su`+`.cut(1)` | `0 ~ 0 ~` を既定、サブ重ね、`bs:wb`、`bs:sw` を主役（ソーベースは future-bass） |
| lead | メロディックなリフレイン。子はウェイト 8 | `ld:mx`、`ld:gl`、`ld:cy`、`.s("square").lpf(3200)` | `ld:ss` / `ld:an` の壁、303、`ld:gr`、ウェイト 9 の `@` |
| hook | kawaii ベル | `plk:mx`、`plk:ch`、`plk:bl`、`plk:mb` | `ld:st` ソースタブ、wobble、`plk:dt` |
| arp | ガラス／チョップ代用。8 子以上 | `plk:fg`、`plk:fc` | `ld:ap` 忙しいソー arp（future-bass 側）、ボーカル WAV を invent |
| chords | `[0,4,8]` add9。短いキラキラ | `[0,2,8]`、`plk:ch`、`plk:sm` | `[0,2,4]`、`[0,4,9]` を add9 と呼ぶ、`plk:ss` ソー壁 |
| pad | **キラキラがこのジャンルの芯** | `ps:mx`、`ps:gb`、`pf:ga`、`pf:sp` | `dr:sl` 低いソー、gabber、毎小節撃つ、strings と同じオンオフ |
| strings | クワイア／広いキラキラ。pad とずらす | `ld:cr`、`pf:ca`、`pf:hl`、`pf:wm` | `dr:sl`、`ld:ss`、violin を invent、毎小節撃つ |

## Why

**キラキラが主役。** Kawaii の派手さはベルとパッドの空気感。スーパーソーを厚く積むと Future Bass（アンセム）になって、Snail's House 側ではなくなる。

**王道 16 小節。** 4 小節王道だけだとすぐ一周する。A A' B C で 16 子。アンセム i–VI–III–VII は future-bass Skill。

**ハーフタイムはスネア位置。** `~ ~ sd ~`。キックを毎曲 `bd ~ bd ~` にしない。`bd*4` にしない。ロールは 8/16 末だけ。

**8 分ベース。** キック 2 打だけだと低域が空洞。ルート 8 分。`.cut(1)`。

**リフレインとウェイト。** 5–8 と 13–16 が同じ決め。`@` 合計 8。合計 9 はドラムからずれる（Issue #58）。

**add9 `[0,4,8]`。** 0=根、4=5 度、8=9 度。`[0,4,9]` は 10 度。

**ヴォーカルチョップ。** 歌サンプルは無い。短い `plk:fg` を arp に置く。

**duck。** キックが orbit 2 を潰す。lead / hook はドライ。ハットに `duckorbit` 禁止。

PCM は `C4:`。

## レシピ

1. スネアは 3 拍目。キックは表から 1 行。`bd*4` にしない
2. ハットは 8 分。ロールは 8 / 16 小節目の末だけ
3. ベースは 8 分ルート追い。`<>` 8 子以上。`.cut(1)`
4. ループは **16 小節** 王道 A A' B C。lead はリフレイン、**各子ウェイト 8**
5. パッドはキラキラ（`ps:mx` 族）。スーパーソーの壁にしない
6. フックはベル。コードは `[0,4,8]`
7. duck はキックだけ。**9 本**。pad と strings のオンオフをずらす
8. フェンスをキーだけ変えて量産しない

鳴らすのは `dj_hermes_apply_song(content, deck)`。`dj_hermes_save_song` は残す指示のときだけ。

## Variations（同じ文法）

| 目的 | 変更 |
| --- | --- |
| 小室 16 小節 | 全 pitched の `.scale` を小室 8 + 小室逆転 4 + クリシェ 4 |
| カノン 16 小節 | 全 pitched の `.scale` をカノン 8 + カノン逆転 4 + クリシェ 4 |
| 逆転を前に | B A C A' など（16 子は保つ） |
| キックを sparse / bounce | ドラム表。前の曲と同じキックにしない |
| チップチューン寄り | lead を `.s("square").lpf(3200)` |
| ベルをチャイムに | hook を `plk:ch` |
| ストリングスを空気に | strings を `pf:ca` / `pf:hl` |
| リード動機 | lift / fall / skip / hold（ウェイト 8） |
| スーパーソーのフェスにしたい | この Skill を使わない。**strudel-genre-future-bass** |

## Pitfalls

1. `bd*4` や `[~ cp]*2` でハウス化する
2. ドラムを 3 `$:` に分ける（キック分離以外）
3. ハットへ `duckorbit`
4. wobble を主役にする
5. サブを重ねる
6. 長い PCM を毎小節撃つ
7. `note("c3'maj")` は root 単音。和音は `[0,4,8]`
8. スーパーソーの壁（`ld:ss` + `plk:ss` + `dr:sl`）をこの床に載せる（それは future-bass）
9. 140 と 174 を `dj` する
10. 「アニソン」とだけ書いて `.scale` を省略する、または I–V–vi–IV を王道として書く
11. フェンスの `.s()` とキック／ベース／lead 次数を全コピーする
12. 4 小節王道だけでループする。lead にリフレインが無い
13. strings を足さずに 8 本。16 引数の `cat`
14. 全曲 `bd ~ bd ~` + 4 小節ごと `[hh*16]`
15. ベース `0 ~ 0 <0 4 0 2>`
16. lead `[4@2 7 9@2  7 4 2 0]`（ウェイト 9）
17. `[0,4,9]` を add9 と呼ぶ
18. Future Bass のアンセム進行を「王道」として書く

## Checklist

- [ ] `setcpm(140/4)` + **9 `$:`**（kick, hats, bass, lead, hook, arp, chords, pad, strings）
- [ ] スネアは拍 3。キックは表の 1 行。`bd*4` ではない
- [ ] ハットは `hh*8`。ロールは 8 / 16 小節目の末だけ
- [ ] ベースは 8 分ルート追い、`<>` 8 子以上、`.cut(1)`
- [ ] **16 小節** 王道 A A' B C（小室／カノンは名前付き）。アンセム i–VI–III–VII ではない
- [ ] lead がリフレイン。**各 `<>` 子のウェイトが 8**。スーパーソーの壁ではない
- [ ] chords `[0,4,8]`。pad はキラキラ。hook はベル
- [ ] ベース PCM は `C4:`、orbit 2、1 本。ハットに duckorbit なし
- [ ] `.s()` は音色パレット（ベル／ガラス／キラキラパッド）
- [ ] `dj_hermes_apply_song(content, deck)`（save は残す指示のときだけ）
