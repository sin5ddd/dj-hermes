---
name: strudel-composition
description: "Use when writing a dj-hermes song: 7–8 $: tracks (drums, bass 1–2, three melody instruments, chords, pad), 4-bar phrases, dj_hermes_apply_song (save only to persist)."
version: 5.11.0
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
- **繰り返し周期の既定は 4 小節**（1 サイクル＝1 小節のまま。`.scale("<…>")` と 4 子以上の `<>` で周期を延ばす）。**Future Bass** と **Kawaii Future Bass** は 16 小節の `.scale`（16 子。`cat` ではない → **strudel-genre-future-bass** / **strudel-genre-kawaii-future-bass**）。**Ambient** は **16 本**・16 小節（4 小節細胞 × 基本 / ドラム / 基本 / 逆行 or クリシェ。転調しない → **strudel-genre-ambient**）。**Minimal** は 14–16 本・16 小節ミュート（キック / 裏拍 OHH / ベースは常時。他だけ `<>` 16 子。和声の `.scale` は 4 子 → **strudel-genre-minimal**）
- **16 小節 `cat` は既定にしない**（ライブ差分が重い）。プリセットの A/B は最大 8 引数
- ジャンルのグリッド・フックは **strudel-genre-*** の Pattern（acid の 303 は音名のクロマチックで `.scale` なし。他は次数 + `.scale` が多い）。**音色は同 Skill のパレット**からスロットごとに選ぶ（フェンスの `.s()` を毎回コピーしない）
- **音色・サンプル**: ドラムは短い `bd`/`sd`/… + 任意 `.bank` または `part:slug`。メロ／コード／パッドはカタログ PCM（`plk:` / `ep:` / `ld:` / `pf:` / `dr:` / `ps:` / **`vc:`**）か、ジャンルパレットが許した波形 / `wt_*` / ライブ `.fm`。波形をメロ／コード／パッドの既定にしない（サブ・303・Reese・wobble・zap はジャンルが芯と書いたスロットだけ。→ **strudel-sound-design** / **strudel-pcm-catalog**）。ボーカルチョップは全ジャンルで `vc:` を使ってよい（自前 WAV を invent しない）

| 場面 | 既定 |
| --- | --- |
| 新規曲・プリセット（`dj_hermes_apply_song`） | 7–8 本、4 小節フレーズ（Future Bass / Kawaii Future Bass は 9 本・16 小節 scale。Ambient は 16 本・16 小節フォーム。Minimal は 14–16 本・16 小節ミュート） |
| 来場者の一言編集 | **1 トラック or 1 メソッド**（全文を作り直さない） |
| 同梱 `songs/<genre>/` | 7–8 本、4 小節フレーズ（Future Bass / Kawaii Future Bass は 9 本・16 小節 scale。Ambient は 16 本・16 小節フォーム。Minimal は 14–16 本・16 小節ミュート） |

## スロット（`$:` 本数の正本）

コメント名は短く固定する（`dj_hermes_patch_track` / live-edit の対象名）。

| # | コメント | 役割 |
| --- | --- | --- |
| 1 | `// drums` | キット 1 本。カンマ並列。kick/hat/snare を 3 `$:` に分けない |
| 2 | `// bass` | フロア / サブ。PCM は `C4:…`、シンセは `C2:…` |
| 3 | `// bass-mid` | 2 本目ベース（Reese ミッド、wobble など）。不要なら **置かない** |
| 4 | `// lead` | 主メロ。PCM は **`.adsr` を `.s()` の直後**（wav を乾かさない） |
| 5 | `// hook` | ジャンルの決めフレーズ（プラック、303、スタブ、EP） |
| 6 | `// arp` | 対旋律 / アルペジオ。**メロディ楽器**（`plk:` / `ld:` / `ep:` + `note()`）。`perc:` はここではない |
| 7 | `// chords` | ブロック和音。`ep:*` で `[0,2,4]` 等。`c3'maj` は root のみなので使わない |
| 8 | `// pad` | ジャンルパレットの pad。**C2**（ドローンなら C1）。PCM は **`.adsr`**。`pf:ff` なら次数 `0`。orbit をリードと分ける |
| 9 | `// strings` | Future Bass と Kawaii Future Bass だけ。長いホールド（カタログに violin は無い）。orbit は pad と同じ |
| 10 | `// vox` | 任意。ボーカルチョップ（`vc:`）。perc と同時に足して本数上限を超えない |

数え方:

- ベース 1 本: drums + bass + lead + hook + arp + chords + pad = **7**。8 本目は `// perc`（毎小節撃たないワンショット）か **`// vox`**（同時に両方は置かない）
- ベース 2 本: drums + bass + bass-mid + lead + hook + arp + chords + pad = **8**。vox は **`// arp` を差し替え**（9 本にしない）
- **duck 例外**: キックだけ別 `$:`（`duckorbit`）。ハットは 2 本目。この 2 本でドラム枠。残り 6 = bass 1 + メロ 3 + chords + pad。2 本目ベースは足さない。vox は arp 差し替え（9 本にしない）
- **Future Bass / Kawaii Future Bass 例外**: duck 分割のうえ `// strings` を足して **9 本**。繰り返しは **16 小節**（`.scale` 16 子。`cat` ではない）。ドラム／ベース／リードは 1 小節ループにしない。`<>` の子は **同じウェイト**（8 分なら 8）。王道・ベルは kawaii、スーパーソーの壁は future-bass（混ぜない）。vox は **`// arp` の `.s("vc:…")`**（10 本にしない）
- **Minimal 例外**: **14–16 本**。キック / 裏拍 OHH / ベースは常時オン（16 子ミュートを書かない）。他は 16 小節ミュート。kick / ohh / chh は別 `$:`。perc / tom / metal は `note()` 可。`// vox` はミュート対象（texture 差し替え、または 15–16 本目）。スロットとマップは **strudel-genre-minimal**
- **Ambient 例外**: **16 本**。繰り返しは **16 小節**（4 小節細胞を 4 ブロック。`cat` ではない）。1–4 基本（キック無し）→ 5–8 ドラム → 9–12 基本 → 13–16 モチーフ逆行または下行バスのクリシェ。**転調しない**。床以外はアクセント。スロットとマップは **strudel-genre-ambient**
- **Electro 例外**: フックはスーパーソー（`ld:ss`）。pitched は `C2:`（arp は `C3:`）。PCM も他ジャンルの `C4:` native に上げない。**`vc:` だけは `C4:`**（録音が C4。C2 に落とすと声が床になる）→ **strudel-genre-electro**
- 目標 **7–8 本**。9 本以上は既定にしない（Future Bass / Kawaii は 9 本、Minimal は 14–16 本、Ambient は 16 本）

各ジャンルのグリッドは Pattern、音色は **音色パレット**（**strudel-genre-***）。

### Lead / pad / arp（乾いた PCM を避ける）

1. **Lead と pad の PCM は乾かさない。** `.s("ld:ss").adsr("0.01:0.3:0.7:0.2")` のように amp ADSR を **`.s()` の直後**（`.gain` の後ろではない）。長い `ld:` / `pf:` は `.adsr` + `.cut(1)` でグリッドに載せる。FX ライザー用の `<>` 間引きとは別。
2. **Pad は低域。** `.scale("C2:…")` が既定。長いドローンは C1。lead / chords の C4 と同じオクターブに置かない。工場 PCM の native は C4 書きだが、pad は意図して下げる。
3. **Arp はメロディ楽器。** `note()` + `plk:` / `ld:` / `ep:`。`s("<~ perc:tm ~>")` を arp にしない。perc は任意 8 本目 `// perc`。

## 音の長さ（役割別）

1 サイクル = 1 小節 = 4 拍。トップレベル **4 原子 = 4 分**、**8 原子 = 8 分**、**16 原子 = 16 分**。`@` を使ったら原子を減らして、その子のウェイト合計をグリッドに揃える（8 分なら 8。Future Bass / Kawaii の `<>` 子も同じ。合計 9 はドラムからズレる）。

| スロット | 既定グリッド | 長音 / 休符 |
| --- | --- | --- |
| pad / strings | 1 小節 1 音 `0`、または `0@3 ~` | **ホールド。** `0 ~ ~ ~` は 4 分 1 打＋無音であり、長いパッドではない |
| chords（ブロック） | `[0,2,4]@2 [0,2,4]@2` または `[0,2,4]`（小節まるごと） | **ホールド。** `[0,2,4] ~ [0,2,4] ~` はスタブ（音が切れる）。スタブがジャンル署名のときだけ残す（DnB / Dubstep の `[0,4]`、Kawaii の短いキラキラ） |
| lead | **8 分**（ウェイト 8）。伸ばす音は `@` | `4 ~ ~ ~` と **4 原子リードは禁止**。16 分埋めは arp と同時にしない |
| hook | **8 分。** ジャンル固定次数はそのまま | 空きは掛け合い用の `~`。伸ばしたい音は `@`。house フック `4 ~ 7 4  2 0 ~ -1` は書き換えない |
| arp | **8 分または 16 分。** 原子を埋める | 4 原子 arp は禁止。末 1 個の `~` は可。`vc:` の 16 分壁は禁止（チョップは疎のまま） |
| bass | ジャンルどおり。この正本は **8 分** | `0 ~ 0 ~` を床にしない |
| drums | 裏拍・2/4 は `~` | 休符のまま。`~@n` はフィル専用 |

対比（取り違えない）:

| 書き方 | エンジン | 耳 |
| --- | --- | --- |
| `a@3` | ウェイト 3、ゲート長も 3 | 同じ音が続く |
| `a ~ ~` | イベント 1、dur=1/3、Rest 2 | 音が切れる |
| `a _ _` / `a @ @` / `a!3` | 未対応（ゴミ atom またはパース失敗） | 使わない |
| `.sustain(0.5)` | Amp ADSR のレベル | 長さの主役はミニ記法の `@` |

掛け合い: メロ 3 本が同時に 16 分アタックを埋めない。arp が 16 分なら lead / hook は 8 分＋`@`（ホールド中はアタックしない）。

## フレーズ長

1 サイクル = 1 小節（エンジン）。**繰り返し周期**を 4 小節にする。

1. 和声: 既定 4 小節 `.scale("<Root:mode …>")`。次数パターンは固定。Future Bass / Kawaii Future Bass は 16 子。Ambient は 4 子細胞 × 4 ブロック（末 4 は逆行またはクリシェ。転調しない）。Minimal の和声は 4 子のまま（16 子は非リズムのミュート）
2. メロ / ベース: `<>` の子を **4 個以上**（1 小節同じフレーズを既定にしない）。Minimal の ostinato は 1 小節のままでよい（16 子はリズム帯以外のオン／オフ）。Future Bass / Kawaii のベースは **8 子以上**。Ambient の lead / hook / arp は 16 子（末 4 で逆行）
3. ドラム: 1 小節骨格は可。4 小節目だけフィル `..., <~ ~ ~ [fill]>`。Future Bass / Kawaii はキックを 4 子以上の `<>` にし、ハット連打は 8/16 小節目の末だけ（4 小節ごと `[hh*16]` は禁止）。Ambient のキックは 5–8 小節だけ
4. 長い PCM FX（ライザー約 15 秒）: `<fx:up ~ ~ ~ ~ ~ ~ ~>`（8 小節に 1 回。`fr` / `nr` / `rf` / `rp` / `rw` も同じ）
5. `cat()`: プリセットで A/B を分けるときだけ、**最大 8 引数**

禁止例（地味・短い）: `note("4 ~ ~ ~")` を毎小節、**4 原子の lead/hook/arp**、コードの `~` パディングをホールドと取り違える、全メロが毎 16 分で同時、メロ／コード／パッドが全部 `triangle`、コードとパッドが同じ次数・同じオクターブ。

## 正本テンプレ（そのまま content に）

ベース 1 本 + perc で 8 本。下の `.s()` は見本。新規 apply はジャンル Skill のパレットから差し替える。

```
// @title live-bed
// @genre techno
setcpm(128/4)

// drums
$: s("bd*4, [~ hh]*4, <~ ~ ~ [~@3 bd ~@4]>").gain(0.55)

// bass
$: note("0 0 2 0  0 2 <4 3 5 2> 0").scale("<C2:minor C2:minor G2:phrygian C2:minor>")
  .s("sawtooth").adsr("0.001:0.08:0.2:0.05").lpf(450).gain(0.45)

// lead
$: note("7@2 6 4  9@2 7 <4 3 7 9>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("ld:ss").adsr("0.01:0.3:0.7:0.2").cut(1).gain(0.16)

// hook
$: note("~ 4 7@2  2 0 4 <2 0 4 7>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("plk:lp").gain(0.18).cut(1)

// arp
$: note("0 4 7 12 7 4 0 4  12 7 4 0 7 4 0 ~").scale("<C5:minor C5:minor G5:phrygian C5:minor>")
  .s("plk:hd").cut(1).gain(0.12)

// chords
$: note("[0,2,4]@2 [0,2,4]@2").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("ep:ky").gain(0.26)

// pad
$: note("<0@3 ~>").scale("<C2:minor C2:minor G2:phrygian C2:minor>")
  .s("pf:ff").adsr("0.2:0.4:0.5:0.4").cut(1).gain(0.16).room(0.3).orbit(2)

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
- メロ 3 本は **同時に全 16 分アタックを埋めない**。arp が 16 分なら lead / hook は 8 分＋`@`。片方が休符またはホールドのとき他方が出る
- 役割ごとに PCM を変える（メロ／コード／パッドを全部 `triangle` にしない。全部 `pf:ff` / `ld:ss` にもしない）
- chords はジャンル表の和音（多くは `ep:*` の `[0,2,4]`、チルポップは `[0,2,6]`）。pad はパレットの pad 列。`pf:ff` なら次数 `0`（`[0,4]` で重ねない）
- コードは **3 音まで**。`pf:ff` を `[0,2,4]` で鳴らさない。6 音スタック禁止（デッキ `MAX_VOICES` は 32）
- サブ同士を重ねない（`bs:su` / `bs:hf` / `bs:dk` / `square`+低い lpf）
- レジスタ: シンセサブは C2 帯。PCM フロアは `C4:` で native。リードは C4。**パッドは C2**（ドローンなら C1。lead と同じ C4 に置かない）。arp はメロディ楽器で C5 付近（**Electro は pitched を C2:**、arp は C3:。PCM も C4: に戻さない。pad はもともと C2 → **strudel-genre-electro**）
- gain 目安: drums 0.50–0.70、bass 0.35–0.50、各メロ 0.12–0.22、chords 0.22–0.32、pad 0.14–0.26
- lead / pad の長い `ld:` / `pf:` は **`.adsr` + `.cut(1)`** で毎小節撃ってよい。`<>` 間引きは FX ライザー（`fx:up` 等）と、使わない長尺ワンショット。slug の意味は strudel-pcm-catalog の INDEX。ジャンル外の長尺はパレットの禁止列
- arp に `perc:` を置かない。perc は 8 本目 `// perc`
- **ボーカルチョップ**は全ジャンルでカタログ `vc:` を使ってよい（下の節。自前 WAV を invent しない）

## ライブ編集ワークフロー（必須）

1. 初回: 正本に近い **7–8 本** content を `dj_hermes_apply_song(content, deck)`（ディスクに書かない。Ambient は 16 本、Future Bass / Kawaii は 9 本、Minimal は 14–16 本）
2. 来場者の要望: **`dj_hermes_get_song(deck)`** → **1 トラック or 1 メソッド**だけ
   - パラメータ 1 個 → `dj_hermes_edit_method`
   - 1 本の `$:` 差し替え/追加 → `dj_hermes_patch_track`
   - 全文 `dj_hermes_apply_song` は大規模変更・新規のみ。残す指示のときだけ `dj_hermes_save_song`
3. バー境界で反映される（チャットにコードだけ書いて終わりにしない）
   - `dj_hermes_edit_method`: `op` は `set` / `add` / `remove`。`method` はドット無し（`lpf`）。`args` は括弧の中身（`400` や `sine.rangex(500,4000)`）
   - `dj_hermes_patch_track`: `op` は `replace` / `remove` / `append`

| 来場者の言い方 | 変更例 |
| --- | --- |
| ハット細かく | `[~ hh]*4` → `hh*8`。**ミニマルは OHH を触らず `// chh` をオン**（→ **strudel-genre-minimal**） |
| ベース動かして | 次数の末尾を `<>` で差し替え |
| 暗い / 明るい | **モード梯子**（→ **strudel-mood-bright-dark**）。副次で lpf |
| ブレイク / フィル | drums に `<>` / `.ply(2)` |
| メロディ足して | 空いている lead/hook/arp を埋める。既に 3 本あるときは 1 本を差し替え。**ミニマルは次数を増やさず synth / pluck をオン** |
| ボーカル / チョップ足して | **`// vox`**（`vc:`）。7 本床なら 8 本目（perc の代わり）。8 本上限・Future Bass / Kawaii は **arp 差し替え**。ミニマルは texture 差し替えか 15–16 本目。アンビエントは 16 本のうちの `// vox` |
| 転調 / 移調 | scale ルート or `.add`/`.sub` |
| コード変えて | `// chords` の `[0,2,4]` または進行の `<>` |
| パッド薄く | `// pad` の `.gain` / `.lpf` |
| 進行変えて | `.scale("<A2:minor D:dorian …>")` の中身を差し替え（pitched 全部で揃える） |

明暗の詳細は **strudel-mood-bright-dark**。

**アンチパターン**: 一言の要望で 8 本全部を `cat(...)` から生成し直す。

## ボーカルチョップ（全ジャンル）

カタログ PCM `vc:pa` / `vc:na` / `vc:ra` / `vc:tu` / `vc:ya` / `vc:yeah`（`samples/vc/`）。録音は **C4**（MIDI 60）、長さ約 **3.2 秒**。どのジャンルでも使ってよい。新規 apply で必ず置く必要はない。来場者が「ボーカル」「チョップ」「yeah」と言ったら置く。slug の意味は **strudel-pcm-catalog** INDEX。自前のボーカル WAV を invent しない。

| 書き方 | 用途 |
| --- | --- |
| `s("vc:pa vc:na vc:tu").gain(0.14).cut(1)` | リズム（録音ピッチのまま） |
| `note("0 4 ~ 7").scale("C4:minor").s("vc:pa").gain(0.14).cut(1)` | 移調。PCM は **`C4:`**（録音が C4 なので native は C4） |
| `s("<vc:yeah ~ ~ ~>").gain(0.16).cut(1)` | ドロップの掛け声。4 小節に 1 回 |
| `.begin(0).end(0.2)` | 短い音節。3.2 秒の頭だけ使う |

ルール:

1. 16 分連打は **必ず `.cut(1)`**（3.2 秒が次のヒットに重なる）
2. `vc:yeah` を毎小節撃たない
3. コメント名は **`// vox`**。本数上限は上の数え方（perc と同時に足して 9 本にしない。Future Bass / Kawaii / DnB / techno-duck は arp 差し替え。Ambient は 16 本のうちの `// vox`）
4. slug は 2 字。**`yeah` だけ 4 字**（音節どおり）
5. Electro の pitched `C2:` ルールは **`vc:` には適用しない**（C2 に落とすと声が床になる）
6. arp スロットに置くときも **`C4:`**（ジャンルの arp `C5:` に上げない。1 オクターブ上になる）
7. ジャンルの芯スロットは奪わない（acid の 303 hook、DnB Reese の square / `bs:rm` / `plk:s5`、house の `[~ cp]*2`）

## Mini-notation（`$:` の文字列内だけ）

公式の Mini-notation 全体は [Mini Notation](https://strudel.cc/learn/mini-notation/)（Strudel）。このエンジンが使うのは次のサブセット。公式の `_`（伸長）・裸の `@`・`!`（複製）・`-`（休符）・`?`・`|`・ユークリッド `bd(3,8)` は未対応。長音は `a@n`、休符は `~`。

| 記法 | 意味 |
| --- | --- |
| `bd sd hh` | **スペース** → 順再生（ウェイト既定 1） |
| `bd,sd` / `[bd,sd]` | **カンマ** → **同時再生** |
| `bd*4` | 1 サイクルに 4 回（密度アップ） |
| `a@2 b` | **`@` elongate** — 時間ウェイト（a が b の 2 倍）。ゲートも伸びる。`<>` の子どうしは **合計ウェイトを揃える**（8 分グリッドなら 8。`[4@2 7 9@2  7 4 2 0]` は 9 なのでズレる） |
| `a ~ ~` | **休符** — `a` のあと無音。音は切れる。`a@3` ではない |
| `~` | 休符（スロット無音）。ドラム裏拍・掛け合い用。長音の代用にしない |
| `a _ _` / `a @ @` / `a!3` | **使わない**（エンジン未対応。公式の `_` / `!` と同じ意味にはならない） |
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
5. **例外で分離** — `.duckorbit` 付きキックだけ別 `$:`。**Minimal** は kick / ohh / chh を別 `$:`（→ **strudel-genre-minimal**）。**Ambient** のキックは 1 本の `// drums` のまま 5–8 だけ鳴らす（ハット分割はしない）
6. 本家 `stack(...)` は使わない
7. **パート名は短く**（`bd` `sd` `hh` `oh` `cp`）。キット差は **`.bank("tr808-hard")` 等**（ディスクは `{bank}_{part}`）。フルネームでリズムを埋めない
8. **FM カタログは `bd:hf` のような `part:slug`**（2〜3 字。**`vc:yeah` だけ 4 字**。→ strudel-pcm-catalog）。`kit:bd` は不可。同梱は `s("bd")` / `.n(0)`

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
$: note("[0,2,4]@2 [0,2,4]@2")
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

Amp ADSR は **`.s()` の直後**。`.gain()` はレベルだけ。`.gain(0.3).attack(…)` とは書かない（エンジンは順番を気にしないが、ADSR を gain の修飾に見せない）。

```
$: note("0 2 3 4").scale("C2:minor").s("sawtooth").adsr("0.01:0.1:0.5:0.2").lpf(600).lpq(8).gain(0.5)
$: s("bd*4, hh*16").hpf(200).gain(0.45)
```

不可: `.vib("<1 4>")`、`stack(...)`、`.cpm(120)`、**`.lfo(...)`**（未実装）、mini の **`bd(3,8)`**（ユークリッド未実装。`(` は unexpected char）。
可: **`.scale("<…>")` 進行**、**`.lpf("<400 1200>")`** / **`.lpf(sine.rangex(500,4000))`** / **`.lpf(sine.range(200,2000).slow(4))`**、**`.add` / `.sub`**（スカラーまたは mini。LFO は不可）、**`.ply`**（整数スカラー）、**`.pan`**（詳細は sound-design）。

同梱サンプル: `bd` `sd` `hh` `oh` `cp`（`samples/cp/00.wav`。ハウス 2/4 は `[~ cp]*2`。テクノキック前は clap を載せない）。ボーカルチョップは `vc:pa` など（`samples/vc/`）。
追加キット・pad/lead の置き方: **`samples/LAYOUT.md`** / 音色は **strudel-sound-design**。カタログ PCM は **strudel-pcm-catalog**（INDEX の `in_bank=no` は書かない。現行キーはすべて `yes`）。

pad / lead / piano でユーザー WAV がある例:

```
$: note("<0@3 ~>").scale("C2:minor").s("pad-ambient_drone01")
  .adsr("0.2:0.4:0.5:0.4").room(0.4).orbit(2).gain(0.35)
$: note("7@2 6 4  9@2 7 <4 9 3 7>").scale("C4:minor").s("lead-supersaw_4oct")
  .adsr("0.01:0.3:0.7:0.2").lpf(2800).cut(1).gain(0.16)
$: note("0 2 4 0").scale("C3:minor").s("piano-acoustic_soft")
  .adsr("0.005:0.3:0.2:0.25").gain(0.35)
```

（ユーザー WAV が無ければカタログ `ep:rs` / `ld:*` / `pf:*` / `plk:*` / `dr:*` / `ps:*` / `vc:*`。メロ／コード／パッドを `triangle` に戻さない。ピアノ感はサンプル。ボーカルは `vc:`。）

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
- 理由なく kick/hat/snare を 3 トラックに分ける（Minimal の kick / ohh / chh 分割は例外）
- 未実装: `.lfo`
- テクノキック前グリッドにハウス `cp` を載せる（`[~ cp]*2` はハウス専用）
- 既定での 16 小節 `cat` 長尺
- mini 内の `kit:bd`（bank を左に書く形）。カタログは `bd:hf`（part:slug）
- 新規曲を 2–5 本の薄いループで出す（デバッグ専用の 2 本版を来場者に使わない）
- 長い PCM（`ld:` / `pf:` / `dr:` / `ps:`）を **ADSR なし**で毎小節撃つ。INDEX の `in_bank=no` を content に書く。lead / pad は `.adsr` + `.cut(1)` ならグリッド可
- lead / pad の PCM を `.adsr` なしで鳴らす（wav を乾かしたまま）
- pad を lead / chords と同じ C4 に置く
- arp を `perc:` にする（メロディ楽器を使う。perc は `// perc`）
- `vc:` を `.cut(1)` なしで 16 分連打する。自前のボーカル WAV を invent する。`vc:yeah` を毎小節撃つ
- メロ／コード／パッドを `triangle` / `sine` / `sawtooth` にする（ジャンル Skill が芯と書いたスロットだけ例外：サブ・303・Reese・wobble・zap）
- ジャンルパレットの禁止キー、INDEX に無いキー
- コード 4 音以上、`pf:ff` を `[0,2,4]` で重ねる
- 新規曲でフェンスの `.s()` を毎回同じ組合せにする
- `.gain(…).attack(…)` / `.gain(…).adsr(…)` — amp ADSR は `.s()` 側（→ **strudel-sound-design**）

## Pitfalls

1. チャットにコードだけ書いて終わりにしない → `dj_hermes_apply_song`（save は残す指示のときだけ。save は演奏を変えない）
2. `stack(...).cpm(170)` を content に入れる → 400
3. `.vib("<1 4>")` など未対応メソッドへ `"<...>"` を渡す → パース失敗（`.lpf("<…>")` と `sine.rangex` は可）
4. mini に `bd(3,8)` や `-` 休符、`a _ _` / `a @ @` / `a!3`（休符は `~` のみ。長音は `a@n`。`(` と `!` は unexpected char）
5. 毎回フル曲を `dj_hermes_save_song` で書き直して差分が巨大になる（save は演奏を変えない。鳴らすのは apply / patch / edit_method）
6. ドラムをフルファイル名で書く → 読めない。短い part + `.bank`
7. `{bank}-{part}.wav` とハイフン連結 → 正は `{bank}_{part}`
8. 1 小節 1 音（`4 ~ ~ ~`）や **4 原子の lead/hook/arp** を既定にする → lead/hook は 8 分＋`@`、arp は 8/16 分。コードの `~` パディングをホールドと取り違えない（ホールドは `@`）
9. 無い PCM キー → 無音。slug は INDEX で確認。FX 長尺は毎小節撃たない。lead / pad は ADSR なしで撃たない
10. 新規 apply でジャンルフェンスの `.s()` を全コピーする（パレットから選ぶ）
11. 同一曲の lead と hook が同じ `.s()`（Reese 分割以外）
12. `<>` の子で `@` 合計が違う（Future Bass / Kawaii のリフレインがドラムからズレる）
13. pad を C4 に置く / arp を `perc:` にする / lead・pad から `.adsr` を外す

## Checklist

- [ ] `setcpm` + **7–8 本**の `$:`（drums、bass 1–2、lead/hook/arp、chords、pad）。Future Bass / Kawaii Future Bass は 9 本（`// strings`）。Ambient は 16 本。Minimal は 14–16 本（kick / ohh / bass 常時）
- [ ] 4 小節フレーズ（`.scale("<…>")` 4 個 または `<>` 4 子）。1 小節同一繰り返しだけにしない。Future Bass / Kawaii は 16 子 scale。Ambient は 16 子（末 4 は逆行またはクリシェ。転調しない）。Minimal は非リズムの 16 子ミュート（和声は 4 子）
- [ ] ドラムは原則 1 本の短い `s("bd …")`（キットは `.bank` / `part:slug`）。duck キックのみ分離。Minimal は kick / ohh / chh を分割
- [ ] ピッチは可能なら次数 + `.scale`（acid の 303 は音名で `.scale` なし）。PCM フロアは `C4:`、シンセサブは `C2:`。**pad は C2**（ドローンなら C1）
- [ ] メロ 3 本は掛け合い。lead/hook は 8 分＋`@`、arp は 8/16 分（4 原子にしない）。arp はメロディ楽器（`perc:` ではない）。コードはホールド（`@2` または小節まるごと。スタブはジャンル署名のときだけ）。pad はジャンルパレット（`pf:ff` なら `note("0")`）+ **`.adsr`** + 別 orbit
- [ ] lead / pad の PCM は `.s(…).adsr(…)`（`.gain` の後ろに付けない）
- [ ] 音色はジャンルの **音色パレット**。同一曲で pitched の `.s()` を重複させない（Reese 分割以外）
- [ ] メロ／コード／パッドはカタログ `in_bank=yes` またはパレットが許した波形 / `wt_*` / `.fm`。ユーザー WAV があればフルネーム。ボーカルチョップは `vc:`（本数上限内の `// vox` または arp 差し替え）
- [ ] ライブ差分は get_song + edit_method / patch_track
- [ ] 全文 apply は初回・大規模変更のみ。save は残す指示のときだけ
