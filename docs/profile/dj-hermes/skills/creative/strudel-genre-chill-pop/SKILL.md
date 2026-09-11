---
name: strudel-genre-chill-pop
description: >-
  Use when writing chill-pop for dj-hermes: Japanese city pop
  (IV–iii–ii–I 下降, maj7, Rhodes), 95–110 BPM. Not EDM I–I–IV–I,
  not 王道進行, not downtempo chill, not house [~ cp]*2.
version: 6.5.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [dj-hermes, music, genre, chill-pop, city-pop]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-kawaii-future-bass
      - strudel-genre-future-bass
---

# dj-hermes × チルポップ（日本のシティポップ）

## Overview

このリポジトリでのチルポップは、**日本のシティポップ**（夕暮れの下降進行、maj7、Rhodes、ナイロンプラック）。BPM 目安 95–110。フェンスは **100**（`setcpm(100/4)`）。

来場者が「チルポップ」と言ったら下表の **シティポップ下降** を書く。単語だけでは Western の I–V–vi–IV や EDM の I–I–IV–I に落ちる。アニソンの **王道進行** は [strudel-genre-kawaii-future-bass](../strudel-genre-kawaii-future-bass/SKILL.md) 側。スーパーソーの Future Bass は [strudel-genre-future-bass](../strudel-genre-future-bass/SKILL.md)。

チル（短調ダウンテンポ）よりコードが前面。ハウス clap は使わない。
`songs/chill-pop/01.strudel` はこのフェンスと同じ（8 本。`// vox` 込み）。

## When

- 来場者が **チルポップ / シティポップ / 柔らかいポップ** を求めるとき
- 「下降進行」「maj7」「Rhodes」と名前を出したとき
- 短調ダウンテンポのチル、パッド主のアンビエント、`[~ cp]*2` のハウス、王道／アニソン床ではないとき
- 新規 apply / プリセット（2–5 本では足りない）

## 進行（Hermes は名前で書く）

親キー C。次数は固定、`.scale("<…>")` の 4 子が 1 コード／小節。pitched 全部で同じ進行（オクターブだけトラックで変える）。PCM フロア / lead / hook / chords（`bs:su` / `ep:*` / `plk:*`）は `C4:` 帯。**pad は C2。**

| 名前 | 度数 | C 親キー | `.scale`（C4 帯の例） |
| --- | --- | --- | --- |
| **シティポップ下降**（既定。「チルポップ」「シティポップ」「夕暮れ」） | IV–iii–ii–I | F–Em–Dm–C | `<F4:lydian E4:phrygian D4:dorian C4:major>` |
| **ウェストコースト循環**（「AOR」「循環」） | I–VI–II–V | C–A7–D7–G7 | `<C4:major A4:mixolydian D4:mixolydian G4:mixolydian>` |
| ツーファイブ。来場者が「ii–V」「ジャズ mill」と言ったとき | ii–V–I–vi | Dm–G–C–Am | `<D4:dorian G4:mixolydian C4:major A4:minor>` |

**取り違えない。** 王道は IV–V–iii–vi（アニソン）。カノンは I–V–vi–IV。どちらもこの床の既定ではない。

下降のモードは C のダイアトニック（IV=lydian、iii=phrygian、ii=dorian、I=major）。`F4:major` にすると Bb が入る。循環の VI/II/V だけ mixolydian（ドミナント 7th）。

## Pattern

8 本（`// vox` 込み）。進行は **シティポップ下降**。メロは順次進行＋ 7 度（次数 6）。アニソンの長い `@` にはしない。

```
// @title visitor-chill-pop
// @genre chill-pop
setcpm(100/4)
// drums
$: s("bd ~ bd ~, [~ sd]*2, hh*8, <~ ~ ~ [bd sd bd sd]>").gain(0.5)
// bass — PCM sub (C3 recording → write C4)
$: note("0 ~ 4 0  0 ~ <4 7 2 0>").scale("<F4:lydian E4:phrygian D4:dorian C4:major>")
  .s("bs:su").gain(0.42)
// lead — stepwise, 7th color, off-beat entry
$: note("~ 2 4 6  4 2 ~ <0 2 4 6>").scale("<F4:lydian E4:phrygian D4:dorian C4:major>")
  .s("plk:ps").adsr("0.01:0.3:0.7:0.2").cut(1).gain(0.16)
// hook — Rhodes (C3 recording → C4 scale)
$: note("0 ~ 2 6  ~ 4 2 <6 4 2 0>").scale("<F4:lydian E4:phrygian D4:dorian C4:major>")
  .s("ep:rs").gain(0.22)
// arp — nylon, includes 7th
$: note("0 2 4 6  4 2 0 4").scale("<F5:lydian E5:phrygian D5:dorian C5:major>")
  .s("plk:ny").gain(0.12).cut(1)
// chords — maj7 / m7 (root, 3rd, 7th); hold, not triangle
$: note("[0,2,6]@2 [0,2,6]@2").scale("<F4:lydian E4:phrygian D4:dorian C4:major>")
  .s("ep:mt").gain(0.24).room(0.25).orbit(2)
// pad — pf:ff is already a fifth; do not write [0,4]
$: note("0").scale("<F2:lydian E2:phrygian D2:dorian C2:major>")
  .s("pf:ff").adsr("0.2:0.4:0.5:0.4").cut(1).gain(0.14).room(0.3).orbit(2)
// vox — sparse city-pop grain, not 16th EDM
$: note("~ 0 ~ ~  ~ ~ ~ ~").scale("<F4:lydian E4:phrygian D4:dorian C4:major>")
  .s("vc:na").gain(0.12).cut(1)
```

`ep:rs` / `ep:mt` / `bs:su` / `pf:ff` は C3 録音。native にするには `.scale("C4:…")` 帯（このフェンスは F4 始まり）。コードはポリフォニックなので `.cut` しない。

## 音色パレット（新規 apply はここから選ぶ）

Pattern はグリッド・次数・スロットの見本。新規曲は下表からスロットごとに 1 つ選び、このフェンスの `.s()` を毎回コピーしない。同一曲の pitched 2 本に同じ `.s()` を使わない。slug の意味は strudel-pcm-catalog の INDEX。lead / pad の PCM は `.s()` の直後に `.adsr`、そのあと `.cut(1)`。FX ライザーだけ `<>` 間引き。シティポップ下降と maj7 `[0,2,6]` は残す。

| スロット | 芯 | 代替 | 禁止 |
| --- | --- | --- | --- |
| drums | キック 1 と 3、スネア 2/4 | `bd:dc` / `bd:lf`、`sd:pp` / `sd:br` | `[~ cp]*2`、`bd:gb`、`hh*8` のハウス化 |
| bass | サブ 1 本 | `bs:su`、`bs:ht`（重ねない） | `bs:hf` 重ね、`bs:wb` |
| lead | 順次＋次数 6 | `plk:ps`、`ld:ny`、`plk:gm` | `ld:ss` アンセム、長い `@` アニソン |
| hook | Rhodes | `ep:rs`、`ep:wr`、`ep:rh` | `plk:ss`、303 |
| arp | ナイロン／爪 | `plk:ny`、`plk:hp`、`plk:kt` | `plk:dt` |
| chords | `[0,2,6]` | `ep:mt`、`ep:ky`、`plk:ep` | `[0,4,9]`、`triangle` |
| pad | **C2** + `.adsr` | `pf:iv`、`pf:ln`、`pf:cl`、`pf:wa`、`pf:ff`+`note("0")` | C4、ADSR なし、`ld:ss`、gabber、`dr:hr`、王道進行 |
| vox | 任意。色として疎、`.cut(1)` | `vc:na` at `C4:` | 16 分 EDM チョップ、ドロップの `vc:yeah`、自前 WAV を invent |

## Why

**シティポップ下降。** Fmaj7–Em7–Dm7–Cmaj7。ルートが 1 度ずつ下がる。I–I–IV–I（旧フェンス）は EDM の明るいループで、日本のシティポップではない。

**maj7 `[0,2,6]`。** 次数 6 が 7 度。三和音 `[0,2,4]` だけだとポップ一般になって 7th の色が消える。Kawaii の add9 `[0,4,8]` や Future Bass のソー壁にはしない。コードは `ep:mt`（フックの `ep:rs` と被らない）。パッドは `pf:ff` の次数 `0`（録音が 5 度。`[0,4]` で重ねない）。

**メロ。** 順次進行（2–4–6）、裏から入る `~`、7 度を色にする。アニソンの `4@2 7 9@2` のような長い伸ばしと跳躍は kawaii 側。

**Rhodes + ナイロン。** フック `ep:rs`、リード `plk:ps`、arp `plk:ny`。ベルは kawaii 床。スーパーソーは future-bass 床。

**ドラム。** キック 1 と 3、スネア 2/4、ハット 16 分。ハウス `cp` は載せない。`bd*4` にすると四つ打ち EDM になる。

## レシピ

- トラックは 8 本: `// drums` `// bass` `// lead` `// hook` `// arp` `// chords` `// pad` `// vox`（新規 apply は 7 本でも可。perc は足さない）
- ドラムは 1 本の `$:`。キック／スネア／ハットに分けない。ハウス `cp` は載せない
- 進行は **シティポップ下降**（来場者が循環／ツーファイブと名前を出したら差し替え）
- ピッチトラックは 4 小節 `.scale`（bass / lead / hook / chords は C4 帯、arp は C5、**pad は C2**）
- PCM フロアは `C4:`。**pad は C2** + `.adsr`。`bs:su` と `bs:hf` は重ねない
- コードは `[0,2,6]` を 3 音まで。パッドは音色パレット（`pf:ff` なら `note("0")`）
- フックは `ep:rs`（C4 帯）。lead PCM は `.s(…).adsr(…)`。リード `plk:ps` と arp `plk:ny` に `.cut(1)`。arp はメロディ楽器（`perc:` ではない）
- メロ／コード／パッドに `triangle` / `sine` / `sawtooth` を使わない
- 長い PCM を ADSR なしで毎小節撃たない。lead / pad は `.adsr` + `.cut(1)` ならグリッド可。`plk:*` / `ep:*` / `bs:*` / `vc:*`、波形、`wt_*`、ライブ `.fm` も使える

鳴らすのは `dj_hermes_apply_song(content, deck)`（次小節、無書き込み）。`dj_hermes_save_song` は残す指示のときだけ。

## Variations（同じ文法）

| 目的 | 変更 |
| --- | --- |
| ウェストコースト循環 | 全 pitched の `.scale` を I–VI–II–V（C4 帯。arp は C5） |
| ツーファイブ | 全 pitched の `.scale` を ii–V–I–vi（C4 帯。arp は C5） |
| ベースを動かして | bass の次数末尾 `<>` だけ（全文作り直さない） |
| もっと明るい | 梯子で major → lydian（全 pitched の mode を同期。下降の IV は既に lydian） |

## Pitfalls

1. コードに歪みを載せすぎる
2. `stack` / `.cpm` / `.lfo`
3. `note("c3'maj")` は root 単音。和音は `[0,2,6]`
4. `ep:rs` に `.scale("C3:…")` を付ける（1 オクターブ下がる）
5. ハウス `[~ cp]*2` を載せる
6. 新規 apply を 3 本のまま出す
7. 100 BPM を 90 チルや 84 ローファイと DJ ペアにする（Transport は 1 つ）
8. I–I–IV–I や王道 IV–V–iii–vi を既定にする（王道は kawaii-future-bass）
9. メロをアニソンの長い `@` にする。コードを add9 `[0,4,8]` にする
10. 「チルポップ」とだけ書いて `.scale` を省略する
11. コード／パッド／メロを `triangle` / `sine` / `sawtooth` にする
12. 新規 apply でフェンスの `.s()` を全コピーする。`ld:ss` や `plk:ss` を載せる

## Checklist

- [ ] 8 本 bundled（drums / bass / lead / hook / arp / chords / pad / vox）。新規 apply は 7 本でも可
- [ ] 4 小節 **シティポップ下降** `<F:lydian E:phrygian D:dorian C:major>`（循環／ツーファイブは名前付き差し替え）
- [ ] chords `[0,2,6]`。lead は順次＋次数 6。PCM フロア / lead / hook / chords は C4 帯。**pad は C2** + `.adsr`。lead PCM は `.adsr`。arp はメロディ楽器
- [ ] ドラムは 1 本。ハウス `cp` なし。`.s()` は音色パレット
- [ ] `dj_hermes_apply_song(content, deck)`。save は残す指示のときだけ
