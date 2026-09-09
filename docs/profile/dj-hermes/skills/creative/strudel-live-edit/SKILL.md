---
name: strudel-live-edit
description: "Use when editing a playing dj-hermes song from natural language: add melody, drum fill, modulate/transpose, brighter/darker."
version: 1.4.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [dj-hermes, music, live-coding, edit, melody, drums, scale, transpose]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-minimal
---

# dj-hermes ライブ編集（自然言語 → 差分）

## Overview

来場者の自然言語を、**どの `$:` をどう書き換えるか** に落とす Skill。  
**他パートを書き換えない**ことが最優先。部分編集 API を使い、全文 `dj_hermes_apply_song` は新規曲・大規模構成変更に限る。

記法の正本は **strudel-composition**。音色は **strudel-sound-design**。

## 共通手順

1. **`dj_hermes_get_song(deck)`** で現状の `tracks[]` / `source` を読む（記憶だけで全文を書かない）  
2. 下の **NL ルーティング表**で対象トラックと操作を決める  
3. **1 意図だけ**適用する（優先順）:
   - メソッド 1 個（`.lpf` / `.gain` / `.add` / `.ply` / `.scale` 等）→ **`dj_hermes_edit_method(deck, track, op, method, args?)`**
     - `op`: `set`（同名は末尾を置換、無ければ追加） / `add`（末尾に追加） / `remove`
   - 1 トラックのチェーン丸ごと差し替え・追加・削除 → **`dj_hermes_patch_track(deck, track, op, code?, name?)`**
   - 新規曲や大規模な再構成のみ → `dj_hermes_apply_song(content, deck)`（ディスクに書かない）。新規は **strudel-composition の 7–8 本**（ジャンル例外あり。ミニマルは 14–16 本）。一言の要望で全部を作り直さない
4. バー境界で反映。チャットにコードだけ書いて終わりにしない  

### edit_method 例

```
# ベースに LPF
dj_hermes_edit_method(deck="A", track="bass", op="set", method="lpf", args="400")

# LFO カットオフ
dj_hermes_edit_method(deck="A", track="bass", op="set", method="lpf", args="sine.rangex(500,4000)")

# gain を外す
dj_hermes_edit_method(deck="A", track="hat", op="remove", method="gain")
```

## NL ルーティング表

| 言い方の例 | 対象 | 操作の要約 |
| --- | --- | --- |
| メロディ足して / lead 欲しい | 空いている `// lead` / `// hook` / `// arp`。既に 3 本あるときは 1 本を差し替え。**ミニマルは次数を増やさず `// synth` / `// pluck` をオン** | 次数 + `.scale` + `@` で長め音。新規全文は composition / ジャンル Skill |
| ボーカル / チョップ足して | **`// vox`**（カタログ `vc:`）。7 本床なら perc の代わりに append。8 本上限・Future Bass / Kawaii は **arp を `vc:` に差し替え**。ミニマルは texture 差し替えか 15–16 本目 | `vc:pa` 等 + `.cut(1)` + `C4:`。自前 WAV を invent しない。→ **strudel-composition** ボーカルチョップ |
| フィル入れて / ブレイク | `// drums` の `s(...)` | `<>` でフィル層 / `*` / `.ply(n)`。Mixer のディレイ/スイッチは **strudel-dj-mix** |
| 転調 / キー上げ下げ | 全 `.scale` の **ルート** | ルート変更 or `.scale("<…>")` 進行（pitched 全部で揃える） |
| 移調 / 半音上げ / 度数上げ | pitched の `$:` | **`.add(n)` / `.sub(n)`**（次数 or 半音） |
| 明るく / 暗く | 全 `.scale` の **モード** | 明暗梯子を ±1 段（lpf は副次） |
| ハット細かく | drums / chh | 既定は `[~ hh]*4` → `hh*8`。**ミニマルは OHH `[~ oh]*4` を触らず、`// chh` を `[hh hh ~ hh]*4` でオン**（裏拍 `&` には hh を置かない。→ **strudel-genre-minimal**） |
| コード変えて | `// chords` | `[0,2,4]` または進行の `<>` |
| パッド薄く / 厚く | `// pad` | `.gain` / `.lpf` / `.room`。orbit はリードと分けたまま |
| フック変えて | `// hook` | ジャンル署名次数は消さない（house の `4 ~ 7 4 …` 等） |

---

## 1. メロディを追加

**ルール**

- 既存 pitched トラックの **Root:mode をコピー**（キーをバラバラにしない）  
- 次数は **0 始まり**。長音は mini **`@`**（`a@2` = a が b の 2 倍の長さ）  
- 空いている `// lead` / `// hook` / `// arp` を埋める。既に 3 本あるときは 1 本だけ差し替え。新規全文は composition の 7–8 本  

```
// lead — 長めノート + 次数（既存の Root:mode をコピー）
$: note("0@2 2 4@3 ~ 7")
  .scale("C4:minor")
  .s("plk:ps").gain(0.18).cut(1)
```

ベースが `C2:minor` なら lead も **同じ mode**、オクターブだけ `C4:` などにする。

---

## 2. ドラムフィルを追加

**原則**: ドラムは 1 本の `s(...)`。骨格を残し、**`<>` でたまにフィル**。

```
// before
$: s("bd*4, [~ sd]*2, [~ hh]*4").gain(0.55)

// after — サイクルごとに空 or フィル
$: s("bd*4, [~ sd]*2, [~ hh]*4, <~ [~@3 bd ~@4] [bd sd hh sd]>").gain(0.55)
```

密度を一時的に上げる:

```
// キック連打風（イベントを n 分割）
$: s("bd*4, [~ sd]*2, [~ hh]*4").ply(2).gain(0.5)
// または層だけ * を上げる
$: s("bd*4, [~ sd]*2, hh*16, <~ [bd sd bd sd]>").gain(0.55)
```

### 本家語彙 → このエンジン

| 本家・要望 | dj-hermes |
| --- | --- |
| `.ply(2)` | **可（スカラー）** — 各イベントを n 分割連打 |
| `chooseCycles("bd","hh","sd")` | `<bd hh sd>` または `..., <~ [bd sd hh sd]>` |
| `.cutoff(sine.rangex(...))` | **可** — `.lpf(sine.rangex(500,4000))` / `.cutoff(sine.range(200,8000).slow(2))` |

---

## 3. 転調・移調

| 意図 | 手段 | 例 |
| --- | --- | --- |
| **転調**（キーが変わる） | 全 `.scale` の **ルート**を同じ量ずらす | `C2:minor` → `D2:minor` |
| **進行** | `.scale("<Root:mode …>")` | 1 バー 1 スケール |
| **移調**（同じキー感で度数ずらし） | **`.add(n)` / `.sub(n)`** | 次数を +n（scale あり） |

```
// 移調: 次数パターンはそのまま、全体を 2 度上げる
$: note("0 2 0 3 0 <2 4>").scale("C2:minor").add(2)
  .s("sawtooth").lpf(500).gain(0.7)

// 転調: ルートだけ変える（次数はそのまま 0=新ルート）
$: note("0 2 0 3").scale("D2:minor").s("sawtooth").lpf(500).gain(0.7)

// 進行（既存）
$: note("0 2 4 0").scale("<A2:minor D:dorian G:mixolydian C:major>")
  .s("sawtooth").lpf(600).gain(0.5)
```

**`.add` / `.sub` の意味（スカラーのみ）**

- `note`/`n` + **scale + 整数次数**: スケール次数を ±n  
- **音名**（`c4`）: 半音 ±n  
- サンプル `s("bd")` ではピッチに効かない（パースは通る）  
- 引数に `"<1 2>"` は不可  

複数 pitched トラックがあるときは、**全部に同じ `.add`** を付けるか、全部の scale ルートを揃えて変える。

---

## 4. 明るく / 暗く（スケール梯子）

**既定**: ルートは固定し、**モードだけ** 1 段動かす。全 `$:` の mode を同期（オクターブ数字 `C2`/`C4` は維持）。

### 明るい → 暗い（並行モード）

| 段 | mode 名（`.scale`） |
| --- | --- |
| 1（最も明るい） | `lydian` |
| 2 | `major` / `ionian` |
| 3 | `mixolydian` |
| 4 | `dorian` |
| 5 | `minor` / `aeolian` |
| 6 | `phrygian` |
| 7（最も暗い） | `locrian` |

```
// before
$: note("0 2 4 0").scale("C2:minor").s("sawtooth").lpf(500).gain(0.6)

// 少し明るく → dorian（1 段上）
$: note("0 2 4 0").scale("C2:dorian").s("sawtooth").lpf(500).gain(0.6)

// 少し暗く → phrygian（1 段下）
$: note("0 2 4 0").scale("C2:phrygian").s("sawtooth").lpf(500).gain(0.6)
```

| 言い方 | 操作 |
| --- | --- |
| 少し明るく | 梯子で 1 段上 |
| 少し暗く | 梯子で 1 段下 |
| メジャーに | `major` |
| マイナーに | `minor` |
| もっとドラマチックに暗く | `phrygian` または `locrian`（locrian は慎重に） |

**副次**（キーは変えない音色側）: `.lpf` を上げる=開けた印象、下げる=こもった印象。  
明暗の第一手段は **scale**。lpf だけで「暗い」にしない。

---

## 本家語彙まとめ

| 要望 | 使う |
| --- | --- |
| `.add(2)` / `.sub(1)` | **可**（スカラーまたは `add("<0 2>")` パターン） |
| `.ply(2)` | **可**（スカラー、1..=16） |
| `chooseCycles` | `<a b c>` |
| 動的 cutoff / `sine.rangex` | **可** — mini `lpf("<…>")` または `lpf(sine.rangex(…))` |
| 長音 | mini `@` |
| 明るく/暗く | モード梯子 ±1 |

不可のまま: `stack(...)`、`.cpm()`、`.lfo(...)`（LFO は `lpf(sine.rangex(...))` 等を使う）。`cp` は同梱。ハウス 2/4 専用。  
可: mini パターン `.lpf("<…>")`、連続 LFO `.lpf(sine.rangex(...))`。

---

## Pitfalls

1. 毎回フル曲を `dj_hermes_save_song` → **get → edit_method / patch_track**（新規は apply_song）
2. lead だけ別キーにする → 既存 `.scale` の Root:mode をコピー  
3. 「暗く」を lpf だけ → まず mode を下げる  
4. `.add` のパターン引数は実装どおり（不明なら composition を見る）  
5. ドラムを kick/hat/snare の 3 `$:` に分けない（duckorbit キックのみ例外。**ミニマルの kick / ohh / chh 分割は例外**）  
6. `get_song` せず記憶の古い content で全文上書き → 他トラック破壊  

## Checklist

- [ ] `dj_hermes_get_song` で現状を読んだ  
- [ ] NL 表で対象 `$:` と操作を決めた  
- [ ] 1 意図だけ（edit_method または patch_track）  
- [ ] scale / add / ply / lpf LFO は実装済みの使い方  
- [ ] 全文 apply は新規・大規模変更のときだけ。save は残す指示のときだけ
