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
```

- 既定で約 30 秒再生（`--seconds 0` は 600 秒）
- サンプルは `./samples`（Sonic Pi 由来 CC0）。曲は `songs/*.strudel`
- 出力デバイスが無い環境ではエラー終了（`cargo test` / build はデバイス不要）

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
