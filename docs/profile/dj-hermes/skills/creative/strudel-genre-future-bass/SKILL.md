---
name: strudel-genre-future-bass
description: >-
  Use when writing Future Bass for strudel-rs: 140 BPM half-time 2-step
  (dubstep/DnB grid), kawaii bells/supersaw, J-pop 王道進行 (IV–V–iii–vi)
  or 小室進行 (vi–IV–V–I). Not four-on-the-floor. Default is Kawaii Future Bass.
version: 6.1.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, future-bass, kawaii, j-pop]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-dubstep
      - strudel-genre-dnb
      - strudel-genre-techno-duck
---

# strudel-rs × Future Bass（Kawaii / J-pop 王道載せ）

## Overview

このリポジトリでの Future Bass は、**ダブステップやドラムンベースのハーフタイムの上に、J-pop 王道進行の長いリードとキラキラした音色を載せたもの**。端的な参照は **Kawaii Future Bass**（Snail's House / Ujico*「Nyan Nyan Angel!」周辺）。

来場者が「アニソン」と言ったら **王道進行** を書く。単語「アニソン」だけでは `.scale` が決まらない。

リズムは [strudel-genre-dubstep](../strudel-genre-dubstep/SKILL.md) と同じ **140 の 2 ステップ**（1 と 3 にキック、スネアは 3）。上物だけが違う。4 つ打ちハウスではない。

| 層 | 取るもの | 取らないもの |
| --- | --- | --- |
| ドラム | ハーフタイム 2 ステップ（体感は BPM の半分） | `bd*4`、ハウス `[~ cp]*2` |
| ベース | キックに揃うサブ 1 本 | ダブステップの wobble（`.lpf(sine.rangex)`）を主役にしない |
| 上物 | 長いリード、ベル／オルゴール、add9、**王道進行**（または小室） | 全部 `triangle`、暗い phrygian 床、名前のない「アニソン進行」 |

テンポ既定は **140**（ダブステップと同じ時計。ペア可）。174 のブレイクに同じ上物を載せるのは別バリエーションで、そのときは **solo**（140 と混ぜない）。

## When

- 依頼が **Future Bass / Kawaii Future Bass / カワイイベース** のとき
- 「ダブステップ（または DnB）にアニソン／J-pop を載せたい」とき → 進行は下表の **王道** が既定
- 来場者が **王道進行** または **小室進行** と名前を出したとき（この床の上）
- 四つ打ちの明るい EDM として書かないとき（それはプログレ／ハウス側）

## 進行（Hermes は名前で書く）

親キー C。次数は固定、`.scale("<…>")` の 4 子が 1 コード／小節。pitched 全部で同じ進行（オクターブだけトラックで変えてよい）。PCM は `C4:` 帯（`SAMPLE_ROOT_HZ` は C4）。

| 名前 | 度数 | C 親キー | `.scale("<…>")` |
| --- | --- | --- | --- |
| **王道進行**（既定。「アニソン」「J-pop 王道」） | IV–V–iii–vi | F–G–Em–Am | `<F4:lydian G4:mixolydian E4:phrygian A4:minor>` |
| **小室進行**（「小室」「90s J-pop」） | vi–IV–V–I | Am–F–G–C | `<A4:minor F4:lydian G4:mixolydian C4:major>` |
| カノン進行（短）。来場者が「カノン」「I–V–vi–IV」と言ったときだけ | I–V–vi–IV | C–G–Am–F | `<C4:major G4:mixolydian A4:minor F4:lydian>` |

**取り違えない。** I–V–vi–IV はカノンであって王道ではない。王道は **IV–V–iii–vi**。

モードは C のダイアトニック（IV=lydian、V=mixolydian、iii=phrygian、vi=minor、I=major）。`F4:major` にすると Bb が入って親キーを外れる。

## Pattern

duck でキックを分離する（Future Bass のポンピング）。ハットに `duckorbit` は付けない。bass / chords / pad を orbit 2。lead / hook / arp はドライのままメロを前に出す。8 本。進行は **王道**。

```
// @title visitor-future-bass
// @genre future-bass
setcpm(140/4)
// kick — 2-step: beats 1 and 3
$: s("bd ~ bd ~").gain(0.85).duckorbit(2).duckattack(0.05).duckdepth(0.8)
// hats — snare on 3; 16th hats; 4th-bar chatter
$: s("~ ~ sd ~, hh*8, <~ ~ ~ [hh*16]>").gain(0.42)
// bass — hits with the kicks; PCM native at C4; 王道 IV–V–iii–vi
$: note("0 ~ 0 <0 4 0 2>").scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor>")
  .s("bs:su").gain(0.4).orbit(2)
// lead — long notes (not 16ths)
$: note("4@2 7 9@2  7 4 <2 0> ~").scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor>")
  .s("ld:ss").gain(0.16).cut(1)
// hook — kawaii bell / music box
$: note("~ 11 ~ 12  ~ 9 ~ <11 12>").scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor>")
  .s("plk:mx").gain(0.18).cut(1)
// arp — glass pluck as vocal-chop stand-in (no vocal wav)
$: note("0 4 ~ 7  4 ~ 9 4").scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor>")
  .s("plk:fg").gain(0.14).cut(1)
// chords — open add9, ducked
$: note("[0,4,9] ~ [0,4,9] ~").scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor>")
  .s("sawtooth").lpf(1800).gain(0.22).orbit(2)
  .attack(0.02).decay(0.2).sustain(0.4).release(0.15)
// pad
$: note("[0,4]").scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor>")
  .s("sine").fm(1.2).fmh(1).fmdec(0.8).fmsus(0.4)
  .room(0.3).orbit(2).gain(0.14)
```

同梱 `songs/future-bass/01.strudel` は古い 4 つ打ちの薄いデモ。新規 apply はこのフェンス。ディスクのグリッドをコピーしない。

## Why

**ハーフタイム 2 ステップ。** 1 サイクル = 1 小節 = 4 拍。`bd ~ bd ~` は拍 1 と 3。`~ ~ sd ~` は拍 3 のスネア。体感は 70 BPM のゆっくりした踏みで、ハットの 16 分だけ速い。`bd*4` だとハウス／テクノになり、Future Bass のドロップではなくなる。

うたてん解説（BPM 70–90 または 140 前後、ハーフタイムの 2 ステップ、ドロップで盛り上げ）と同じ軸。Kawaii 側のキラキラは **上物の音色** で出す（ベル、オルゴール、ガラスプラック、スーパーソー）。チップチューン寄りの矩形は lead の差し替えで可。

**長いリード。** lead は `@` で伸ばす。16 分で埋めない。和声は上表。次数は固定、`.scale("<…>")` が 4 小節。

**add9 `[0,4,9]`。** Future Bass ドロップの開いたボイシング（根音・5 度・9 度）。三和音 `[0,2,4]` に戻さない。パッドは 5 度 `[0,4]` のまま（コードと帯域を分ける）。

**ヴォーカルチョップ。** このエンジンに歌サンプルは無い。短い `plk:fg` / `plk:ss` を arp に置く。無いキーを invent しない。

**duck。** キックが orbit 2 を潰す。bass / chords / pad を同じ orbit に載せる。lead / hook は orbit 1 のままメロが埋もれないようにする。ハットに `duckorbit` を付けると 16 分のたびにエンベロープが再トリガする。

**ダブステップとの差。** 同じ 140・同じ 2 ステップでも、ダブステップ Skill の主役は wobble ベース。こちらはサブをキックに揃え、主役はキラキラした上物。`.lpf(sine.rangex(80, 600))` をこの床に載せない。

PCM は `C4:`（`SAMPLE_ROOT_HZ` は C4。`bs:su` / `ld:ss` / `plk:mx` / `plk:fg` は native）。シンセサブを足して `bs:su` と重ねない。

## レシピ

1. ドラムは 2 ステップ。`bd*4` にしない。clap を 2/4 に置かない
2. 上物は長い lead + kawaii（ベル／ガラス）。進行は **王道**（来場者が小室／カノンと名前を出したら差し替え）
3. コードは `[0,4,9]`。`.scale` は上表の 4 子。I–V–vi–IV を王道と呼ばない
4. サブは `bs:su` 1 本、orbit 2。wobble を主役にしない
5. duck はキックだけ。ハットに duckorbit 禁止。合計 8 本

鳴らすのは `strudel_apply_song(content, deck)`（次小節、無書き込み）。`strudel_save_song` は残す指示のときだけ。

## Variations（同じ文法）

| 目的 | 変更 |
| --- | --- |
| 小室進行 | 全 pitched の `.scale` を `<A4:minor F4:lydian G4:mixolydian C4:major>` |
| カノン進行 | 全 pitched の `.scale` を `<C4:major G4:mixolydian A4:minor F4:lydian>` |
| チップチューン寄り | lead を `.s("square").lpf(3200)` |
| ベルをチャイムに | hook を `plk:ch`（`plk:mx` の代わり） |
| チョップを短く | arp を `plk:ss` + `.cut(1)` |
| DnB 時計 | `setcpm(174/4)` にして drums を [strudel-genre-dnb](../strudel-genre-dnb/SKILL.md) の break `*2` に差し替え。上物はこのフェンスのまま。**174 は solo**（140 と DJ しない） |
| もっと明るい | 梯子で major → lydian（全 pitched の mode を同期。王道の IV は既に lydian） |

## Pitfalls

1. `bd*4` や `[~ cp]*2` でハウス化する
2. ドラムを kick / snare / hat の 3 `$:` に分ける（キック分離以外）
3. ハットへ `duckorbit`（16 分が env を retrigger）
4. wobble `.lpf(sine.rangex(…))` をこの床の主役にする（それは dubstep Skill）
5. `bs:su` の上に square サブ / `bs:hf` / `bs:dk` を重ねる
6. `in_bank=no` の `ld:ac` / `pf:al` / `dr:*` / `ps:*` → 無音。長い PCM は `ld:ss` と `pf:ff`
7. `note("c3'maj")` は root 単音。和音は `[0,4,9]`
8. ディスクの `songs/future-bass/01.strudel`（4 つ打ち）を完成形としてコピーする
9. 140 と 174 を `dj` する（共有時計。片方の BPM が捨てられる）
10. 「アニソン」とだけ書いて `.scale` を省略する、または I–V–vi–IV を王道として書く

## Checklist

- [ ] `setcpm(140/4)` + **8 `$:`**（kick, hats, bass, lead, hook, arp, chords, pad）
- [ ] 2 ステップ（`bd ~ bd ~` + 拍 3 の `sd`）。`bd*4` ではない
- [ ] 上物が長い lead + kawaii（ベル、ガラス）。暗い wobble 床ではない
- [ ] chords `[0,4,9]`。進行は **王道** `<F4:lydian G4:mixolydian E4:phrygian A4:minor>`（小室／カノンは名前付き差し替え）
- [ ] `bs:su` は `C4:`、orbit 2、サブ 1 本。ハットに duckorbit なし
- [ ] `strudel_apply_song(content, deck)`（save は残す指示のときだけ）
