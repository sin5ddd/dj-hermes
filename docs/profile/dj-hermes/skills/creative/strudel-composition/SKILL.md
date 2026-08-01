---
name: strudel-composition
description: "Use when writing strudel-rs patterns (mini-notation + $: tracks). Prefer consolidated drum s() patterns."
version: 2.1.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, composition, mini-notation, drums]
    related_skills:
      - strudel-data-format
      - strudel-sound-design
---

# strudel-rs 作曲（Composition）

## Overview
strudel-rs は **曲ファイル**として `setcpm` + **`$:` トラック**だけを受け付ける。mini-notation は文字列の中（`s("...")` / `note("...")`）で使う。保存は必ず `strudel_save_song`。

**ドラムは原則 1 本の `$:` `s(...)` に統合する**（本家 Strudel らしい書き方）。kick / hat / snare を別トラックに分けない。

## 保存テンプレ（そのまま content に）

```
// @title demo
setcpm(120/4)
// drums (space = sequence, comma = simultaneous)
$: s("[bd hh [bd,sd] hh]*2").gain(0.75)
// bass
$: note("c2 c2 eb2 g2").s("sawtooth").lpf(500).gain(0.55)
```

密度が違う層をまとめるとき（推奨）:

```
// @title demo-parallel
setcpm(124/4)
// drums
$: s("bd*4, hh*8, ~ sd ~ sd").gain(0.55)
// bass
$: note("c2 c2 eb2 g2").s("sawtooth").lpf(450).gain(0.5)
```

## Mini-notation（`$:` の文字列内だけ）

| 記法 | 意味 |
| --- | --- |
| `bd sd hh` | **スペース** → 順に再生（サイクルを等分） |
| `bd,sd` / `[bd,sd]` | **カンマ** → **同時再生**（同じ時間スロット） |
| `bd*4` | 1 サイクルに 4 回 |
| `~` | 休符 |
| `[a b]` | 細分化シーケンス |
| `<a b c>` | サイクルまたぎで 1 つずつ（同時ではない） |
| `bd*4, hh*8` | 層ごとの密度を保ったまま並列 |

```
// 短いグリッド（教科書）
$: s("[bd hh [bd,sd] hh]*2").gain(0.75)
// 並列（4つ打ち + 8分ハット + 裏スネア）
$: s("bd*4, hh*8, ~ sd ~ sd").gain(0.55)
// 16 小節は cat の各引数に並列パターンを載せる
$: s(cat("bd*4, hh*8, ~ sd ~ sd", "bd*4, hh*16, ~ sd ~ [sd sd]")).gain(0.55)
```

## ドラム統合ルール（必須）

1. **原則 1 トラック**: `// drums` + 1 本の `$:` `s(...)`  
2. **密度が違う層** → トップレベル並列: `bd*4, hh*8, ~ sd ~ sd`  
3. **短いループ** → グリッド: `[bd hh [bd,sd] hh]*2`  
4. **例外で分離してよい場合だけ**  
   - `.duckorbit` 付きキック（duck を hat/snare に付けない）  
   - 層ごとに全く違う FX / orbit が必要  
5. 統合時の `.gain` は **1 つの妥協値**（イベント単位 gain は不可）  
6. 本家の `stack(...)` 関数は **使わない**（同時は mini の `,` で書く）

## ベース / メロディの圧縮

### 1. 音名 + `<>`（小節差分だけサイクル切替）

```
$: note("c2 eb2 c2 f2 c2 <eb2 g2> <g2 eb2> <eb2 <f2 g2>>").s("sawtooth").lpf(600).gain(0.5)
```

### 2. 次数 + `.scale`（推奨・本家寄り）

次数は **0 始まり・ルート相対**。マイナス可（`C2:major` の `-1` → B1、`C2:minor` の `-1` → Bb1）。

```
// C minor: 0=c 1=d 2=eb 3=f 4=g 5=ab 6=bb 7=c+oct
$: note("0 2 0 3 0 <2 4> <4 2> <2 <3 4>>").scale("C2:minor").s("sawtooth").lpf(600).gain(0.5)
// head の n(...) も可（メソッド .n(1) サンプル index とは別）
$: n("0 0 2 4").scale("C2:minor").s("sine").lpf(400).gain(0.55)
```

| scale 引数 | 意味 |
| --- | --- |
| `C2:minor` | root=C2、自然短（ベースはオクターブ明示） |
| `C:major` | root オクターブ省略 → **C4** |
| `A2:minor:pentatonic` | 複合モード |

対応モード（v1）: `major`/`ionian`, `minor`/`aeolian`, `dorian`, `phrygian`, `lydian`, `mixolydian`, `locrian`, `major:pentatonic`/`pentatonic`, `minor:pentatonic`, `chromatic`。

ルール:

1. 似た小節 → `<>` で差分  
2. 同一スケール → 次数 + `.scale("RootOct:mode")`  
3. ベースは **`C2:` のようにオクターブを付ける**（省略は 4）  
4. 構造の違うブロック（break/B）は `cat` で区切ってよい  
5. `.scale("<C F>:minor")` のような動的 scale は不可（スカラーのみ）

## チェーン（スカラー引数のみ）

```
$: note("0 2 3 4").scale("C2:minor").s("sawtooth").lpf(600).lpq(8).gain(0.5)
$: s("bd*4, hh*16").hpf(200).gain(0.45)
```

不可: `.lpf("<400 1200>")`、`.vib("<1 4>")`、`stack(...)`、`.cpm(120)`。

## テンポ

- `setcpm(30)` → BPM 120（30 cycles/min × 4 beats）  
- `setcpm(120/4)` → BPM 120 と同じ書き方  
- 体感テンポを上げるのは主に mini の `*2` / `.fast(2)`  

## `cat()` について

- **曲ファイルでは可**: `s(cat("a", "b", "c"))`（1 引数 = 1 小節、`<a b c>` と同じ考え方）  
- 16 小節デモは `cat` + 並列ドラムが標準  
- 本家 JS の `arrange` / `stack` 関数は不可  

## 禁止

- 本家 JS: `stack(...)`、`.cpm()`、裸の `s("...")` 行（`$:` 無し）  
- kick / hat / snare を **duck 等の理由もなく** 3 トラックに分けること  

## Pitfalls

1. チャットにコードだけ書いて保存しない  
2. `stack(...).cpm(170)` を content に入れる → 400  
3. 引数にミニ記法パターンを入れる → 非対応  
4. duck 付きキックと hat を 1 本にまとめない  

## Checklist

- [ ] `setcpm` + 複数 `$:`  
- [ ] ドラムは原則 1 本の `s(...)`（`,` 同時 / スペース順）  
- [ ] ベースは可能なら次数 + `.scale("X2:mode")`（負次数 = ルート下）  
- [ ] mini は引用符の中だけ  
- [ ] `strudel_save_song` で保存（必要なら `deck`）  
