---
name: strudel-genre-future-bass
description: >-
  Use when writing Future Bass for dj-hermes: 140 BPM half-time 2-step,
  16-bar loop (not 4-bar), refrain lead, long strings track, J-pop 王道
  plus reverse and bass cliché so songs do not share one progression.
  Kawaii bells/supersaw. Not four-on-the-floor.
version: 6.3.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [dj-hermes, music, genre, future-bass, kawaii, j-pop]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-dubstep
      - strudel-genre-dnb
      - strudel-genre-techno-duck
---

# dj-hermes × Future Bass（Kawaii / J-pop 王道載せ）

## Overview

このリポジトリでの Future Bass は、**ダブステップやドラムンベースのハーフタイムの上に、J-pop 進行の長いリードとキラキラした音色を載せたもの**。端的な参照は **Kawaii Future Bass**（Snail's House / Ujico*「Nyan Nyan Angel!」周辺）。

来場者が「アニソン」と言ったら **王道進行** を書く。単語「アニソン」だけでは `.scale` が決まらない。

リズムは [strudel-genre-dubstep](../strudel-genre-dubstep/SKILL.md) と同じ **140 の 2 ステップ**（1 と 3 にキック、スネアは 3）。上物だけが違う。4 つ打ちハウスではない。

**ループは 16 小節**（4 小節の 4 倍）。4 小節 `.scale` だけだとすぐ一周して飽きる。`cat` は使わない（16 子の `.scale("<…>")`）。

| 層 | 取るもの | 取らないもの |
| --- | --- | --- |
| ドラム | ハーフタイム 2 ステップ（体感は BPM の半分） | `bd*4`、ハウス `[~ cp]*2` |
| ベース | キックに揃うサブ 1 本 | ダブステップの wobble（`.lpf(sine.rangex)`）を主役にしない |
| 上物 | **リフレインする**長いリード、ベル／オルゴール、add9、**16 小節の進行**（王道＋逆転＋クリシェ）、長いストリングス | 全部 `triangle`、暗い phrygian 床、4 小節王道の使い回し、名前のない「アニソン進行」 |

テンポ既定は **140**（ダブステップと同じ時計。ペア可）。174 のブレイクに同じ上物を載せるのは別バリエーションで、そのときは **solo**（140 と混ぜない）。

## When

- 依頼が **Future Bass / Kawaii Future Bass / カワイイベース** のとき
- 「ダブステップ（または DnB）にアニソン／J-pop を載せたい」とき → 進行は下の **16 小節フォーム**（王道が前半の既定）
- 来場者が **王道進行** または **小室進行** と名前を出したとき（この床の上。4 小節だけでは出さない）
- 四つ打ちの明るい EDM として書かないとき（それはプログレ／ハウス側）

## 進行（16 小節。Hermes は名前で書く）

親キー C。次数は固定、`.scale("<…>")` の **16 子**が 1 コード／小節。pitched 全部で同じ 16 小節（オクターブだけトラックで変えてよい）。PCM は `C4:` 帯（`SAMPLE_ROOT_HZ` は C4）。arp は C5。

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
| **カノン**（来場者が「カノン」「I–V–vi–IV」と言ったときだけ） | カノン 8 + カノン逆転 4 + クリシェ 4。カノン 4 子: `C4:major G4:mixolydian A4:minor F4:lydian`。逆転: `F4:lydian A4:minor G4:mixolydian C4:major`（IV–vi–V–I） |

**取り違えない。** I–V–vi–IV はカノンであって王道ではない。王道は **IV–V–iii–vi**。王道の逆転は **vi–iii–V–IV** であり、カノンでも小室でもない。

**クリシェ**はここでは **下行バス**（I のルートから B–A–G）。次数 0 がルートを辿るので、ベースは `0` のままで C–B–A–G になる。半音クロマティック（Bb を挟む）は書かない（親キー C のダイアトニックを外れる）。

モードは C のダイアトニック（IV=lydian、V=mixolydian、iii=phrygian、vi=minor、I=major、vii=locrian）。`F4:major` にすると Bb が入って親キーを外れる。

既定 16 子（pitched にこれを載せる。arp はオクターブ 5）:

```
<F4:lydian G4:mixolydian E4:phrygian A4:minor F4:lydian G4:mixolydian E4:phrygian A4:minor A4:minor E4:phrygian G4:mixolydian F4:lydian C4:major B4:locrian A4:minor G4:mixolydian>
```

## Pattern

duck でキックを分離する。ハットに `duckorbit` は付けない。bass / chords / pad / **strings** を orbit 2。lead / hook / arp はドライのままメロを前に出す。**9 本**（composition の Future Bass 例外）。進行は上の **A A' B C**。

`// lead` は **リフレイン**: 1–4 小節は前振り（休符多め）、5–8 と 13–16 が同じ決めフレーズ、9–12 は逆転の上で変化。決めフレーズを 16 小節ずっと変えない、逆に 4 小節で全部終わらせない。

```
// @title visitor-future-bass
// @genre future-bass
setcpm(140/4)
// kick — 2-step: beats 1 and 3
$: s("bd ~ bd ~").gain(0.85).duckorbit(2).duckattack(0.05).duckdepth(0.8)
// hats — snare on 3; 16th hats; fill every 4th of 16
$: s("~ ~ sd ~, hh*8, <~ ~ ~ [hh*16] ~ ~ ~ [hh*16] ~ ~ ~ [hh*16] ~ ~ ~ [hh*16]>").gain(0.42)
// bass — hits with the kicks; PCM native at C4; 16-bar A A' B C
$: note("0 ~ 0 <0 4 0 2>").scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor F4:lydian G4:mixolydian E4:phrygian A4:minor A4:minor E4:phrygian G4:mixolydian F4:lydian C4:major B4:locrian A4:minor G4:mixolydian>")
  .s("bs:su").gain(0.4).orbit(2)
// lead — bars 1–4 setup, 5–8 refrain, 9–12 reverse, 13–16 refrain return
$: note("<[~ 4 ~ 7  ~ 9 4 2] [~ 7 4 9  7 ~ 4 2] [4 ~ 9 7  ~ 4 2 0] [~ 4 7 9  4 2 ~ 7] [4@2 7 9@2  7 4 2 0] [4@2 7 9@2  7 4 0 2] [4@2 7 9@2  7 4 2 ~] [4@2 7 9@2  7 4 2 0] [~ 9 7 4  2 0 ~ 4] [9 ~ 7 4  ~ 2 0 4] [7 4 ~ 2  0 ~ 4 7] [~ 4 2 0  4 7 ~ 9] [4@2 7 9@2  7 4 2 0] [4@2 7 9@2  7 4 0 2] [4@2 7 9@2  7 4 2 ~] [4@2 7 9@2  7 4 2 0]>")
  .scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor F4:lydian G4:mixolydian E4:phrygian A4:minor A4:minor E4:phrygian G4:mixolydian F4:lydian C4:major B4:locrian A4:minor G4:mixolydian>")
  .s("ld:ss").gain(0.16).cut(1)
// hook — kawaii bell; denser on refrain bars
$: note("<[~ 11 ~ 12] [~ 9 ~ 11] [~ 11 ~ 12] [~ 9 ~ 7] [~ 11 ~ 12] [~ 9 ~ 11] [~ 11 ~ 12] [11 12 9 11] [~ 12 ~ 9] [~ 11 ~ 7] [~ 9 ~ 4] [~ 7 ~ 4] [~ 11 ~ 12] [~ 9 ~ 11] [~ 11 ~ 12] [11 12 9 11]>")
  .scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor F4:lydian G4:mixolydian E4:phrygian A4:minor A4:minor E4:phrygian G4:mixolydian F4:lydian C4:major B4:locrian A4:minor G4:mixolydian>")
  .s("plk:mx").gain(0.18).cut(1)
// arp — glass pluck as vocal-chop stand-in (no vocal wav)
$: note("0 4 ~ 7  4 ~ 9 4").scale("<F5:lydian G5:mixolydian E5:phrygian A5:minor F5:lydian G5:mixolydian E5:phrygian A5:minor A5:minor E5:phrygian G5:mixolydian F5:lydian C5:major B5:locrian A5:minor G5:mixolydian>")
  .s("plk:fg").gain(0.14).cut(1)
// chords — open add9, ducked
$: note("[0,4,9] ~ [0,4,9] ~").scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor F4:lydian G4:mixolydian E4:phrygian A4:minor A4:minor E4:phrygian G4:mixolydian F4:lydian C4:major B4:locrian A4:minor G4:mixolydian>")
  .s("plk:ss").gain(0.2).orbit(2)
// pad
$: note("<0 ~ ~ ~>").scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor F4:lydian G4:mixolydian E4:phrygian A4:minor A4:minor E4:phrygian G4:mixolydian F4:lydian C4:major B4:locrian A4:minor G4:mixolydian>")
  .s("pf:ff").gain(0.14).room(0.3).orbit(2)
// strings — long choir/wide (no violin WAV in the catalog)
$: note("<0 ~ ~ ~>").scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor F4:lydian G4:mixolydian E4:phrygian A4:minor A4:minor E4:phrygian G4:mixolydian F4:lydian C4:major B4:locrian A4:minor G4:mixolydian>")
  .s("ld:cr").gain(0.12).room(0.45).orbit(2)
```

同梱 `songs/future-bass/01.strudel` はこのフェンスと同じ（2 ステップ、16 小節 A A' B C、リフレイン lead、strings）。新規 apply の `.s()` は下のパレットから選ぶ。

## 音色パレット（新規 apply はここから選ぶ）

Pattern はグリッド・次数・スロットの見本。新規曲は下表からスロットごとに 1 つ選び、このフェンスの `.s()` を毎回コピーしない。同一曲の pitched 2 本に同じ `.s()` を使わない。slug の意味は strudel-pcm-catalog の INDEX。長尺（`ld:` / `dr:` / `pf:` / `ps:`、`plk:fp` / `plk:sp`）は `.cut(1)` か `note("<0 ~ ~ ~>")`。wobble を主役にしない。カタログに violin は無い。ストリングス役はクワイア／広いパッドで代用する。

| スロット | 芯 | 代替 | 禁止 |
| --- | --- | --- | --- |
| drums | 2 ステップ（`bd ~ bd ~` + 拍 3 の `sd`） | `bd:8t`、`sd:tr`、`hh:ch` | `bd*4`、`[~ cp]*2` |
| bass | `bs:su` at `C4:` | （サブは 1 本） | `bs:hf` / square サブを重ねる、`bs:wb` 主役 |
| lead | 16 小節リフレイン（長いノート） | `ld:ss`、`ld:st`、`ld:mx`、`.s("square").lpf(3200)` | 303、`ld:gr`、1 小節だけ変えて終わり |
| hook | kawaii | `plk:mx`、`plk:ch`、`plk:bl`、`plk:mb` | wobble、`plk:dt` |
| arp | チョップ代用 | `plk:fg`、`plk:fc` | ボーカル WAV を invent |
| chords | `[0,4,9]` | `plk:ss`、`plk:sm` | `[0,2,4]` に戻す |
| pad | | `ps:mx`、`ps:gb`、`pf:sp`、`pf:ga`、`pf:ff`+`note("<0 ~ ~ ~>")` | `dr:hr`、gabber |
| strings | 長いホールド | `ld:cr`、`pf:ca`、`pf:hl`、`pf:wm` | 毎小節撃つ、`ld:ss`（lead と被る）、violin を invent |

## Why

**ハーフタイム 2 ステップ。** 1 サイクル = 1 小節 = 4 拍。`bd ~ bd ~` は拍 1 と 3。`~ ~ sd ~` は拍 3 のスネア。体感は 70 BPM のゆっくりした踏みで、ハットの 16 分だけ速い。`bd*4` だとハウス／テクノになり、Future Bass のドロップではなくなる。

うたてん解説（BPM 70–90 または 140 前後、ハーフタイムの 2 ステップ、ドロップで盛り上げ）と同じ軸。Kawaii 側のキラキラは **上物の音色** で出す（ベル、オルゴール、ガラスプラック、スーパーソー）。チップチューン寄りの矩形は lead の差し替えで可。

**16 小節。** 4 小節王道だけだとドロップがすぐ一周する。16 子の `.scale` で和声が 16 小節。`cat` の 16 引数はライブ差分が重いので使わない。

**リフレイン。** lead の `<>` は 16 子（1 子＝1 小節）。5–8 と 13–16 が同じ決めフレーズ（`4@2 7 9@2  7 4 2 0` 系）。1–4 は入り、9–12 は逆転の上で次数を変える。フックはリフレイン小節だけ粒を増やす。

**逆転とクリシェ。** 30 曲が全部 IV–V–iii–vi の 4 小節だと区別がつかない。後半を vi–iii–V–IV にし、締めを I–vii–vi–V の下行バスにする。小室／カノンは名前が付いたときだけ、同じ 16 小節の組み方で差し替える。

**add9 `[0,4,9]`。** Future Bass ドロップの開いたボイシング（根音・5 度・9 度）。三和音 `[0,2,4]` に戻さない。コードは短いスーパーソー `plk:ss`。パッドとストリングスは `note("<0 ~ ~ ~>")`（4 小節に 1、16 小節で 4 回。ブロック頭に当たる）。

**ストリングス。** カタログに violin は無い。`ld:cr`（クワイア）や `pf:ca` / `pf:hl` / `pf:wm` を長いホールドにする。lead のスーパーソーと `.s()` を共有しない。orbit 2 でキックに duck される。

**ヴォーカルチョップ。** このエンジンに歌サンプルは無い。短い `plk:fg` / `plk:ss` を arp に置く。無いキーを invent しない。

**duck。** キックが orbit 2 を潰す。bass / chords / pad / strings を同じ orbit に載せる。lead / hook は orbit 1 のままメロが埋もれないようにする。ハットに `duckorbit` を付けると 16 分のたびにエンベロープが再トリガする。

**ダブステップとの差。** 同じ 140・同じ 2 ステップでも、ダブステップ Skill の主役は wobble ベース。こちらはサブをキックに揃え、主役はキラキラした上物とリフレイン。`.lpf(sine.rangex(80, 600))` をこの床に載せない。

PCM は `C4:`（`SAMPLE_ROOT_HZ` は C4）。シンセサブを足して `bs:su` と重ねない。

## レシピ

1. ドラムは 2 ステップ。`bd*4` にしない。clap を 2/4 に置かない
2. ループは **16 小節**。上物はリフレイン lead + kawaii。進行は **A A' B C**（来場者が小室／カノンと名前を出したら 16 子を差し替え）
3. コードは `[0,4,9]`。I–V–vi–IV を王道と呼ばない。4 小節王道だけで終わらせない
4. サブは `bs:su` 1 本、orbit 2。wobble を主役にしない
5. duck はキックだけ。ハットに duckorbit 禁止。**9 本**（strings を足す）

鳴らすのは `dj_hermes_apply_song(content, deck)`（次小節、無書き込み）。`dj_hermes_save_song` は残す指示のときだけ。

## Variations（同じ文法）

| 目的 | 変更 |
| --- | --- |
| 小室 16 小節 | 全 pitched の `.scale` を小室 8 + 小室逆転 4 + クリシェ 4 |
| カノン 16 小節 | 全 pitched の `.scale` をカノン 8 + カノン逆転 4 + クリシェ 4 |
| 逆転を前に | B A C A' など 4 ブロックの順だけ入れ替える（16 子は保つ） |
| チップチューン寄り | lead を `.s("square").lpf(3200)` |
| ベルをチャイムに | hook を `plk:ch`（`plk:mx` の代わり） |
| チョップを短く | arp を `plk:ss` + `.cut(1)` |
| ストリングスを空気に | strings を `pf:ca` / `pf:hl`（`ld:cr` の代わり）。毎小節撃たない |
| DnB 時計 | `setcpm(174/4)` にして drums を [strudel-genre-dnb](../strudel-genre-dnb/SKILL.md) の break `*2` に差し替え。上物はこのフェンスのまま。**174 は solo**（140 と DJ しない） |
| もっと明るい | 梯子で major → lydian（全 pitched の mode を同期。王道の IV は既に lydian。クリシェの I は major） |

## Pitfalls

1. `bd*4` や `[~ cp]*2` でハウス化する
2. ドラムを kick / snare / hat の 3 `$:` に分ける（キック分離以外）
3. ハットへ `duckorbit`（16 分が env を retrigger）
4. wobble `.lpf(sine.rangex(…))` をこの床の主役にする（それは dubstep Skill）
5. `bs:su` の上に square サブ / `bs:hf` / `bs:dk` を重ねる
6. 長い PCM（`ld:` / `dr:` / `pf:` / `ps:`）を毎小節撃たない
7. `note("c3'maj")` は root 単音。和音は `[0,4,9]`
8. 四つ打ちのハウス／プログレ曲を Future Bass としてコピーする
9. 140 と 174 を `dj` する（共有時計。片方の BPM が捨てられる）
10. 「アニソン」とだけ書いて `.scale` を省略する、または I–V–vi–IV を王道として書く
11. 新規 apply でフェンスの `.s()` を全コピーする。pad を毎回 `pf:ff` にする
12. 4 小節王道だけでループする。lead にリフレイン（5–8 と 13–16 の繰り返し）が無い
13. strings を足さずに 8 本のまま出す。16 引数の `cat` で 16 小節を組む

## Checklist

- [ ] `setcpm(140/4)` + **9 `$:`**（kick, hats, bass, lead, hook, arp, chords, pad, strings）
- [ ] 2 ステップ（`bd ~ bd ~` + 拍 3 の `sd`）。`bd*4` ではない
- [ ] **16 小節** `.scale`（16 子）。`cat` ではない
- [ ] lead がリフレイン（決めフレーズが後半で戻る）。暗い wobble 床ではない
- [ ] chords `[0,4,9]`。進行は **A A' B C**（小室／カノンは名前付きの 16 小節差し替え）
- [ ] `bs:su` は `C4:`、orbit 2、サブ 1 本。ハットに duckorbit なし。strings も orbit 2
- [ ] `.s()` は音色パレット（ベル／ガラス／スーパーソー／ストリングス役を役割で変える）
- [ ] `dj_hermes_apply_song(content, deck)`（save は残す指示のときだけ）
