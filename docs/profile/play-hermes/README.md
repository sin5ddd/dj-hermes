# Hermes プロファイル見本: `play-hermes`

`dj-hermes play` 用の **隔離プロファイル** サンプルです。  
1 曲を live 編集します。DJ ミックス（xfade / mix / デッキ B）は扱いません。

運用の共通手順は [docs/profile/dj-hermes/README.md](../dj-hermes/README.md) を参照。

## このディレクトリに含まれるもの

| ファイル | 内容 |
| --- | --- |
| `config.yaml` | MCP / toolset 隔離 / agent 設定。mix ツールは呼ばない旨を system_prompt に書く |
| `SOUL.md` | 1 曲 live 編集向け人格 |
| `profile.yaml` | プロファイル説明 |
| `.no-bundled-skills` | バンドル skills を載せないマーカー |
| `skills/creative/strudel-seqtrak/` | SEQTRAK MIDI 作曲スキル（play 専用の正本） |

**スキルの正本はほぼ `dj-hermes` 側**です。インストール時に `dj-hermes` の creative スキルをコピーし、`strudel-dj-mix` と `strudel-dj-hype` を外します。例外が **SEQTRAK MIDI**（`skills/creative/strudel-seqtrak/`）。こちらは play 専用なのでこのディレクトリが正本です。

**含めないもの（マシン固有・秘密）**

- `.env` / `auth.json` / OAuth トークン
- 絶対パス（`dj-hermes` のフルパス）
- `model.provider` / API キー
- `sessions/` / `state.db` / `memories/` / キャッシュ

## インストール

```bash
hermes profile create play-hermes --no-skills \
  --description "Public exhibit: live-edit one Strudel song via strudel MCP."

PROFILE_DIR="$(hermes --profile play-hermes config path | xargs dirname)"
# 使えない場合の例: PROFILE_DIR="$HOME/.hermes/profiles/play-hermes"

cp docs/profile/play-hermes/config.yaml   "$PROFILE_DIR/"
cp docs/profile/play-hermes/SOUL.md       "$PROFILE_DIR/"
cp docs/profile/play-hermes/profile.yaml  "$PROFILE_DIR/"
cp docs/profile/play-hermes/.no-bundled-skills "$PROFILE_DIR/"

# 作曲スキルは dj-hermes の正本から（dj-mix / dj-hype は入れない）
mkdir -p "$PROFILE_DIR/skills"
cp -R docs/profile/dj-hermes/skills/creative "$PROFILE_DIR/skills/"
rm -rf "$PROFILE_DIR/skills/creative/strudel-dj-mix"
rm -rf "$PROFILE_DIR/skills/creative/strudel-dj-hype"
cp -R docs/profile/play-hermes/skills/creative/strudel-seqtrak \
     "$PROFILE_DIR/skills/creative/"

hermes --profile play-hermes model
```

Windows（PowerShell 例）:

```powershell
hermes profile create play-hermes --no-skills
$dst = Join-Path $env:LOCALAPPDATA "hermes\profiles\play-hermes"
Copy-Item docs\profile\play-hermes\config.yaml, `
          docs\profile\play-hermes\SOUL.md, `
          docs\profile\play-hermes\profile.yaml, `
          docs\profile\play-hermes\.no-bundled-skills `
          -Destination $dst -Force
New-Item -ItemType Directory -Force -Path (Join-Path $dst "skills") | Out-Null
Copy-Item -Recurse -Force docs\profile\dj-hermes\skills\creative `
          (Join-Path $dst "skills\creative")
Remove-Item -Recurse -Force (Join-Path $dst "skills\creative\strudel-dj-mix")
Remove-Item -Recurse -Force (Join-Path $dst "skills\creative\strudel-dj-hype")
Copy-Item -Recurse -Force docs\profile\play-hermes\skills\creative\strudel-seqtrak `
          (Join-Path $dst "skills\creative\strudel-seqtrak")
hermes --profile play-hermes model
```

演奏本体は `dj-hermes play`（既定プロファイル `play-hermes`）。`dj` はこれまでどおり `dj-hermes` です。

SEQTRAK 展示は `dj-hermes play --midi-only --midi-port SEQTRAK`（ポート確認は `play --midi-list`）。スキルは **strudel-seqtrak**。
