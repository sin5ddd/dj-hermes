---
name: strudel-genre-dubstep
description: "Use when writing Dubstep tracks in Strudel."
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel, music, genre, dubstep, bass]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-dnb
      - strudel-genre-electro
      - strudel-genre-chill
      - strudel-genre-house
      - strudel-genre-future-bass
      - strudel-genre-progressive-house
---

# Strudel × ダブステップ（Dubstep）

## Overview
ハーフタイム感の強い低速〜中速グルーヴ（体感 70–75 BPM のハーフ／140–150 の裏）。重いサブ、ワブル（周期的なフィルター／振幅変調）ベース、スネアはおおむね 3 拍目、キックはまばら。ドロップでベースが主役、ビルドでは余白と上昇感。Brostep 寄りの攻撃的ワブルから、深い Dub 寄りの空間系まで幅がある。

## When to Use
- ダブステップ、ブロステップ、ディープダブ寄り
- ハーフタイム＋ワブル／サブベースが核の曲

Don't use for: 4つ打ちハウス、高速フルグリッドの DnB（近いがテンポ感と sn 配置が違う）、ビートレス・アンビエント。

## テンポと骨格
```
// @genre dubstep
// ハーフタイム・ドラム（1サイクル＝ゆっくり2拍子感）
s("bd ~ ~ ~ sd ~ ~ ~").gain(0.9).room(0.12)
// バリエーション
s("bd ~ bd ~ sd ~ ~ bd  ~ ~ ~ sd ~ bd ~ ~").gain(0.88)

// 薄いハット／パーカ
s("hh*8").gain(0.18).hpf(9000)
s("~ ~ cp ~").gain(0.25)

// サブ（長い）
note("c1 ~ ~ ~ ~ ~ ~ ~").s("sine")
  .lpf(100).attack(0.01).release(0.6).gain(0.75)

// ワブル・ベース（主役）
note("c1*8").s("sawtooth")
  .lpf(sine.range(80,1200).fast(4)).lpq(12)
  .gain(0.5).orbit(2)
// ワブル速度をパターン化
note("<c1 c1 eb1 f1>*4").s("sawtooth")
  .lpf(sine.range(100,2000).fast("<2 4 8 16>"))
  .lpq(14).distort(0.3).gain(0.45)
```

## 制作レシピ
1. **ドラム**: キックまばら、スネアはハーフタイムの「後ろ」（例: サイクル後半）。`bd*4` は使わない。
2. **テンポ**: `cpm(140)` 前後でパターンをハーフに書く、または `cpm(70)` で素直に書く。
3. **サブ**: `sine` / 暗い低音、ほぼ mono 的に安定。ドロップで存在感。
4. **ワブル**: `lpf` を `sine`/`tri` で周期変調。`fast(2|4|8|16)` でうねり速度。`lpq` め高め。
5. **ドロップ構成**: Intro（ドラム薄）→ Build（上昇・空き）→ Drop（ワブル全開）→ Break。
6. **空間**: sn/fx に room・delay。ワブルとサブは dry 寄り、別 orbit。

## 音作りの要点（strudel-sound-design）
- 波形: ワブルは `sawtooth`/`square`、サブは `sine`
- 必須: 動的 `lpf` + 高め `lpq`（アシッドより「周期」が規則的）
- 歪み: ドロップで軽い `distort`/`shape`
- 連続変調: `note("c1*8")` や `seg(16)` でフィルターが滑らかに動くようにする
- duck: キックでワブル軌道を軽く `duckorbit` するとクラブ的

## ミニパターン例
```
stack(
  s("bd ~ ~ ~ sd ~ ~ ~").gain(.92),
  s("hh*16").gain(.12).hpf(10000),
  note("c1").s("sine").lpf(90).gain(.7).slow(1),
  note("c1*8").s("sawtooth")
    .lpf(sine.range(120,1800).fast(4)).lpq(14)
    .decay(.3).sustain(.4).gain(.4).orbit(2)
).cpm(140)
```

### ビルド→ドロップの切り替わり（簡易）
```
// ビルド: ワブル遅め・ドラム抜き気味
// ドロップ: fast(8) と distort、bd/sd を厚く
note("c1*8").s("sawtooth")
  .lpf(sine.range(80,2000).fast("<1 2 4 8>".slow(2)))
  .lpq(16).gain(0.45)
```

## DnB / Electro との切り分け
| | Dubstep | DnB | Electro |
| --- | --- | --- | --- |
| 拍感 | ハーフタイム | 高速ブレイク | 中速マシン |
| sn | 遅い裏打ち | 細かく多数 | ファンキー配置 |
| ベース | ワブル周期 | サブ＋Reese | 短い電子音 |
| cpm目安 | 70 or 140 | 160–180 | 110–130 |

## Common Pitfalls
1. `bd*4` のまま → ただの遅いハウス／テックになる。ハーフタイム配置にする。
2. ワブルが音の頭だけで止まる → 持続音＋`seg`/連打で lpf を連続更新。
3. サブとワブルを同軌道・同帯域で飽和 → orbit 分け、サブはより低く gain 調整。
4. 全編ワブル全開 → ドロップ前後の余白がないと疲れる。
5. アシッドと混同 → アシッドは16分フレーズ旋律、ダブは周期LFO的うねりが主。

## Verification Checklist
- [ ] ドラムがハーフタイム（sn が疎）
- [ ] サブまたはワブルがドロップの主役
- [ ] lpf が周期的に動いている（ワブル）
- [ ] キック／サブ／ワブルが帯域・orbit で整理されている
- [ ] `@genre dubstep` 任意
