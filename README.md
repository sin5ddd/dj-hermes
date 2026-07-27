# strudel-rs

Strudel 記法の曲ファイルをリアルタイム演奏する Rust 製 CLI（デュアルデッキ・バークオンタイズ）。

実装は進行中です。概要・制約・モジュール構成は [AGENTS.md](./AGENTS.md) を参照してください。
詳細タスクは [docs/plans/2026-07-27_020000-strudel-rs-final.md](./docs/plans/2026-07-27_020000-strudel-rs-final.md) が正本です。

## インストール

Rust toolchain（stable）が必要です。

```bash
# GitHub から（private の場合は git 認証済みであること）
cargo install --git https://github.com/sin5ddd/strudel-rust --locked

# クローン済みリポジトリから
git clone https://github.com/sin5ddd/strudel-rust.git
cd strudel-rust
cargo install --path . --locked
```

インストール先は通常 `~/.cargo/bin/strudel-rs`（Windows は `%USERPROFILE%\.cargo\bin\strudel-rs.exe`）。
`PATH` に `cargo` の bin が入っていれば、どのディレクトリからでも `strudel-rs` を呼べます。

曲ファイル（`songs/`）とサンプル WAV（`samples/`）はリポジトリ同梱です。インストール後もデモを鳴らすときは、リポジトリをクローンしてそのルートで `play` するか、曲パスを明示してください。

```bash
# 動作確認
strudel-rs help
cd /path/to/strudel-rust   # songs/ と samples/ がある場所
strudel-rs play songs/smoke.strudel --headless
```

Linux では ALSA 開発ヘッダが必要なことがあります（例: `libasound2-dev`）。

## いま聴けるもの（最小 play）

リポジトリルートで（未 install なら `cargo run --` を先頭に付ける）:

```bash
strudel-rs play songs/smoke.strudel
strudel-rs play songs/smoke.strudel --seconds 15
# TUI なし（メタログのみ・スクリプト向け）
strudel-rs play songs/smoke.strudel --headless
# デュアルデッキ live UI（曲は省略可）
strudel-rs dj songs/techno1.strudel songs/ambient1.strudel
strudel-rs dj
# 開発時
cargo run -- dj songs/techno1.strudel songs/ambient1.strudel
```

- **既定はループ再生**（終了: TUI なら `q` / Esc、`--headless` なら Ctrl+C）
- `--seconds N`: N 秒で自動停止（スクリプト向け）
- **既定はミニ記法ライブハイライト TUI**（曲ソース表示・再生中 atom を ANSI 強調）
- `--headless`: 旧来のメタログのみ（TTY 不要・CI / パイプ向け）
- **`dj [SONG_A] [SONG_B]`**: **ハイライト + コマンド行**のライブ UI + `songs/` ウォッチャ（デモ / DJ 向け）
  - 画面上段: **左 = デッキ A / 右 = デッキ B** のミニ記法ハイライト（同時表示）
  - 中段: **A/B の Hi・Mid・Lo EQ**（各 3 行・短スライダー。現状は見た目＋ドラッグのみ／チャンネル EQ DSP は未接続）
  - その下: **クロスフェーダー**（最大 10 文字幅 `XF A ──□── B`。□ は白背景。クリック／ドラッグ）
  - 下段: ログ + `»` プロンプト
  - A のみ / B のみ / 両方省略も可（空デッキから `a load` / `b load`）
  - コマンド例（コロン不要）:
    - `a load songs/techno1.strudel` / `b load songs/ambient1.strudel`
    - `b head 33`（次の小節境界で B を曲の 33 小節目から再生。別名 `cue`。1 始まり）
    - `x 4`（反対側デッキへ 4 小節 xfade）/ `b x 4`（明示的に B へ）
    - `a mute kick` / `bpm 128` / `hush` / `status` / `quit`
  - `--text`: ハイライトなしの rustyline テキスト REPL
  - 互換: `play --repl` / `play --repl-text` も同じセッションを起動（A/B 2 曲可）
- サンプルは `./samples`（Sonic Pi 由来 CC0）。曲は `songs/*.strudel`
- 出力デバイスが無い環境ではエラー終了（`cargo test` / build はデバイス不要）

## パターン記法の拡張（Task 23）

ローカル FX・シンセ（パーボイス）と Deck 内 orbit をサポートしています。

| 系統 | メソッド例 |
| --- | --- |
| 波形 / ノイズ | `sine` `sawtooth` `square` `triangle` `white` `pink` `brown` |
| ウェーブテーブル | `wt_sine` `wt_bright` `wt_organ`（手続き生成の 1 周期） |
| 変調 | `.vib("4:12")` `.fm(4).fmh(1.5)` `.noise(0.2)` `.penv(12)` |
| フィルタ | `.lpf(800)` `.lpq(2)` `.hpf(200)` `.bpf(1000)` `.lpenv(4).lpa(0.01)` |
| サンプル | `.bank("tr808")` `.clip(0.5)` `.legato(1.2)` `.cut(1)` `.n(0)` |
| orbit / duck | `.orbit(2)` `.duckorbit(2).duckattack(0.15).duckdepth(0.9)` |
| ダイナミクス | `.compressor("-20:4:6:.003:.1")`（Mixer マスターへ last-write） |

orbit は **デッキ単位で 4 本**（id 1..4）。Deck A と B の orbit は共有しません。  
`delay` / `room` は未実装で、Task 24 で orbit バスへ接続予定です。

```
// kick が pad の orbit を duck
$: s("bd*4").gain(0.9).duckorbit(2).duckattack(0.15).duckdepth(0.9)
$: note("c3'maj").s("sawtooth").lpf(800).orbit(2).gain(0.4)
```

## HTTP API

`play` 起動時に **HTTP API** が立ち上がります（既定 `http://127.0.0.1:17878`）。

| フラグ / 環境変数 | 意味 |
| --- | --- |
| `--port N` | 待受ポート |
| `--no-api` | API を起動しない |
| `STRUDEL_API_PORT` | 既定ポート上書き（CLI 未指定時） |
| `STRUDEL_API` | ベース URL（MCP / スクリプト向け。例: `http://127.0.0.1:17878`） |

| メソッド | パス | 内容 |
| --- | --- | --- |
| GET | `/health` | 生存確認 |
| GET | `/status` | デッキ・BPM・曲内小節・ゲイン |
| GET | `/events` | 状態 SSE |
| PUT | `/code` | 単発パターンをデッキへ |
| POST | `/song/load` | `.strudel` をロード |
| POST | `/xfade` | クロスフェード |
| POST | `/bpm` | マスター BPM |
| POST | `/mute` | トラック mute/unmute |
| POST | `/head` | デッキ頭出し（1 始まり小節、次バーで同期） |
| POST | `/hush` | 全停止 |

```bash
# 本体（別ターミナル）— songs/ samples/ のあるディレクトリで
strudel-rs play --headless songs/smoke.strudel

# 操作例
curl -s http://127.0.0.1:17878/status
curl -s -X PUT -H "Content-Type: application/json" \
  -d '{"code":"note(\"c3 e3 g3\").s(\"triangle\")","deck":"A"}' \
  http://127.0.0.1:17878/code
```

## MCP 登録

`strudel-rs mcp` は **stdio の薄いブリッジ**です。音声デバイスは開きません。  
**先に** `strudel-rs play`（API 付き）を起動してから、MCP クライアントを繋いでください。本体未起動時はツールが接続エラーを返します（仕様）。

提供ツール: `strudel_set_code` / `strudel_load_song` / `strudel_xfade` / `strudel_set_bpm` / `strudel_mute` / `strudel_head` / `strudel_hush` / `strudel_status`

### 手順

1. 演奏本体を起動する（API `:17878`）
2. MCP クライアントに下記を登録する
3. チャットから `strudel_status` や `strudel_set_code` を呼ぶ

```bash
# ターミナル 1 — 演奏（API も同時に立つ）
cd /path/to/strudel-rust
strudel-rs dj songs/smoke.strudel
# または headless 単曲:
# strudel-rs play --headless songs/smoke.strudel
```

### Hermes（`config.yaml`）

```yaml
mcp_servers:
  strudel:
    command: strudel-rs
    args: ["mcp"]
    # ポートを変えている場合:
    # env:
    #   STRUDEL_API: "http://127.0.0.1:17878"
```

`command` にフルパスを書く例:

```yaml
mcp_servers:
  strudel:
    command: /home/you/.cargo/bin/strudel-rs   # Windows 例: C:\Users\you\.cargo\bin\strudel-rs.exe
    args: ["mcp"]
```

### Claude Desktop / Cursor 系（`mcp.json`）

設定ファイルの場所はクライアントによります（例: Claude Desktop の `claude_desktop_config.json`、Cursor の MCP 設定）。

```json
{
  "mcpServers": {
    "strudel": {
      "command": "strudel-rs",
      "args": ["mcp"],
      "env": {
        "STRUDEL_API": "http://127.0.0.1:17878"
      }
    }
  }
}
```

フルパス版:

```json
{
  "mcpServers": {
    "strudel": {
      "command": "/home/you/.cargo/bin/strudel-rs",
      "args": ["mcp"]
    }
  }
}
```

### 動作確認（手動）

```bash
# 本体起動済みの状態で
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"t","version":"0"}}}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
  | strudel-rs mcp
```

`tools/list` の応答に `strudel_set_code` / `strudel_head` など 8 ツールが出ればブリッジは生きています。

### 同梱デモ曲

#### Task 23 展示（DJ 切替デモ向け・短め）

| ファイル | 内容 | テンポ |
| --- | --- | --- |
| `songs/techno1.strudel` | duck/orbit + FM ベース + compressor | 126 BPM |
| `songs/ambient1.strudel` | `wt_organ` / `wt_bright` + vib + 和音（サンプル不要） | 126 BPM |

```bash
cargo run -- play songs/techno1.strudel
cargo run -- dj songs/techno1.strudel songs/ambient1.strudel
# 起動後: x 4 で A→B クロスフェード
cargo test --test e2e
```

#### 16 小節ループ

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

16 小節曲は `<...>` で 16 サイクル分の展開を持ち、そのままループする。Strudel 記法（`setcpm` / `$:` / `// @title` メタデータ）で書いているので REPL からのコピペ改造もしやすい。メタデータの書き方は [Strudel: Music metadata](https://strudel.cc/learn/metadata/) に合わせている。

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
