# Hermes プロファイル見本: `dj-hermes`

展示ブース用の **隔離プロファイル** サンプルです。  
個人用 Hermes 設定・キー・memory を混ぜず、**strudel MCP 操作 + Strudel 作曲スキル**だけを LLM に渡します。

運用手順の本編は [docs/exhibit/README.md](../../exhibit/README.md) を参照。

## このディレクトリに含まれるもの

| ファイル / ディレクトリ | 内容 |
| --- | --- |
| `config.yaml` | MCP / toolset 隔離 / agent 設定（パス・キーなし）。`skills` ツールのみ有効 |
| `SOUL.md` | 展示向け人格 |
| `profile.yaml` | プロファイル説明 |
| `.no-bundled-skills` | バンドル skills を載せないマーカー（公式カタログは入れない） |
| `skills/creative/strudel-*` | 作曲用ローカル skills（15 本） |

**含めないもの（マシン固有・秘密）**

- `.env` / `auth.json` / OAuth トークン
- 絶対パス（`strudel-rs` のフルパス）
- `model.provider` / API キー
- `sessions/` / `state.db` / `memories/` / キャッシュ

## 同梱 Strudel skills

**ライブ短いループ**向けの作曲ガイド。プロファイル配下へコピーして使う。

| スキル | 用途 |
| --- | --- |
| `strudel-composition` | 正本: 短いループ + mini + ライブ差分 save |
| `strudel-data-format` | ファイル形式・メタデータ（2–5 トラック目安） |
| `strudel-sound-design` | 波形・エフェクト・音色（ライブで触るツマミ） |
| `strudel-genre-*` | ジャンル別の短いレシピ（16 小節 cat は書かない） |

エージェントは `skills_list` / `skill_view` で必要なものだけ読む（progressive disclosure）。  
まず `strudel-composition`。`skill_manage` の書き込みは見本 config で `skills.write_approval: true`（承認待ち）。

### 開発中の共有ディレクトリ（任意）

skills を `puredata-hermes` 側で編集し続けたい場合は、コピーの代わりに `config.yaml` へ:

```yaml
skills:
  write_approval: true
  external_dirs:
    - C:/Users/you/Hermes/Projects/puredata-hermes/skills
    # または: ${PUREDATA_HERMES}/skills
```

外部 dir は **書き込み可能なと agent がそこを更新しうる**ので、展示ブースではプロファイル内コピー + `write_approval` を推奨。

## インストール（他端末）

```bash
# 1) 空の展示用プロファイルを作成（バンドル skills なし）
hermes profile create dj-hermes --no-skills \
  --description "Public exhibit booth: live Strudel DJ via strudel MCP only."

# 2) 見本をプロファイルへコピー
#    HERMES の profiles ルートは環境により異なる:
#      Linux/macOS: ~/.hermes/profiles/dj-hermes/
#      Windows:     %LOCALAPPDATA%\hermes\profiles\dj-hermes\
PROFILE_DIR="$(hermes --profile dj-hermes config path | xargs dirname)"
# config path が使えない場合の例:
#   PROFILE_DIR="$HOME/.hermes/profiles/dj-hermes"

cp docs/profile/dj-hermes/config.yaml   "$PROFILE_DIR/"
cp docs/profile/dj-hermes/SOUL.md       "$PROFILE_DIR/"
cp docs/profile/dj-hermes/profile.yaml  "$PROFILE_DIR/"
cp docs/profile/dj-hermes/.no-bundled-skills "$PROFILE_DIR/"
# Strudel skills（creative/ ごと）
mkdir -p "$PROFILE_DIR/skills"
cp -R docs/profile/dj-hermes/skills/creative "$PROFILE_DIR/skills/"

# 3) 演奏 API が http://127.0.0.1:17878 で生きていること
#    （strudel-rs dj / play。ポートを変えたら config の url も合わせる）

# 4) 展示用モデル・認証（個人用と分離推奨）
hermes --profile dj-hermes model

# 5) 確認
hermes --profile dj-hermes tools list --platform cli
#   → skills が enabled、他の危険 toolset は disabled
hermes --profile dj-hermes skills list --source local --enabled-only
#   → strudel-composition など 15 本
hermes --profile dj-hermes mcp list
# dj/play 起動後（MCP は HTTP /mcp — exe spawn なし）:
hermes --profile dj-hermes mcp test strudel
```

Windows（PowerShell 例）:

```powershell
hermes profile create dj-hermes --no-skills
$dst = Join-Path $env:LOCALAPPDATA "hermes\profiles\dj-hermes"
Copy-Item docs\profile\dj-hermes\config.yaml, `
          docs\profile\dj-hermes\SOUL.md, `
          docs\profile\dj-hermes\profile.yaml, `
          docs\profile\dj-hermes\.no-bundled-skills `
          -Destination $dst -Force
New-Item -ItemType Directory -Force -Path (Join-Path $dst "skills") | Out-Null
Copy-Item -Recurse -Force docs\profile\dj-hermes\skills\creative `
          (Join-Path $dst "skills\creative")
hermes --profile dj-hermes model
hermes --profile dj-hermes skills list --source local --enabled-only
```

## セキュリティ要点

1. **専用 profile**（個人用と混ぜない）
2. **内蔵 toolset は skills 以外 off** + `agent.disabled_toolsets` で x_search 等の自動有効化も封じる  
   （`skills` は skill_view 用に **許可**。shell / file は禁止のまま）
3. MCP は **strudel のみ**、`strudel_hush` は exclude（緊急停止はオペレータの `/hush` / Esc）
4. **バンドル skills は載せない**（`.no-bundled-skills`）。同梱は strudel 作曲用 15 本だけ（**strudel-rs 記法のみ**）
5. `skills.write_approval: true` で skill ファイルの作成・編集をオペレータ承認制に
6. `hermes tools disable` を後から再実行すると `platform_toolsets` が書き換わることがある → 変更後は必ず `tools list` で確認
7. **`tools.tool_search.enabled: off`** — MCP を tool_search の後ろに隠さない（ローカル小モデル向け）
8. 展示はネット不通を想定し **ローカル小モデル** を既定にする

## 小モデル向けの曲保存契約

skills / SOUL / MCP は次で揃えている:

- content = `setcpm(...)` + **短い** `$:` 行のみ（2–5 トラック目安。`stack` / `.cpm` 禁止）
- 既定は 1 サイクル + `<>`。長尺 `cat` は非既定
- ライブ編集は **同名 + deck で上書き**（1 パラメータ差分）
- 保存は `strudel_save_song(name, content, deck?)` のみ
- 未実装メソッド（`.lfo` / `.add`）や未同梱 `cp` は例に出さない
- 検査: `python scripts/lint_strudel_skills.py`

## ライブ profile からの再エクスポート

手元の動いている profile を見本に反映するときは、**秘密と絶対パスを落とす**:

```bash
# 例: 生きている config をサニタイズして上書き（手作業推奨）
# - mcp_servers.strudel.url → http://127.0.0.1:17878/mcp（command/args は使わない）
# - model / base_url セクション削除
# - .env / auth.json はコピーしない
# - skills/creative は puredata-hermes 側と diff を見て同期
```

## 関連

- [docs/exhibit/README.md](../../exhibit/README.md) — ブース手順
- [docs/exhibit/hermes-dj-hermes.yaml.example](../../exhibit/hermes-dj-hermes.yaml.example) — 旧スニペット（本ディレクトリが正本）
- 元スキル置き場（開発）: `puredata-hermes/skills/creative/strudel-*`
