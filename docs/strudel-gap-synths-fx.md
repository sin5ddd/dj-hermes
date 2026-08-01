# 本家 Strudel にあって strudel-rs 未実装のシンセ / FX

本ドキュメントは、[Strudel 公式](https://strudel.cc/learn/synths/) の **Synths / Effects**（および関連する Samples 音源まわり）のうち、**strudel-rs に無い、または一部だけあるもの**を列挙する。

- **正本の実装判定:** `src/code.rs` の `apply_method`、`src/sound.rs` の `WAVEFORM_NAMES` / `resolve_sound`、`src/synth.rs` の `Wave` / `NoiseKind` / `builtin_wavetable`
- **対応済みの使い方（Skill）:** `docs/profile/dj-hermes/skills/creative/strudel-sound-design/SKILL.md`（正本は `puredata-hermes/skills/creative/strudel-sound-design/`）
- **公式参照:**
  - Synths: https://strudel.cc/learn/synths/
  - Effects: https://strudel.cc/learn/effects/
  - Samples: https://strudel.cc/learn/samples/
- **更新方針:** 新メソッドを `apply_method` に足したら本表から外し、Skill に追記する。逆に本家だけにある機能を曲や Hermes プロンプトに書かない。

> デモ向けサブセットである。未実装は「バグ」ではなく **意図的スコープ外** が多い。

---

## 凡例

| 記号 | 意味 |
| --- | --- |
| **未** | メソッド / sound 名として認識されない（パースエラー or unknown sound） |
| **部分** | 名前はあるが本家より機能が狭い |
| **実装済** | スカラー引数の範囲で実用可（下表「実装済サマリ」） |

**横断ギャップ（ほぼ全パラメータ共通）**

| 本家 | strudel-rs |
| --- | --- |
| メソッド引数にミニ記法・Pattern（例: `.lpf("<200 800>")`） | **未** — 数値 / `"a:b:c"` コロン列のみ |
| 連続 LFO で毎サンプル変調（`tri.range` 等） | **未** — イベント発火時の固定値。`seg` も無し |
| 多段同一 FX（`lpf` を直列2段） | 本家同様 **上書き**（1 ボイス 1 設定） |

---

## 実装済サマリ（ギャップ表の対）

ここに無いものを「未実装候補」として扱う。

### 音源 sound 名

| カテゴリ | 名前 |
| --- | --- |
| 波形 | `sine`, `sawtooth`/`saw`, `square`, `triangle`/`tri` |
| ノイズ音源 | `white`, `pink`, `brown`/`brownian` |
| 内蔵 wavetable | `wt_sine`/`wt_demo0`, `wt_bright`/`wt_demo1`, `wt_organ`/`wt_demo2` |
| サンプル | `SampleBank` にあるもの（同梱例: `bd`, `sd`, `hh`, `oh`）+ `.bank` 接頭辞 |

### メソッド（概略）

`s`/`sound`, `note`/`n`, `gain`, `velocity`/`vel`, `pan`（スカラー）, ADSR 一式, `lpf`/`hpf`/`bpf` + Q, `vib`/`vibmod`, `fm`/`fmh`/`fmattack`/`fmdecay`/`fmsustain`, `noise`(mix), `penv`/`pattack`/`pdecay`, `lpenv`/`lpattack`…`lprelease`, `begin`/`end`/`speed`/`bank`/`clip`/`legato`/`cut`, `fast`/`slow`, `orbit`, `duckorbit`/`duckattack`/`duckdepth`, `delay`/`delaytime`/`delayfeedback`, `room`/`roomsize`, `compressor`

詳細は sound-design Skill を参照。

---

## 1. シンセ音源（Synths）— 未 / 部分

出典: https://strudel.cc/learn/synths/

### 1.1 完全未実装（sound / エンジン）

| 本家 | 概要 | 状態 |
| --- | --- | --- |
| `crackle` + `density` | クラックルノイズ | **未** |
| `user` + `partials` / `phases` | 加算合成・任意倍音 | **未** |
| `partials`（saw 等への適用） | 倍音マグニチュード制御 | **未** |
| `phases` | 倍音位相 | **未** |
| ZZFX 一式 `z_sawtooth` / `z_tan` / `z_noise` / `z_sine` / `z_square` 等 | サイズコーディング向けシンセ | **未**（unknown sound） |
| ZZFX 専用パラメータ | `zrand`, `curve`, `slide`, `deltaSlide`, `zmod`, `zcrush`, `zdelay`, `pitchJump`, `pitchJumpTime`, `lfo`, `tremolo`（ZZFX 文脈） | **未**（unsupported method） |
| 本家デフォルト大量 `wt_*`（AKWF 等） | 1000+ ウェーブテーブル | **未** — ネット `samples('bubo:waveforms')` も無し |
| `supersaw` / `pulse` 等の拡張波形名 | ドキュメント・コミュニティ例で頻出 | **未**（専用オシレータ無し。`square` で代用する程度） |

### 1.2 部分実装

| 本家 | strudel-rs | ギャップ |
| --- | --- | --- |
| Basic waveforms | **実装済** | アンチエイリアス無し（ナイーブ波形） |
| Noise white/pink/brown | **実装済** | 簡易 IIR。本家と音色は同一でない |
| `.noise`（オシレータへピンク混入） | **実装済** | 引数の Pattern 変調は不可 |
| Vibrato `vib` / `vibmod` | **実装済** | 同上・スカラーのみ |
| FM `fm` / `fmh` / `fmattack` / `fmdecay` / `fmsustain` | **部分** | **1 オペのみ**。本家の `fmh2`…`fmh8` 等 **複数オペ番号付き API は未** |
| `fmenv` (`lin`/`exp`) | **未** | FM エンベロープ形状切替なし |
| Wavetable `wt_*` | **部分** | 内蔵 3 種（+ demo 別名）のみ。`loopBegin`/`loopEnd` で走査 **未**。外部セット読み込み **未** |
| `note` デフォルト triangle | **実装済** | |

### 1.3 関連するが「シンセ音源」以外

| 本家 | 状態 |
| --- | --- |
| `_scope` / `_spectrum` 等ビジュアル | **未**（端末 viz は `dj` の punchcard 等・別系統） |
| `samples('github:...')` 遅延ロード | **未**（オフライン `samples/` のみ） |

---

## 2. エフェクト（Effects）— 未 / 部分

出典: https://strudel.cc/learn/effects/

### 2.1 ローカル FX（パーボイス）

| 本家 | 状態 | 備考 |
| --- | --- | --- |
| `gain` / `velocity` | **実装済** | |
| ADSR / `adsr` | **実装済** | |
| `lpf`/`lpq`/`hpf`/`hpq`/`bpf`/`bpq` | **実装済** | biquad。本家と Q レンジ・キャラは非一致の可能性 |
| `ftype` (12db / ladder / 24db) | **未** | 次数・タイプ固定 |
| `fanchor` | **未** | |
| `vowel` | **未** | フォルマント |
| LPF env: `lpa`…`lpenv` | **部分** | 実装あり。**HPF/BPF 用** `hpenv`/`bpenv` および `hpa`… 系は **未** |
| Pitch env: `penv`/`pattack`/`pdecay` | **部分** | `prelease`/`pcurve`/`panchor`/`psustain` は **未** |
| `detune` | **未** | |
| `stretch`（phase vocoder） | **未** | |
| `coarse` | **未** | 疑似リサンプル |
| `crush` | **未** | ビットクラッシュ |
| `shape` | **未** | ウェーブシェイプ |
| `distort` / `dist` | **未** | |
| `tremolo` / `tremolosync` / `tremolodepth` / `tremoloskew` / `tremolophase` / `tremoloshape` / `am` | **未** | 振幅変調 |
| `pan` | **実装済** | 0=左 … 0.5=中央 … 1=右。等パワー。**スカラー引数のみ**（`pan("0 1")` の Pattern 引数は未） |
| `jux` / `juxBy` | **未** | パターン変換寄り |
| `postgain` / `post` / `dry` | **未** | |
| `phaser` / `phaserdepth` / `phasercenter` / `phasersweep` | **未** | |
| `compressor` | **部分** | パターンから設定可だが **マスター寄り last-write**。本家のイベントごとチェーンとは位置が異なる |

### 2.2 グローバル / orbit FX

| 本家 | 状態 | 備考 |
| --- | --- | --- |
| `orbit` / `o` | **部分** | **デッキ内 1..4 のみ**。マルチチャンネル outs、`orbit("2,3,4")` 複数コピー **未** |
| `delay` / `delaytime` / `delayfeedback` | **部分** | 簡易ライン。`delayspeed` **未** |
| `room` / `roomsize` | **部分** | 簡易 reverb。本家フルパラメータ未満 |
| `roomfade` / `rfade` | **未** | |
| `roomlp` / `rlp` | **未** | |
| `roomdim` / `rdim` | **未** | |
| `iresponse` / `ir` | **未** | インパルス応答 |
| `duck` / `duckorbit` / `duckattack` / `duckdepth` | **実装済** | デッキ内 orbit。本家とアルゴリズム差はあり得る |

### 2.3 その他 Dynamics / クロスフェード

| 本家 | 状態 |
| --- | --- |
| パターン関数 `xfade(a, pos, b)` | **未**（strudel-rs の DJ Mixer xfade / クロスフェーダーは **別物**） |

---

## 3. サンプルまわり（Samples）— 音源・再生オプションのギャップ

出典: https://strudel.cc/learn/samples/

| 本家 | 状態 |
| --- | --- |
| デフォルト Dirt 大量キット | **未** — 同梱は最小ドラム等（Sonic Pi 由来 CC0） |
| URL / github / strudel.json ロード | **未** |
| Shabda 等外部 API | **未** |
| `loop` / `loopBegin` / `loopEnd` | **未** |
| `loopAt` / `fit` | **未** |
| `chop` / `slice` / `splice` / `striate` / `scrub` | **未** |
| multi-sample リージョン | **未** |
| `begin` / `end` / `speed` / `n` / `bank` / `cut` / `clip` / `legato` | **実装済**（負 speed 逆再生等の細部は本家と差があり得る） |

---

## 4. 優先度メモ（将来拡張の目安）

デモ品質で効きやすい順の **目安**（コミットメントではない）:

1. **高:** メソッド引数の Pattern 化（またはよく使う LFO の固定セット）— 本家コピペの失敗要因 1 位
2. **中:** `distort` / `crush`、HPF/BPF filter env、`fmenv`、FM 多オペの一部
3. **中:** room 拡張（`rlp`/`rdim`）、`delayspeed`
4. **低:** ZZFX、加算 `partials`、IR reverb、`phaser`、granular `chop`…
5. **非目標寄:** ネットサンプル、DAW マルチアウト orbit、Web 専用 viz 関数

---

## 5. メンテナンス手順

1. 公式 Synths / Effects ページで新セクションが増えていないか確認
2. `rg 'name =>' src/code.rs`（`apply_method`）と `WAVEFORM_NAMES` を突合
3. 本ファイルの「未」を減らし、Skill に「使える」を増やす
4. プラン正本 `docs/plans/2026-07-27_020000-strudel-rs-final.md` のティア表と矛盾があれば、**実装を正**として本ファイルを更新

---

## 改訂履歴

| 日付 | 内容 |
| --- | --- |
| 2026-07-30 | 初版。本家 docs + `code.rs`/`sound.rs`/`synth.rs` 突合 |
