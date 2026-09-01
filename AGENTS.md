# AGENTS.md — strudel-rs

このリポジトリで作業する AI エージェント向けのプロジェクト概要と実装上の制約です。
詳細なタスク分解は `docs/plans/2026-07-27_020000-strudel-rs-final.md` を正本とします。
**完了タスクの詳細手順**は `docs/plans/done/` に切り出し済み（索引: `docs/plans/done/README.md`）。

---

## 何を作るか

**strudel-rs** は、Strudel 記法で書かれた曲ファイルをリアルタイム演奏する Rust 製 CLI です。

| 利用者       | 操作手段                                                                          |
| ------------ | --------------------------------------------------------------------------------- |
| 人間（編集） | 演奏はオンメモリ。ディスク反映は明示 save。エディタ変更を鳴らすには `/a load` または `/a reload` |
| 人間（操作） | live TUI: 自然文→Hermes、`/` 付きでローカルコマンド。`--text` は rustyline 裸コマンド |
| LLM / Hermes | TUI から `hermes -z`（profile `dj-hermes`）+ MCP。HTTP API の `POST /mcp`（stdio `strudel-rs mcp` は非推奨デバッグ用） |

主な体験:

- 曲A・曲B（2 デッキ）を **同一 Transport（共有グローバル時計）** で同期再生
- その上に **ミキサー層**（A/B 音量フェーダー、EQ・フィルター、クロスフェード/切替）
- コード変更・曲切替などは **バークオンタイズ**（次のバー境界が原則。バッファまたぎは安全性優先で 1 バッファ以内のずれを許容）
- Strudel 記法をできるだけそのまま流す。**和音記法（`c3'maj` 等）も初期対応**
- パース失敗時は現行の演奏を継続し、エラーだけ報告する

---

## 中核概念

### Song（曲）

`.strudel` ファイル 1 つ = 1 曲。**Strudel REPL からコピペしやすい記法**を優先する。

```
// @title smoke
// @by strudel-rs
setcpm(30)
// drums (space = sequence, comma = simultaneous)
$: s("[bd hh [bd,sd] hh]*2").gain(0.75)
// bass
$: note("c2 c2 eb2 g2").s("sawtooth").lpf(400).gain(0.7)
```

- `setcpm(N)` / `setcpm(120/4)`: cycles per minute（Strudel と同じ）。1 cycle = 1 bar（4 beats）なのでエンジン BPM は `N * 4`
- `setcps(x)` も可（BPM = `x * 240`）
- ミニ記法: スペース=順再生、カンマ=同時再生、`@n`=時間ウェイト（elongate）。ドラム例: `s("bd*4, [~ sd]*2, [~ hh]*4")`
- ベース/メロ: 次数 + `.scale("C2:minor")`（0 始まり・ルート相対、**負次数可**。`C2:major` の `-1` → B1）。音名 + ネスト `<>` での小節差分圧縮も可
- `$:` の直前コメント（`// drums`）をトラック名にする。コメント無しは `$0`, `$1`, …
- メタデータは [Strudel 流のコメントタグ](https://strudel.cc/learn/metadata/): `// @title …` / `// @by …` / `// @license …` など（`/* … */` ブロックや 1 行複数タグも可）
- レガシー互換: `// title: …` / `bpm:` / `title:` / `name: code` / `---` も引き続きパース可能
- 独自ミニパーサで読む（serde/toml は使わない）

### Deck（デッキ）= 曲A / 曲B

Song をロードして鳴らす再生ユニット ×2。両デッキは同一 Transport を参照する。
曲ファイルの `bpm` は「マスター未設定時の初期値」。マスター BPM に統一する。

### Mixer（ミキサー層）

曲A/B の上に乗る第3層。担当:

1. A/B それぞれの音量フェーダー
2. EQ・フィルター
3. A/B クロスフェードおよび曲切替

デッキは「何を鳴らすか」、ミキサーは「どう混ぜて出すか」。

### Bar-Quantized Transition

状態遷移は audio スレッド内で `global_sample >= target_bar_sample` になった瞬間に pending → current へ差し替える。
デモ用途のため、バッファまたぎはサンプル精度より**クリック等の事故防止**を優先（バッファ内スプリットはしない）。
クリック防止に約 5ms のゲインランプ。`hush` 以外の遷移は原則バークオンタイズ。

### XFade

ミキサー層で `gainA = cos(θ)`, `gainB = sin(θ)`（θ: 0 → π/2）。開始もバー境界。

---

## 想定アーキテクチャ

```
入力（TUI / REPL / HTTP / MCP）
        │  すべて同じ Command チャネルへ
        ▼
   Scheduler (audio thread)
   pending をバー境界でのみ適用
        │
        ▼
 Deck A  ◄── Transport ──►  Deck B
        │
        ▼
      Mixer（フェーダー / EQ・フィルター / xfade）
        │
        ▼
   AudioBackend (cpal / NullBackend) → 出力
```

- コマンド経路: `crossbeam` のチャネル（SPSC 想定）
- **演奏 CLI は手動起動**。Hermes 向け MCP は演奏プロセスの **`POST /mcp`（Streamable HTTP）**。stdio `strudel-rs mcp` は非推奨。audio の多重起動をツール経路から起こさない
- 単一バイナリ + サブコマンド（`play` / `mcp` / `list` 等）。別 crate の workspace 分割はしない

---

## リポジトリ構成（目標）

```
strudel-rust/                 # このリポジトリのルート
├── Cargo.toml                # package name: strudel-rs
├── src/
│   ├── main.rs               # サブコマンド分岐
│   ├── backend.rs            # AudioBackend + NullBackend
│   ├── transport.rs          # 共有時計 (BPM, bar)
│   ├── mini.rs               # ミニ記法 tokenizer / AST / eval
│   ├── code.rs               # メソッドチェーン + note_to_hz + 和音展開
│   ├── sound.rs              # 波形リスト → サンプル の sound 解決
│   ├── song.rs               # 曲ファイルパーサ
│   ├── synth.rs              # Voice（波形/ノイズ + ADSR + パーボイス FX）
│   ├── sample.rs             # SampleBank + サンプル再生
│   ├── deck.rs               # 曲A/B: トラック集合 + ボイスプール
│   ├── mixer.rs              # フェーダー / EQ・フィルター / xfade・切替
│   ├── engine.rs             # Scheduler / Command
│   ├── mixer.rs              # フェーダー / EQ・フィルター / xfade（Task 14）
│   ├── highlight.rs          # ミニ記法ライブハイライト（Task 26）
│   ├── live_ui.rs            # highlight TUI（Task 16 / 26）
│   ├── repl.rs               # rustyline REPL（Task 16）
│   ├── api.rs                # REST + SSE
│   └── mcp.rs                # rmcp, stdio → HTTP ブリッジ
├── songs/                    # デモ曲 (.strudel)
├── samples/                  # WAV (bd, sd, hh 等・Sonic Pi 由来 CC0。カスタム追加可)
├── tests/                    # e2e 等（NullBackend 駆動）
├── docs/plans/               # 実装プラン（正本）
└── hermes_push.sh            # Hermes 向け HTTP ラッパ（後段タスク）
```

実装順はプランの Task 1〜22 に従う。現時点では cargo プロジェクト未初期化の可能性がある。

---

## 技術スタック

| 領域         | 選択                                                  |
| ------------ | ----------------------------------------------------- |
| 言語         | Rust stable, edition 2021                             |
| 音声 I/O     | cpal（Linux では ALSA。システム依存は ALSA のみ想定） |
| コマンド     | crossbeam                                             |
| ファイル監視 | notify                                                |
| REPL         | rustyline                                             |
| HTTP         | axum + tokio + serde_json 等                          |
| MCP          | rmcp（server + transport-io）、ブリッジ用に reqwest   |
| パーサ       | 自前（依存を増やさない）                              |

Release プロファイル目安（プラン）: `opt-level = 3`, `lto = true`, `strip = true`。
目標リソース: release バイナリ < 8MB、RSS < 60MB、演奏中 CPU < 8%（1 コア）程度。

開発ホストが Windows でもよいが、**第一ターゲットは Linux + ALSA**。ヘッドレス検証は `NullBackend` で行う。

---

## 実装上の制約・非目標

エージェントは次を勝手に広げないこと。詳細な決定経緯はプラン末尾「Risks / Open Questions」「追加メモ」。

1. **和音記法は初期対応**（`c3'maj` 等を展開）。対応コード種は Task 7 で表を固定し、未対応はエラー。
2. **sound 解決:** 固定波形リスト → なければ SampleBank（フォールバック）。`note().s("bd")` 可。詳細はプラン「Strudel 音源・エフェクト対応方針」。
3. **1 bar = 4 beats 固定。**
4. **MCP は HTTP ブリッジのみ**（演奏は手動起動）。stdio 直結で audio を持たない。本体未起動時のツールエラーは仕様。
5. **3 層:** 曲A / 曲B / Mixer。フェーダー・EQ・切替は Mixer に寄せる。マスター BPM 共有。曲ごとの独立テンポはしない。
6. **バッファまたぎ**はスプリットせず安全性優先（最大 1 バッファずれ許容）。
7. **cpal は I/O のみ。** Synths/Effects/Samples 相当は自前 DSP。Strudel 全機能は目標にせず **ティアA** を Task 1–22 の完了条件とする（B/C は任意 Task 23–24）。
8. **Punchcard/Pianoroll 可視化**は任意 Task 25（完了）。`dj` live UI で `F10` / `/viz` により body を highlight ⇔ punchcard 切替。エディタ埋め込みは非対応。
9. **ミニ記法ライブハイライト**は任意 Task 26。`play` の**既定表示**（曲ソース + 再生中 atom の ANSI 強調）。旧メタログのみは `--headless`。audio スレッドでは span 計算しない（UI 再評価）。
10. 依存をむやみに増やさない。パーサジェネレータや重いシリアライズ層は避ける。ネット経由サンプルロードはデモ範囲外。

---

## コマンド操作面（完成時のイメージ）

### CLI / REPL 例

```
./strudel-rs dj songs/house-01.strudel songs/four-on-the-floor-01.strudel   # live UI + API(:17878)
# または空起動: ./strudel-rs dj
# プロンプト: a load … / b load … / x 4
```

### HTTP

- 操作: `PUT`/`POST` 系（code, load, xfade, mute, hush, bpm 等）
- 購読: `GET /events`（SSE）、`GET /status`
- MCP: `POST /mcp`（Streamable HTTP、Hermes 用）

### MCP ツール

Mixer: `mixer_eq` / `mixer_filter` / `mixer_crossfader` / `xfade` / `mix` / `set_bpm`  
Deck: `load_song` / `apply_song` / `list_songs` / `save_song` / `mute` / `head`  
Transport: `hush` / `status`  
（いずれも `strudel_*` プレフィックス。コード直書きの `set_code` は MCP から削除済み。鳴らす全文は `apply_song`、曲ファイルは `load_song`、残すのは `save_song`）

Hermes 登録例:

```yaml
mcp_servers:
    strudel:
        url: "http://127.0.0.1:17878/mcp"
```

---

## テスト方針

| 種類             | 手段                                                                    |
| ---------------- | ----------------------------------------------------------------------- |
| 単体             | 各モジュールの `#[cfg(test)]`（tokenize, transport, mini, song 等）     |
| バークオンタイズ | バー頭以外で状態が変わらないことの自動テスト                            |
| E2E              | `tests/e2e.rs` を NullBackend で駆動。クリッピング無し、DJ 切替シナリオ |

実機音出し確認（Task 1 のサイン波、最終デモ）は Linux + ALSA 環境で行う。
Windows 上では `cargo test`（NullBackend 経路）を優先する。

Rust の build/test/clippy 実行時は、利用可能なら `cargo-runner` スキル（出力フィルタ）を使う。

---

## 作業ルール（このリポジトリ）

1. **プラン正本:** `docs/plans/2026-07-27_020000-strudel-rs-final.md`。タスクを飛ばしたり、未承認のスコープ拡大をしない。
2. **コミット:** ユーザーが明示的に依頼するまで **git の** commit / stage / push をしない（グローバル規則）。このリポジトリは **git のみ**（Git LFS で `samples/**/*.wav`）。jj をコロケートしない。公式 jj は Git LFS の smudge をしないため、作業コピーが pointer と実 WAV で食い違う。
3. **品質:** 触ったモジュールのテストを通す。audio スレッド内でアロケーションやロック待ちを増やさないよう注意する。PR 前は CI 相当をローカルで通す（下記 **fmt 必須**）。
4. **エラー:** パース失敗で演奏を止めない。API/REPL の両方で失敗理由を返す。
5. **ドキュメント:** コードコメントと README は標準の平易な文章。造語や曖昧な断定を避ける。
6. **セキュリティ:** ローカル bind（127.0.0.1）前提の API。公開 bind や認証は現スコープ外だが、パス traversal（曲ロード）や無制限入力には注意する。

### rustfmt（CI で繰り返し落ちやすい）

CI の `fmt` ジョブは `cargo fmt --all -- --check` のみで、**自動整形しない**。未整形のまま push すると毎回失敗する。

**エージェント / 作業者は次を守る:**

1. **Rust を編集したコミット・PR の直前に必ず** `cargo fmt --all` を実行する（確認だけでなく整形まで）。
2. 続けて `cargo fmt --all -- --check` が exit 0 であることを確認してから push する。
3. 実装完了の自己チェックや PR 作成フローでは、`cargo test` / `clippy` の**前または同列**で fmt を行う（後回しにしない）。
4. Windows でも同じ。行末や差分が小さく見えても rustfmt は差分を出すことがある。

```bash
# PR / push 前の最小セット（この順を推奨）
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

`cargo-runner` フィルタは build/test/clippy 用。**fmt には使わない**（素の `cargo fmt` をそのまま実行）。

---

## 現状と次の一手

| 項目               | 状態                                                   |
| ------------------ | ------------------------------------------------------ |
| リポジトリ         | GitHub private（`sin5ddd/strudel-rust`）+ CI/CD 基盤   |
| cargo プロジェクト | Task 1–19 + Task 21 + Task 23–26 完了 |
| 実装タスク         | Task 20 任意 → Task 22 計測（Task 25 viz 完了）      |

次に実装する場合の入口:

1. Task 22: リソース計測 + README 展示手順
2. Task 20（任意）: 汎用 ctl スクリプト + MCP クライアント設定例（Hermes 専用ランタイムは作らない）

詰まった点・設計判断はプラン末尾の「詰まりログ」「追加メモ」「Risks / Open Questions」に追記する。
