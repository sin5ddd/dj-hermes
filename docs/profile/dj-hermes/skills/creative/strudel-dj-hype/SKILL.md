---
name: strudel-dj-hype
description: "Use when フロアを沸かせて, 沸かせて, 盛り上げて, ドロップ, 上げて, 沸かせるMIX, hype the floor, drop it."
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [dj-hermes, dj, mixer, hype, drop, fill]
    related_skills:
      - strudel-dj-mix
---

# フロアを沸かせる（状況 → 既存 mix 1 回）

来場者の「沸かせて」は **ライザー → B 固定にしない**。低レベル表は **strudel-dj-mix**。こちらは状況で `to` と `kind` を選ぶ。

`apply_song` / hush / `mixer_eq` 4 連はしない。デッキを切らずに短く止めるときだけ `dj_hermes_mixer_tape` を足してよい。

## 毎回の手順

1. **`dj_hermes_status`**（必須）
2. `mix` が残っていたら何もしない（重ねない）
3. クロスフェーダーが端でない → 主電源へ 100%
   - 主電源: `crossfader < 0.5` → A、それ以外 → B
   - `dj_hermes_mixer_crossfader(pos=0 または 1)`（即時）
4. 下の表で `kind` と `to` を決めて **`dj_hermes_mix` を 1 回**。同じ曲のまま止めて盛り上げるなら mix の代わりに `dj_hermes_mixer_tape(on=true, len="4n", reps=2)`（四分音符×2 連）
5. 任意でトラックミュートは 1 本まで（`dj_hermes_mute`）。同じデッキのドロップに mix の `mute_track` は使わない（戻らない）

## 状況 → `to`

| 状況 | `to` |
| --- | --- |
| 静かなデッキに曲がある | 静かな方 |
| 静かなデッキが空、または両デッキが同じスロット | **主電源**（空 B へ切らない） |
| 主電源の `drums` がミュート済み（`muted_a` / `muted_b`） | 主電源、`kind=drop` |

`to` = 主電源の fill は同じ曲のドロップ（終わりに同じデッキへスナップ）。

## 状況 → `kind`（既存 10 種だけ）

ジャンルは主電源スロットのフォルダ。`kind=vinyl` は既定 8 小節。かすれ（バンドパス）と wow 振幅が徐々に大きくなり、カットイン。保持の `dj_hermes_mixer_vinyl` は使わない（ドロップ後も残る）。

| 主電源ジャンル | 既定 kind | 代わり（同じ系統を連続しない） |
| --- | --- | --- |
| house / four-on-the-floor / techno-duck / progressive-house / acid / electro / minimal | `hpf` / `echo` / `vinyl` | `lpf` / `flash` |
| dnb / dnb-reese / dubstep / future-bass | `roll` または `drop` | `riser` |
| kawaii-future-bass / chill-pop | `riser` | `flash` / `drop` / `vinyl` |
| ambient / chill / lofi-hiphop | `lpf`（切替なら `long`） | `vinyl`（roll/drop は使わない） |

**`kind=riser` + `to=B` を既定にしない。** B が静かで曲が載っているときだけ `to=B`。

## ミュート（任意）

- ブレイク: 主電源の `pad` または `chords` をミュート（キックは残す）
- ドラムを落としてからドロップ: `drums` をミュート → 次の「沸かせて」で unmute + `drop`

ポストミックスのフィルター／ディレイ／ビニール保持は `dj_hermes_mixer_filter` / `dj_hermes_mixer_fx` / `dj_hermes_mixer_vinyl`（fill が終わっても残る。沸かしては使わない）。チャンネル EQ は **1.0 = 0 dB、0 = キル**（ブースト無し）。
