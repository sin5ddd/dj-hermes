---
name: strudel-genre-ambient
description: >-
  Use when writing ambient for dj-hermes: 70 BPM, pad as primary,
  16 tracks, 16-bar form (plain 4 / drums 4 / plain 4 / retrograde
  or cliché 4), rest-heavy melody, low gain. Not a 4-bar loop, not
  a key change, not chill drums, not house clap, not 7 thin tracks.
version: 6.1.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [dj-hermes, music, genre, ambient]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-pcm-catalog
      - strudel-data-format
---

# dj-hermes × アンビエント

## Overview

キックは薄い、またはブロックのあいだ無し。パッドが主。メロは休符多め。gain は低め。
BPM 目安 60–90。フェンスは **70**（`setcpm(70/4)`）。
ハウスやチルより疎い。4 小節だけだと進行がすぐ一周するので、**このジャンルだけループは 16 小節**（4 小節の 4 倍。`cat` ではない）。転調はしない。

同梱 `songs/ambient/` は下の **16 本・16 小節**フェンス。新規 apply も同じ本数・フォーム。

## When

- 来場者がアンビエント、ドローン、空間、静かなパッド、ゆったりしたコード進行を求めるとき
- キック前のテクノ、ハウスの clap、チルの 2/4 スネアではないとき
- 4 小節ループや 7 本の薄い床では足りないとき

## フォーム（16 小節。転調しない）

和声の細胞は **4 小節**。それを 4 回並べて 16 子にする。親キーは変えない。

| ブロック | 小節 | 中身 |
| --- | --- | --- |
| **基本** | 1–4 | パッド／コード／メロ。キック無し |
| **ドラムあり** | 5–8 | 同じ 4 小節進行 + `bd:lf`（と perc アクセント） |
| **基本** | 9–12 | 1–4 に戻す（キック無し） |
| **逆行 or クリシェ** | 13–16 | どちらか一方。転調しない |

既定の 4 子（C 短調のダイアトニック i–VI–iv–v）:

```
C4:minor Ab4:lydian F4:dorian G4:phrygian
```

`Ab4:major` は Db が入って親キーを外れる。VI は `Ab4:lydian`。

**13–16 は次のどちらか**（両方は書かない。曲ごとに変える）:

| 名前 | 何をするか | 4 子 |
| --- | --- | --- |
| **逆行**（既定） | 4 子の進行を逆順。lead / hook / arp の次数も逆 | `G4:phrygian F4:dorian Ab4:lydian C4:minor` |
| **クリシェ** | 下行バス i–bVII–VI–v。次数パターンは 1–12 のまま | `C4:minor Bb4:mixolydian Ab4:lydian G4:phrygian` |

16 子（既定＝逆行。pitched 全部で同じ並び。オクターブだけトラックで変える）:

```
<C4:minor Ab4:lydian F4:dorian G4:phrygian C4:minor Ab4:lydian F4:dorian G4:phrygian C4:minor Ab4:lydian F4:dorian G4:phrygian G4:phrygian F4:dorian Ab4:lydian C4:minor>
```

pad は **C2**、arp は **C5**、choir は **C3**、`vc:` は **C4**。PCM フロアは C4。

禁止: 4 小節 `.scale` のまま出す。後半でルートを Eb や C major に移す（転調）。16 引数の `cat`。

## スロット（16 本）

コメント名は固定（`dj_hermes_patch_track`）。床 6 本はほぼ常時。残りは **アクセント**（16 子で出し入れ）。7 本にまとめない。

| # | コメント | いつ | 役割 |
| --- | --- | --- | --- |
| 1 | `// pad` | 常時 | **主**。C2、`.adsr`、`pf:ff` なら次数 `0` |
| 2 | `// bass` | 常時 | 疎なサブ 1 本 |
| 3 | `// chords` | 常時 | `[0,2,4]` 小節ホールド |
| 4 | `// lead` | 常時（休符多） | 主メロ。PCM は `.adsr`。13–16 で逆行 |
| 5 | `// hook` | 常時（長い音） | 13–16 で逆行 |
| 6 | `// arp` | 常時（疎） | **メロディ楽器** + `note()`。`perc:` ではない |
| 7 | `// drums` | **5–8 だけ** | `bd:lf` 1 打。`bd*4` にしない |
| 8 | `// perc` | 5–8 の隙間 | キックと同時にしないワンショット |
| 9 | `// drone` | 1 と 9 | 長い `dr:`。基本ブロックの頭だけ |
| 10 | `// air` | 4 / 8 / 12 / 16 | 短いエア／ノイズ |
| 11 | `// fx` | 8 と 16 | 境界のウーシュ／スイープ |
| 12 | `// bells` | 13–16 | 疎なベル。lead と同じ `.s()` にしない |
| 13 | `// vox` | 9–12 | `vc:` 疎、`.cut(1)`、`C4:` |
| 14 | `// choir` | 13–16 | クワイア。pad と別 slug |
| 15 | `// counter` | 13–16 | 対旋律。逆行のとき lead の元モチーフをここに残す |
| 16 | `// swell` | 8 と 16 | 2 本目パッド。C2、`.adsr` |

14 本にするときは `// bells` と `// swell` を落とす。13 本以下にしない。

`<>` の 1 小節ループは **ブラケットで 1 子**。`bd:lf ~ ~ ~` を `<>` に裸で書くと 4 小節になる。

## Pattern

```
// @title visitor-ambient
// @genre ambient
setcpm(70/4)
// pad
$: note("0").scale("<C2:minor Ab2:lydian F2:dorian G2:phrygian C2:minor Ab2:lydian F2:dorian G2:phrygian C2:minor Ab2:lydian F2:dorian G2:phrygian G2:phrygian F2:dorian Ab2:lydian C2:minor>")
  .s("pf:ff").adsr("0.4:0.8:0.7:0.8").cut(1).gain(0.22).room(0.5).orbit(2)
// bass
$: note("0 ~ ~ ~").scale("<C4:minor Ab4:lydian F4:dorian G4:phrygian C4:minor Ab4:lydian F4:dorian G4:phrygian C4:minor Ab4:lydian F4:dorian G4:phrygian G4:phrygian F4:dorian Ab4:lydian C4:minor>")
  .s("bs:su").gain(0.26)
// chords
$: note("[0,2,4]").scale("<C4:minor Ab4:lydian F4:dorian G4:phrygian C4:minor Ab4:lydian F4:dorian G4:phrygian C4:minor Ab4:lydian F4:dorian G4:phrygian G4:phrygian F4:dorian Ab4:lydian C4:minor>")
  .s("ep:mt").gain(0.16)
// lead
$: note("<[~ 7 ~ ~] [~ ~ 4 ~] [~ 9 ~ 7] [4 ~ ~ ~] [~ 7 ~ ~] [~ ~ 4 ~] [~ 9 ~ 7] [4 ~ ~ ~] [~ 7 ~ ~] [~ ~ 4 ~] [~ 9 ~ 7] [4 ~ ~ ~] [~ ~ ~ 4] [7 ~ 9 ~] [~ 4 ~ ~] [~ ~ 7 ~]>")
  .scale("<C4:minor Ab4:lydian F4:dorian G4:phrygian C4:minor Ab4:lydian F4:dorian G4:phrygian C4:minor Ab4:lydian F4:dorian G4:phrygian G4:phrygian F4:dorian Ab4:lydian C4:minor>")
  .s("ld:et").adsr("0.08:0.5:0.6:0.5").cut(1).gain(0.12)
// hook
$: note("<[0@2 ~ 4@2 ~] [0@2 ~ 4@2 ~] [0@2 ~ 4@2 ~] [0@2 ~ 4@2 ~] [0@2 ~ 4@2 ~] [0@2 ~ 4@2 ~] [0@2 ~ 4@2 ~] [0@2 ~ 4@2 ~] [0@2 ~ 4@2 ~] [0@2 ~ 4@2 ~] [0@2 ~ 4@2 ~] [0@2 ~ 4@2 ~] [4@2 ~ 0@2 ~] [4@2 ~ 0@2 ~] [4@2 ~ 0@2 ~] [4@2 ~ 0@2 ~]>")
  .scale("<C4:minor Ab4:lydian F4:dorian G4:phrygian C4:minor Ab4:lydian F4:dorian G4:phrygian C4:minor Ab4:lydian F4:dorian G4:phrygian G4:phrygian F4:dorian Ab4:lydian C4:minor>")
  .s("plk:am").gain(0.14).cut(1)
// arp
$: note("<[~ 4 ~ 7] [~ 4 ~ 7] [~ 4 ~ 7] [~ 4 ~ 7] [~ 4 ~ 7] [~ 4 ~ 7] [~ 4 ~ 7] [~ 4 ~ 7] [~ 4 ~ 7] [~ 4 ~ 7] [~ 4 ~ 7] [~ 4 ~ 7] [7 ~ 4 ~] [7 ~ 4 ~] [7 ~ 4 ~] [7 ~ 4 ~]>")
  .scale("<C5:minor Ab5:lydian F5:dorian G5:phrygian C5:minor Ab5:lydian F5:dorian G5:phrygian C5:minor Ab5:lydian F5:dorian G5:phrygian G5:phrygian F5:dorian Ab5:lydian C5:minor>")
  .s("plk:lp").cut(1).gain(0.1)
// drums — bars 5–8 only
$: s("<~ ~ ~ ~ [bd:lf ~ ~ ~] [bd:lf ~ ~ ~] [bd:lf ~ ~ ~] [bd:lf ~ ~ ~] ~ ~ ~ ~ ~ ~ ~ ~>").gain(0.18)
// perc — gaps in the drum block
$: s("<~ ~ ~ ~ ~ perc:cm ~ perc:tg ~ ~ ~ ~ ~ ~ ~ ~>").gain(0.1)
// drone — heads of the plain blocks
$: note("<0 ~ ~ ~ ~ ~ ~ ~ 0 ~ ~ ~ ~ ~ ~ ~>").scale("<C2:minor Ab2:lydian F2:dorian G2:phrygian C2:minor Ab2:lydian F2:dorian G2:phrygian C2:minor Ab2:lydian F2:dorian G2:phrygian G2:phrygian F2:dorian Ab2:lydian C2:minor>")
  .s("dr:ad").adsr("0.5:1.0:0.6:1.0").cut(1).gain(0.1).room(0.45).orbit(2)
// air — section edges
$: s("<~ ~ ~ fx:ha ~ ~ ~ fx:nh ~ ~ ~ fx:ha ~ ~ ~ fx:wd>").gain(0.1).cut(1)
// fx — end of drum block and end of loop
$: s("<~ ~ ~ ~ ~ ~ ~ fx:wh ~ ~ ~ ~ ~ ~ ~ fx:sw>").gain(0.14).cut(1)
// bells — last 4
$: note("<~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ [~ 11 ~ ~] ~ [~ 7 ~ ~] ~>").scale("<C5:minor Ab5:lydian F5:dorian G5:phrygian C5:minor Ab5:lydian F5:dorian G5:phrygian C5:minor Ab5:lydian F5:dorian G5:phrygian G5:phrygian F5:dorian Ab5:lydian C5:minor>")
  .s("plk:bl").gain(0.08).cut(1)
// vox — second plain block
$: s("<~ ~ ~ ~ ~ ~ ~ ~ ~ vc:na ~ ~ ~ ~ ~ ~>").gain(0.1).cut(1)
// choir — last 4
$: note("<~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ [0@3 ~] [0@3 ~] [0@3 ~] [0@3 ~]>").scale("<C3:minor Ab3:lydian F3:dorian G3:phrygian C3:minor Ab3:lydian F3:dorian G3:phrygian C3:minor Ab3:lydian F3:dorian G3:phrygian G3:phrygian F3:dorian Ab3:lydian C3:minor>")
  .s("ld:cr").adsr("0.3:0.6:0.7:0.6").cut(1).gain(0.1).room(0.4).orbit(2)
// counter — original lead motif under the retrograde
$: note("<~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ [~ 7 ~ ~] [~ ~ 4 ~] [~ 9 ~ 7] [4 ~ ~ ~]>").scale("<C4:minor Ab4:lydian F4:dorian G4:phrygian C4:minor Ab4:lydian F4:dorian G4:phrygian C4:minor Ab4:lydian F4:dorian G4:phrygian G4:phrygian F4:dorian Ab4:lydian C4:minor>")
  .s("ld:ny").adsr("0.08:0.5:0.6:0.5").cut(1).gain(0.1)
// swell — bars 8 and 16
$: note("<~ ~ ~ ~ ~ ~ ~ 0 ~ ~ ~ ~ ~ ~ ~ 0>").scale("<C2:minor Ab2:lydian F2:dorian G2:phrygian C2:minor Ab2:lydian F2:dorian G2:phrygian C2:minor Ab2:lydian F2:dorian G2:phrygian G2:phrygian F2:dorian Ab2:lydian C2:minor>")
  .s("pf:cl").adsr("0.4:0.8:0.5:0.8").cut(1).gain(0.12).room(0.5).orbit(2)
```

`pf:ff` は録音済みの 5 度（C+G）。`[0,2,4]` でも `[0,4]` でも鳴らさない。パッドは `note("0")` の移調だけ。コードは `ep:mt`。

70 BPM の 1 小節は約 3.4 秒。`dr:ad` は約 17 秒なので 1 と 9 だけ。`fx:up`（約 15 秒）は置かない。

## 音色パレット（新規 apply はここから選ぶ）

Pattern はグリッド・次数・ミュートの見本。スロットごとに 1 つ選ぶ。同一曲の pitched 2 本に同じ `.s()` を使わない。slug は strudel-pcm-catalog の INDEX。lead / pad / drone / choir / swell の長い PCM は `.adsr` + `.cut(1)`。

| スロット | 芯 | 代替 | 禁止 |
| --- | --- | --- | --- |
| pad | **主**。C2 + `.adsr` | `pf:ff`+次数 `0`、`pf:cl`、`pf:wa`、`ps:sh` | スーパーソー、C4、ADSR なし、`bd*4` の上に載せるだけ |
| bass | 疎なサブ 1 本 | `bs:su` | `bs:hf` 重ね、wobble |
| chords | `[0,2,4]` ホールド | `ep:mt`、薄い `ld:fp` | `triangle`、毎拍スタブ、`pf:ff` を `[0,2,4]` |
| lead | 休符多め + `.adsr` | `ld:et`、`ld:fl`、`ld:si` | `ld:ss` アンセム、`ld:zp`、ADSR なし |
| hook | 長いノート | `plk:am`、`ld:cr`（choir と被らない） | gabber、`plk:ss` |
| arp | メロディ楽器 | `plk:lp`、`ld:si`、`plk:hp` | `perc:`、16 分埋め |
| drums | 5–8 だけ `bd:lf` | `bd:lf`、無し（その曲だけ） | `bd*4`、1–16 常時、house clap |
| perc | 5–8 の隙間 | `perc:cm`、`perc:tg`、`perc:cb` | arp に置く、16 分埋め |
| drone | 1 と 9 | `dr:ad`、`dr:uw`、`dr:fg` | 毎小節、`dr:sl` ソー壁 |
| air | 境界 | `fx:ha`、`fx:nh`、`fx:wd` | ライザー毎 4 小節 |
| fx | 8 と 16 | `fx:wh`、`fx:sw` | `fx:up`、`fx:gb`、毎小節 |
| bells | 13–16 疎 | `plk:bl`、`plk:ch` | lead と同じ slug、16 分 |
| vox | 9–12 疎、`C4:`、`.cut(1)` | `vc:na`、`vc:ra` | 16 分 EDM、毎小節 `vc:yeah`、自前 WAV |
| choir | 13–16 | `ld:cr`、`pf:ca` | pad と同じ slug、16 小節常時 |
| counter | 13–16 | `ld:ny`、`plk:ps` | lead と同じ `.s()` |
| swell | 8 と 16。C2 + `.adsr` | `pf:cl`、`ps:sh` | pad と同じ slug、毎小節 |

## Why

4 小節の i–i–v–i だと、70 BPM でも十数秒で一周して進行を楽しめない。細胞は 4 小節のまま、**16 子で 4 ブロック**にする。ドラムは 2 ブロック目だけ。最後は転調ではなく、同じキーの **逆行**か **下行バスのクリシェ**。

16 本は同時に壁を作るためではない。床（pad / bass / chords / メロ 3）のほかは、ブロック境界と 13–16 のアクセント。Minimal の「キック常時 + ミュート展開」とは逆で、キックは短い間だけ。

lead / pad の長い PCM は乾かさない（`.s()` の直後に `.adsr`）。pad は C2。arp は `note()` + プラック／リードで、perc は `// perc`。

PCM フロアは `C4:`。`bs:su` と `bs:hf` は重ねない。

## レシピ

1. **16 本**。7 本のまま出さない
2. ループは **16 小節**（4 子 × 4 ブロック）。`cat` ではない
3. 1–4 基本（キック無し）→ 5–8 ドラム → 9–12 基本 → 13–16 逆行 **または** クリシェ
4. 転調しない（ルートを関係調へ移さない）
5. pad が主。C2 + `.adsr`。コードは 3 音まで
6. arp はメロディ楽器。perc は `// perc`
7. メロ（lead / hook / arp）は休符多め、gain 0.08–0.16
8. 長い FX / drone は連続小節に置かない

鳴らすのは `dj_hermes_apply_song(content, deck)`。`dj_hermes_save_song` は残す指示のときだけ。

## Variations（同じ文法）

| 目的 | 変更 |
| --- | --- |
| クリシェ | 全 pitched の `.scale` の末 4 子を `C4:minor Bb4:mixolydian Ab4:lydian G4:phrygian`（オクターブはトラックどおり）。lead 次数は 1–12 のまま |
| キック無し | `// drums` と `// perc` を 16 子すべて `~`（本数は 16 のまま） |
| 14 本 | `// bells` と `// swell` を置かない |
| パッドを雲に | pad を `pf:cl`（swell は `ps:sh`） |
| チョップを前に | vox を 5–8（drums と同時。gain さらに下げる） |

## Pitfalls

1. 連打キック / `bd*4` でアンビエントが崩れる
2. `stack` / `.cpm` / `.lfo`
3. gain 過大
4. `note("c3'maj")` は root 単音。和音は `[0,2,4]`
5. `pf:ff` を `[0,2,4]` で鳴らす
6. `bs:su` の上に `bs:hf` を重ねる
7. 新規 apply を 7 本・4 小節のまま出す
8. 70 BPM を 126 テクノと DJ ペアにする（Transport は 1 つ）
9. フェンスの `.s()` を全コピーする。パッド以外をスーパーソーや zap にする
10. 転調する（後半を Eb や平行長調にする）
11. arp を `perc:` にする。pad を C4 にする。lead / pad から `.adsr` を外す
12. `<>` の 1 小節をブラケット無しにする
13. `fx:up` を毎小節撃つ。`dr:ad` を **ADSR なし**で毎小節撃つ（ドローンは `.adsr` + `.cut(1)` ならグリッド可。フェンスの間引きは残す）
14. 16 引数の `cat`

## Checklist

- [ ] `setcpm(70/4)` + **16 本**（pad / bass / chords / lead / hook / arp が床。drums は 5–8。残りはアクセント）
- [ ] **16 小節** `.scale`（4 子 × 3 + 逆行またはクリシェ）。転調していない。`cat` ではない
- [ ] 1–4 と 9–12 はキック無し。5–8 だけ `bd:lf`
- [ ] pad は C2 + `.adsr`。lead の PCM も `.adsr`。arp はメロディ楽器
- [ ] `.s()` は音色パレット。同一曲の pitched を重複させない
- [ ] `dj_hermes_apply_song(content, deck)`（save は残す指示のときだけ）
