# Strudel skills（プロファイル同梱・strudel-rs 専用）

展示用 profile `dj-hermes` が使う **ローカル skills** です。  
本家 Strudel REPL 記法は含みません。保存形式は `setcpm` + `$:` のみ。  
ドラムは原則 **1 本の `s(...)`**（スペース=順再生、カンマ=同時再生）。正本は `strudel-composition`。

Hermes の公式バンドル skills とは別物で、`.no-bundled-skills` により公式カタログは入れません。

## 配置

```
skills/
  creative/
    strudel-composition/
    strudel-data-format/
    strudel-sound-design/
    strudel-genre-*/
```

各ディレクトリに `SKILL.md` が必要（agentskills.io / Hermes 互換）。

## 正本と同期

**正本はこのリポジトリの `docs/profile/dj-hermes/skills/`** です。  
live profile へは次をコピーします:

```powershell
$src = "docs\profile\dj-hermes\skills\creative"
$dst = Join-Path $env:LOCALAPPDATA "hermes\profiles\dj-hermes\skills\creative"
New-Item -ItemType Directory -Force -Path $dst | Out-Null
Copy-Item -Recurse -Force "$src\*" $dst
```

Linux/macOS:

```bash
PROFILE_DIR="${HERMES_HOME:-$HOME/.hermes}/profiles/dj-hermes"
mkdir -p "$PROFILE_DIR/skills"
cp -R docs/profile/dj-hermes/skills/creative "$PROFILE_DIR/skills/"
```

## 静的チェック

コードフェンス内に `stack(` / `.cpm(` 等が無いことを検査:

```bash
python scripts/lint_strudel_skills.py
```

## セキュリティ

- `skills` toolset は skill の **一覧・閲覧**用
- 見本 config は `skills.write_approval: true`（skill ファイルの書き込みは承認制）
- shell / file toolset は無効のまま
- `tools.tool_search.enabled: off` で MCP フル schema を常時表示（小モデル向け）
