---
name: strudel-composition
description: "Use when writing a dj-hermes song: 7–8 $: tracks (drums, bass 1–2, three melody instruments, chords, pad), 4-bar phrases, dj_hermes_apply_song (save only to persist)."
version: 5.4.2
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [dj-hermes, music, composition, mini-notation, live-coding, drums]
    related_skills:
      - strudel-data-format
      - strudel-sound-design
      - strudel-pcm-catalog
      - strudel-mood-bright-dark
      - strudel-dj-mix
      - strudel-live-edit
---

# dj-hermes 作曲（Composition）— 8 トラック / 4 小節フレーズ

## Overview

dj-hermes の曲は **`$:` を重ねたループを、演奏しながら 1 本ずつ書き換える**。  
**新規 apply / プリセット**は薄い 2–5 本では足りない。DSP に余裕がある前提で、**7–8 本・4 小節フレーズ**を既定にする。

- 演奏形式: `setcpm` + **`$:` トラック**のみ（鳴らすのは `dj_hermes_apply_song`）
- mini-notation は文字列の中だけ（`s("...")` / `note("...")`）
- **新規の既定: 7–8 本**（ドラム＋ベース 1～2＋メロディ楽器 3＋コード＋パッド）
- **繰り返し周期の既定は 4 小節**（1 サイクル＝1 小節のまま。`.scale("<…>")` と 4 子以上の `<>` で周期を延ばす）。**Future Bass** は 16 小節の `.scale`（16 子。`cat` ではない → **strudel-genre-future-bass**）。**Minimal Techno** は 16 小節のミュートマップ（各トラックの `<>` 16 子。和声の `.scale` は 4 子のままでよい → **strudel-genre-minimal-techno**）
- **16 小節 `cat` は既定にしない**（ライブ差分が重い）。プリセットの A/B は最大 8 引数
- ジャンルのグリッド・フック次数は **strudel-genre-*** の Pattern。**音色は同 Skill のパレット**からスロットごとに選ぶ（フェンスの `.s()` を毎回コピーしない）
- **音色・サンプル**: ドラムは短い `bd`/`sd`/… + 任意 `.bank` または `part:slug`。メロ／コード／パッドはカタログ PCM（`plk:` / `ep:` / `ld:` / `pf:` / `dr:` / `ps:`）か、ジャンルパレットが許した波形 / `wt_*` / ライブ `.fm`。波形をメロ／コード／パッドの既定にしない（サブ・303・Reese・wobble・zap はジャンルが芯と書いたスロットだけ。→ **strudel-sound-design** / **strudel-pcm-catalog**）

| 場面 | 既定 |
| --- | --- |
| 新規曲・プリセット（`dj_hermes_apply_song`） | 7–8 本、4 小節フレーズ（Future Bass は 9 本・16 小節 scale。Minimal Techno は 8 本・16 小節ミュート） |
| 来場者の一言編集 | **1 トラック or 1 メソッド**（全文を作り直さない） |
| 同梱 `songs/<genre>/` | 7–8 本、4 小節フレーズ（Future Bass は 9 本・16 小節 scale。Minimal Techno は 8 本・16 小節ミュート） |

## スロット（`$:` 本数の正本）

コメント名は短く固定する（`dj_hermes_patch_track` / live-edit の対象名）。

| # | コメント | 役割 |
| --- | --- | --- |
| 1 | `// drums` | キット 1 本。カンマ並列。kick/hat/snare を 3 `$:` に分けない |
| 2 | `// bass` | フロア / サブ。PCM は `C4:…`、シンセは `C2:…` |
| 3 | `// bass-mid` | 2 本目ベース（Reese ミッド、wobble など）。不要なら **置かない** |
| 4 | `// lead` | 主メロ |
| 5 | `// hook` | ジャンルの決めフレーズ（プラック、303、スタブ、EP） |
| 6 | `// arp` | 対旋律 / アルペジオ / メロディック perc |
| 7 | `// chords` | ブロック和音。`ep:*` で `[0,2,4]` 等。`c3'maj` は root のみなので使わない |
| 8 | `// pad` | ジャンルパレットの pad。`pf:ff` なら次数 `0`（録音が 5 度）。orbit をリードと分ける |
| 9 | `// strings` | Future Bass だけ。長いクワイア／広いパッド（カタログに violin は無い）。orbit は pad と同じ |

数え方:

- ベース 1 本: drums + bass + lead + hook + arp + chords + pad = **7**。8 本目は `// perc`（毎小節撃たないワンショット）か対旋律。Minimal Techno の 8 本目は `// fx`
- ベース 2 本: drums + bass + bass-mid + lead + hook + arp + chords + pad = **8**
- **duck 例外**: キックだけ別 `$:`（`duckorbit`）。ハットは 2 本目。この 2 本でドラム枠。残り 6 = bass 1 + メロ 3 + chords + pad。2 本目ベースは足さない
- **Future Bass 例外**: duck 分割のうえ `// strings` を足して **9 本**。繰り返しは **16 小節**（`.scale` 16 子。`cat` ではない）
- **Minimal Techno 例外**: **8 本**（7 + `// fx`）。繰り返しは **16 小節ミュート**（キック常時、他は `<>` 16 子でオンオフ。和声は 4 小節 `.scale`。`cat` ではない）。ハット既定は裏拍オープン `[~ oh]*4`（下表の `hh*8` にしない）。マップとダーク FX は **strudel-genre-minimal-techno**
- **Electro 例外**: フックはスーパーソー（`ld:ss`）。pitched は `C2:`（arp は `C3:`）。PCM も他ジャンルの `C4:` native に上げない → **strudel-genre-electro**
- 目標 **7–8 本**。9 本以上は既定にしない（上の Future Bass 例外だけ 9 本）

各ジャンルのグリッドは Pattern、音色は **音色パレット**（**strudel-genre-***）。

## フレーズ長

1 サイクル = 1 小節（エンジン）。**繰り返し周期**を 4 小節にする。

1. 和声: 既定 4 小節 `.scale("<Root:mode …>")`。次数パターンは固定。Future Bass は 16 子。Minimal Techno の和声は 4 子のまま（16 子はミュート）
2. メロ / ベース: `<>` の子を **4 個以上**（1 小節同じフレーズを既定にしない）。Minimal Techno の ostinato は 1 小節のままでよい（16 子はオン／オフ）
3. ドラム: 1 小節骨格は可。4 小節目だけフィル `..., <~ ~ ~ [fill]>`
4. 長い PCM FX: `<fx:up ~ ~ ~>`（4 小節に 1 回）
5. `cat()`: プリセットで A/B を分けるときだけ、**最大 8 引数**

禁止例（地味・短い）: `note("4 ~ ~ ~")` を毎小節、全メロが毎 16 分で同時、メロ／コード／パッドが全部 `triangle`、コードとパッドが同じ次数・同じオクターブ。

## 正本テンプレ（そのまま content に）

ベース 1 本 + perc で 8 本。下の `.s()` は見本。新規 apply はジャンル Skill のパレットから差し替える。

```
// @title live-bed
// @genre techno
setcpm(128/4)

// drums
$: s("bd*4, [~ hh]*4, <~ ~ ~ [~@3 bd ~@4]>").gain(0.55)

// bass
$: note("0 0 2 <4 3 5 2>").scale("<C2:minor C2:minor G2:phrygian C2:minor>")
  .s("sawtooth").lpf(450).gain(0.45)
  .attack(0.001).decay(0.08).sustain(0.2).release(0.05)

// lead
$: note("~ 7 6 <4 9 3 7>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("ld:ss").gain(0.16).cut(1)

// hook
$: note("4 ~ 7 <4 2 0 4>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("plk:lp").gain(0.18).cut(1)

// arp
$: note("0 4 7 12  7 4 0 ~").scale("<C5:minor C5:minor G5:phrygian C5:minor>")
  .s("plk:hd").gain(0.12).cut(1)

// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("ep:ky").gain(0.26)

// pad
$: note("0").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("pf:ff").gain(0.16).room(0.3).orbit(2)

// perc — 4 小節に 1 回
$: s("<~ ~ ~ perc:cm>").gain(0.2)
```

2 トラック版は **デモやデバッグ専用**。来場者の新規曲やプリセットには使わない。

```
// @title demo-thin
setcpm(120/4)
$: s("bd*4, [~ sd]*2, [~ hh]*4").gain(0.55)
$: note("0 0 2 4").scale("C2:minor").s("sawtooth").lpf(450).gain(0.5)
```

## 掛け合い・ミックス（地味さ対策）

- 新規 apply は対象 **strudel-genre-*** の **音色パレット** からスロットごとに選ぶ。Pattern フェンスの `.s()` を全文コピーしない
- 同一曲の pitched 2 本に同じ `.s()` を使わない。例外は DnB Reese（square サブ + saw ミッド）だけ
- メロ 3 本は **同時に全 16 分を埋めない**。片方が休符のとき他方が出る
- 役割ごとに PCM を変える（メロ／コード／パッドを全部 `triangle` にしない。全部 `pf:ff` / `ld:ss` にもしない）
- chords はジャンル表の和音（多くは `ep:*` の `[0,2,4]`、チルポップは `[0,2,6]`）。pad はパレットの pad 列。`pf:ff` なら次数 `0`（`[0,4]` で重ねない）
- コードは **3 音まで**。`pf:ff` を `[0,2,4]` で鳴らさない。6 音スタック禁止（デッキ `MAX_VOICES` は 32）
- サブ同士を重ねない（`bs:su` / `bs:hf` / `bs:dk` / `square`+低い lpf）
- レジスタ: シンセサブは C2 帯。PCM フロアは `C4:` で native。リードは C4 以上。**Electro は pitched を C2:**（arp は C3:。PCM も C4: に戻さない → **strudel-genre-electro**）
- gain 目安: drums 0.50–0.70、bass 0.35–0.50、各メロ 0.12–0.22、chords 0.22–0.32、pad 0.14–0.26
- 長い `ld:` / `pf:` / `dr:` / `ps:`（約 8–17 秒）は毎小節撃たない。`s("<ld:ss ~ ~ ~>")` のように `<>` で間引く。slug の意味は strudel-pcm-catalog の INDEX。ジャンル外の長尺はパレットの禁止列

## ライブ編集ワークフロー（必須）

1. 初回: 正本に近い **7–8 本** content を `dj_hermes_apply_song(content, deck)`（ディスクに書かない）
2. 来場者の要望: **`dj_hermes_get_song(deck)`** → **1 トラック or 1 メソッド**だけ
   - パラメータ 1 個 → `dj_hermes_edit_method`
   - 1 本の `$:` 差し替え/追加 → `dj_hermes_patch_track`
   - 全文 `dj_hermes_apply_song` は大規模変更・新規のみ。残す指示のときだけ `dj_hermes_save_song`
3. バー境界で反映される（チャットにコードだけ書いて終わりにしない）
   - `dj_hermes_edit_method`: `op` は `set` / `add` / `remove`。`method` はドット無し（`lpf`）。`args` は括弧の中身（`400` や `sine.rangex(500,4000)`）
   - `dj_hermes_patch_track`: `op` は `replace` / `remove` / `append`

| 来場者の言い方 | 変更例 |
| --- | --- |
| ハット細かく | `[~ hh]*4` → `hh*8` |
| ベース動かして | 次数の末尾を `<>` で差し替え |
| 暗い / 明るい | **モード梯子**（→ **strudel-mood-bright-dark**）。副次で lpf |
| ブレイク / フィル | drums に `<>` / `.ply(2)` |
| メロディ足して | 空いている lead/hook/arp を埋める。既に 3 本あるときは 1 本を差し替え |
| 転調 / 移調 | scale ルート or `.add`/`.sub` |
| コード変えて | `// chords` の `[0,2,4]` または進行の `<>` |
| パッド薄く | `// pad` の `.gain` / `.lpf` |
| 進行変えて | `.scale("<A2:minor D:dorian …>")` の中身を差し替え（pitched 全部で揃える） |

明暗の詳細は **strudel-mood-bright-dark**。

**アンチパターン**: 一言の要望で 8 本全部を `cat(...)` から生成し直す。

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
// 4 小節目だけフィル（1 サイクル骨格のまま）
$: s("bd*4, [~ sd]*2, [~ hh]*4, <~ ~ ~ [~@3 bd ~@4]>").gain(0.55)
```

## ドラム統合ルール

1. **原則 1 トラック**: `// drums` + 1 本の `$:` `s(...)`
2. **定番骨格** → `bd*4, [~ sd]*2, [~ hh]*4`（ハウス 2/4 は `[~ cp]*2`。テクノは clap なし）
3. **不均等** → `@`
4. **小節っぽい差分** → パターン内の `<>`（まずこれ）。プリセットは 4 小節フィル
5. **例外で分離** — `.duckorbit` 付きキックだけ別 `$:`
6. 本家 `stack(...)` は使わない
7. **パート名は短く**（`bd` `sd` `hh` `oh` `cp`）。キット差は **`.bank("tr808-hard")` 等**（ディスクは `{bank}_{part}`）。フルネームでリズムを埋めない
8. **FM カタログは `bd:hf` のような `part:slug`**（2〜3 字。→ strudel-pcm-catalog）。`kit:bd` は不可。同梱は `s("bd")` / `.n(0)`

ユーザーキットがあるとき:

```
$: s("bd*4, [~ sd]*2, [~ hh]*4").bank("tr808-hard").gain(0.55)
```

bank を付けないと同梱 `samples/bd/` 等が使われる。

## ベース / メロディ

次数は **0 始まり・ルート相対**。マイナス可（`C2:major` の `-1` → B1、`C2:minor` の `-1` → Bb1）。

```
// C minor: 0=c 1=d 2=eb 3=f 4=g 5=ab 6=bb 7=c+oct
$: note("0 2 0 3 0 <2 4 5 3>").scale("C2:minor").s("sawtooth").lpf(600).gain(0.5)
// head の n(...) も可（メソッド .n(1) サンプル index とは別）
$: n("0 0 2 4").scale("C2:minor").s("sine").lpf(400).gain(0.55)
// 和音は並列次数; 移調は .add/.sub（スカラーまたは mini。LFO は不可）
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

同じ次数パターンのまま、**サイクルごとにルート/モードを切り替える**。プリセットの 4 小節はこれを既定にする。

```
// 4 サイクルで Am → Ddor → Gmix → Cmaj（次数 0 がルートを辿る）
$: note("0 2 4 0").scale("<A2:minor D:dorian G:mixolydian C:major>")
  .s("sawtooth").lpf(600).gain(0.5)
// 和音レイヤも同じ進行を共有
$: note("[0,2,4] ~ [0,2,4] ~")
  .scale("<A4:minor D4:dorian G4:mixolydian C4:major>")
  .s("ep:ky").gain(0.28)
```

ルール:

- `<>` の **内側は空白区切り**の `Root:mode`（`:` はスケール名の一部）
- **1 引数 = 1 サイクル（1 bar）**。4 個なら 4 小節で一周
- 次数パターンは固定のまま、スケール側で進行を出す（ライブで `<>` の中身を差し替えやすい）
- オクターブ省略は root 4（`D:dorian` → D4）。ベース用は `A2:minor` のように明示
- 固定キーは従来どおり `.scale("C2:minor")`。プリセットでは進行を優先

ルール（一般）:

1. 似た変化 → **`<>` で差分**（長尺 `cat` より先）
2. 同一キー → 次数 + `.scale("RootOct:mode")`
3. **進行** → 次数固定 + `.scale("<Root:mode …>")`（プリセット既定は 4 個）
4. シンセベースは **`C2:` / `A2:` のようにオクターブを付ける**。PCM ベースは **`C4:`**（native）

## チェーン

```
$: note("0 2 3 4").scale("C2:minor").s("sawtooth").lpf(600).lpq(8).gain(0.5)
$: s("bd*4, hh*16").hpf(200).gain(0.45)
```

不可: `.vib("<1 4>")`、`stack(...)`、`.cpm(120)`、**`.lfo(...)`**（未実装）、mini の **`bd(3,8)`**（ユークリッド未実装。`(` は unexpected char）。
可: **`.scale("<…>")` 進行**、**`.lpf("<400 1200>")`** / **`.lpf(sine.rangex(500,4000))`** / **`.lpf(sine.range(200,2000).slow(4))`**、**`.add` / `.sub`**（スカラーまたは mini。LFO は不可）、**`.ply`**（整数スカラー）、**`.pan`**（詳細は sound-design）。

同梱サンプル: `bd` `sd` `hh` `oh` `cp`（`samples/cp/00.wav`。ハウス 2/4 は `[~ cp]*2`。テクノキック前は clap を載せない）。
追加キット・pad/lead の置き方: **`samples/LAYOUT.md`** / 音色は **strudel-sound-design**。カタログ PCM は **strudel-pcm-catalog**（INDEX の `in_bank=no` は書かない。現行キーはすべて `yes`）。

pad / lead / piano でユーザー WAV がある例:

```
$: note("0 2 4 7").scale("C3:minor").s("pad-ambient_drone01").room(0.4).orbit(1).gain(0.35)
$: note("7 6 <4 9>").scale("C4:minor").s("lead-supersaw_4oct").lpf(2800).gain(0.16)
$: note("0 2 4 0").scale("C3:minor").s("piano-acoustic_soft").gain(0.35)
```

（ユーザー WAV が無ければカタログ `ep:rs` / `ld:*` / `pf:*` / `plk:*` / `dr:*` / `ps:*`。メロ／コード／パッドを `triangle` に戻さない。ピアノ感はサンプル。）

## テンポ

- `setcpm(30)` → BPM 120（30 cycles/min × 4 beats）
- `setcpm(120/4)` → 同じ
- 体感テンポ上げは mini の `*2`（バー全体に敷く）。メソッド `.fast(2)` はイベント開始を半分に潰すだけで次サイクルを取り直さない → フルバーが前半だけになる

## `cat()` について

- **曲ファイルでは可**: `s(cat("a", "b"))`（1 引数 = 1 小節）
- **既定では使わない**。4 小節は `.scale("<…>")` と `<>` で足りる
- 来場者が「A メロと B メロをはっきり分けたい」など明確なときだけ。**最大 8 引数**
- 本家 JS の `arrange` / `stack` は不可

## 禁止

- 本家 JS: `stack(...)`、`.cpm()`、裸の `s("...")` 行（`$:` 無し）
- 理由なく kick/hat/snare を 3 トラックに分ける
- 未実装: `.lfo`
- テクノキック前グリッドにハウス `cp` を載せる（`[~ cp]*2` はハウス専用）
- 既定での 16 小節 `cat` 長尺
- mini 内の `kit:bd`（bank を左に書く形）。カタログは `bd:hf`（part:slug）
- 新規曲を 2–5 本の薄いループで出す（デバッグ専用の 2 本版を来場者に使わない）
- 長い PCM（`ld:` / `pf:` / `dr:` / `ps:`）を毎小節撃つ。INDEX の `in_bank=no` を content に書く
- メロ／コード／パッドを `triangle` / `sine` / `sawtooth` にする（ジャンル Skill が芯と書いたスロットだけ例外：サブ・303・Reese・wobble・zap）
- ジャンルパレットの禁止キー、INDEX に無いキー
- コード 4 音以上、`pf:ff` を `[0,2,4]` で重ねる
- 新規曲でフェンスの `.s()` を毎回同じ組合せにする

## Pitfalls

1. チャットにコードだけ書いて終わりにしない → `dj_hermes_apply_song`（save は残す指示のときだけ。save は演奏を変えない）
2. `stack(...).cpm(170)` を content に入れる → 400
3. `.vib("<1 4>")` など未対応メソッドへ `"<...>"` を渡す → パース失敗（`.lpf("<…>")` と `sine.rangex` は可）
4. mini に `bd(3,8)` や `-` 休符（休符は `~` のみ。`(` は unexpected char）
5. 毎回フル曲を `dj_hermes_save_song` で書き直して差分が巨大になる（save は演奏を変えない。鳴らすのは apply / patch / edit_method）
6. ドラムをフルファイル名で書く → 読めない。短い part + `.bank`
7. `{bank}-{part}.wav` とハイフン連結 → 正は `{bank}_{part}`
8. 1 小節 1 音のフック（`4 ~ ~ ~`）を既定にする → 4 子の `<>` か 4 小節 scale
9. 無い PCM キー → 無音。slug は INDEX で確認。長い PCM は毎小節撃たない
10. 新規 apply でジャンルフェンスの `.s()` を全コピーする（パレットから選ぶ）
11. 同一曲の lead と hook が同じ `.s()`（Reese 分割以外）

## Checklist

- [ ] `setcpm` + **7–8 本**の `$:`（drums、bass 1–2、lead/hook/arp、chords、pad）。Future Bass は 9 本（`// strings`）。Minimal Techno は 8 本（`// fx`）
- [ ] 4 小節フレーズ（`.scale("<…>")` 4 個 または `<>` 4 子）。1 小節同一繰り返しだけにしない。Future Bass は 16 子 scale。Minimal Techno は 16 子ミュート（和声は 4 子）
- [ ] ドラムは原則 1 本の短い `s("bd …")`（キットは `.bank` / `part:slug`）。duck キックのみ分離
- [ ] ピッチは可能なら次数 + `.scale`。PCM は `C4:`、シンセサブは `C2:`
- [ ] メロ 3 本は掛け合い。コード 3 音まで。pad はジャンルパレット（`pf:ff` なら `note("0")`）で別 orbit
- [ ] 音色はジャンルの **音色パレット**。同一曲で pitched の `.s()` を重複させない（Reese 分割以外）
- [ ] メロ／コード／パッドはカタログ `in_bank=yes` またはパレットが許した波形 / `wt_*` / `.fm`。ユーザー WAV があればフルネーム
- [ ] ライブ差分は get_song + edit_method / patch_track
- [ ] 全文 apply は初回・大規模変更のみ。save は残す指示のときだけ
