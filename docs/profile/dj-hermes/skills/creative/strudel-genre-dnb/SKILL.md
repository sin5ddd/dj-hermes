---
name: strudel-genre-dnb
description: "Use when writing Drum and Bass in Strudel."
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel, music, genre, dnb, drum-and-bass]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-electro
      - strudel-genre-chill
      - strudel-genre-ambient
      - strudel-genre-dubstep
---

# Strudel × ドラムンベース（Drum and Bass）

## Overview
高速（体感 160–180 BPM 相当）のブレイクビーツと重いサブベース。キックとスネアの配置が複雑で、ハーフタイム感も持つ。ベースは長い低音や Reese 的うねり。高域はロールや高速ハット。BPM/cpm は高速、またはパターンを `fast` する。

## When to Use
- DnB、ジャングル寄り、リキッド寄りを含むブレイクビーツ高速曲
- 複雑なドラム＋サブベースが核の時

Don't use for: 4つ打ちハウス／ミニマル、ビートレス・アンビエント。

## テンポと骨格
```
// @genre drum-and-bass, dnb
// ブレイク（例: 2サイクルで1フレーズ）
s("bd ~ ~ ~ ~ sd ~ ~ bd ~ bd ~ ~ sd ~ ~")
  .fast(2).gain(0.9)
// より密集
s("bd [~ bd] ~ sd  bd ~ [sd bd] sd").fast(2)

// 高速ハット
s("hh*16").gain(0.22).hpf(9000).fast(1)

// サブベース（長く）
note("<c1 ~ ~ ~ eb1 ~ ~ f1 ~ ~ ~ c1 ~ bb0 ~ ~>")
  .s("sine").lpf(120)
  .attack(0.01).release(0.4).gain(0.7)

// Reese風（うねり）
note("c1*2").s("sawtooth")
  .lpf(sine.range(80,400).fast(3)).lpq(6)
  .gain(0.4).orbit(2)
```

## 制作レシピ
1. **ドラム**: Amen/ブレイク的配置。`bd`/`sd` の非均等。`fast(2)` で加速。
2. **ハーフタイム感**: スネアを少なめの位置に置き「ゆっくり聞こえるが中身は速い」。
3. **サブ**: `sine` か暗い `saw`、超低 `note`（c1以下）、長い release。
4. **Reese**: 2系統デチューン相当 → 近い音程 stack や lpf の速い変調。
5. **高域**: `hh*16`、ロールは `ply` や短い繰り返し。
6. **切替**: ドラムのみ／ベースドロップなど、セクションで要素のオンオフ。

## 音作りの要点
- サブとキックの衝突に注意（同時打は片方を短く or 帯域分け）
- ドラムに薄い `room`、サブは dry
- リキッド寄り: 柔らかい pad + 遅め lpf、ジャングル寄り: ブレイク強調 + ノイズ

## ミニパターン例
```
stack(
  s("bd ~ ~ sd ~ bd sd ~").fast(2).gain(.9),
  s("hh*8").fast(2).gain(.2),
  note("c1").s("sine").lpf(100).gain(.65).slow(1)
).cpm(170)
```

## Common Pitfalls
1. `bd*4` のまま → ただの速いハウスになる。ブレイク配置にする。
2. サブが melodic すぎ／高すぎ → c1–f1 付近、lpf 低め。
3. 全部 fast しすぎて潰れる → ドラムだけ fast、メロは half も検討。
4. 無音の隙間ゼロ → DnB も「抜き」がグルーヴ。

## Verification Checklist
- [ ] ドラムがブレイク／非4つ打ち
- [ ] 体感テンポが DnB 域（または fast で同等）
- [ ] サブベースがローを支えている
- [ ] キックとサブが濁りすぎない
- [ ] `@genre drum-and-bass` 任意
