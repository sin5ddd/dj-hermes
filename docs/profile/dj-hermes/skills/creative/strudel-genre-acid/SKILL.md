---
name: strudel-genre-acid
description: "Use when writing Acid tracks in Strudel."
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel, music, genre, acid, techno]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-electro
      - strudel-genre-minimal-techno
      - strudel-genre-house
---

# Strudel × アシッド（Acid）

## Overview
TB-303的なレゾナント・スクエア／ソーの「ジュワァ」が核のジャンル。16分の刻み、カットオフとレゾナンスのオートメーション、スライド感。キックは4つ打ち寄り。ハーモニーより音色変化でグルーヴを作る。BPM目安 120–135。

## When to Use
- アシッド／アシッドハウス／アシッドテクノを Strudel で書く時
- 303風ベースラインとフィルター・スウィープが主役の曲

Don't use for: 穏やかなチル／アンビエント（→ 各ジャンル Skill）、高速ブレイク主体の DnB。

## テンポと骨格
```
// メタデータ例
// @title Acid Line
// @genre acid, techno
// @by ...

// キック 4つ打ち
s("bd*4").gain(0.9)

// ハット
s("hh*8").gain(0.3).hpf(8000)

// 303ライン（主役）
note("<c2 c2 eb2 f2 c2 bb1 c2 g2>*2")
  .s("sawtooth")
  .lpf("<400 800 2000 1200 600 3000 900 500>")
  .lpq("<12 18 22 16 20 24 14 18>")
  .lpenv("<2 3 4 3>")
  .decay(0.2).sustain(0.1).release(0.05)
  .gain(0.55)
```

## 制作レシピ
1. **リズム**: `bd*4` + タイトな `hh*8` or `hh*16`。スネアは控えめ（`~ sd ~ sd`）。
2. **ベースライン**: 短い音価の16分〜8分。`note` + `sawtooth`/`square`。
3. **フィルター**: `lpf` をパターンで動かす。`lpq` を高め（12–30）に。
4. **エンベロープ**: `decay` 短め、`sustain` 低め → プルック〜レゾナント・テイル。
5. **アクセント**: `gain` や `velocity` を一部だけ上げる。`?` で間引きも可。
6. **スライド感**: 隣接点の短いスラー、または `penv` でピッチ包絡を薄く。

## 音作りの要点（strudel-sound-design）
- 波形: `sawtooth` / `square`（triangle は弱い）
- 必須: `lpf` + `lpq` + できれば `lpenv`
- 空間: `room` は薄め（0.1–0.3）。ベースは dry 寄り
- 歪み: 軽い `distort` / `shape` でエッジ

## ミニパターン例
```
// ユークリッド寄りアシッド・キック
s("bd(3,8), hh*8").bank("RolandTR909")

// クラシックな16分ライン
note("c2 eb2 f2 g2 eb2 f2 c2 bb1".fast(2))
  .s("sawtooth").lpf(sine.range(300,2500).slow(4)).lpq(18)
  .decay(.15).sustain(0).gain(.5)
```

## Common Pitfalls
1. `lpq` を上げすぎて耳が痛い → 18前後から、`gain` も下げる。
2. リバーブ過多で303が溶ける → ベース軌道は dry / 別 orbit。
3. ノートを長くしすぎて「ジュワッ」にならない → decay/sustain を締める。
4. ハウスと区別がつかない → フィルターと16分ラインを主役に、コードパッドは控える。

## Verification Checklist
- [ ] 303的ラインが聴こえの中心になっている
- [ ] lpf/lpq（+lpenv）が時間変化している
- [ ] キックは4つ打ち寄りで邪魔していない
- [ ] BPM/サイクル感がダンス向け（遅すぎ・速すぎない）
- [ ] `@genre acid` 等メタデータを付けた（任意）
