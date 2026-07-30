---
name: strudel-composition
description: "Use when writing Strudel patterns or mini-notation."
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel, music, composition, mini-notation, patterns]
    related_skills:
      - strudel-data-format
      - strudel-sound-design
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

# Strudel 楽曲の書き方（Composition）

## Overview
Strudel は Tidal Cycles 由来の "Mini-Notation" という小さい記法でリズムパターンを書く。この Skill は、`.strudel` コード内で楽曲を記述するための記法（シーケンス、ネスト、ユークリッドリズム等）、パターン生成関数、タイムモディファイア、パラメータ制御をまとめる。

## When to Use
- ユーザーが Strudel で実際に曲を書く・パターンを組む時
- mini-notation の記法やファクトリ関数を使う時
- 音符・リズム・構成をプログラム的に操作したい時

Don't use for: 保存形式・メタデータ（→ strudel-data-format）、シンセ/エフェクト音作り（→ strudel-sound-design）。

## Mini-Notation の基本
バッククォート `` ` `` で複数行、ダブルクォート `"` で単一行、シングルクォート `'` で非パース文字列。

### サイクル内のイベント列
```
note("c e g b")   // 4音が1サイクルに圧縮、各音は1/4サイクル
note("c d e f g a b")  // 音が増えると各音長は短くなる（サイクル長は一定）
```

### 乗算（*）
```
note("[e5 b4 d5 c5]*2")      // 1サイクル中に2回
note("[e5 b4 d5 c5]*2.75")   // 小数も可
```

### 除算（/）
```
note("[e5 b4 d5 c5]/2")      // 2サイクルにわたって再生
```

### 山括弧 < >
イベント数で長さを決定（ピアノロール的）。
```
note("<e5 b4 d5 c5>")        // /4 と同等
note("<e5 b4 d5 c5>*8")      // 1サイクルに8音
```

### 括弧ネスト [ ]
時間を細分化。内側の列は外側1イベントの長さになる。
```
note("e5 [b4 c5] d5 [c5 b4]")
```

### 休符
`~` または `-`。
```
note("[b4 [~ c5] d5 e5]")
```

### 並列／ポリフォニー（,）
```
note("[g3,b3,e4]")           // コード
note("<[g3,b3,e4] [a3,c3,e4] [b3,d3,f#4]>*2")
```

### 伸長（@）
時間的重み。
```
note("<[g3,b3,e4]@2 [a3,c3,e4] [b3,d3,f#4]>*2")
```

### 複製（!）
早くせず繰り返し。
```
note("<[g3,b3,e4]!2 [a3,c3,e4] [b3,d3,f#4]>*2")
```

### ランダム（?, |）
```
note("[g3,b3,e4]*8?")        // 50%で除去
note("[g3,b3,e4]*8?0.1")    // 10%で除去
note("[g3,b3,e4] | [a3,c3,e4] | [b3,d3,f#4]")  // ランダム選択
```

### ユークリッドリズム（(beats,segments,offset)）
```
s("bd(3,8,0)")   // 3拍/8セグメント、位置0（Pop Clave）
s("bd(3,8)")     // offset省略可
s("bd(3,8,3), hh cp")
```

## パターン生成関数（ファクトリ）
| 関数 | mini |
| --- | --- |
| `cat(x,y)` (slowcat) | `""` |
| `seq(x,y)` (fastcat) | `"x y"` |
| `stack(x,y)` (polyrhythm) | `"x,y"` |
| `stepcat([3,x],[1,y])` (timecat) | `"x@3 y@2"` |
| `polymeter(a,b,c, x,y)` | `"{a b c, x y}"` |
| `polymeterSteps(2,x,y,z)` | `"{x y z}%2"` |
| `silence` | `"~"` |

```
cat("e5","b4",["d5","c5"]).note()
stack("g3","b3",["e4","d4"]).note()
arrange([4,"(3,8)"],[2,"(5,8)"]).note()
n(run(4)).scale("C4:pentatonic")      // 0..3 の離散パターン
"hh".s().struct(binary(5))            // "1 0 1"
```

## タイムモディファイア
| 関数 | mini |
| --- | --- |
| `slow(2)` | `/2` |
| `fast(2)` | `*2` |
| `euclid(3,8)` | `(3,8)` |
| `euclidRot(3,8,1)` | `(3,8,1)` |

```
s("bd hh sd hh").slow(2)
s("bd hh sd hh").fast(2)
note("c3").euclid(3,8)             // キューバン・トレシージョ
note("c3").euclidRot(3,16,14)      // サンバ
note("c d e g").rev()               // 反転
note("c d e g").palindrome()        // 往復
note("0 1 2 3".scale('A minor')).iter(4)
s("bd ~ sd cp").ply("<1 2 3>")
s("bd*2 hh*3 [sd bd]*2 perc").zoom(0.25,0.75)
s("hh*8").swing(4)                  // swingBy(1/3,4)
s(",hh*2").cpm(90)                  // 90 BPM
```

## コントロールパラメータ
各パラメータは独立して制御できる。
```
note("c e g b")
 .cutoff("<500 1000 2000>")
 .gain(0.8)
 .s("sawtooth")
 .log()
```

### 値の修飾（演算子）
```
note("c e g").add(2)        // 数値+2
note("c e g").mul(2)        // ×2
note("c e g").range(0,1)    // 0..1 → 範囲スケール
```
チェーンで param 関数を重ね可能:
```
note(cat('c','e','g'))
```

## Common Pitfalls
1. サイクル長が固定なのを忘れる → 「音符を足すとテンポが上がる」と誤解。足すと各音が短くなる。
2. パラメータ無しの文字列を鳴らそうとする → `note` 等で包む必要あり。
3. `*` と `/` を逆にする → `*` は早く、`/` は遅く。
4. ユークリッドの offset を忘れて位相が合わない → hh 等と重ねて聴く。

## Verification Checklist
- [ ] バッククォート/ダブルクォート/シングルクォートの使い分けが正しい
- [ ] サイクル内イベント数と乗除算でテンポ意図が一致している
- [ ] コード・休符・ネストが `,` `[]` `~` で正しく書けている
- [ ] ユークリッド記法 `(beats,segments,offset)` の引数順が正しい
- [ ] 各音が `note`/`s` 等の param 関数で包まれている
