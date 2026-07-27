# strudel-rs

Strudel 記法の曲ファイルをリアルタイム演奏する Rust 製 CLI（デュアルデッキ・バークオンタイズ）。

実装は進行中です。概要・制約・モジュール構成は [AGENTS.md](./AGENTS.md) を参照してください。
詳細タスクは [docs/plans/2026-07-27_020000-strudel-rs-final.md](./docs/plans/2026-07-27_020000-strudel-rs-final.md) が正本です。

## 目標（完成時）

```bash
./strudel-rs                          # REPL + HTTP API(:7878) + 曲ファイル監視
# エディタで songs/*.strudel を保存 → 次の小節から反映
# Hermes: strudel-rs mcp を MCP サーバとして登録
```

## いま聴けるもの（最小 play）

リポジトリルートで:

```bash
cargo run -- play songs/smoke.strudel
cargo run -- play songs/smoke.strudel --seconds 15
# TUI なし（メタログのみ・スクリプト向け）
cargo run -- play songs/smoke.strudel --headless
# REPL + 曲ファイル監視（バー量子化ロード / xfade など）
cargo run -- play --repl songs/techno16.strudel
```

- **既定はループ再生**（終了: TUI なら `q` / Esc、`--headless` なら Ctrl+C）
- `--seconds N`: N 秒で自動停止（スクリプト向け）
- **既定はミニ記法ライブハイライト TUI**（曲ソース表示・再生中 atom を ANSI 強調）
- `--headless`: 旧来のメタログのみ（TTY 不要・CI / パイプ向け）
- **`--repl`**: **ハイライト + コマンド行**のライブ UI + `songs/` ウォッチャ（デモ向け）
  - 画面上段: **左 = デッキ A / 右 = デッキ B** のミニ記法ハイライト（同時表示）
  - 下段: ログ + `»` プロンプト
  - コマンド例（コロン不要）:
    - `a load songs/techno16.strudel` / `b load songs/house16.strudel`
    - `x 4`（反対側デッキへ 4 小節 xfade）/ `b x 4`（明示的に B へ）
    - `a mute kick` / `bpm 128` / `hush` / `status` / `quit`
- **`--repl-text`**: ハイライトなしの rustyline テキスト REPL（同じコマンド体系）
- サンプルは `./samples`（Sonic Pi 由来 CC0）。曲は `songs/*.strudel`
- 出力デバイスが無い環境ではエラー終了（`cargo test` / build はデバイス不要）

### 同梱デモ曲（16 小節ループ）

| ファイル | ジャンル | テンポ |
| --- | --- | --- |
| `songs/techno16.strudel` | ダークテクノ | 128 BPM |
| `songs/house16.strudel` | ハウス | 122 BPM |
| `songs/dnb16.strudel` | DnB | 174 BPM |
| `songs/acid16.strudel` | アシッド / ミニマル | 130 BPM |
| `songs/garage16.strudel` | UK ガレージ / 2-step | 132 BPM |
| `songs/smoke.strudel` | スモーク（短め） | 120 BPM |

```bash
cargo run -- play songs/techno16.strudel
cargo run -- play songs/house16.strudel --seconds 45
```

各曲は `<...>` で 16 サイクル分の展開を持ち、そのままループする。Strudel 記法（`setcpm` / `$:`）で書いているので REPL からのコピペ改造もしやすい。

## 開発メモ

- 第一ターゲット: Linux + ALSA（`libasound2-dev`）
- ヘッドレス検証: `NullBackend` + `cargo test`
- パッケージ名: `strudel-rs`（このリポジトリルート）
- ライブラリ: `strudel_rs`（`src/lib.rs`）+ バイナリ `strudel-rs`

### ローカル品質チェック（CI 相当）

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings   # Linux では libasound2-dev が必要
cargo test
cargo build --release
```

### Jujutsu (jj)

このリポジトリは **git と jj をコロケート**している（`.git` + `.jj`）。通常の `git` コマンドもそのまま使える。

```bash
# 初回クローン後（まだ .jj が無い場合）
jj git init --colocate
jj bookmark track master --remote=origin

jj status
jj log -r '::@' --limit 10
```

`.jj/` は gitignore 対象。`git clean -xdf` すると `.jj` も消える点に注意。

### CI / CD

| 経路 | 内容 |
| --- | --- |
| Push / PR → `master` | `.github/workflows/ci.yml` — fmt, clippy, test, release build（ubuntu + windows） |
| タグ `v*` | `.github/workflows/release.yml` — バイナリを GitHub Release に添付 |
| Dependabot | cargo / github-actions を週次 |

リリース例:

```bash
git tag v0.1.0
git push origin v0.1.0
```
