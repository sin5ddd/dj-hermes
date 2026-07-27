# AGENTS.md — strudel-rs

このリポジトリで作業する AI エージェント向けのプロジェクト概要と実装上の制約です。
詳細なタスク分解は `docs/plans/2026-07-27_020000-strudel-rs-final.md` を正本とします。

---

## 何を作るか

**strudel-rs** は、Strudel 記法で書かれた曲ファイルをリアルタイム演奏する Rust 製 CLI です。

| 利用者 | 操作手段 |
| --- | --- |
| 人間（編集） | エディタで `songs/*.strudel` を保存 → ファイル監視が検知 → **次の小節境界**で反映 |
| 人間（操作） | REPL（rustyline）でデッキ操作・即時コマンド |
| LLM / Hermes | HTTP API（axum, REST + SSE）および MCP server（`strudel-rs mcp`, stdio） |

主な体験:

- 曲A・曲B（2 デッキ）を **同一 Transport（共有グローバル時計）** で同期再生
- その上に **ミキサー層**（A/B 音量フェーダー、EQ・フィルター、クロスフェード/切替）
- コード変更・曲切替などは **バークオンタイズ**（次のバー境界が原則。バッファまたぎは安全性優先で 1 バッファ以内のずれを許容）
- Strudel 記法をできるだけそのまま流す。**和音記法（`c3'maj` 等）も初期対応**
- パース失敗時は現行の演奏を継続し、エラーだけ報告する

---

## 中核概念

### Song（曲）

`.strudel` ファイル 1 つ = 1 曲。ヘッダに `bpm`、本文は `名前: パターンコード`。

```
bpm: 126
---
kick:  s("bd*4").gain(0.9)
bass:  note("c2 c2 eb2 g2").s("sawtooth").lpf(400).gain(0.7)
hat:   s("hh*8").gain(0.3)
```

`#` コメント可。独自ミニパーサで読む（serde/toml は使わない）。

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
入力（editor watcher / REPL / HTTP / MCP）
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
- **演奏 CLI は手動起動**。MCP は **別プロセス**（stdio）で本体 HTTP（`127.0.0.1:7878`）へ中継。stdio 直結で audio を多重起動しない
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
│   ├── watcher.rs            # notify → Command
│   ├── repl.rs
│   ├── api.rs                # REST + SSE
│   └── mcp.rs                # rmcp, stdio → HTTP ブリッジ
├── songs/                    # デモ曲 (.strudel)
├── samples/                  # WAV (bd, sd, hh 等・CC0)
├── tests/                    # e2e 等（NullBackend 駆動）
├── docs/plans/               # 実装プラン（正本）
└── hermes_push.sh            # Hermes 向け HTTP ラッパ（後段タスク）
```

実装順はプランの Task 1〜22 に従う。現時点では cargo プロジェクト未初期化の可能性がある。

---

## 技術スタック

| 領域 | 選択 |
| --- | --- |
| 言語 | Rust stable, edition 2021 |
| 音声 I/O | cpal（Linux では ALSA。システム依存は ALSA のみ想定） |
| コマンド | crossbeam |
| ファイル監視 | notify |
| REPL | rustyline |
| HTTP | axum + tokio + serde_json 等 |
| MCP | rmcp（server + transport-io）、ブリッジ用に reqwest |
| パーサ | 自前（依存を増やさない） |

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
8. **Punchcard/Pianoroll 可視化**は任意 Task 25。端末 TUI 近似で実現可能。エディタ埋め込みは非対応。音声コア完了後。
9. 依存をむやみに増やさない。パーサジェネレータや重いシリアライズ層は避ける。ネット経由サンプルロードはデモ範囲外。

---

## コマンド操作面（完成時のイメージ）

### CLI / REPL 例

```
./strudel-rs                          # play: REPL + API(:7878) + watcher
:load A songs/techno1.strudel
:load B songs/ambient1.strudel
:xfade B 8
```

### HTTP

- 操作: `PUT`/`POST` 系（code, load, xfade, mute, hush, bpm 等）
- 購読: `GET /events`（SSE）、`GET /status`

### MCP ツール（予定）

`set_code` / `load_song` / `switch_deck` / `xfade` / `hush` / `get_state` など（`strudel_*` プレフィックス）。

Hermes 登録例:

```yaml
mcp_servers:
  strudel:
    command: /path/to/strudel-rs
    args: ["mcp"]
```

---

## テスト方針

| 種類 | 手段 |
| --- | --- |
| 単体 | 各モジュールの `#[cfg(test)]`（tokenize, transport, mini, song 等） |
| バークオンタイズ | バー頭以外で状態が変わらないことの自動テスト |
| E2E | `tests/e2e.rs` を NullBackend で駆動。クリッピング無し、DJ 切替シナリオ |

実機音出し確認（Task 1 のサイン波、最終デモ）は Linux + ALSA 環境で行う。
Windows 上では `cargo test`（NullBackend 経路）を優先する。

Rust の build/test/clippy 実行時は、利用可能なら `cargo-runner` スキル（出力フィルタ）を使う。

---

## 作業ルール（このリポジトリ）

1. **プラン正本:** `docs/plans/2026-07-27_020000-strudel-rs-final.md`。タスクを飛ばしたり、未承認のスコープ拡大をしない。
2. **コミット:** ユーザーが明示的に依頼するまで commit / stage しない（グローバル規則）。このリポジトリは **git と jj（Jujutsu）コロケート**。`.jj/` はローカルのみ（gitignore）。エージェントはユーザー指示がない限り `git` で操作してよい。jj を使う場合は `jj bookmark track master --remote=origin` 済み想定。
3. **品質:** 触ったモジュールのテストを通す。audio スレッド内でアロケーションやロック待ちを増やさないよう注意する。PR 前は CI 相当（`cargo fmt --check` / `clippy -D warnings` / `cargo test`）をローカルで通す。
4. **エラー:** パース失敗で演奏を止めない。API/REPL の両方で失敗理由を返す。
5. **ドキュメント:** コードコメントと README は標準の平易な文章。造語や曖昧な断定を避ける。
6. **セキュリティ:** ローカル bind（127.0.0.1）前提の API。公開 bind や認証は現スコープ外だが、パス traversal（曲ロード）や無制限入力には注意する。

---

## 現状と次の一手

| 項目 | 状態 |
| --- | --- |
| リポジトリ | GitHub private（`sin5ddd/strudel-rust`）+ CI/CD 基盤 |
| cargo プロジェクト | Task 1–8 完了（lib `strudel_rs` + bin） |
| 実装タスク | Task 9 以降（プラン Progress を更新しながら進める） |

次に実装する場合の入口:

1. Task 9: サンプル / sound フォールバック
2. Task 10–12: Song / Deck / Engine
3. 以降: Mixer → watcher → REPL → HTTP → MCP

詰まった点・設計判断はプラン末尾の「詰まりログ」「追加メモ」「Risks / Open Questions」に追記する。
