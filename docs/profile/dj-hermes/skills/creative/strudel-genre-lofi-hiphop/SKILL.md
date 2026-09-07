---
name: strudel-genre-lofi-hiphop
description: >-
  Use when writing lo-fi hip hop for strudel-rs: 75–90 BPM, dusty
  bd:lf, keys, slow hats. Not house [~ cp]*2.
version: 5.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, lofi, hiphop]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × ローファイ・ヒップホップ

## Overview

遅め BPM、乾いたドラム、柔らかいキー。BPM 目安 75–90。フェンスは **84**（`setcpm(84/4)`）。
キックは `bd:lf`。ハットは遅い。ハウスの clap グリッドは使わない。
`songs/lofi-hiphop/01.strudel` はこのフェンスと同じ（7 本）。

## When

- 来場者がローファイ、ヒップホップ、ダスト、EP キーを求めるとき
- ハウス `[~ cp]*2`、シティポップ下降のチルポップ、パッド主のアンビエントではないとき
- 新規 apply / プリセット（2–5 本では足りない）

## Pattern

```
// @title visitor-lofi
// @genre lofi-hiphop
setcpm(84/4)
// drums
$: s("bd:lf ~ ~ sd, [~ hh]*4, <~ ~ ~ [bd:lf sd bd:lf sd]>").gain(0.44)
// bass
$: note("0 ~ 2 <0 0 3 2>").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("bs:su").gain(0.38)
// lead
$: note("~ 4 7 <5 4 0 2>").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("plk:lf").gain(0.18).cut(1)
// hook
$: note("[0,2,4] ~ [0,3,5] ~").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("ep:rs").gain(0.22)
  .attack(0.05).decay(0.3).sustain(0.5).release(0.3)
// arp
$: note("~ 0 4 7  ~ 4 0 2").scale("<C5:minor C5:minor F5:dorian C5:minor>")
  .s("plk:ny").gain(0.12).cut(1)
// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("ep:mt").gain(0.2).room(0.3).orbit(1)
// pad
$: note("0").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("pf:ff").gain(0.14).room(0.35).orbit(1)
```

ハウス `[~ cp]*2` は使わない。ハットは `[~ hh]*4` のまま（`hh*8` にしない）。

## レシピ

- トラックは 7 本: `// drums` `// bass` `// lead` `// hook` `// arp` `// chords` `// pad`（任意で perc）
- ドラムは 1 本の `$:`。キック／スネア／ハットに分けない。`bd:lf` + 遅いハット
- ピッチトラックは 4 小節 `.scale("<C4:minor C4:minor F4:dorian C4:minor>")`（コード・パッドは C3、arp は C5）
- PCM は `C4:`。シンセサブは `C2:`。フロアは `bs:su` だけ（`bs:hf` と重ねない）
- フックは `ep:rs`（C3 録音 → native は C4 スケール）。リードは `plk:lf`
- コードは `ep:mt` の `[0,2,4]` を 3 音まで。パッドは `pf:ff` の `note("0")`
- メロ／コード／パッドに `triangle` / `sine` を使わない
- `in_bank=no` の長い PCM は書かない。使えるのは `ld:ss` `pf:ff` `plk:*` `ep:*` `perc:*` `bs:*`、波形、`wt_*`、ライブ `.fm`

## Pitfalls

1. 高速ハットでハウス化する（`hh*8` や `[~ cp]*2`）
2. `stack` / `.cpm` / `.lfo`
3. `note("c3'maj")` は root 単音。和音は `[0,2,4]`
4. `bs:su` の上に `bs:hf` や別のサブを重ねる
5. `ep:rs` / `plk:lf` に `.scale("C3:…")` を付ける（1 オクターブ下がる）
6. 新規 apply を 3 本のまま出す
7. 84 BPM を 124 ハウスと DJ ペアにする（Transport は 1 つ）

## Checklist

- [ ] 7–8 本（drums / bass / lead / hook / arp / chords / pad。任意 perc）
- [ ] 4 小節 `.scale("<…>")`。ドラムは 1 本
- [ ] `strudel_apply_song(content, deck)`。save は残す指示のときだけ
