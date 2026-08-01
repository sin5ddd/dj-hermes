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
// drums — smart parallel (* densify, @ elongate)
$: s("bd*4, [~ sd]*2, [~ hh]*4").gain(0.55)
// bass
$: note("0 0 2 4").scale("C2:minor").s("sawtooth").lpf(500).gain(0.55)
```

変化のある小節（休符バーとフィル）:

```
// @title demo-break
setcpm(124/4)
// drums
$: s("bd*4, [~ sd]*2, [~ hh]*4, <~ [~@3 bd ~@4]>").gain(0.55)
// bass
$: note("0 2 3 4").scale("C2:minor").s("sawtooth").lpf(450).gain(0.5)
```

## Mini-notation（`$:` の文字列内だけ）

| 記法 | 意味 |
| --- | --- |
| `bd sd hh` | **スペース** → 順再生（ウェイト既定 1 でサイクルを分割） |
| `bd,sd` / `[bd,sd]` | **カンマ** → **同時再生** |
| `bd*4` | 1 サイクルに 4 回（密度アップ） |
| `a@2 b` | **`@` elongate** — 時間ウェイト（a が b の 2 倍の長さ。既定 1） |
| `~` | 休符 |
| `[a b]` | 細分化シーケンス |
| `<a b c>` | サイクルまたぎで 1 つずつ（同時ではない） |
| `bd*4, [~ sd]*2` | 層ごとの密度を保った並列 |

```
// 推奨ドラム（4つ打ち + 裏 snare + オフビート hh）
$: s("bd*4, [~ sd]*2, [~ hh]*4").gain(0.55)
// 不均等グリッド（@ = weight）
$: s("[~@3 bd ~@4]").gain(0.8)
// ユーザー例フル
$: s("bd*4, [~ sd]*2, [~ hh]*4, <~ [~@3 bd ~@4]>").gain(0.55)
// 16 小節は cat に並列パターン
$: s(cat("bd*4, [~ sd]*2, [~ hh]*4", "bd*4, [~ sd]*2, hh*16")).gain(0.55)
```

## ドラム統合ルール（必須）

1. **原則 1 トラック**: `// drums` + 1 本の `$:` `s(...)`  
2. **定番骨格** → `bd*4, [~ sd]*2, [~ hh]*4`（または `hh*8`）  
3. **不均等** → `@`（例: `bd ~@3`、`[~@3 bd ~@4]`）  
4. **小節差分** → `<>` / `cat`  
5. **例外で分離** — `.duckorbit` 付きキックなどチェーンが違うときだけ  
6. 本家 `stack(...)` は使わない（同時は mini の `,`）

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
- [ ] ドラムは原則 1 本の `s(...)`（`,` 並列、`*` 密度、`@` 伸長）  
- [ ] ベースは可能なら次数 + `.scale("X2:mode")`（負次数 = ルート下）  
- [ ] mini は引用符の中だけ  
- [ ] `strudel_save_song` で保存（必要なら `deck`）  
