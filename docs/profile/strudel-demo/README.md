# Hermes プロファイル見本: `strudel-demo`

展示ブース用の **隔離プロファイル** サンプルです。  
個人用 Hermes 設定・キー・memory を混ぜず、**strudel MCP 操作だけ**を LLM に渡します。

運用手順の本編は [docs/exhibit/README.md](../../exhibit/README.md) を参照。

## このディレクトリに含まれるもの

| ファイル | 内容 |
| --- | --- |
| `config.yaml` | MCP / toolset 隔離 / agent 設定（パス・キーなし） |
| `SOUL.md` | 展示向け人格 |
| `profile.yaml` | プロファイル説明 |
| `.no-bundled-skills` | バンドル skills を載せないマーカー |

**含めないもの（マシン固有・秘密）**

- `.env` / `auth.json` / OAuth トークン
- 絶対パス（`strudel-rs` のフルパス）
- `model.provider` / API キー
- `sessions/` / `state.db` / `memories/` / キャッシュ

## インストール（他端末）

```bash
# 1) 空の展示用プロファイルを作成（skills なし）
hermes profile create strudel-demo --no-skills \
  --description "Public exhibit booth: live Strudel DJ via strudel MCP only."

# 2) 見本をプロファイルへコピー
#    HERMES の profiles ルートは環境により異なる:
#      Linux/macOS: ~/.hermes/profiles/strudel-demo/
#      Windows:     %LOCALAPPDATA%\hermes\profiles\strudel-demo\
PROFILE_DIR="$(hermes --profile strudel-demo config path | xargs dirname)"
# config path が使えない場合の例:
#   PROFILE_DIR="$HOME/.hermes/profiles/strudel-demo"

cp docs/profile/strudel-demo/config.yaml   "$PROFILE_DIR/"
cp docs/profile/strudel-demo/SOUL.md       "$PROFILE_DIR/"
cp docs/profile/strudel-demo/profile.yaml  "$PROFILE_DIR/"
cp docs/profile/strudel-demo/.no-bundled-skills "$PROFILE_DIR/"

# 3) strudel-rs の場所を合わせる（PATH に無い場合）
#    config.yaml の mcp_servers.strudel.command を絶対パスに変更
#    または PATH に strudel-rs を通す

# 4) 展示用モデル・認証（個人用と分離推奨）
hermes --profile strudel-demo model

# 5) 確認
hermes --profile strudel-demo tools list --platform cli
hermes --profile strudel-demo mcp list
# dj 起動後:
hermes --profile strudel-demo mcp test strudel
```

Windows（PowerShell 例）:

```powershell
hermes profile create strudel-demo --no-skills
$dst = Join-Path $env:LOCALAPPDATA "hermes\profiles\strudel-demo"
Copy-Item docs\profile\strudel-demo\config.yaml, `
          docs\profile\strudel-demo\SOUL.md, `
          docs\profile\strudel-demo\profile.yaml, `
          docs\profile\strudel-demo\.no-bundled-skills `
          -Destination $dst -Force
hermes --profile strudel-demo model
```

## セキュリティ要点

1. **専用 profile**（個人用と混ぜない）
2. **内蔵 toolset は全部 off** + `agent.disabled_toolsets` で x_search 等の自動有効化も封じる
3. MCP は **strudel のみ**、`strudel_hush` は exclude（緊急停止はオペレータの `/hush` / Esc）
4. `hermes tools disable` を後から再実行すると `platform_toolsets` が書き換わることがある → 変更後は必ず `tools list` で確認

## ライブ profile からの再エクスポート

手元の動いている profile を見本に反映するときは、**秘密と絶対パスを落とす**:

```bash
# 例: 生きている config をサニタイズして上書き（手作業推奨）
# - mcp_servers.strudel.command → strudel-rs
# - model / base_url セクション削除
# - .env / auth.json はコピーしない
```

## 関連

- [docs/exhibit/README.md](../../exhibit/README.md) — ブース手順
- [docs/exhibit/hermes-strudel-demo.yaml.example](../../exhibit/hermes-strudel-demo.yaml.example) — 旧スニペット（本ディレクトリが正本）
