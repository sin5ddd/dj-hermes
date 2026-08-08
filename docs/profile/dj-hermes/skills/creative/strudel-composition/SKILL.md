---
name: strudel-composition
description: "Use when writing strudel-rs live patterns: short loops, mini-notation, iterative save."
version: 3.3.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, composition, mini-notation, live-coding, drums]
    related_skills:
      - strudel-data-format
      - strudel-sound-design
      - strudel-live-edit
---

# strudel-rs 作曲（Composition）— ライブ短いループ

## Overview

strudel-rs の曲は **短いループを `$:` で重ね、演奏しながら少しずつ書き換える** のが本筋。

- 保存形式: `setcpm` + **`$:` トラック**のみ（`strudel_save_song`）
- mini-notation は文字列の中だけ（`s("...")` / `note("...")`）
- **既定は 1 サイクル骨格 + `<>` で小差分**（2–5 トラック）
- **16 小節 `cat` の長尺アレンジは既定にしない**（ライブ差分が重い）
- **音色・サンプル**: ドラムは短い `bd`/`sd`/… + 任意 `.bank`。pad/lead/FX はフルネームまたはシンセ（→ **strudel-sound-design** / `samples/LAYOUT.md`）

## 正本テンプレ（そのまま content に）

```
// @title live-basic
// @genre techno
setcpm(128/4)

// bass — 同じ骨格; <> がサイクルごとに末尾を変える
$: note("0 2 0 3 0 <2 4> <4 2> <2 <3 6>>")
  .scale("C2:minor")
  .s("sawtooth").lpf(500).gain(0.7)
  .attack(0.001).decay(0.06).sustain(0.15).release(0.04)

// drums — 1 本（* 密度, , 並列, <> で snare/oh）
$: s("bd*4, [~ <sd oh>]*2, [~ hh]*4").gain(0.5)

// lead
$: note("7 6 <4 9> <3 [4 2]>")
  .scale("C2:minor")
  .s("square").lpf(3200).gain(0.16)
  .attack(0.001).decay(0.5).sustain(0.1).release(0.03)

// chords — 並列次数 [6,8]
$: note("0 2 4 [6,8] 0 2 4 [7,9]")
  .scale("C4:minor")
  .s("sawtooth").gain(0.35).bpf(1500)
```

もっと短い 2 トラック版:

```
// @title demo
setcpm(120/4)
$: s("bd*4, [~ sd]*2, [~ hh]*4").gain(0.55)
$: note("0 0 2 4").scale("C2:minor").s("sawtooth").lpf(450).gain(0.5)
```

## ライブ編集ワークフロー（必須）

1. 初回: 正本に近い **短い** content を `strudel_save_song(name, content, deck)`  
2. 来場者の要望: **`strudel_get_song(deck)`** → **1 トラック or 1 メソッド**だけ  
   - パラメータ 1 個 → `strudel_edit_method`  
   - 1 本の `$:` 差し替え/追加 → `strudel_patch_track`  
   - 全文 `strudel_save_song` は大規模変更・新規のみ  
3. バー境界で反映される（チャットにコードだけ書いて終わりにしない）  
詳細レシピは **strudel-live-edit**。

| 来場者の言い方 | 変更例 |
| --- | --- |
| ハット細かく | `[~ hh]*4` → `hh*8` |
| ベース動かして | 次数の末尾を `<>` で差し替え |
| 暗い / 明るい | **モード梯子**（→ **strudel-live-edit**）。副次で lpf |
| ブレイク / フィル | drums に `<>` / `.ply(2)`（→ live-edit） |
| メロディ足して | lead `$:` + `@`（→ live-edit） |
| 転調 / 移調 | scale ルート or `.add`/`.sub`（→ live-edit） |
| コード足して | chord の `$:` を 1 本追加 or `[6,8]` を変える |
| 進行変えて | `.scale("<A2:minor D:dorian …>")` の中身を差し替え |

自然言語の編集レシピの詳細は **strudel-live-edit**。

**アンチパターン**: 毎回 8–16 引数の `cat(...)` を一から生成する。

## Mini-notation（`$:` の文字列内だけ）

| 記法 | 意味 |
| --- | --- |
| `bd sd hh` | **スペース** → 順再生（ウェイト既定 1） |
| `bd,sd` / `[bd,sd]` | **カンマ** → **同時再生** |
| `bd*4` | 1 サイクルに 4 回（密度アップ） |
| `a@2 b` | **`@` elongate** — 時間ウェイト（a が b の 2 倍） |
| `~` | 休符 |
| `[a b]` | 細分化シーケンス |
| `<a b c>` | サイクルまたぎで 1 つずつ（同時ではない） |
| `bd*4, [~ sd]*2` | 層ごとの密度を保った並列 |

```
// 定番ドラム
$: s("bd*4, [~ sd]*2, [~ hh]*4").gain(0.55)
// 不均等グリッド
$: s("[~@3 bd ~@4]").gain(0.8)
// たまにフィル（1 サイクル骨格のまま）
$: s("bd*4, [~ sd]*2, [~ hh]*4, <~ [~@3 bd ~@4]>").gain(0.55)
```

## ドラム統合ルール

1. **原則 1 トラック**: `// drums` + 1 本の `$:` `s(...)`  
2. **定番骨格** → `bd*4, [~ sd]*2, [~ hh]*4`  
3. **不均等** → `@`  
4. **小節っぽい差分** → パターン内の `<>`（まずこれ）  
5. **例外で分離** — `.duckorbit` 付きキックだけ別 `$:`  
6. 本家 `stack(...)` は使わない  
7. **パート名は短く**（`bd` `sd` `hh` `oh`）。キット差は **`.bank("tr808-hard")` 等**（ディスクは `{bank}_{part}`）。フルネームでリズムを埋めない  
8. **`bd:00` は不可** → `s("bd")` または `.n(0)`  

ユーザーキットがあるとき:

```
$: s("bd*4, [~ sd]*2, [~ hh]*4").bank("tr808-hard").gain(0.55)
```

bank を付けないと同梱 `samples/bd/` 等が使われる。

## ベース / メロディ

次数は **0 始まり・ルート相対**。マイナス可（`C2:major` の `-1` → B1、`C2:minor` の `-1` → Bb1）。

```
// C minor: 0=c 1=d 2=eb 3=f 4=g 5=ab 6=bb 7=c+oct
$: note("0 2 0 3 0 <2 4> <4 2>").scale("C2:minor").s("sawtooth").lpf(600).gain(0.5)
// head の n(...) も可（メソッド .n(1) サンプル index とは別）
$: n("0 0 2 4").scale("C2:minor").s("sine").lpf(400).gain(0.55)
// 和音は並列次数; 移調は .add/.sub（スカラー）
$: note("0 2 4 [6,8]").scale("C4:minor").s("sawtooth").gain(0.3)
$: note("0 2 4").scale("C2:minor").add(2).s("sawtooth").lpf(500).gain(0.5)
```

| scale 引数 | 意味 |
| --- | --- |
| `C2:minor` | root=C2、自然短（ベースはオクターブ明示） |
| `C:major` | オクターブ省略 → **C4** |
| `A2:minor:pentatonic` | 複合モード |
| `<A2:minor D:dorian G:mixolydian C:major>` | **コード進行**（1 サイクル＝1 スケール） |

対応モード（v1）: `major`/`ionian`, `minor`/`aeolian`, `dorian`, `phrygian`, `lydian`, `mixolydian`, `locrian`, `major:pentatonic`/`pentatonic`, `minor:pentatonic`, `chromatic`。

### コード進行テク（`.scale("<…>")`）

同じ次数パターンのまま、**サイクルごとにルート/モードを切り替える**。

```
// 4 サイクルで Am → Ddor → Gmix → Cmaj（次数 0 がルートを辿る）
$: note("0 2 4 0").scale("<A2:minor D:dorian G:mixolydian C:major>")
  .s("sawtooth").lpf(600).gain(0.5)
// 和音レイヤも同じ進行を共有
$: note("[0,2,4] ~ [0,2,4] ~")
  .scale("<A2:minor D:dorian G:mixolydian C:major>")
  .s("triangle").lpf(1400).gain(0.28)
```

ルール:

- `<>` の **内側は空白区切り**の `Root:mode`（`:` はスケール名の一部）  
- **1 引数 = 1 サイクル（1 bar）**。4 個なら 4 小節で一周  
- 次数パターンは固定のまま、スケール側で進行を出す（ライブで `<>` の中身を差し替えやすい）  
- オクターブ省略は root 4（`D:dorian` → D4）。ベース用は `A2:minor` のように明示  
- 固定キーは従来どおり `.scale("C2:minor")`  

ルール（一般）:

1. 似た変化 → **`<>` で差分**（長尺 `cat` より先）  
2. 同一キー → 次数 + `.scale("RootOct:mode")`  
3. **進行** → 次数固定 + `.scale("<Root:mode …>")`  
4. ベースは **`C2:` / `A2:` のようにオクターブを付ける**  

## チェーン（スカラー引数のみ）

```
$: note("0 2 3 4").scale("C2:minor").s("sawtooth").lpf(600).lpq(8).gain(0.5)
$: s("bd*4, hh*16").hpf(200).gain(0.45)
```

不可: `.vib("<1 4>")`、`stack(...)`、`.cpm(120)`、**`.lfo(...)`**（未実装）。  
可: **`.scale("<…>")` 進行**、**`.lpf("<400 1200>")`** / **`.lpf(sine.rangex(500,4000))`**、**`.add` / `.sub` / `.ply`**（詳細は live-edit / sound-design）。

同梱サンプル: `bd` `sd` `hh` `oh`（`cp` は同梱無し → `sd`/`oh`、またはユーザー `{bank}_cp`）。  
追加キット・pad/lead の置き方: **`samples/LAYOUT.md`** / 音色は **strudel-sound-design**。

pad / lead / piano でユーザー WAV がある例:

```
$: note("0 2 4 7").scale("C3:minor").s("pad-ambient_drone01").room(0.4).orbit(1).gain(0.35)
$: note("7 6 <4 9>").scale("C4:minor").s("lead-supersaw_4oct").lpf(2800).gain(0.16)
$: note("0 2 4 0").scale("C3:minor").s("piano-acoustic_soft").gain(0.35)
```

（ファイルが無ければ `wt_organ` / `square` / `triangle` 等のシンセに戻す。ピアノ感はサンプル推奨。）

## テンポ

- `setcpm(30)` → BPM 120（30 cycles/min × 4 beats）  
- `setcpm(120/4)` → 同じ  
- 体感テンポ上げは mini の `*2` / `.fast(2)`  

## `cat()` について

- **曲ファイルでは可**: `s(cat("a", "b"))`（1 引数 = 1 小節）  
- **既定では使わない**。短いループ + `<>` で足りる  
- 来場者が「A メロと B メロをはっきり分けたい」など明確なときだけ  
- 本家 JS の `arrange` / `stack` は不可  

## 禁止

- 本家 JS: `stack(...)`、`.cpm()`、裸の `s("...")` 行（`$:` 無し）  
- 理由なく kick/hat/snare を 3 トラックに分ける  
- 未実装: `.lfo`  
- 同梱に無い `cp` を bank なしで使う  
- 既定での 16 小節 `cat` 長尺  
- mini 内の `bd:00` / `kit:bd`（コロン不可）  

## Pitfalls

1. チャットにコードだけ書いて保存しない  
2. `stack(...).cpm(170)` を content に入れる → 400  
3. 引数にミニ記法パターンを入れる → 非対応  
4. 毎回フル曲を `strudel_save_song` で書き直して差分が巨大になる  
5. ドラムをフルファイル名で書く → 読めない。短い part + `.bank`  
6. `{bank}-{part}.wav` とハイフン連結 → 正は `{bank}_{part}`  

## Checklist

- [ ] `setcpm` + 2–5 本の `$:`（短いまま）  
- [ ] ドラムは原則 1 本の短い `s("bd …")`（キットは `.bank`）  
- [ ] ピッチは可能なら次数 + `.scale`  
- [ ] pad/lead/piano はフルネーム WAV またはシンセ  
- [ ] ライブ差分は get_song + edit_method / patch_track  
- [ ] 全文 save は初回・大規模変更のみ  
