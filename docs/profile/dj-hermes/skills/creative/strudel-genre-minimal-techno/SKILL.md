---
name: strudel-genre-minimal-techno
description: "Use when writing Minimal Techno in Strudel."
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel, music, genre, minimal, techno]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-acid
      - strudel-genre-house
      - strudel-genre-electro
---

# Strudel × ミニマルテクノ（Minimal Techno）

## Overview
要素を極限まで減らす。キック中心、ハイハットやパーカッションの微細な変化、小さなフィルター／パンの動きで時間を進める。メロディより空間と反復。BPM目安 120–130。音数は少ないが、タイミングと音色の揺らぎが命。

## When to Use
- ミニマルテクノ、リダクション重視のテクノ
- 「音を足す」より「ずらす・薄く変える」で展開したい時

Don't use for: メロディ密集のハウス・ポップ、高速 DnB、ドローン主体のアンビエント（近いがビートの有無で分ける）。

## テンポと骨格
```
// @genre minimal, techno
s("bd*4").gain(0.9).orbit(1)

// 微細ハット
s("hh*16").gain(0.18).hpf(10000)
  .gain("<0.18 0.12 0.2 0.1>".slow(2))

// 薄いパーカッション
s("~ cp ~ ~").gain(0.35).room(0.2)

// ワンノート的トーン
note("c2*4").s("triangle")
  .lpf(sine.range(200,800).slow(8))
  .attack(0.01).decay(0.3).sustain(0.2).release(0.1)
  .gain(0.35).orbit(2)
```

## 制作レシピ
1. **コア**: `bd*4` を固定。他は最小限。
2. **変化**: 音色・pan・lpf・gain を **ゆっくり**（`slow(4)`〜`slow(16)`）。
3. **時間操作**: `iter` / `linger` / `zoom` / `early`/`late` で微ずれ。
4. **レイヤー**: 同時発音は2–4系統まで。orbit を分ける。
5. **ダック**: キックでパッド軌道を軽く `duckorbit`。
6. **展開**: 8–16サイクルごとに1要素だけ足す／引く。

## 音作りの要点
- 音は短くても長くてもよいが **派手にしない**
- `room` は均一で薄く（0.15–0.4）
- メロディラインは1–3音まで
- 連続変調は `seg(8)` 等でイベントを増やす

## ミニパターン例
```
stack(
  s("bd*4").gain(.95),
  s("hh*8").gain(sine.range(0.08,0.22).slow(4)),
  s("~ ~ sd ~").gain(.3).room(.15),
  note("c3").s("sawtooth").lpf(400).lpq(2)
    .slow(2).gain(.25).room(.3)
)
```

## Common Pitfalls
1. 音を足して埋める → ミニマルが崩れる。変化はパラメータで。
2. フィルターを速く振りすぎ → アシッド化。slow を使う。
3. 全トラック同 orbit で room 上書き → orbit 分離。
4. 展開が無さすぎて単調 → 8–16c で1パラメータだけ動かすルールを。

## Verification Checklist
- [ ] 同時に目立つ要素が少ない（キック＋α）
- [ ] 変化が「微細」で時間軸が長い
- [ ] メロディが主張しすぎない
- [ ] orbit / room の衝突がない
- [ ] `@genre minimal, techno` 任意
