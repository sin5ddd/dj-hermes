# Sample layout rules (strudel-rs)

このディレクトリの SampleBank の置き方と命名規約です。  
エンジンは起動時に **1 階層だけ**読みます（再帰しません）。キーは **小文字**です。

正本のライセンス方針は `LICENSE.md`。フォーマット変換の例は `README.md`。

---

## フォーマット

- **16-bit PCM mono WAV / 48 kHz**（同梱キットと同じ）
- ステレオは先頭チャンネルのみ使われる想定

```bash
ffmpeg -y -i source.wav -ac 1 -ar 48000 -sample_fmt s16 samples/path/to/out.wav
```

---

## ロード単位（エンジン）

| ディスク | sound キー | 変種 |
| --- | --- | --- |
| `samples/<name>/00.wav`, `01.wav`, … | `<name>` | ファイル名ソート順 = `.n(0)`, `.n(1)`, … |
| `samples/<name>.wav`（直下の単体） | `<name>` | 変種 1 本のみ |

- mini の sound atom に使える文字: 英数字と `.` `#` `-` `_` `'` `:`  
  **`part:slug`** はフォルダ内のファイル stem（`s("bd:8b")` → `samples/bd/8b.wav`）。  
  **`part:2`** は整数 index（`.n(2)` と同じ）。`kit:bd` のように bank を左に書く形は今も不可（キットは `.bank`）
- 波形名（`sine` `saw` `white` `wt_*` 等）と衝突する名前は使わない（波形側が優先）

---

## 二系統のキット方針

### A. ドラム — 短いパート名 + `.bank`（リズム可読性）

人間が読むパターンは短いパート名のままにする:

```
$: s("bd hh [bd,sd] hh").bank("tr808-hard")
$: s("bd*4, [~ sd]*2, [~ hh]*4").bank("accdrum-jazz")
```

| コード | 解決キー | ディスク例 |
| --- | --- | --- |
| `s("bd")` | `bd` | `samples/bd/00.wav`（同梱デフォルト） |
| `s("bd").n(1)` | `bd` の n=1 | `samples/bd/01.wav` |
| `s("bd").bank("tr808-hard")` | **`tr808-hard_bd`** | `samples/tr808-hard_bd.wav` または `samples/tr808-hard_bd/00.wav` |

- `.bank("X")` + `s("part")` → キーは **`x_part`**（bank も part も小文字化。連結は **アンダースコア 1 つ**）
- **bank 名にキット／硬度などの特性**を載せる（例: `tr808-hard`, `tr808-soft`, `accdrum-jazz`）
- **1 本のドラム `$:` につき bank は 1 つ**（そのチェーン全体に効く）
- キットを混ぜるときは `$:` を分ける

推奨 part（短い名前）:

| part | 役割 |
| --- | --- |
| `bd` | kick |
| `sd` | snare |
| `hh` | closed hat |
| `oh` | open hat |
| `cp` | clap（同梱 `samples/cp/00.wav`。ハウス 2/4 のドライクラップ。スネア代用ではない） |
| `rim` `tom` `perc` … | 必要なら同様に `{bank}_{part}` |

フォルダ内のファイル名例（変種の説明用。キーはフォルダ名）:

```
samples/tr808-hard_bd/
  bd-s01.wav    # short 01 → ソートで .n(0) になるよう命名
  bd-s02.wav
```

同キャラの微差だけを 1 フォルダに入れる。short と deep などキャラが違うなら **bank を分ける**か **別 part/別キー**にする。

### B. 音程楽器・FX・pad/lead/piano — フルネーム（bank なし）

リズム構文に載せない（またはワンショット 1 発）。  
sound 名 = **ファイル stem 全体**（意味を名前に載せる）。

```
samples/
  pad-ambient_drone01.wav
  pad-ambient_bright01.wav
  lead-supersaw_4oct.wav
  lead-supersaw_4oct-pluck.wav
  piano-acoustic_soft.wav      # 単音 one-shot（録音ピッチ ≈ C3 推奨）
  piano-acoustic_hard.wav
  piano-electric_rhodes.wav    # EP / ローズ系など character で区別
  atmo-noise.wav
  fx-riser_short01.wav
```

```
$: note("0 2 4 7").scale("C3:minor").s("pad-ambient_drone01")
$: note("7 6 4").scale("C4:minor").s("lead-supersaw_4oct")
$: note("0 2 4 0").scale("C3:minor").s("piano-acoustic_soft").gain(0.35)
$: note("<0 2 4 7>/2").scale("C4:major").s("piano-electric_rhodes")
  .attack(0.01).release(0.4).room(0.3).orbit(1).gain(0.28)
$: s("fx-riser_short01")
```

命名の目安（閉じた語彙を増やすときは Skill も更新）:

```
{family}-{character}_{detail}{##}.wav
```

例:

| family | character 例 | detail 例 |
| --- | --- | --- |
| `pad` | `ambient` | `drone01`, `bright01` |
| `lead` | `supersaw` | `4oct`, `4oct-pluck` |
| `piano` | `acoustic` / `electric` | `soft`, `hard`, `rhodes` |
| `reese` / `atmo` / `fx` | … | … |

`tr808` はドラム bank 側で使う（フル名 lead と混同しない）。

`note().s("sample")` のピッチ基準はおおよそ **C3**（speed スケール）。  
**ピアノ・長い pad / tone は C3 付近の単音**で録音すると移調しやすい（本格マルチサンプル鍵盤は非対応に近い）。

---

## 推奨ディレクトリ見取り図

```
samples/
  # --- git 同梱デモ（短いパート名・Git LFS）---
  bd/00.wav 01.wav
  sd/00.wav 01.wav
  hh/00.wav
  oh/00.wav
  cp/00.wav          # house 2/4 dry clap (FM)
  plk/lp.wav  # C3 short FM pluck
  plk/bl.wav   # C3 inharmonic bell / glass (ratio 3.5)
  ep/ky.wav     # C3 EP: harmonic 2×/3× tines + .fm(2).fmh(1) attack
  perc/fm.wav  # unpitched metallic hit (not a stab or kick)
  plk/s5.wav  # hollow C+G fifth (no third)
  plk/s3.wav  # C3 major triad C–E–G (bright counterpart)
  bs/rm.wav      # C3 mid Reese glue, 800–1200 Hz
  pf/ff.wav   # C3 fifth pad (sustained C+G, ~8.2 s)
  bs/su.wav    # C2 clean sine sub (native C2 / C4:… like house)
  bs/hf.wav  # C2 tight house floor bass (with the kick; not Eb)
  bs/dk.wav     # C3 dark full-range Reese (sub + mid)
  ld/ss.wav  # C3 classic supersaw lead (~8.2 s / 4 bars @ 120)
  fx/nr.wav # unpitched noise riser (~3.2 s)
  fx/up.wav    # unpitched uplifter (~2.8 s)
  fx/id.wav  # unpitched DnB impact (~0.5 s)
  fx/sd.wav    # unpitched sub drop (~1.1 s)
  LICENSE.md
  README.md
  LAYOUT.md          # 本ファイル

  # --- 追加キット（同じ 1 階層・Git LFS でコミット可）---
  tr808-hard_bd.wav
  tr808-hard_sd.wav
  tr808-hard_hh.wav
  tr808-hard_oh.wav
  tr808-soft_bd.wav
  …
  pad-ambient_drone01.wav
  pad-ambient_bright01.wav
  lead-supersaw_4oct.wav
  piano-acoustic_soft.wav
  piano-acoustic_hard.wav
  piano-electric_rhodes.wav
  fx-riser_short01.wav
```

- rust-fm-synthe カタログ: `samples/<part>/<slug>.wav`（例 `bd/8b.wav` → `s("bd:8b")`）。slug 表と説明は `docs/profile/dj-hermes/skills/creative/strudel-pcm-catalog/`。同梱 `00.wav` は上書きしない
- **同梱 CC0 キット**（`bd/` `cp/` と上記 FM ワンショット）はリポジトリに残す  
- **追加の `samples/<name>.wav` / `samples/<name>/00.wav`** は Git LFS（`.gitattributes` の `*.wav`）。gitignore されない  
- スクラッチ出力は `/out/` `/recordings/` のまま git 外  
- Dirt-Samples やライセンス不明キットを **vendor しない**

---

## 呼び出し早見

| 意図 | コード |
| --- | --- |
| デフォルトキック | `s("bd")` |
| デフォルト変種 | `s("bd").n(1)` |
| 808 hard キットでリズム | `s("bd*4, [~ sd]*2, hh*8").bank("tr808-hard")` |
| ambient pad | `note("…").scale("…").s("pad-ambient_drone01")` |
| piano / EP | `note("…").scale("…").s("piano-acoustic_soft")` |
| FX one-shot | `s("fx-riser_short01")` |
| 同梱 FM catalog | `s("bd:8b")` / `note("0").scale("C4:minor").s("bs:hf")` |

---

## やってはいけないこと

1. `samples/pad/ambient/drone.wav` のような **深い階層**（エンジンは読まない）  
2. 1 フォルダに `bd-*.wav` と `sd-*.wav` を混在（全部 **同じ sound の変種**になる）  
3. mini 内で `TR808:bd`（bank が左）。カタログは `bd:8b`（part が左、slug は 2〜3 字）  
4. `.bank("tr808-hard")` なのにファイルが `tr808-hard-bd.wav`（ハイフン連結）— 正は **`tr808-hard_bd`**  
5. 波形名の流用（`sine.wav` 等）

---

## 関連

- rust-fm-synthe `part:slug` カタログ: `docs/profile/dj-hermes/skills/creative/strudel-pcm-catalog/SKILL.md`
- 音色レシピ・役割分担: `docs/profile/dj-hermes/skills/creative/strudel-sound-design/SKILL.md`
- ドラム統合・7–8 本 / 4 小節フレーズ: `docs/profile/dj-hermes/skills/creative/strudel-composition/SKILL.md`
- 同梱マッピング: `LICENSE.md`
