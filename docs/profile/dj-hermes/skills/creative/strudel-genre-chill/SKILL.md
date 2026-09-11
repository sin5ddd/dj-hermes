---
name: strudel-genre-chill
description: >-
  Use when writing chill / downtempo for dj-hermes: 80–100 BPM feel,
  soft drums, bs:hf floor only (do not stack bs:su). Not ambient and
  not chill-pop.
version: 5.2.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [dj-hermes, music, genre, chill]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# dj-hermes × チル / ダウンテンポ

## Overview

遅め〜中庸、柔らかいドラム、控えめメロ。BPM 目安 80–100。フェンスは **90**（`setcpm(90/4)`）。
アンビエントよりドラムがある。チルポップほどコードを前面に出さない。
`songs/chill/01.strudel` はこのフェンスと同じ（7 本）。

## When

- 来場者がチル、ダウンテンポ、ゆるいビートを求めるとき
- パッド主のアンビエント、シティポップ下降のチルポップ、ハウス clap ではないとき
- 新規 apply / プリセット（2–5 本では足りない）

## Pattern

```
// @title visitor-chill
// @genre chill
setcpm(90/4)
// drums
$: s("bd:hf ~ ~ bd:hf ~ ~ sd ~, [~ hh]*4, <~ ~ ~ oh>").gain(0.42)
// bass
$: note("0 ~ 2 ~ 0 <3 4 2 0>").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("bs:hf").gain(0.4)
// lead
$: note("~ 4 ~ 7 ~ <6 9 7 4>").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("plk:ps").adsr("0.01:0.3:0.7:0.2").cut(1).gain(0.16)
// hook
$: note("0@2 4 7@2 ~").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("plk:am").gain(0.18).cut(1)
// arp
$: note("~ 7 12 7  4 0 ~ 2").scale("<C5:minor C5:minor F5:dorian C5:minor>")
  .s("plk:hp").gain(0.12).cut(1)
// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("ep:rs").gain(0.22).room(0.3).orbit(2)
// pad
$: note("0").scale("<C2:minor C2:minor F2:dorian C2:minor>")
  .s("pf:ff").adsr("0.2:0.4:0.5:0.4").cut(1).gain(0.16).room(0.35).orbit(2)
```

`bs:hf` のみ。`bs:su` は重ねない。

## 音色パレット（新規 apply はここから選ぶ）

Pattern はグリッド・次数・スロットの見本。新規曲は下表からスロットごとに 1 つ選び、このフェンスの `.s()` を毎回コピーしない。同一曲の pitched 2 本に同じ `.s()` を使わない。slug の意味は strudel-pcm-catalog の INDEX。lead / pad の PCM は `.s()` の直後に `.adsr`、そのあと `.cut(1)`。FX ライザーだけ `<>` 間引き。ドラムの間引きは残す。

| スロット | 芯 | 代替 | 禁止 |
| --- | --- | --- | --- |
| drums | キック間引き、`[~ hh]*4` | `bd:hf` / `bd:lf`、`sd:br` | `hh*8`、house clap、`bd*4` |
| bass | `bs:hf` のみ | （重ねない） | `bs:su` 重ね、`bs:wb` |
| lead | | `plk:ps`、`plk:am`、`ld:ny` | `ld:ss`、スーパーソー |
| hook | | `plk:am`、`ep:rs` | 303、`plk:ss` |
| arp | | `plk:hp`、`plk:kl` | `plk:dt` |
| chords | `[0,2,4]` | `ep:rs`、`ep:mt` | シティポップ maj7 を既定に、`triangle` |
| pad | **C2** + `.adsr` | `pf:cl`、`pf:ln`、`dr:fg`、`pf:ff`+`note("0")` | C4、ADSR なし、gabber、`dr:hr` |
| vox | 任意 8 本目 `// vox`。疎、`.cut(1)` | `vc:na`、`vc:ra` at `C4:` | 16 分埋め、毎小節 `vc:yeah`、自前 WAV を invent |

## レシピ

- トラックは 7 本: `// drums` `// bass` `// lead` `// hook` `// arp` `// chords` `// pad`（任意で perc または vox）
- ドラムは 1 本の `$:`。キック／スネア／ハットに分けない。キックは間引き、ハットは `[~ hh]*4`
- ピッチトラックは 4 小節 `.scale`（bass / lead / hook / chords は C4、arp は C5、**pad は C2**）
- PCM フロアは `C4:`。**pad は C2** + `.adsr`。フロアは `bs:hf` だけ（サブ同士を重ねない）
- コードは 3 音まで。パッドは音色パレット（`pf:ff` なら `note("0")`）
- メロ／コード／パッドに `triangle` / `sine` / `wt_organ` を使わない
- メロは掛け合い。gain 0.12–0.18。lead PCM は `.s(…).adsr(…)`。プラックに `.cut(1)`。arp はメロディ楽器（`perc:` ではない）
- 長い PCM を ADSR なしで毎小節撃たない。lead / pad は `.adsr` + `.cut(1)` ならグリッド可。`plk:*` / `ep:*` / `bs:*` / `vc:*`、波形、`wt_*`、ライブ `.fm` も使える

## Pitfalls

1. 高速 DnB 配置や `hh*8` 連打を持ってくる
2. `stack` / `.cpm` / `.lfo`
3. `bs:su` を `bs:hf` の下に足す
4. `note("c3'maj")` は root 単音。和音は `[0,2,4]`
5. キック／スネア／ハットを 3 本の `$:` に分ける
6. 新規 apply を 3 本のまま出す
7. 90 BPM を 70 アンビエントや 100 チルポップと DJ ペアにする（Transport は 1 つ）
8. 新規 apply でフェンスの `.s()` を全コピーする。シティポップ maj7 やスーパーソーを既定にする

## Checklist

- [ ] 7–8 本（drums / bass / lead / hook / arp / chords / pad。任意 perc または vox）
- [ ] 4 小節 `.scale("<…>")`。**pad は C2** + `.adsr`。lead PCM は `.adsr`。arp はメロディ楽器。ドラムは 1 本。`.s()` は音色パレット
- [ ] `dj_hermes_apply_song(content, deck)`。save は残す指示のときだけ
