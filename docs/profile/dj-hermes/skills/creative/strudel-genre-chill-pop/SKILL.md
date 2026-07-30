---
name: strudel-genre-chill-pop
description: "Use when writing Chill Pop in Strudel."
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel, music, genre, chill-pop, pop, downtempo]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-chill
      - strudel-genre-lofi-hiphop
      - strudel-genre-future-bass
      - strudel-genre-progressive-house
      - strudel-genre-ambient
---

# Strudel × チルポップ（Chill Pop）

## Overview
歌ものポップのフックと、低〜中テンポの柔らかいビートを合わせたムード。**キャッチーなメロディ／ボーカル的フレーズが主役**で、ドラムとコードは支え。lo-fi hip hop よりメロが前、Future Bass より刺激とダックが弱い、汎用チルより「曲として歌える」。BPM目安 80–110。

## When to Use
- チルポップ、インドポップ的リラックス歌もの、柔らかい synth-pop
- フックのあるメロ＋穏やかビートを Strudel で書く時

Don't use for:
- ブーンバップ＋jazz＋vinyl 主体 → `strudel-genre-lofi-hiphop`
- ムードのみでメロが無い → `strudel-genre-chill`
- キラキラ・強いサイドチェインの EDM → `strudel-genre-future-bass`
- ビートレス → `strudel-genre-ambient`

## 近縁との差
| | Chill Pop | lo-fi hip hop | Chill | Future Bass |
| --- | --- | --- | --- | --- |
| 主役 | 歌えるメロ | ビート＋jazzコード | 雰囲気 | コードチョップ |
| ドラム | シンプル四拍〜軽い | ブーンバップ | ゆるい | ハーフタイム |
| 質感 | クリア〜少し暖色 | vinyl/crush | 任意 | 明るい・太い |
| 構成 | Verse/Hook 意識 | ループ | ループ | ドロップ |

## テンポと骨格
```
// @genre chill-pop, pop
// シンプルで柔らかいドラム
s("bd ~ ~ ~ bd ~ sd ~").gain(0.72).room(0.15)
s("hh*8").gain(0.16).hpf(8000)
// クラップ薄く
s("~ ~ cp ~").gain(0.35).room(0.2)

// 明るいが刺さらないコード
note("<[c3,e3,g3] [a2,c3,e3] [f2,a2,c3] [g2,b2,d3]>")
  .s("triangle")
  .attack(0.08).release(0.7)
  .lpf(1600).room(0.3).delay(0.25)
  .gain(0.32).slow(2)

// フック・メロ（歌えるフレーズ）
note("e4 ~ g4 a4  ~ g4 e4 d4  c4 ~ e4 g4  ~ a4 g4 e4")
  .s("triangle")
  .attack(0.02).release(0.45)
  .lpf(2800).delay(0.35).room(0.25)
  .gain(0.34)

// ベース（シンプルなルート）
note("<c2 ~ e2 ~  a1 ~ c2 ~  f1 ~ a1 ~  g1 ~ b1 ~>").s("sine")
  .lpf(200).attack(0.03).release(0.35).gain(0.48)
```

## 制作レシピ
1. **BPM**: `cpm(90–100)` 前後が扱いやすい。
2. **フック先に決める**: 4–8 音の歌えるフレーズを先に書く。
3. **ドラムは薄く支える**: kick は少なめ〜普通、sn/cp は控えめ。
4. **コード**: I–V–vi–IV や vi–IV–I–V などポップ定番でも可。長すぎるドローンは避ける。
5. **音色**: クリアめ。crush は無し〜極薄（lo-fi にしすぎない）。
6. **空間**: delay をメロに多め、room は中程度。
7. **構成**: Intro（コード）→ Verse（メロ控えめ）→ Hook（メロ全開）の差を gain/音数で作る。

## 音作りの要点
- メロの `gain` をコードより少し前に
- attack はメロ短め、パッド長め
- ベースは単純なルート運動（複雑にしない）
- サビ相当では `hh` 密度やコードの開放（lpf 上げ）で持ち上げ

## ミニパターン例
```
stack(
  s("bd ~ ~ bd ~ ~ sd ~").gain(.7).room(.14),
  s("hh*8").gain(.14).hpf(8500),
  note("<[c3,e3,g3,b3] [a2,c3,e3,g3] [f2,a2,c3,e3] [g2,b2,d3,f3]>")
    .s("triangle").attack(.1).release(.8).lpf(1500).gain(.28).room(.28).slow(2),
  note("g4 a4 c5 a4  g4 e4 d4 e4").s("triangle")
    .delay(.32).lpf(3000).gain(.33),
  note("c2 ~ a1 ~ f1 ~ g1 ~").s("sine").lpf(180).gain(.5)
).cpm(96)
```

## Common Pitfalls
1. メロが弱く雰囲気だけ → chill 汎用になってしまう。フックを明示。
2. lo-fi ノイズと crush 過多 → chill pop の「歌もの感」が消える。
3. Future Bass のハーフタイム＋強いダック → ジャンルがずれる。
4. ドラムを詰めすぎ → ポップの余白とメロ帯域を圧迫。

## Verification Checklist
- [ ] 歌える／覚えられるメロまたはフックがある
- [ ] ドラムがメロを邪魔していない
- [ ] テンポが低〜中、攻撃的でない
- [ ] lo-fi hip hop ほど dirty にしていない（意図時を除く）
- [ ] `@genre chill-pop` 任意
