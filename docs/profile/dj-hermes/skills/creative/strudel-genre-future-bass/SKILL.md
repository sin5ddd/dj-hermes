---
name: strudel-genre-future-bass
description: >-
  Use when writing Future Bass (not Kawaii) for dj-hermes: 140 BPM
  trap-influenced half-time, supersaw stacks and eurobeat-like flash,
  8th-note bass, 16-bar anthem loop, equal-weight <> children, hat
  rolls only at phrase ends. Not 王道, not kawaii bells, not four-on-the-floor,
  not hh*16 every 4 bars, not kick-only bass.
version: 8.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [dj-hermes, music, genre, future-bass, supersaw, eurobeat]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-kawaii-future-bass
      - strudel-genre-dubstep
      - strudel-genre-dnb
      - strudel-genre-techno-duck
---

# dj-hermes × Future Bass（スーパーソー / ユーロビート派生の派手さ）

## Overview

このリポジトリの **Future Bass** は、トラップ由来のハーフタイムの上に **スーパーソーを厚く積んだ派手さ**。ユーロビートから借りるのはキックの四つ打ちではなく、**デチューンしたソーのアンセム感・忙しいアルペジオ・オクターブの跳ね**。参照の方向はフェスのドロップ（広いソーコード、8 分ベース、シンコペしたキック）。

**Kawaii Future Bass とは別物。** キラキラパッド、ベル、オルゴール、王道／小室は [strudel-genre-kawaii-future-bass](../strudel-genre-kawaii-future-bass/SKILL.md)。来場者が「アニソン」「カワイイベース」「王道」と言ったらそちら。こちらにベルと王道を載せない。

| 層 | 取るもの | 取らないもの |
| --- | --- | --- |
| ドラム | ハーフタイム（スネアは 3 拍目）。キックは曲ごとに表から選ぶ。ハットは 8 分。ロールは **8 / 16 小節目の末**だけ | 全曲 `bd ~ bd ~`、`bd*4`（四つ打ちユーロビートそのもの）、ハウス `[~ cp]*2`、**4 小節ごとの `[hh*16]`** |
| ベース | コードのルートを **8 分**で追う。オクターブ `7` を混ぜて派手に。ミッドがある 1 本（`.cut(1)`） | キックにだけ揃う `0 ~ 0 ~`、1 小節ループ、wobble 主役 |
| 上物 | **スーパーソー**のリード／スタブ／コード、忙しい arp、**16 小節のアンセム進行**。`<>` の子は **同じウェイト** | kawaii ベル、オルゴール、王道 IV–V–iii–vi、全部 `triangle`、ウェイト 9 のリフレイン |

テンポ既定は **140**（ダブステップと同じ時計。ペア可）。174 のブレイクに同じ上物を載せるのは別バリエーションで、そのときは **solo**（140 と混ぜない）。

同梱 `songs/future-bass/*.strudel` は **旧・未分化**（2 ステップ固定、王道、ベル、4 小節ごと `hh*16`、キック揃えベース、リフレインのウェイト 9）。新規の Future Bass はこの Skill。Kawaii は隣の Skill。曲ファイルは別作業。

## When

- 依頼が **Future Bass / フューチャーベース** で、スーパーソーやフェスのドロップを指しているとき
- 「派手」「アンセム」「スーパーソー」のとき
- **使わないとき**: 「カワイイベース」「アニソン」「王道」「ベル」「オルゴール」→ **strudel-genre-kawaii-future-bass**
- 四つ打ちの明るい EDM として書かないとき（`bd*4` はプログレ／ハウス。ユーロビートのキックをここへ持ち込まない）

## グリッドをずらさない（Issue #58）

1 小節 = ウェイト合計で割る。8 分グリッドなら **子の `@` 合計は 8**。16 分なら 16。`<>` の子ごとに合計が違うと、その小節だけ音符がドラムからずれる。

| 書き方 | 合計 | 結果 |
| --- | --- | --- |
| `[~ 4 ~ 7  ~ 9 4 2]` | 8 | 8 分に乗る |
| `[4@2 7 9@2 7 4 2]` | 2+1+2+1+1+1 = **8** | 長い音でもグリッド維持 |
| `[4@2 7 9@2  7 4 2 0]` | 2+1+2+1+1+1+1 = **9** | **禁止。** リフレインだけ遅れて聞こえる |

長い音は `@` で伸ばす。伸ばした分、その子の原子を減らす。`.cut(1)` で同じ次数を 2 回書くとレトリガする（ホールドにならない）。

新規の lead / hook / bass は、書き終わったら **各 `<>` 子のウェイトを数える**。8 と 9 が混ざる文字列を貼らない。

## 進行（16 小節。王道は使わない）

親キー A 短調（アンセムの定番 i–VI–III–VII = Am–F–C–G）。次数は固定、`.scale("<…>")` の **16 子**が 1 コード／小節。pitched 全部で同じ 16 小節（オクターブだけトラックで変えてよい）。PCM は `C4:` 帯。arp は C5。

4 小節ブロックを 4 つ並べる。**新規曲を 4 小節の使い回しにしない。王道 IV–V–iii–vi は kawaii Skill。**

| ブロック | 名前 | 度数 | A 親キー | 4 子 |
| --- | --- | --- | --- | --- |
| A | **アンセム**（既定） | i–VI–III–VII | Am–F–C–G | `A4:minor F4:lydian C4:major G4:mixolydian` |
| A' | 同じ進行の繰り返し | 同じ | 同じ | 同じ 4 子 |
| B | **リフト**（III 始まり） | III–VII–i–VI | C–G–Am–F | `C4:major G4:mixolydian A4:minor F4:lydian` |
| C | **下降** | i–VII–VI–v | Am–G–F–Em | `A4:minor G4:mixolydian F4:lydian E4:phrygian` |

既定フォームは **A A' B C**。

名前付き差し替え（16 子を組み替える。4 小節に戻さない。王道にしない）:

| 名前 | 16 小節 |
| --- | --- |
| **カノン**（来場者が「カノン」「I–V–vi–IV」と言ったときだけ） | カノン 8 + カノン逆転 4 + 下降 4。カノン 4 子: `C4:major G4:mixolydian A4:minor F4:lydian`。逆転: `F4:lydian A4:minor G4:mixolydian C4:major` |

**取り違えない。** 王道は IV–V–iii–vi で、kawaii 側。I–V–vi–IV はカノンであって王道ではない。小室 vi–IV–V–I も kawaii 側。

モードはダイアトニック（i=minor、VI=lydian、III=major、VII=mixolydian、v=phrygian）。`F4:major` にすると Bb が入って親キーを外れる。

既定 16 子（pitched にこれを載せる。arp はオクターブ 5）:

```
<A4:minor F4:lydian C4:major G4:mixolydian A4:minor F4:lydian C4:major G4:mixolydian C4:major G4:mixolydian A4:minor F4:lydian A4:minor G4:mixolydian F4:lydian E4:phrygian>
```

## ドラム（曲ごとに表から 1 行）

スネアはハーフタイムなので **3 拍目**（`~ ~ sd ~`）。キックは **1 と 3 の 2 ステップを全曲の既定にしない。** トラップはキックが 1 と「2 の裏」、または 1 と 4。体感は 70 BPM の踏み、ハットの 8 分だけ速い。

ハットの床は `hh*8`（8 分）。**32 分ロールは 8 小節目と 16 小節目の最後の 8 分だけ。** 4 小節ごとに小節まるごと `hh*16` にすると、どの曲も同じ「ハイハット連打」になる。

新規曲は下表から **キック 1 行**を選ぶ。前の Future Bass と同じキック文字列を使わない。

| 名 | キック（4 子。16 小節で 4 周） | ハット | いつ |
| --- | --- | --- | --- |
| **skip** | `<[bd ~ ~ ~] [bd [~ bd] ~ ~] [bd ~ ~ bd] [bd ~ ~ ~]>` | `hh*8` + 下のロール層 | **既定。** 1、2 の裏、4 |
| **sparse** | `<[bd ~ ~ ~] [bd ~ ~ ~] [bd [~ bd] ~ ~] [bd ~ ~ ~]>` | `[~ hh]*4`（裏拍） | 空きが多いドロップ |
| **bounce** | `<[bd ~ [bd ~] ~] [bd ~ ~ ~] [bd [~ bd] ~ ~] [bd ~ ~ bd]>` | `hh*8`、4 小節目に `oh` 1 つ可 | シンコペ |
| **two-step** | `<[bd ~ bd ~] [bd ~ ~ ~] [bd ~ bd ~] [bd ~ ~ bd]>` | `hh*8` + ロール層 | **4 子すべて `bd ~ bd ~` にはしない** |

ロール層（skip / bounce / two-step に並列。16 子。8 と 16 だけ末 8 分が `[hh*4]` = 32 分 4 粒）:

```
<~ ~ ~ ~ ~ ~ ~ [~ ~ ~ ~ ~ ~ ~ [hh*4]] ~ ~ ~ ~ ~ ~ ~ [~ ~ ~ ~ ~ ~ ~ [hh*4]]>
```

禁止: `<~ ~ ~ [hh*16] ~ ~ ~ [hh*16] ~ ~ ~ [hh*16] ~ ~ ~ [hh*16]>`（4 小節ごとのフルバー連打）。`bd*4` も禁止（四つ打ちユーロビートになってドロップではなくなる）。

キックは duck 用に **別 `$:`**。ハット＋スネアは 2 本目。ハットに `duckorbit` は付けない。

## ベース（8 分、8 子以上）

ベースは和声の床で、**キックのコピーではない。** ルートを 8 分で追い、派手さ用にオクターブ `7` を混ぜる。1 小節 `0 ~ 0 <0 4 0 2>` だと低音がスカスカで、30 曲が同じに聞こえる。

- 次数の `<>` は **8 子以上**（16 小節 scale の上を 2 周する）。1 小節固定は禁止
- 各子は 8 分 × 8（ウェイト 8）。ルート `0` を軸に、5 度 `4` とオクターブ `7` と下行 `2`
- コードが休む 8 分ではベースも休んでよい
- PCM は 1 本。8 分で撃つなら **`.cut(1)`**。ミッドが欲しいときは `bs:sw`（スーパーソーベース）か `bs:ht`。サブを 2 本にしない
- `bs:su` をキック位置だけ撃つ書き方は、1 曲の「空き」バリエーションに限る。既定にしない

## Pattern

duck でキックを分離する。ハットに `duckorbit` は付けない。bass / chords / pad / **strings** を orbit 2。lead / hook / arp はドライ。**9 本**。進行は上の **アンセム A A' B C**。

このフェンスは **skip キック + 8 分ベース + スーパーソー** の見本。新規曲はキック表・ベース次数・lead 次数を **書き直す**。フェンスの `note("…")` / キック文字列をキーだけ変えて量産しない。同一曲の pitched 2 本に同じ `.s()` を使わない（lead が `ld:ss` なら chords は `plk:ss`、strings は `dr:sl`）。

`// lead` は **リフレイン**: 1–4 は前振り、5–8 と 13–16 が同じ決め（各子ウェイト 8）、9–12 はリフトの上で変化。

```
// @title visitor-future-bass
// @genre future-bass
setcpm(140/4)
// kick — trap skip (not 2-step on every song, not bd*4)
$: s("<[bd ~ ~ ~] [bd [~ bd] ~ ~] [bd ~ ~ bd] [bd ~ ~ ~]>").gain(0.85).duckorbit(2).duckattack(0.05).duckdepth(0.8)
// hats — snare on 3; 8th hats; 32nd roll on last 8th of bars 8 and 16 only
$: s("~ ~ sd ~, hh*8, <~ ~ ~ ~ ~ ~ ~ [~ ~ ~ ~ ~ ~ ~ [hh*4]] ~ ~ ~ ~ ~ ~ ~ [~ ~ ~ ~ ~ ~ ~ [hh*4]]>").gain(0.42)
// bass — 8ths follow the chord root with octave jumps; 8-bar phrase
$: note("<[0 0 ~ 0  0 ~ 7 4] [0 0 0 ~  0 4 ~ 7] [0 ~ 0 0  4 0 7 ~] [0 0 ~ 4  0 ~ 2 7] [0 0 0 0  ~ 0 4 7] [0 ~ 0 4  0 7 ~ 2] [0 0 ~ 0  4 ~ 7 0] [0 4 0 ~  7 0 2 0]>")
  .scale("<A4:minor F4:lydian C4:major G4:mixolydian A4:minor F4:lydian C4:major G4:mixolydian C4:major G4:mixolydian A4:minor F4:lydian A4:minor G4:mixolydian F4:lydian E4:phrygian>")
  .s("bs:sw").gain(0.42).cut(1).orbit(2)
// lead — supersaw refrain; every child weight 8
$: note("<[~ 4 ~ 7  ~ 9 7 4] [~ 7 4 9  7 ~ 4 2] [4 ~ 9 7  ~ 4 2 0] [~ 4 7 9  4 7 ~ 11] [4@2 7 9@2 7 4 2] [4@2 7 9@2 7 4 0] [4@2 7 9@2 7 2 ~] [4@2 7 9@2 7 4 2] [~ 9 7 4  2 0 ~ 4] [9 ~ 7 4  ~ 2 0 4] [7 4 ~ 2  0 ~ 4 7] [~ 4 2 0  4 7 ~ 9] [4@2 7 9@2 7 4 2] [4@2 7 9@2 7 4 0] [4@2 7 9@2 7 2 ~] [4@2 7 9@2 7 4 2]>")
  .scale("<A4:minor F4:lydian C4:major G4:mixolydian A4:minor F4:lydian C4:major G4:mixolydian C4:major G4:mixolydian A4:minor F4:lydian A4:minor G4:mixolydian F4:lydian E4:phrygian>")
  .s("ld:ss").gain(0.16).cut(1)
// hook — supersaw stab, not a kawaii bell
$: note("<[~ 7 ~ 11  ~ 9 ~ 7] [~ 11 ~ 9  ~ 7 ~ 4] [7 ~ 11 9  ~ 7 4 2] [~ 9 ~ 7  ~ 4 ~ 11] [7 11 ~ 12  11 7 9 7] [~ 11 ~ 12  9 ~ 7 4] [7 12 ~ 11  9 ~ 12 7] [7 11 12 9  11 7 4 2] [~ 12 ~ 9  ~ 7 ~ 4] [~ 11 ~ 7  ~ 4 ~ 0] [~ 9 ~ 4  ~ 7 ~ 2] [~ 7 ~ 4  ~ 2 ~ 0] [7 11 ~ 12  11 7 9 7] [~ 11 ~ 12  9 ~ 7 4] [7 12 ~ 11  9 ~ 12 7] [7 11 12 9  11 7 4 2]>")
  .scale("<A4:minor F4:lydian C4:major G4:mixolydian A4:minor F4:lydian C4:major G4:mixolydian C4:major G4:mixolydian A4:minor F4:lydian A4:minor G4:mixolydian F4:lydian E4:phrygian>")
  .s("ld:st").gain(0.18).cut(1)
// arp — busy euro-flash 8ths (not glass bell)
$: note("<[0 4 7 12  7 4 0 7] [0 7 4 12  7 0 4 7] [4 0 7 12  4 7 0 4] [0 7 12 4  7 4 0 12] [0 4 7 12  9 7 4 0] [4 7 12 7  4 0 7 12] [0 4 12 7  4 7 0 4] [7 12 4 0  7 4 12 7]>")
  .scale("<A5:minor F5:lydian C5:major G5:mixolydian A5:minor F5:lydian C5:major G5:mixolydian C5:major G5:mixolydian A5:minor F5:lydian A5:minor G5:mixolydian F5:lydian E5:phrygian>")
  .s("ld:ap").gain(0.12).cut(1)
// chords — bounced supersaw add9 [0,4,8]; 4-bar
$: note("<[[0,4,8] ~ ~ [0,4,8]  [0,4,8] ~ [0,4,8] ~] [[0,2,8] ~ [0,2,8] ~  ~ [0,4,8] ~ [0,4,8]] [[0,4,8] ~ ~ [0,4,8]  [0,2,8] ~ [0,2,8] ~] [[0,4,8] ~ [0,4,8] [0,4,8]  ~ [0,2,8] ~ ~]>")
  .scale("<A4:minor F4:lydian C4:major G4:mixolydian A4:minor F4:lydian C4:major G4:mixolydian C4:major G4:mixolydian A4:minor F4:lydian A4:minor G4:mixolydian F4:lydian E4:phrygian>")
  .s("plk:ss").gain(0.2).orbit(2)
// pad — wide saw bed, thinned (not sparkly kawaii pad)
$: note("<0 ~ ~ ~ 0 ~ ~ ~ ~ ~ 0 ~ 0 ~ ~ ~>").scale("<A4:minor F4:lydian C4:major G4:mixolydian A4:minor F4:lydian C4:major G4:mixolydian C4:major G4:mixolydian A4:minor F4:lydian A4:minor G4:mixolydian F4:lydian E4:phrygian>")
  .s("pf:sp").gain(0.14).room(0.25).orbit(2)
// strings — low supersaw drone (not choir / music-box)
$: note("<~ ~ 0 ~ ~ ~ 0 ~ 0 ~ ~ ~ ~ ~ 0 ~>").scale("<A4:minor F4:lydian C4:major G4:mixolydian A4:minor F4:lydian C4:major G4:mixolydian C4:major G4:mixolydian A4:minor F4:lydian A4:minor G4:mixolydian F4:lydian E4:phrygian>")
  .s("dr:sl").gain(0.12).orbit(2)
```

新規 apply の `.s()` は下のパレットから選ぶ。キック／ベース／lead の次数は表から **別の組**にする。

## リード動機（フェンスの 16 子を使い回さない）

次数は 0 始まり。各子ウェイト 8。曲ごとに 1 行を選び、5–8 と 13–16 を同じ決めに戻す。

| 名 | 決め（ウェイト 8） | 前振りの味 |
| --- | --- | --- |
| **lift** | `[4@2 7 9@2 7 4 2]` | 休符始まり、4 と 7 と 9 |
| **fall** | `[9@2 7 4@2 2 0 4]` | 高い 9 から下りる |
| **skip** | `[4 7 ~ 9  7@2 4 2]` | 8 分と欠拍 |
| **hold** | `[4@4 7@2 9 7]` | 長い 4 のあと短い飾り（4+2+1+1=8） |

`[4@2 7 9@2  7 4 2 0]`（合計 9）はどの動機にも使わない。

## 音色パレット（新規 apply はここから選ぶ）

Pattern はグリッド・次数・スロットの見本。新規曲は下表からスロットごとに 1 つ選び、フェンスの `.s()` を毎回コピーしない。同一曲の pitched 2 本に同じ `.s()` を使わない。slug の意味は strudel-pcm-catalog の INDEX。長尺（`ld:` / `dr:` / `pf:` / `ps:`）は `.cut(1)` か間引き。wobble を主役にしない。

| スロット | 芯 | 代替 | 禁止 |
| --- | --- | --- | --- |
| drums | 上のキック表 1 行 + 3 拍目 `sd` + `hh*8` + 8/16 末ロール | `bd:8t`、`bd:8d`、`sd:tr`、`hh:tt`、`hh:hs` | `bd*4`、`[~ cp]*2`、全曲 `bd ~ bd ~`、4 小節ごと `[hh*16]` |
| bass | 8 分ルート＋オクターブ。`bs:sw` / `bs:ht` / `bs:8s`+`.cut(1)` at `C4:` | `bs:su`+`.cut(1)`（8 分のとき） | `0 ~ 0 ~` を既定、サブ重ね、`bs:wb` 主役、1 小節ループ |
| lead | スーパーソー 16 小節リフレイン。子はウェイト 8 | `ld:ss`、`ld:st`、`ld:an`、`ld:us` | ベル、`ld:mx`、`plk:ch`、303、`ld:gr`、ウェイト 9 の `@` |
| hook | ソーのスタブ | `ld:st`、`ld:an`、`plk:ss`（lead が `ld:ss` のとき） | `plk:mx`、`plk:bl`、`plk:mb`、オルゴール |
| arp | 忙しい 8 分 | `ld:ap`、`ld:sw`、`plk:s5` | `plk:fg` / `plk:fc` ガラス（kawaii 側）、1 小節使い回し |
| chords | `[0,4,8]` add9。跳ねる 8 分のスーパーソー | `[0,2,8]`、`plk:ss`、`ld:st`（lead と別 slug） | `[0,2,4]` 三和音、`[0,4,9]` を add9 と呼ぶ、ベルコード |
| pad | 広いソー床を間引く | `pf:sp`、`dr:sl` 以外の薄い `pf:*` | `ps:mx`、`ps:gb`（キラキラは kawaii）、gabber、毎小節撃つ |
| strings | 低いスーパーソードローン。pad とずらす | `dr:sl`、`dr:rw`、`ld:an`（lead が `ld:ss` のとき） | `ld:cr` クワイア、`ld:mx`、violin を invent、`ld:ss`（lead と被る） |

## Why

**ハーフタイムはスネア位置。キックはトラップ。派手さはソー。** 1 サイクル = 1 小節 = 4 拍。`~ ~ sd ~` が拍 3。キックを毎曲 `bd ~ bd ~` にするとダブステップ 30 曲になる。`bd*4` にすると四つ打ちユーロビート／ハウスになり、Future Bass のドロップではなくなる。ユーロビートから借りるのは **スーパーソーの厚みと忙しい arp** だけ。

ハットの 16 分フルバーを 4 小節おきに置くとメトロノームのロールに聞こえる。ロールは末 8 分の `[hh*4]` を **8 小節と 16 小節**に置く。

**8 分ベース。** サブがキックと同じ 2 打だけだと低域が空洞。ルート 8 分にオクターブ `7` を足して跳ねる。ミッドのある `bs:sw` を 1 本。

**16 小節アンセム。** i–VI–III–VII を 8 小節、リフト 4、下降 4。王道 IV–V–iii–vi は kawaii。scale だけ 16 でもドラム／ベース／arp が 1 小節なら短く聞こえる。キック 4 子、ベース・arp 8 子、lead 16 子。

**リフレインとウェイト。** lead の `<>` は 16 子。5–8 と 13–16 が同じ決め。`@` はその子の合計を 8 に保つ。合計 9 の `[4@2 7 9@2  7 4 2 0]` はドラムからずれる（Issue #58）。

**add9 `[0,4,8]`。** 0=根、4=5 度、8=9 度。`[0,4,9]` は 10 度。コードは短いスーパーソー `plk:ss` で 8 分に跳ねる。

**Kawaii との差。** こちらはソーのスタック。ベル・オルゴール・キラキラパッド・王道・クワイアは隣の Skill。ダブステップとの差は wobble を主役にしないこと（同じ 140 でも）。

PCM は `C4:`。シンセサブを `bs:sw` と重ねない。

## レシピ

1. スネアは 3 拍目。キックは表から 1 行。全曲 `bd ~ bd ~` にしない。`bd*4` にしない
2. ハットは 8 分。ロールは 8 / 16 小節目の末だけ。4 小節ごと `[hh*16]` は禁止
3. ベースは 8 分でルート＋オクターブ。`<>` は 8 子以上。`.cut(1)`
4. ループは **16 小節** アンセム。lead はスーパーソーのリフレイン、**各子ウェイト 8**。王道にしない
5. コードは `[0,4,8]` のソー。跳ねる 8 分
6. 上物はスーパーソー族。ベル／オルゴールは kawaii Skill
7. duck はキックだけ。**9 本**。pad と strings のオンオフをずらす
8. フェンスのキック／ベース／lead 次数をキーだけ変えて量産しない

鳴らすのは `dj_hermes_apply_song(content, deck)`（次小節、無書き込み）。`dj_hermes_save_song` は残す指示のときだけ。

## Variations（同じ文法）

| 目的 | 変更 |
| --- | --- |
| カノン 16 小節 | 全 pitched の `.scale` をカノン 8 + 逆転 4 + 下降 4（王道にはしない） |
| キックを sparse / bounce | ドラム表。前の曲と同じキックにしない |
| リード動機 | lift / fall / skip / hold を差し替え（ウェイト 8） |
| スタブを厚く | hook を `ld:an`（lead は `ld:ss` のまま） |
| DnB 時計 | `setcpm(174/4)` にして drums を [strudel-genre-dnb](../strudel-genre-dnb/SKILL.md) の break `*2` に差し替え。**174 は solo** |
| Kawaii にしたい | この Skill を使わない。**strudel-genre-kawaii-future-bass** |

## Pitfalls

1. `bd*4` や `[~ cp]*2` でハウス／四つ打ちユーロビート化する
2. ドラムを kick / snare / hat の 3 `$:` に分ける（キック分離以外）
3. ハットへ `duckorbit`
4. wobble `.lpf(sine.rangex(…))` を主役にする（それは dubstep）
5. サブを重ねる
6. 長い PCM を毎小節撃つ
7. `note("c3'maj")` は root 単音。和音は `[0,4,8]`
8. 王道／小室／ベル／オルゴール／キラキラパッドをこの床に載せる（それは kawaii）
9. 140 と 174 を `dj` する
10. 「アニソン」とだけ書いてこの Skill で王道を書く
11. フェンスの `.s()` とキック／ベース／lead 次数を全コピーする
12. 4 小節アンセムだけでループする。lead にリフレインが無い
13. strings を足さずに 8 本のまま出す。16 引数の `cat`
14. 全曲 `bd ~ bd ~` + 4 小節ごと `[hh*16]`
15. ベース `0 ~ 0 <0 4 0 2>`
16. lead `[4@2 7 9@2  7 4 2 0]`（ウェイト 9）
17. `[0,4,9]` を add9 と呼ぶ。9 度は次数 **8**
18. lead と chords の両方を `ld:ss` にする

## Checklist

- [ ] `setcpm(140/4)` + **9 `$:`**（kick, hats, bass, lead, hook, arp, chords, pad, strings）
- [ ] スネアは拍 3。キックは表の 1 行。`bd*4` ではない
- [ ] ハットは `hh*8`。ロールは 8 / 16 小節目の末だけ
- [ ] ベースは 8 分ルート＋オクターブ、`<>` 8 子以上、`.cut(1)`
- [ ] **16 小節** アンセム `.scale`（16 子）。王道ではない。`cat` ではない
- [ ] lead がスーパーソーのリフレイン。**各 `<>` 子のウェイトが 8**
- [ ] chords `[0,4,8]` のソー。ベル／オルゴールなし
- [ ] ベース PCM は `C4:`、orbit 2、1 本。ハットに duckorbit なし
- [ ] `.s()` は音色パレット（スーパーソー族。kawaii ベルは使わない）
- [ ] `dj_hermes_apply_song(content, deck)`（save は残す指示のときだけ）
