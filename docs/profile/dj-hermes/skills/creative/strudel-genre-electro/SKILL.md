---
name: strudel-genre-electro
description: "Use when writing Electro tracks in Strudel."
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel, music, genre, electro]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-acid
      - strudel-genre-house
      - strudel-genre-minimal-techno
      - strudel-genre-dubstep
---

# Strudel × エレクトロ（Electro）

## Overview
80年代エレクトロ／デトロイト系の、機械的で硬質なビート。キック＋スネア（クラップ）のブレイク寄り配置、電子ベース、冷たいリード。ファンクの骨格をマシンが叩く。4つ打ち一辺倒ではなく空きとシンコペが多い。BPM目安 110–130。

## When to Use
- エレクトロ、エレクトロ・ファンク、古いマシンビート感の曲
- 乾いたドラムと硬質シンセが欲しい時

Don't use for: 4つ打ち determinism のハウス本体、感情的パッド主体のチル／アンビエント。

## テンポと骨格
```
// @genre electro
// ブレイク寄りのドラム
s("bd ~ bd bd ~ sd ~ bd ~ ~ bd sd ~ bd ~ sd")
  .bank("RolandTR808").gain(0.85)
// または圧縮形
s("bd [~ bd] bd sd").gain(0.9)

// 乾いたハット
s("hh*8").gain(0.25).hpf(9000).crush(8)

// 電子ベース
note("<c2 ~ eb2 f2 ~ c2 g1 ~>*2")
  .s("square").lpf(600).lpq(4)
  .decay(.2).sustain(0).gain(.5)

// 冷たいリード
note("<c4 eb4 f4 g4>*2").s("triangle")
  .lpf(2000).delay(0.2).delayfeedback(0.3).gain(0.35)
```

## 制作レシピ
1. **ドラム**: 808/909系。`bd` と `sd`/`cp` の間を空ける。完全な `bd*4` は避けるか崩す。
2. **質感**: `crush` / `coarse` でローファイ機械感。リバーブは最小。
3. **ベース**: 短い decay の `square`/`pulse`。ファンキーな休符。
4. **リード**: 冷たい `triangle`/`sine`、控えめディレイ。
5. **構成**: 2–4小節フレーズの繰り返し＋ドラムのバリエーション。

## 音作りの要点
- ドライ > ウェット
- トランジェントをはっきり（短い attack）
- 空間系は delay 薄め、room はほぼ無し〜0.15
- パーカッションに `hpf` で分離

## ミニパターン例
```
stack(
  s("bd ~ ~ bd sd ~ bd ~").bank("RolandTR808"),
  s("~ hh ~ hh*2 ~ hh ~ hh").gain(.3),
  note("c2 ~ c2 eb2 ~ f2 ~ g1").s("square").lpf(500).decay(.18).sustain(0)
)
```

## Common Pitfalls
1. ハウスと同じ `bd*4` 固定 → エレクトロ感が消える。空きを作る。
2. リバーブ過多 → マシン感が溶ける。
3. アシッドと混同 → 303スウィープ主役ではなく、ドラム・配置が主役。
4. 音を詰めすぎ → シンコペと沈黙がグルーヴ。

## Verification Checklist
- [ ] ドラムに空き／シンコペがある
- [ ] 音色が乾いて硬質（crush/短いdecay）
- [ ] ベースがファンキーに休符を持つ
- [ ] 過剰な room がない
- [ ] `@genre electro` 任意付与
