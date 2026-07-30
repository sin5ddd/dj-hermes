---
name: strudel-genre-lofi-hiphop
description: "Use when writing lo-fi hip hop in Strudel."
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel, music, genre, lofi, lo-fi, hip-hop, chill]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-chill
      - strudel-genre-ambient
      - strudel-genre-house
      - strudel-genre-chill-pop
---

# Strudel × ローファイ・ヒップホップ（lo-fi hip hop）

## Overview
YouTube の "lofi hip hop radio" 的な、勉強・作業 BGM 向けビート。**低〜中テンポのブーンバップ／ヘッドノッド、ジャズ寄りコード、ビニールノイズ、鈍ったドラム、サイドチェイン少なめの穏やかさ**が核。チルの下位〜近縁だが、より **ヒップホップのキック／スネア配置とサンプル感**が強い。BPM目安 70–90（よくあるのは 75–85）。

## When to Use
- lo-fi hip hop / chillhop / study beats
- ジャズコード＋柔らかいブーンバップを Strudel で書く時

Don't use for: 攻撃的なトラップ、クラブ向けハウス、ビートレスの純アンビエント（近いがドラムとスウィングの有無で分ける）、Future Bass のキラキラ・チョップ主役。

## チル（Chill）との切り分け
| | lo-fi hip hop | Chill (汎用) |
| --- | --- | --- |
| ドラム | ブーンバップ（kick-snare の頭打ち） | ゆるい四拍子〜適当 |
| コード | m7 / maj7 のジャズ寄り | 任意の暖かい進行 |
| 質感 | vinyl / crush / 鈍いハイ | 必ずしもローファイでない |
| メロ | 短いピアノ／ギター的フレーズ | 任意 |

迷ったら: **ブーンバップ＋ジャズコード＋ノイズ層**なら本 Skill。ムードだけ緩いなら `strudel-genre-chill`。

## テンポと骨格
```
// @genre lo-fi, hip-hop, chillhop
// ブーンバップ骨格（1サイクル＝1小節想定）
s("bd ~ ~ bd  ~ ~ sd ~  bd ~ ~ ~  ~ sd ~ ~").gain(0.75).room(0.15)
// シンプル形
s("bd ~ sd ~").gain(0.78).room(0.12)

// ハット（少しハネる）
s("hh*8").gain(0.14).hpf(6000).crush(9)
// または開閉
s("[hh ~ hh hh]*2").gain(0.16).hpf(7000)

// ジャズ寄りコード（長い）
note("<[d3,f3,a3,c4] [g2,b2,d3,f3] [c3,e3,g3,b3] [f2,a2,c3,e3]>")
  .s("triangle")
  .attack(0.08).release(1.2)
  .lpf(1400).room(0.4).delay(0.28)
  .gain(0.32).slow(2)

// 短いピアノ的メロ
note("~ f4 a4 c5  ~ a4 g4 f4  ~ e4 g4 a4  ~ g4 f4 e4")
  .s("triangle").lpf(2200)
  .attack(0.01).release(0.5).delay(0.35)
  .gain(0.22).slow(2)

// ベース（ルート、丸い）
note("<d2 ~ ~ a1  g1 ~ ~ f1  c2 ~ ~ g1  f1 ~ ~ a1>")
  .s("sine").lpf(180).attack(0.02).release(0.4).gain(0.5)

// ビニール／部屋ノイズ層
s("pink").gain(0.025).hpf(800).lpf(6000).room(0.2)
// crackle が使えるなら
// s("crackle*4").density(0.05).gain(0.04)
```

## 制作レシピ
1. **BPM**: `cpm(80)` 前後。急ぎすぎない。
2. **ドラム**: kick と snare のヒップホップ配置。ベロシティをばらす（`gain` パターン）。
3. **スウィング**: `swing(4)` やハットの不均等でヘッドノッド。
4. **コード**: m7 / maj7 / ii–V–I っぽい進行。長く release。
5. **ローファイ処理**: 全体またはドラムに軽い `crush` / `coarse`、ハイを `lpf`/`hpf` で丸める。
6. **ノイズ層**: 常時ごく小さい `pink`/`crackle`。主役にしない。
7. **メロ**: 1–2 小節の短いフレーズを繰り返し、時々音を抜く。
8. **空間**: 中くらいの room + 短い delay。クラブ的な巨大リバーブは避ける。

## 音作りの要点（strudel-sound-design）
- ドラム: アタックを少し鈍く（サンプルが無ければ `lpf` でハイを落とす）
- コード: `triangle` / 柔らかい `saw` + lpf 1200–1800
- 歪みは極薄。刺さる `lpq` は使わない
- マスター的に明るくしすぎない（ハイ抑制）
- サイドチェインはあっても弱い（Future Bass ほど潰さない）

## ミニパターン例
```
stack(
  s("bd ~ ~ bd ~ ~ sd ~").gain(.72).room(.12),
  s("hh*8").gain(.12).hpf(6500).crush(10),
  note("<[d3,f3,a3,c4] [bb2,d3,f3,a3]>").s("triangle")
    .attack(.1).release(1).lpf(1300).gain(.3).room(.35).slow(2),
  note("d2 ~ a1 ~  g1 ~ f1 ~").s("sine").lpf(160).gain(.48),
  s("pink").gain(0.02).hpf(1000)
).cpm(82).swing(4)
```

## Common Pitfalls
1. 4つ打ちハウスのまま → ブーンバップ配置にする。
2. Future Bass 化（キラキラ短いスタブ＋強いダック）→ コードは長く、ダックは弱く。
3. ノイズが大きすぎ → gain 0.02–0.04 程度。
4. テンポ 100超 → lo-fi 感が薄れる。80前後へ。
5. ジャズ進行を無視してパワーコードだけ → ジャンル色が弱い（意図なら chill へ）。

## Verification Checklist
- [ ] BPM がおおむね 70–90
- [ ] kick/snare がヒップホップ型
- [ ] m7/maj7 系など暖かいコードがある
- [ ] 軽い crush/ノイズ/丸いハイのいずれかで lo-fi 質感
- [ ] 余白と swing がある
- [ ] `@genre lo-fi, hip-hop` 任意
