---
name: strudel-genre-future-bass
description: "Use when writing Future Bass in Strudel."
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel, music, genre, future-bass, edm, marshmello]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-progressive-house
      - strudel-genre-dubstep
      - strudel-genre-chill
      - strudel-genre-house
      - strudel-genre-dnb
---

# Strudel × フューチャーベース（Future Bass）

## Overview
Marshmello に近い軸の EDM。**ハーフタイム寄り〜中速のビート、明るく厚いコードスタブ（チョップ）、サイドチェイン感、キラキラしたリード／プラック**が核。ダブステップのワブルより「和音の躍動とポップなメロ」が主役。BPM目安 140–160（ハーフ感で聴こえることも）または 70–80 台の表記。

Alone / Happier 的な、ポップに寄せた festival future bass を主対象にする。

## When to Use
- Marshmello 風、Future Bass、メロディックなチョップド・コード EDM
- 明るいコードスタブ＋ドロップの「ホンッ」というダック感

Don't use for: ワブル主役のクラシック・ダブステップ、Avicii 型の長いスーソース・4つ打ちアンセム（→ progressive-house）、ディープハウス。

## テンポと骨格
```
// @genre future-bass, edm
// ハーフタイム風ドラム
s("bd ~ ~ ~ sd ~ ~ ~").gain(0.9)
s("bd ~ bd ~ sd ~ ~ ~").gain(0.88)  // バリエ
// 高速ロール・ハット
s("hh*16").gain(0.16).hpf(10000)
s("~ ~ ~ ~ ~ ~ hh*8 ~").gain(0.2)   // フィル的

// チョップド・コード（スタブ）
note("<[c3,e3,g3,b3] ~ [e3,g3,b3,d4] ~ [a2,c3,e3,g3] ~ [g2,b2,d3,f3] ~>*2")
  .s("sawtooth")
  .attack(0.005).decay(0.2).sustain(0.15).release(0.15)
  .lpf(3200).lpq(2)
  .gain(0.4).room(0.25).delay(0.2)

// キラキラ・リード
note("~ e4 g4 b4  ~ a4 g4 e4").s("triangle")
  .attack(0.01).release(0.3).delay(0.35).room(0.3).gain(0.3)

// サブ
note("<c2 ~ ~ ~ e2 ~ ~ ~ a1 ~ ~ ~ g1 ~ ~ ~>").s("sine")
  .lpf(120).gain(0.65)
```

## 制作レシピ
1. **ドラム**: ハーフタイム（sd が疎）。フィルで `hh`/`sd` ロール。
2. **コードスタブ**: 短い decay の厚い和音をリズムに乗せる（チョップ）。
3. **ダック感**: キックでコード軌道を `duckorbit`、またはコードをキック位置で抜く。
4. **リード**: ポップで短いフレーズ、ディレイ多め。
5. **レイヤー**: サブ + ミッドコード + 高域プラック/ベル。
6. **ドロップ**: コードとドラムが同時に厚くなる。ブレイクはメロ＋パッドのみ。

## 音作りの要点
- コードは **短く明るい**（prog house の長いパッドと逆）
- `saw` スタックや高め lpf で「太いキラキラ」
- ピッチ感: メジャー／明るいマイナー、テンション（add9 等）歓迎
- ワブルは主役にしない（使うならワンショット的）
- ボーカルチョップ相当: 短い `note` フレーズを繰り返す

## Progressive House / Dubstep との差
| | Future Bass | Prog House (Avicii) | Dubstep |
| --- | --- | --- | --- |
| 拍 | ハーフ〜中間 | 4つ打ち | ハーフ |
| コード | 短いチョップ | 長いパッド＋スーソース | 薄い |
| 主役 | 和音リズム＋メロ | アンセム・メロ | ワブル／サブ |
| 気分 | ポップ・明るい | 高揚・壮大 | 重い・うねる |

## ミニパターン例
```
stack(
  s("bd ~ ~ ~ sd ~ ~ ~").gain(.9),
  s("hh*8").gain(.15),
  note("<[c4,e4,g4] ~ [e4,g4,b4] ~>*2").s("sawtooth")
    .decay(.18).sustain(.1).lpf(3500).gain(.38).room(.22),
  note("c2 ~ ~ ~").s("sine").lpf(100).gain(.6),
  note("~ g4 b4 e5").s("triangle").delay(.3).gain(.28)
).cpm(150)
```

## Common Pitfalls
1. 4つ打ちのまま長いパッド → prog house 化。スタブとハーフタイムを意識。
2. ワブル全開 → dubstep 化。和音チョップを残す。
3. コードを伸ばしすぎ → future bass の「跳ね」が消える。
4. 高域だけキラキラでロー無し → サブを必ず敷く。

## Verification Checklist
- [ ] ドラムがハーフタイム寄り（または意図した groove）
- [ ] 短い明るいコードスタブがある
- [ ] キックに対するダック／抜け感がある
- [ ] ポップなリードまたはフックがある
- [ ] `@genre future-bass` 任意
