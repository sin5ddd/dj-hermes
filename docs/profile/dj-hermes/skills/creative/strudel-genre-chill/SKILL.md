---
name: strudel-genre-chill
description: >-
  Use when writing chill / downtempo for strudel-rs: 80–100 BPM feel,
  soft drums, bs:hf floor only (do not stack bs:su). Not ambient and
  not chill-pop.
version: 5.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, chill]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
---

# strudel-rs × チル / ダウンテンポ

## Overview

遅め〜中庸、柔らかいドラム、控えめメロ。BPM 目安 80–100。フェンスは **90**（`setcpm(90/4)`）。
アンビエントよりドラムがある。チルポップほどコードを前面に出さない。
`songs/chill/01.strudel` はまだ薄いデモ。目標はこのフェンスの 7 本。

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
  .s("triangle").lpf(1800).gain(0.16)
// hook
$: note("0@2 4 7@2 ~").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("plk:am").gain(0.18).cut(1)
// arp
$: note("~ 7 12 7  4 0 ~ 2").scale("<C5:minor C5:minor F5:dorian C5:minor>")
  .s("plk:hp").gain(0.12).cut(1)
// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("<C3:minor C3:minor F3:dorian C3:minor>")
  .s("triangle").lpf(1200).gain(0.22).room(0.3).orbit(2)
  .attack(0.04).release(0.3)
// pad
$: note("[0,4]").scale("<C3:minor C3:minor F3:dorian C3:minor>")
  .s("wt_organ").lpf(900).gain(0.16).room(0.35).orbit(2)
  .attack(0.12).release(0.5)
```

`bs:hf` のみ。`bs:su` は重ねない。

## レシピ

- トラックは 7 本: `// drums` `// bass` `// lead` `// hook` `// arp` `// chords` `// pad`（任意で perc）
- ドラムは 1 本の `$:`。キック／スネア／ハットに分けない。キックは間引き、ハットは `[~ hh]*4`
- ピッチトラックは 4 小節 `.scale("<C4:minor C4:minor F4:dorian C4:minor>")`（コード・パッドは C3、arp は C5）
- PCM は `C4:`。シンセサブは `C2:`。フロアは `bs:hf` だけ（サブ同士を重ねない）
- コードは `[0,2,4]` を 3 音まで。パッドは `[0,4]`（`wt_organ`）
- メロは掛け合い。gain 0.12–0.18。プラックに `.cut(1)`
- `in_bank=no` の長い PCM は書かない。使えるのは `ld:ss` `pf:ff` `plk:*` `ep:*` `perc:*` `bs:*`、波形、`wt_*`、ライブ `.fm`

## Pitfalls

1. 高速 DnB 配置や `hh*8` 連打を持ってくる
2. `stack` / `.cpm` / `.lfo`
3. `bs:su` を `bs:hf` の下に足す
4. `note("c3'maj")` は root 単音。和音は `[0,2,4]`
5. キック／スネア／ハットを 3 本の `$:` に分ける
6. `songs/chill/01.strudel` を正本だと思って 3 本のまま apply する
7. 90 BPM を 70 アンビエントや 100 チルポップと DJ ペアにする（Transport は 1 つ）

## Checklist

- [ ] 7–8 本（drums / bass / lead / hook / arp / chords / pad。任意 perc）
- [ ] 4 小節 `.scale("<…>")`。ドラムは 1 本
- [ ] `strudel_apply_song(content, deck)`。save は残す指示のときだけ
