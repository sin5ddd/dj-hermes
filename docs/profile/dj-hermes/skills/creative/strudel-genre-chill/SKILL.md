---
name: strudel-genre-chill
description: "Use when writing Chill / downtempo in Strudel."
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel, music, genre, chill, downtempo]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-ambient
      - strudel-genre-house
      - strudel-genre-dnb
      - strudel-genre-dubstep
      - strudel-genre-future-bass
      - strudel-genre-progressive-house
      - strudel-genre-lofi-hiphop
      - strudel-genre-chill-pop
---

# Strudel × チル（Chill / Downtempo）

## Overview
ジャンル名というよりムード。低〜中テンポ、柔らかいドラム、温かいコード、控えめなメロディ。ロファイ／チルアウト／ダウンテンポ寄り。攻撃的トランジェントや高レゾナンスは避ける。BPM目安 70–100（または half-time 感）。

## When to Use
- チル、ダウンテンポ、リラックス向けビートもの（ジャンルが曖昧なとき）
- ビートはあるが攻撃的にしたくない時

Don't use for: クラブ向け強い4つ打ち（→ house）、ビートレス空間（→ ambient）、高速 DnB。
- **ブーンバップ＋ジャズコード＋vinyl** → `strudel-genre-lofi-hiphop` を優先
- **歌もの／ポップメロ＋柔らかいビート** → `strudel-genre-chill-pop` を優先

## テンポと骨格
```
// @genre chill, downtempo
// ゆるドラム
s("bd ~ ~ sd ~ ~ bd ~  ~ ~ sd ~ bd ~ ~ ~")
  .gain(0.7).room(0.2)
s("hh*8").gain(0.2).hpf(7000).gain("<0.2 0.12 0.18 0.1>")

// 暖かいコード
note("<[c3,e3,g3] [a2,c3,e3] [f2,a2,c3] [g2,b2,d3]>")
  .s("triangle")
  .attack(0.1).release(0.8)
  .lpf(1400).room(0.35).delay(0.25)
  .gain(0.35).slow(2)

// ソフトベース
note("<c2 ~ e2 f2 ~ a1 ~ g1>")
  .s("sine").lpf(250)
  .attack(0.05).release(0.4).gain(0.45)

// 短いメロ（控えめ）
note("~ ~ e4 ~  g4 ~ a4 ~  ~ e4 ~ d4  ~ c4 ~ ~")
  .s("triangle").lpf(2000).delay(0.3)
  .gain(0.25).slow(2)
```

## 制作レシピ
1. **テンポ**: `cpm(80)` 前後、またはパターンを `slow(2)`。
2. **ドラム**: 密度低め。`swing(4)` でゆるく。ベロシティにばらつき。
3. **コード**: 明るすぎない進行（maj7/min7 感）。長い release。
4. **メロ**: 隙間多め。1フレーズを繰り返さず変化を小さく。
5. **音色**: lpf 低め、lpq 低め、distort なし〜極薄。
6. **空間**: delay + 中程度 room。乾きすぎ／濡れすぎない。

## 音作りの要点
- すべての attack をわずかに遅く（0.02–0.15）
- ハイを `hpf` で刺さらないように
- ロファイ寄りなら軽い `crush(9)` や `coarse`
- アンビエントとの差: **ドラムとベースの拍が残る**

## ミニパターン例
```
stack(
  s("bd ~ ~ sd ~ bd ~ sd").gain(.65).room(.18),
  s("hh*4").gain(.15),
  note("<[c3,eb3,g3] [bb2,d3,f3]>").s("triangle")
    .attack(.12).release(.9).lpf(1200).gain(.3).room(.3).slow(2)
).cpm(86).swing(4)
```

## 近縁ジャンル
| 寄せたい方向 | 使う Skill |
| --- | --- |
| ブーンバップ・ジャズ・vinyl | `strudel-genre-lofi-hiphop` |
| 歌メロ・ポップフック | `strudel-genre-chill-pop` |
| ビートほぼ無し | `strudel-genre-ambient` |

## Common Pitfalls
1. ハウスと同じ音圧・同じ `bd*4` → チルにならない。密度と cpm を下げる。
2. 高 lpq アシッド → 刺激が強すぎる。
3. アンビエント化して拍が消える → 軽いドラムは残す（意図的なら ambient Skill へ）。
4. メロを詰め込みすぎ → 余白がチルの本体。
5. lo-fi hip hop / chill pop の型なのに汎用チルのまま → 専用 Skill へ indirection。

## Verification Checklist
- [ ] テンポが低〜中、ドラムが柔らかい
- [ ] コードが暖かく、release に余裕
- [ ] 刺激的な resonance/歪みが無い
- [ ] 余白（休符）がある
- [ ] lo-fi / chill-pop の方が適切でないか確認した
- [ ] `@genre chill` 任意
