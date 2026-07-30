---
name: strudel-genre-progressive-house
description: "Use when writing Progressive House in Strudel."
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel, music, genre, progressive-house, edm, avicii]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-house
      - strudel-genre-electro
      - strudel-genre-future-bass
      - strudel-genre-chill
---

# Strudel × プログレッシブハウス（Progressive House）

## Overview
Avicii に近い軸のダンスミュージック。4つ打ちキックはハウスと同型だが、**長いビルド、情感のあるコード／メロディ、ドロップでのスーソースやプラック**が特徴。フォークギター的リフやボーカルフックを乗せることも多い（いわゆるメロディック／フェスティバル寄りプログレ）。BPM目安 126–130。通常のハウスより「曲としての起承転結」とメロが前面。

※ 狭義のプログレ（深い反復・催眠）と、Avicii 型のメロディック・プログレ／EDM は違う。この Skill は **Avicii 型＝メロディックな progressive / festival house** を主対象にする。

## When to Use
- Avicii 風、メロディック・プログレ、フェスティバルハウス
- 4つ打ち＋大きなコード進行＋キャッチーなリード／ドロップ

Don't use for: ディープで地味なミニマル、ワブル主役のダブステップ、Future Bass のチョップド・コード主体（→ future-bass）。

## テンポと骨格
```
// @genre progressive-house, edm
// 4つ打ち
s("bd*4").gain(0.95)
s("~ cp ~ cp").gain(0.65).room(0.2)
s("hh*8").gain(0.28)
s("~ oh ~ oh").gain(0.3)

// 暖かい進行コード（長い）
note("<[c3,e3,g3,b3] [a2,c3,e3,g3] [f2,a2,c3,e3] [g2,b2,d3,f3]>")
  .s("sawtooth")
  .attack(0.3).decay(0.4).sustain(0.6).release(0.8)
  .lpf(1800).room(0.45).delay(0.3)
  .gain(0.32).slow(2)

// スーソース系ドロップ・リード
note("<c4 e4 g4 b4 a4 g4 e4 d4>*2")
  .s("sawtooth")
  .detune(0.12) // 使えなければ stack で微ずれ
  .lpf(sine.range(800,5000).slow(1))
  .attack(0.02).release(0.25)
  .gain(0.4).room(0.3)

// ベース（ルート追随）
note("<c2 c2 a1 a1 f1 f1 g1 g1>")
  .s("sawtooth").lpf(400).decay(0.25).sustain(0.3).gain(0.5)
```

## 制作レシピ
1. **Intro**: パッド＋薄い perc、キック後入れ。
2. **Build**: ハイハット密度↑、ノイズライザー相当（`white` + hpf 上昇）、コードを厚く。
3. **Drop**: `bd*4` 全開、スーソース／プラック主メロ、ベース同期。
4. **Breakdown**: キック抜き、情感コード＋メロだけ。
5. **メロ**: 歌える簡易フレーズ。ペンタやメジャースケール多め。
6. **層**: パッド / プラック / リード / サブ を分け、drop で同時に鳴らす。

## 音作りの要点
- スーソース感: `sawtooth` + やや広めの音（stack で ±少しの note）+ lpf 開く
- パッドは長い attack、リードは短い attack
- room/delay はメロと sn に多め、キックとサブは dry
- duck: キックでパッド／スーソース軌道を軽く潰す

## 通常ハウスとの差
| | House | Progressive (Avicii型) |
| --- | --- | --- |
| メロ | 控え〜スタブ | 正面のフック |
| 構成 | ループ寄り | ビルド／ドロップ明確 |
| 音色 | 暖か・地味可 | スーソース・明るい |
| 感情 | グルーヴ | アンセム／高揚 |

## ミニパターン例
```
stack(
  s("bd*4"),
  s("~ cp ~ cp").room(.18),
  s("[hh oh]*4").gain(.28),
  note("<[c3,e3,g3] [a2,c3,e3] [f2,a2,c3] [g2,b2,d3]>")
    .s("sawtooth").attack(.25).release(.6).lpf(1600).gain(.3).room(.4).slow(2),
  note("c4 e4 g4 e4 a4 g4 e4 d4").s("sawtooth").lpf(3500).gain(.35)
).cpm(128)
```

## Common Pitfalls
1. メロなしでループのみ → ただのハウスになる。フックを1つ決める。
2. ドロップで帯域パンパン → パッド gain を下げ、リードを通す。
3. Future Bass 化 → チョップド・コードとトラップハットにしない（4つ打ち維持）。
4. ビルドなしでいきなりフル → Avicii 型の「溜め」が消える。

## Verification Checklist
- [ ] bd*4 と 2/4 cp がある
- [ ] 感情的なコード or 歌えるリードがある
- [ ] ビルド／ドロップの差がある（または意図したループ）
- [ ] スーソース／明るい saw 系がドロップで目立つ
- [ ] `@genre progressive-house` 任意
