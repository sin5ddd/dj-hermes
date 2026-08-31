# Strudel skills（プロファイル同梱・strudel-rs 専用）

展示用 profile `dj-hermes` が使う **ローカル skills** です。  
本家 Strudel REPL 記法は含みません。保存形式は `setcpm` + `$:` のみ。

**思想**: 短いループを重ね、演奏しながら content を逐次書き換える（16 小節 `cat` は非既定）。  
正本は `strudel-composition`。

Hermes の公式バンドル skills とは別物で、`.no-bundled-skills` により公式カタログは入れません。

## 配置

```
skills/
  creative/
    strudel-composition/   # 正本（ライブ短いループ + mini）
    strudel-live-edit/     # 自然言語 → 差分編集（メロディ/フィル/転調/明暗）
    strudel-data-format/   # ファイル形式
    strudel-sound-design/  # 音色・FX・サンプル用法（bank / フルネーム）・役割レシピ
    strudel-pcm-catalog/   # rust-fm-synthe part:slug（INDEX 付き）
    strudel-genre-*/       # ジャンル別・短いレシピ
```

各ディレクトリに `SKILL.md` が必要（agentskills.io / Hermes 互換）。

## リポジトリ正本とプロファイルへのコピー

**正本はこのリポジトリの `docs/profile/dj-hermes/skills/`** です。

```powershell
$src = "docs\profile\dj-hermes\skills\creative"
$dst = Join-Path $env:LOCALAPPDATA "hermes\profiles\dj-hermes\skills\creative"
Copy-Item -Recurse -Force $src $dst
```

```bash
mkdir -p "$PROFILE_DIR/skills"
cp -R docs/profile/dj-hermes/skills/creative "$PROFILE_DIR/skills/"
```

## 検査

```bash
python scripts/lint_strudel_skills.py
```

## Hermes 側の注意

- `skills` toolset は skill の **一覧・閲覧**用
- 見本 config は `skills.write_approval: true`（skill ファイルの書き込みは承認制）
