---
name: strudel-sound-design
description: "Use when designing Strudel synths or effects."
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel, music, sound-design, synthesis, effects]
    related_skills:
      - strudel-data-format
      - strudel-composition
      - strudel-genre-acid
      - strudel-genre-electro
      - strudel-genre-minimal-techno
      - strudel-genre-house
      - strudel-genre-dnb
      - strudel-genre-ambient
      - strudel-genre-chill
      - strudel-genre-dubstep
      - strudel-genre-progressive-house
      - strudel-genre-future-bass
      - strudel-genre-lofi-hiphop
      - strudel-genre-chill-pop
---

# Strudel サウンドメイク（Sound Design）

## Overview
Strudel はサンプリングに加え、WebAudio ベースのシンセと内蔵エフェクトを持つ。この Skill は、波形・加算合成・FM・ウェーブテーブル・ZZFX などの音源と、フィルタ/ADSR/リバーブ/ディレイ等のエフェクト（信号チェーン順）をまとめる。

## When to Use
- ユーザーが Strudel で音そのものを作る・音色を設計する時
- シンセ（FM/加算/ウェーブテーブル/ZZFX）を使う時
- フィルタ・エンベロープ・空間系エフェクトをかける時

Don't use for: パターン記法（→ strudel-composition）、保存形式（→ strudel-data-format）。

## 基本波形
`sound`（または `s`）で選択。note のみで sound 未指定の場合デフォルトは triangle。
```
note("c2 >".fast(2)).sound("sine")      // sine / sawtooth / square / triangle
```

## ノイズ
`white` / `pink` / `brown`。
```
sound("white")._scope()
sound("bd*2,*8").decay(.04).sustain(0)._scope()   // ハイハット代わり
note("c3").noise("<0.1 0.25 0.5>")                // 任意波形へ加算
s("crackle*4").density("<0.01 0.04 0.2 0.5>".slow(2))
```

## 加算合成（partials / phases）
```
note("c2 >".fast(2)).sound("sawtooth")
 .partials([1,1,"<1 0>","<1 0>","<1 0>","<1 0>","<1 0>"])
note("c2 >".fast(2)).sound("user").partials([1,0,0.3,0,0.1,0,0,0.3])
note("c2 >").fast(2).sound("user").partials(randL(200)).phases(randL(200))
```
- `partials`: 各倍音の相対マグニチュード（先頭=基本波）。
- `phases`: 各倍音の位相。

## ビブラート
```
note("a e").vib("<.5 1 2 4 8 16>")        // 周波数(Hz)
note("a e").vib(4).vibmod("<.25 .5 1 2 12>")  // 深度(半音): で周波数も可
```

## FM 合成
```
note("c e g b g e").fm(4).fmh("<1 2 1.5 1.61>")  // 調波比
 .fmattack("<0 .05 .1 .2>")                       // アタック
 .fmdecay("<.01 .05 .1 .2>").fmsustain(.4)        // ディケイ/サステイン
 .fmenv("")                                        // ランプ型 lin|exp
```
- `fmh` 整数/単純比は自然、小数/複雑比は金属的。
- 後ろに数字で個別FM指定可（例 `fmh2`, `fmatt5`）。

## ウェーブテーブル合成
`wt_` 接頭辞のサンプルは1サイクル波形として読込（loop=1）。
```
samples('bubo:waveforms')
note("<[g3,b3,e4]!2 [a3,c3,e4] [b3,d3,f#4]>")
 .n("<1 2 3 4 5 6 7 8 9 10>/2").s('wt_flute')
 .loopBegin(0).loopEnd(1)
```

## ZZFX
20パラメータのミニシンセ。
```
note("c2 eb2 f2 g2").s("{z_sawtooth z_tan z_noise z_sine z_square}%4")
 .attack(0.001).decay(0.1).sustain(.8).release(.1)
 .curve(1).slide(0).noise(0).zmod(0).zcrush(0).zdelay(0)
 .pitchJump(0).pitchJumpTime(0).lfo(0).tremolo(0.5)
```

## 信号チェーン順（重要）
1. 音発生（サンプル/オシレータ）
2. デチューン系（detune, penv）
3. 以下順次（呼んだ分だけ適用、同じエフェクトは上書きされる）:
   stretch → gain → lpf → hpf → bandpass → vowel → coarse → crush → shape → distort → tremolo → compressor → pan → phaser → post
4. 分配: dry + 送り先（delay, room リバーブ）

注意: 単一エフェクトはパターン内で複数書いても最後が優先（`lpf().distort().lpf()` は不可）。

## フィルタ
| 関数 | 意味 | 追加パラメータ |
| --- | --- | --- |
| `lpf` (cutoff) | ローパス | `lpq`(共振) |
| `hpf` (hcutoff) | ハイパス | `hpq` |
| `bpf` (bandf) | バンドパス | `bpq` |
| `ftype` | 0=12db,1=ladder,2=24db | `fanchor`,`lpenv` |

mini-notation では `:` で q を指定: `"1000:10"`。
```
s("bd*16").lpf("1000:0 1000:10 1000:20")
note("c f g g a c d4").fast(2).sound('sawtooth').lpf(200).fanchor(0).lpenv(3).lpq(1).ftype("")
```

## 母音フィルタ
```
note("[c2 >]*2").s('sawtooth').vowel(">")
s("bd sd mt ht bd [~ cp] ht lt").vowel("[a|e|i|o|u]")
```
vowel 値: a e i o u ae aa oe ue y uh un en an on

## 振幅エンベロープ（ADSR）
```
attack(0..) / decay(0..) / sustain(0..1) / release(0..)
note("[c3 bb2 f3 eb3]*2").sound("sawtooth").lpf(600).adsr(".1:.1:.5:.2")
```

## 振幅変調（トレモロ）
```
note("d d d# d".fast(4)).s("supersaw").tremolosync("4").tremoloskew("<1 .5 0>")
```

## リバーブ / ディレイ
```
.room(0.5).size(10)        // リバーブ量 / サイズ
.rdim(400).rlp(5000)       // ディケイ / ローパス
.delay(0.25).delaytime(.3).delayfeedback(.5)
```
単一オービットにつきリバーブ/ディレイは1つ。複数パターンで同じ orbit を共有すると上書きされる → 別 orbit を割り当て。

## Phaser
```
n(run(8)).scale("D:pentatonic").s("sawtooth").release(0.5)
 .phaser("<1 2 4 8>").phaserdepth("<0 .5 .75 1>").phasercenter("<800 2000 4000>").phasersweep("<800 2000 4000>")
```

## Orbits（出力ルーティング）
- デフォルト orbit=1。`.orbit(2)` で別チャンネルに。
- マルチチャンネル設定時は orbit i → チャンネル 2i, 2i+1（DAW録音用）。
- `.orbit("2,3,4")` で複数コピー（音量3倍注意、gain下げる）。

## Duck（サイドチェイン風）
```
$: n(run(16)).scale("c:minor:pentatonic").s("sawtooth").delay(.7).orbit(2)
$: s("bd:4!4").beat("0,4,8,11,14",16).duckorbit(2).duckattack(0.2).duckdepth(1)
```

## Common Pitfalls
1. フィルタ等をチェーンで2回書く → 最後が上書きされる。1回のみ。
2. 同じ orbit を複数パターンで共有 → room/size 等が上書きされて予期せぬ音に。orbit を分ける。
3. 連続的な LFO フィルタを期待 → 音発生ごとにしかサンプルされない。`seg(16)` 等でイベント増やす。
4. `.orbit("2,3,4")` で単純に音量が3倍 → gain 下げる。

## Verification Checklist
- [ ] 波形/シンセ種別が sound(s) で指定されている（noteのみなら triangle になる）
- [ ] フィルタは1種1回しか書いていない
- [ ] リバーブ/ディレイを共有するパターンは別 orbit を使っている
- [ ] 連続変調が必要なら seg 等でイベント数を確保している
- [ ] 信号チェーン順（gain→lpf→…→post→delay/room）を意識している
