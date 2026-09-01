# strudel-rs

Strudel 記法の曲ファイルをリアルタイム演奏する Rust 製 CLI（デュアルデッキ・バークオンタイズ）。

実装は進行中です。概要・制約・モジュール構成は [AGENTS.md](./AGENTS.md) を参照してください。
詳細タスクは [docs/plans/2026-07-27_020000-strudel-rs-final.md](./docs/plans/2026-07-27_020000-strudel-rs-final.md) が正本です。

## インストール

Rust toolchain（stable）が必要です。WAV サンプルは **Git LFS** です（`git-lfs` を入れてからクローン）。詳細は [samples/README.md](./samples/README.md)。

```bash
# 初回（マシンごと）
git lfs install

# GitHub から（private の場合は git 認証済みであること）
cargo install --git https://github.com/sin5ddd/strudel-rust --locked

# クローン済みリポジトリから
git clone https://github.com/sin5ddd/strudel-rust.git
cd strudel-rust
git lfs pull   # 既にクローン済みで *.wav が数行のポインタなら
cargo install --path . --locked
```

インストール先は通常 `~/.cargo/bin/strudel-rs`（Windows は `%USERPROFILE%\.cargo\bin\strudel-rs.exe`）。
`PATH` に `cargo` の bin が入っていれば、どのディレクトリからでも `strudel-rs` を呼べます。

曲ファイル（`songs/`）とサンプル WAV（`samples/`）はリポジトリ同梱です。インストール後もデモを鳴らすときは、リポジトリをクローンしてそのルートで `play` するか、曲パスを明示してください。

```bash
# 動作確認
strudel-rs help
cd /path/to/strudel-rust   # songs/ と samples/ がある場所
strudel-rs play songs/house-01.strudel --headless
```

Linux では ALSA 開発ヘッダが必要なことがあります（例: `libasound2-dev`）。

## いま聴けるもの（最小 play）

リポジトリルートで（未 install なら `cargo run --` を先頭に付ける）:

```bash
strudel-rs play songs/house-01.strudel
strudel-rs play songs/house-01.strudel --seconds 15
# TUI なし（メタログのみ・スクリプト向け）
strudel-rs play songs/house-01.strudel --headless
# デュアルデッキ live UI（曲は省略可）。両曲は 124 BPM で同じ Transport。
strudel-rs dj songs/house-01.strudel songs/four-on-the-floor-01.strudel
strudel-rs dj
# 開発時
cargo run -- dj songs/house-01.strudel songs/four-on-the-floor-01.strudel
# genre examples (same Transport BPM per pair)
strudel-rs play songs/four-on-the-floor-01.strudel --seconds 12
strudel-rs play songs/house-01.strudel --headless --seconds 8
strudel-rs dj songs/house-01.strudel songs/four-on-the-floor-01.strudel
strudel-rs play songs/dnb-01.strudel --seconds 12
strudel-rs play songs/techno-duck-01.strudel --headless --seconds 8
# 130 BPM acid / 303 filter env (play solo)
strudel-rs play songs/acid-01.strudel --seconds 12
```

Recipes (Cursor `SKILL.md` + playable `songs/<genre>-01.strudel`): [docs/profile/dj-hermes/skills/creative/](./docs/profile/dj-hermes/skills/creative/).

- **既定はループ再生**（終了: TUI なら `q` / Esc、`--headless` なら Ctrl+C）
- `--seconds N`: N 秒で自動停止（スクリプト向け）
- **既定はミニ記法ライブハイライト TUI**（曲ソース表示・再生中 atom を ANSI 強調）
- `--headless`: 旧来のメタログのみ（TTY 不要・CI / パイプ向け）
- **`dj [SONG_A] [SONG_B]`**: **ハイライト + コマンド行**のライブ UI + `songs/` ウォッチャ（デモ / DJ 向け）
  - 画面上段: **左 = デッキ A / 右 = デッキ B** のミニ記法ハイライト（同時表示）
  - **`F10` または `/viz`**: 上段を **punchcard** に切替。上段=ドラム（`$:` ごとレーン・**一色**）、下段=ノートのピアノロール（**楽器＝note `$:` ごと色分け**）。`/viz on` / `/viz off` も可
  - **`F9` または `/vfx`**（別名 `/dopa` `/flash`）: ヒットに合わせた VFX（ドラム固有色・波紋・メロディビーム・ライザー色相）。既定 On。ヘルプ行の `[VFX]` をクリックしても切替。ハットなどの細かいヒットは局所のみ（全画面の点滅はしない）
  - 中段: **A/B の Hi・Mid・Lo EQ**（各 3 行・短スライダー。中央 0.5＝フラット、±12 dB。Mixer チャンネル EQ に連動）
  - その下: **クロスフェーダー**（最大 10 文字幅 `XF A ──□── B`。□ は白背景。クリック／ドラッグ）
  - 下段: ログ + `»` プロンプト
  - A のみ / B のみ / 両方省略も可（空デッキから `/a load` / `/b load`）
  - **入力モデル（live TUI）**
    - **自然文**（例: `暗くして`）→ Hermes（既定プロファイル `dj-hermes`、MCP 経由で操作）
    - **F12** → マイク録音トグル → **ローカル STT**（`STRUDEL_STT_BASE_URL`）→ 同じ Hermes 経路（画面に Hermes は出ない）
    - **`/` 付き**（例: `/x 4` `/bpm 128` `/a load house-01`）→ ローカル即時コマンド
    - `--no-hermes` または Hermes 未検出時: 裸入力もローカル（従来どおり）
    - `--no-voice` で F12 音声を明示オフ
  - ローカルコマンド例:
    - `/a load songs/house-01.strudel` / `/b load songs/four-on-the-floor-01.strudel`
    - `/b head 33`（次の小節境界で B を曲の 33 小節目から。別名 `cue`。1 始まり）
    - `/x 4`（反対側デッキへ 4 小節 xfade）/ `/b x 4`
    - `/a mute kick` / `/bpm 128` / `/status` / `/help` / `/viz` / `/vfx`
    - オペレータ: `/hush` `/quit`
  - `--text`: ハイライトなしの rustyline テキスト REPL（**裸コマンドのまま**。Hermes は TUI のみ）
  - 互換: `play --repl` / `play --repl-text` も同じセッションを起動（A/B 2 曲可）
  - 展示向け Hermes 手順・プロンプトインジェクション対策: [docs/exhibit/README.md](./docs/exhibit/README.md)
- サンプルは `./samples`（Sonic Pi 由来 CC0、**Git LFS**）。曲は `songs/*.strudel`
- **記法 → リズム / 和声 / DJ の対応**と再利用スキル: [docs/profile/dj-hermes/skills/creative/README.md](./docs/profile/dj-hermes/skills/creative/README.md)（例: `songs/four-on-the-floor-01.strudel`, `songs/house-01.strudel`）
- 出力デバイスが無い環境ではエラー終了（`cargo test` / build はデバイス不要）

## パターン記法の拡張（Task 23–24）

ローカル FX・シンセ（パーボイス）と Deck 内 orbit（delay / room 含む）をサポートしています。

| 系統 | メソッド例 |
| --- | --- |
| 波形 / ノイズ | `sine` `sawtooth` `square` `triangle` `white` `pink` `brown` |
| ウェーブテーブル | `wt_sine` `wt_bright` `wt_organ`（手続き生成の 1 周期） |
| 変調 | `.vib("4:12")` `.fm(4).fmh(1.5)` `.noise(0.2)` `.penv(12)` |
| フィルタ | `.lpf(800)` `.lpq(2)` `.hpf(200)` `.bpf(1000)` `.lpenv(4).lpa(0.01)` |
| サンプル | `.bank("tr808")` `.clip(0.5)` `.legato(1.2)` `.cut(1)` `.n(0)` |
| orbit / duck | `.orbit(2)` `.duckorbit(2).duckattack(0.15).duckdepth(0.9)` |
| delay / room | `.delay(0.5)` `.delay("0.5:0.25:0.8")` `.delaytime(0.25)` `.delayfeedback(0.6)` `.room(0.4)` `.room("0.9:4")` `.roomsize(2)` |
| ダイナミクス | `.compressor("-20:4:6:.003:.1")`（Mixer マスターへ last-write） |

orbit は **デッキ単位で 4 本**（id 1..4）。Deck A と B の orbit は共有しません。  
`delay` / `room` は **orbit 共有の global FX**（同 orbit 上は last-write）。`delayfeedback` は 0.95 未満にクランプされます。

本家 Strudel にあって **未実装のシンセ / FX 一覧**: [docs/strudel-gap-synths-fx.md](./docs/strudel-gap-synths-fx.md)

```
// kick が pad の orbit を duck
$: s("bd*4").gain(0.9).duckorbit(2).duckattack(0.15).duckdepth(0.9)
$: note("c3'maj").s("sawtooth").lpf(800).orbit(2).gain(0.4).room(0.35).roomsize(3)
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
| GET | `/status` | デッキ・BPM・曲内小節・ゲイン・EQ・filter・crossfader |
| GET | `/events` | 状態 SSE |
| PUT | `/code` | 単発パターンをデッキへ（スクリプト向け） |
| POST | `/song/load` | `.strudel` をロード（bare 名は `~/.config/strudel-rs/songs/` → `songs/`） |
| POST | `/song/save` | ユーザー曲ライブラリに保存（**のみ** `~/.config/strudel-rs/songs/`。basename 限定。任意で deck ロード） |
| POST | `/xfade` | N バー クロスフェード |
| POST | `/mix` | DJ マクロ（long / cut / fill / hold） |
| POST | `/bpm` | マスター BPM |
| POST | `/mixer/eq` | A/B チャンネル EQ（hi/mid/lo、即時） |
| POST | `/mixer/filter` | マスター LPF/HPF（即時、`null` でバイパス） |
| POST | `/mixer/crossfader` | 即時クロスフェーダー位置 0..=1 |
| POST | `/mute` | トラック mute/unmute |
| POST | `/head` | デッキ頭出し（1 始まり小節、次バーで同期） |
| POST | `/hush` | 全停止 |
| POST | `/mcp` | MCP Streamable HTTP（Hermes 用。JSON-RPC） |

```bash
# 本体（別ターミナル）— songs/ samples/ のあるディレクトリで
strudel-rs play --headless songs/house-01.strudel

# 操作例
curl -s http://127.0.0.1:17878/status
curl -s -X POST -H "Content-Type: application/json" \
  -d '{"deck":"A","lo":0.3,"hi":0.7}' \
  http://127.0.0.1:17878/mixer/eq
curl -s -X POST -H "Content-Type: application/json" \
  -d '{"pos":0.5}' \
  http://127.0.0.1:17878/mixer/crossfader
```

## MCP 登録（Hermes）

演奏プロセスの HTTP API 上に **Streamable HTTP MCP**（`POST /mcp`）があります。  
音声デバイスは MCP 経路では開きません。**先に** `strudel-rs play` / `dj`（API 付き）を起動してから Hermes を繋いでください。本体未起動時は接続エラーになります（仕様）。

提供ツール（Mixer → Deck → Transport）:

| グループ | ツール |
| --- | --- |
| Mixer | `strudel_mixer_eq` / `strudel_mixer_filter` / `strudel_mixer_crossfader` / `strudel_xfade` / `strudel_mix` / `strudel_set_bpm` |
| Deck | `strudel_load_song` / `strudel_apply_song` / `strudel_list_songs` / `strudel_save_song` / `strudel_mute` / `strudel_head` |
| Transport | `strudel_hush` / `strudel_status` |

曲の差し替えは **`strudel_load_song`**（`.strudel` ファイル）または **`strudel_apply_song`**（全文・無書き込み）。新規の永続化は **`strudel_save_song`**（書き込み先は `~/.config/strudel-rs/songs/` のみ、演奏は変えない）。HTTP `PUT /code` はスクリプト用に残置。

### 手順

1. 演奏本体を起動する（API `:17878`）
2. Hermes プロファイルに下記を登録する（`command` で exe を spawn しない）
3. チャットから `strudel_status` や `strudel_mixer_eq` を呼ぶ

```bash
# ターミナル 1 — 演奏（API + /mcp も同時に立つ）
cd /path/to/strudel-rust
strudel-rs dj songs/house-01.strudel songs/four-on-the-floor-01.strudel
# または headless 単曲:
# strudel-rs play --headless songs/house-01.strudel
```

### 音声入力（F12 / VAD → ローカル STT → Hermes）

live TUI で **F12** を押すと録音開始、もう一度 F12 で停止（最大 7 秒）。  
`POST {STRUDEL_STT_BASE_URL}/v1/audio/transcriptions` で文字化し、既存の Hermes oneshot に渡します。Hermes の対話画面は出ません。クラウド音声 API（xAI / ElevenLabs など）は使いません。

展示で HP に Whisper を置く手順は [docs/exhibit/stt-hp.md](./docs/exhibit/stt-hp.md)。

```bash
# Surface（演奏機）— HP の STT が起動済みであること
export STRUDEL_STT_BASE_URL=http://192.168.x.x:8090
# 任意:
# export STRUDEL_STT_API_KEY=booth-token
# export STRUDEL_STT_LANGUAGE=ja
# export STRUDEL_VOICE_MAX_SECS=7
# export STRUDEL_VOICE_MODE=push    # または vad
strudel-rs dj songs/house-01.strudel songs/four-on-the-floor-01.strudel
# F12 で話す → 認識テキストが Hermes → MCP → 音が変わる
```

`STRUDEL_STT_BASE_URL` 未設定・マイク無し・`--no-voice` のときは音声のみ無効（TUI / キーボード自然文は従来どおり）。  
default 入力デバイスを使います（`arecord -l` で確認）。

### Hermes（`config.yaml`）

展示ブースでは **専用プロファイル `dj-hermes`** を使い、strudel MCP 以外のツールを無効にしてください（詳細: [docs/exhibit/README.md](./docs/exhibit/README.md)）。

```yaml
mcp_servers:
  strudel:
    url: "http://127.0.0.1:17878/mcp"
    # ポートを変えている場合は URL のポートも合わせる
    tools:
      exclude: [strudel_hush]   # 緊急停止は TUI の /hush
```

`GET /events` は状態 SSE であり MCP ではありません。Hermes の `url` は必ず **`/mcp`** を指してください。

### 動作確認

```bash
# 本体起動済みの状態で
curl -s -X POST http://127.0.0.1:17878/mcp \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"t","version":"0"}}}'

hermes --profile dj-hermes mcp test strudel
# → Transport: HTTP … Connected, 12 tools
```

`tools/list`（または `mcp test`）で `strudel_mixer_eq` / `strudel_mix` / `strudel_load_song` / `strudel_apply_song` / `strudel_save_song` / `strudel_status` など **17 ツール**が出れば生きています（`strudel_set_code` は含みません）。

デバッグ用に **非推奨** の `strudel-rs mcp`（stdio → REST ブリッジ）も残していますが、Hermes からは使いません。

### 同梱デモ曲

#### 展示（DJ 切替デモ向け・同じ 124 BPM）

| ファイル | 内容 | テンポ |
| --- | --- | --- |
| `songs/house-01.strudel` | ハウス clap 2/4 + `plk:lp` | 124 BPM |
| `songs/four-on-the-floor-01.strudel` | kick+offbeat hats + sine/triangle（サンプル不要でも鳴る） | 124 BPM |

```bash
cargo run -- play songs/house-01.strudel
cargo run -- dj songs/house-01.strudel songs/four-on-the-floor-01.strudel
# 起動後: x 4 で A→B クロスフェード
cargo test --test e2e
```

#### ジャンル曲（`songs/<genre>-01.strudel` … `-10`）

| ファイル | ジャンル | テンポ |
| --- | --- | --- |
| `songs/techno-duck-01.strudel` | テクノ duck/orbit + FM ベース | 126 BPM |
| `songs/electro-01.strudel` | エレクトロ（同じ 126 で techno-duck と組める） | 126 BPM |
| `songs/dnb-01.strudel` | DnB | 174 BPM |
| `songs/acid-01.strudel` | アシッド / 303 filter env | 130 BPM |
| `songs/house-01.strudel` | ハウス | 124 BPM |

```bash
cargo run -- play songs/techno-duck-01.strudel
cargo run -- play songs/house-01.strudel --seconds 45
```

デモ曲は **短い `$:` ループ**（2–5 トラック）が既定。`<>` でサイクル差分を出し、演奏しながらコードを少しずつ書き換える想定（16 小節 `cat` の長尺アレンジではない）。Strudel 記法（`setcpm` / `$:` / `// @title`）なので REPL からのコピペ改造もしやすい。メタデータは [Strudel: Music metadata](https://strudel.cc/learn/metadata/) に合わせている。

**ドラムのミニ記法:** スペース=順、カンマ=同時、`@n`=時間ウェイト（elongate）。デモは原則 1 本の `$:`（例: `s("bd*4, [~ sd]*2, [~ hh]*4")`）。duck 付きキックだけは別トラックに残す。

**ベース / 次数:** `note("0 2 0 3 0 <2 4>").scale("C2:minor")` のように 0 始まりのスケール次数が使える（ルート相対、負の次数可。例: `C2:major` の `-1` → B1）。

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

### Version control

このリポジトリは **git + Git LFS** のみ。`samples/**/*.wav` は LFS。jj をコロケートしない（jj は LFS smudge をしない）。

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
