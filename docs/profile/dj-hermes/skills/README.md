# Strudel skills（プロファイル同梱）

展示用 profile `dj-hermes` が使う **ローカル skills** です。  
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

## ソース

開発の正本は `puredata-hermes/skills/creative/strudel-*` 側にある想定です。  
このディレクトリは **展示プロファイルに載せるためのコピー**です。スキル本文を直すときは正本を直してからここへ再コピーしてください。

```powershell
# 例: puredata-hermes 正本 → この見本
$src = "..\..\..\..\skills\creative"   # リポジトリ配置に合わせて調整
$dst = "creative"
Get-ChildItem $src -Directory | Where-Object Name -like 'strudel-*' | ForEach-Object {
  Copy-Item $_.FullName (Join-Path $dst $_.Name) -Recurse -Force
}
```

## セキュリティ

- `skills` toolset は skill の **一覧・閲覧**用
- 見本 config は `skills.write_approval: true`（skill ファイルの書き込みは承認制）
- shell / file toolset は無効のまま
