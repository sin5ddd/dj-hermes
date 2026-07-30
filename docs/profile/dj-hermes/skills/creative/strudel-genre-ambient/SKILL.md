---
name: strudel-genre-ambient
description: "Use when writing Ambient music in Strudel."
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel, music, genre, ambient]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-chill
      - strudel-genre-minimal-techno
      - strudel-genre-dnb
      - strudel-genre-lofi-hiphop
      - strudel-genre-chill-pop
---

# Strudel × アンビエント（Ambient）

## Overview
ビートを前面に出さない（またはごく薄い）。持続音、ドローン、パッド、空間系エフェクトが主。聴くというより「いる」音楽。メロディは希薄で、テクスチャと残響で時間を溶かす。BPMはほぼ無拍〜非常に遅い。

## When to Use
- アンビエント、ドローン、空間的サウンドスケープ
- ビートレス／極薄ビートの BGM 的作品

Don't use for: 明確な4つ打ちダンス（ハウス等）、高速ブレイクの DnB。チルは近いが「拍とメロの残し方」で分ける。

## テンポと骨格
```
// @genre ambient
// ドローン
note("c2").s("sawtooth")
  .attack(4).decay(2).sustain(0.8).release(6)
  .lpf(600).room(0.9).size(8)
  .gain(0.25).slow(4)

// パッド和音（遅い）
note("<[c3,e3,g3,b3] [a2,c3,e3,g3] [f2,a2,c3,e3]>")
  .s("triangle")
  .attack(3).release(5)
  .lpf(sine.range(400,1400).slow(16))
  .room(0.85).gain(0.2)
  .slow(2)

// テクスチャ・ノイズ
s("pink").gain(0.04).hpf(2000).room(0.7).slow(2)

// ごく薄いパルス（任意）
s("~ ~ bd ~").gain(0.15).hpf(100).room(0.5).slow(2)
```

## 制作レシピ
1. **時間**: `slow(2)`〜`slow(16)`、長い ADSR（attack/release 秒単位）。
2. **和音**: 変化は数サイクルに1回。テンション多めでも可（ぶつかっても room で溶ける）。
3. **空間**: `room` 大きめ（0.6–1.0）、`size`、薄い `delay`。
4. **テクスチャ**: `pink`/`brown`、`wt_` ウェーブテーブル、低 gain。
5. **ビート**: 基本なし。入れるなら gain 0.1台、hpf で存在だけ。
6. **変調**: lpf を非常に遅く。`seg` で滑らかにサンプリング。

## 音作りの要点
- gain は全体的に低め（クリップと疲労を避ける）
- トランジェントを消す（attack 長）
- 単音ドローン + ゆっくり動くパッドの2層が安定
- orbit 分けでリバーブ競合を避ける

## ミニパターン例
```
stack(
  note("c2").s("sine").attack(8).release(8).gain(.2).room(.9).slow(4),
  note("<[e3,g3,b3] [d3,f3,a3]>").s("triangle")
    .attack(4).release(6).lpf(900).gain(.15).room(.8).slow(2)
)
```

## Common Pitfalls
1. キックを普通の音量で入れる → アンビエントが「薄いテクノ」になる。
2. 短い decay のプラック連打 → ジャンル感が消える。
3. 変化が速すぎ → slow と長いエンベロープを優先。
4. gain 積みすぎ → 複数レイヤーは必ず小さく。

## Verification Checklist
- [ ] ビートが主役になっていない
- [ ] 長い attack/release と大きな room がある
- [ ] メロディが主張しすぎない
- [ ] 全体 gain が穏やか
- [ ] `@genre ambient` 任意
