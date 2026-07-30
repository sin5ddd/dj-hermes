---
name: strudel-data-format
description: "Use when writing .strudel files or music metadata tags."
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel, music, file-format, metadata]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-genre-acid
      - strudel-genre-electro
      - strudel-genre-minimal-techno
      - strudel-genre-house
      - strudel-genre-dnb
      - strudel-genre-ambient
      - strudel-genre-chill
      - strudel-genre-dubstep
      - strudel-genre-progressive-house
      - strudel-genre-future-bass
      - strudel-genre-lofi-hiphop
      - strudel-genre-chill-pop
---

# Strudel データ形式（`.strudel`）

## Overview
`.strudel` は Strudel 楽曲コード（JavaScript + mini-notation 文字列）を保存するための暫定拡張子です。この Skill はファイル形式の規約と、楽曲メタデータ（コメント内の `@title` 等タグ）の書き方を定義します。メタデータは Strudel 本体からは無視されますが、検索・管理ツールや他ソフトで楽曲情報を取り出すために使われます。

## When to Use
- ユーザーが `.strudel` ファイルを新規作成・保存する時
- 楽曲にタイトル・作者・ライセンス等のメタデータを付与したい時
- Strudel 楽曲をフォルダで管理・検索するツールを作る時

Don't use for: 実際の音作り（→ strudel-sound-design）、パターン記法の記述（→ strudel-composition）。

## `.strudel` ファイルの基本
- 拡張子: `.strudel`（暫定規約）
- 中身は通常の Strudel コード（JS + mini-notation 文字列）そのまま
- メタデータはコード中のコメントとして記述する

## メタデータ記法
行コメントでタグを書く:
```
// @title My Cool Song
// @by John Doe
// @license CC-BY-SA-4.0
```

代替構文（ブロックコメント）:
```
/*
 @title My Cool Song
 @by John Doe
 @license CC-BY-SA-4.0
*/
```

1行に複数タグを書くことも可:
```
// @title My Cool Song @by John Doe @license CC-BY-SA-4.0
```

`title` のみ、先頭に引用符で書く別構文あり（ファイル先頭に限定）:
```
// "My Cool Song" @by John Doe
```

## タグ一覧
| タグ | 意味 |
| --- | --- |
| `@title` | 曲名 |
| `@by` | 作者（カンマ区切り、`<>` でリンク可: `@by John Doe `） |
| `@license` | ライセンス（SPDX識別子、例: `CC-BY-SA-4.0`） |
| `@details` | 補足情報 |
| `@url` | 関連URL（リポジトリ、Soundcloud等） |
| `@genre` | ジャンル（pop, jazz 等） |
| `@album` | アルバム名 |
| `@tag` | 任意のタグ |

## 複数値の書き方
一部のタグはカンマ・改行・タグ重複で複数値を取る:
```
/*
 @by John Doe
 @by Jane Doe
 @genre pop, jazz
 @url https://example.com
 @url https://example.org
*/
```
接頭辞を付けて使い分けも可:
```
/* song @by John Doe samples @by Jane Doe */
...note("a3 c#4 e4 a4") // @by Sandy Sue
```

## 複数行の値
リスト非対応タグ（@details等）は複数行の値を取れる:
```
/*
@details I wrote this song in February 19th, 2023.
 It was around midnight and I was lying on
 the sofa in the living room.
*/
```

## オンラインREPLでの検索
- 作者検索: `by: Ada L`
- ジャンル検索: `genre: unicorns`
- メタデータ未指定時は `@title`/`@by`/`@tag` に一致するものが表示される

## ツール作者への注意
メタデータの構文が正しいとは限らない。不正な値が来ても壊れないよう堅牢に実装すること。

## Common Pitfalls
1. メタデータをコメント外（実コード）に書く → Strudel が構文エラーになる。
2. SPDX識別子を間違える（例: `CC-BY` のみ）→ ライセンス検索で引っかからない。
3. ツール側で値を信じ込む → 不正値でパース落ち。必ずバリデーション＋フォールバック。

## Verification Checklist
- [ ] 拡張子が `.strudel` である
- [ ] メタデータがコメント内にあり、実コードを壊していない
- [ ] タグ名が公式一覧に存在する（@title/@by/@license/@details/@url/@genre/@album/@tag）
- [ ] 複数値はカンマ・改行・重複いずれかの規約に従っている
- [ ] ツールが不正値でもクラッシュしない
