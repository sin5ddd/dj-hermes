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

## 開発メモ

- 第一ターゲット: Linux + ALSA（`libasound2-dev`）
- ヘッドレス検証: `NullBackend` + `cargo test`
- パッケージ名: `strudel-rs`（このリポジトリルート）
