---
name: strudel-genre-house
description: "Use when writing House tracks in Strudel."
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel, music, genre, house]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-acid
      - strudel-genre-electro
      - strudel-genre-minimal-techno
      - strudel-genre-chill
      - strudel-genre-progressive-house
      - strudel-genre-future-bass
---

# Strudel × ハウス（House）

## Overview
4つ打ちキックが土台。オフビートのオープンハット、クラップ／スネアは2と4、暖かいベース、ピアノやコード・ヴォーカル的フレーズ。グルーヴは「乗り」。ディープ／テック／プログレ等の派生でも骨格は同じ。BPM目安 118–128。

## When to Use
- ハウス、ディープハウス、テックハウス寄りの4つ打ち
- ダンスフロア向けの乗りと温かさが欲しい時

Don't use for: 空き多めのエレクトロ、要素極小のミニマル、高速 DnB。

## テンポと骨格
```
// @genre house
// キック
s("bd*4").gain(0.95)

// クラップ 2と4
s("~ cp ~ cp").gain(0.7).room(0.15)

// ハット（クローズ＋オープン）
s("hh*8").gain(0.28)
s("~ oh ~ oh").gain(0.35).room(0.1)

// ベース
note("<c2 c2 eb2 f2 c2 bb1 ab1 bb1>")
  .s("sawtooth").lpf(500).lpq(3)
  .decay(0.25).sustain(0.3).release(0.1)
  .gain(0.5)

// コード／スタブ
note("<[c3,eb3,g3] [bb2,d3,f3] [ab2,c3,eb3] [bb2,d3,f3]>*2")
  .s("triangle").lpf(1200)
  .attack(0.02).release(0.2).gain(0.3).room(0.25)
```

## 制作レシピ
1. **グリッド**: `bd*4` 固定。`cp` は 2/4。`hh` は8分 or 16分。
2. **オープンハット**: 裏（オフビート）に `oh`。
3. **ベース**: ルート中心、ウォームな lpf（300–800Hz）。
4. **コード**: 2–4和音、短い stab か柔らかいパッド。
5. **グルーヴ**: 軽い `swing(4)` や `late(0.01)` で人間味。
6. **空間**: クラップとパッドに薄い room/delay。キックとベースは dry 寄り。

## 音作りの要点
- 暖かさ: `sawtooth`/`triangle`、低め lpf、歪みは軽く
- キックとベースの帯域分離（ベースに hpf 薄くでも可）
- duck: キックでパッド軌道を軽く潰すとクラブ感

## ミニパターン例
```
stack(
  s("bd*4"),
  s("~ cp ~ cp").room(.12),
  s("[hh oh]*4").gain(.3),
  note("c2 eb2 f2 g2").s("sawtooth").lpf(450).decay(.3).sustain(.2)
).cpm(124)
```

## Common Pitfalls
1. キック以外を埋めすぎ → 乗りが死ぬ。空きを残す。
2. ベースを明るくしすぎ → ハウスの「腰」が消える。lpf を下げる。
3. アシッド化 → 高 lpq の16分主役にしない（テックハウス分岐は別判断）。
4. cp を毎拍 → 2と4のフックが弱くなる。

## Verification Checklist
- [ ] bd*4 と 2/4 の cp がある
- [ ] オフビートのハット／oh がある
- [ ] ベースが暖かくローを支えている
- [ ] 全体が「乗れる」4つ打ち
- [ ] `@genre house` 任意
