# Strudel skills（プロファイル同梱・dj-hermes 専用）

展示用 profile `dj-hermes` が使う **ローカル skills** です。  
本家 Strudel REPL 記法は含みません。保存形式は `setcpm` + `$:` のみ。

**思想**: 7–8 本の `$:` を 4 小節フレーズで重ね、演奏しながら 1 本ずつ書き換える（16 小節 `cat` は非既定）。  
正本は `strudel-composition`。

Hermes の公式バンドル skills とは別物で、`.no-bundled-skills` により公式カタログは入れません。

## 配置

```
skills/
  creative/
    strudel-composition/   # 正本（7–8 本 + 4 小節フレーズ + mini）
    strudel-live-edit/     # 自然言語 → 差分編集（メロディ/フィル/転調/明暗）
    strudel-dj-mix/        # A/B ミックス（dj_hermes_mix 1 呼び）
    strudel-dj-hype/       # フロアを沸かせて（状況 → mix 1 回）
    strudel-data-format/   # ファイル形式
    strudel-sound-design/  # 音色・FX・サンプル用法（bank / フルネーム）・役割レシピ
    strudel-pcm-catalog/   # rust-fm-synthe part:slug（INDEX 付き）
    strudel-genre-*/       # ジャンル別スロット（グリッド + 8 本例）
```

各ディレクトリに `SKILL.md` が必要（agentskills.io / Hermes 互換）。

楽曲レシピの索引（記法 → リズム / 和声 / DJ）は [creative/README.md](./creative/README.md)。

## リポジトリ正本とプロファイルへのコピー

**正本はこのリポジトリの `docs/profile/dj-hermes/skills/creative/`** です（旧 `docs/skills/` から移した）。

```powershell
$src = "docs\profile\dj-hermes\skills\creative"
$dst = Join-Path $env:LOCALAPPDATA "hermes\profiles\dj-hermes\skills\creative"
Copy-Item -Recurse -Force $src $dst
```

```bash
mkdir -p "$PROFILE_DIR/skills"
cp -R docs/profile/dj-hermes/skills/creative "$PROFILE_DIR/skills/"
```

## Hermes 側の注意

- `skills` toolset は skill の **一覧・閲覧**用
- 見本 config は `skills.write_approval: true`（skill ファイルの書き込みは承認制）
