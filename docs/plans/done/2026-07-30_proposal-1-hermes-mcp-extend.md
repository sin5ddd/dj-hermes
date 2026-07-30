# Proposal 1 — Hermes からの操作拡張（MCP ツール拡張 + デモ操作テンプレ）

> この展示会用に、**Hermes が strudel-rs の DJ 操作を直接行える**状態にする拡張案。
> 変化の契機は「Hermes チャット内の短文プロンプト」とし、**展示フロー** で示す。

---

## Goal

Hermes の MCP ツール一覧を `strudel_set_*` だけでなく **ミキサー操作** まで拡張し、
展示ブースでは来場者が「もっと暗く」→ Hermes → strudel-rs / 「ファンキーに」→ Hermes → strudel-rs という体験を可能にする。

---

## 前提

- Task 17–19: API, MCP server, error handling **完了**
- Task 26: ライブハイライト TUI **完了**
- 本体プロセス（API `17878`）は展示では **手動起動前提**（`./strudel-rs play`）
- MCP server は **別プロセス**（`./strudel-rs mcp`）で HTTP API へブリッジ

---

##  exposes する操作イメージ（来場者への説明文）

```
1 / 2：BPM / トラック / フェーダーは「次小節から変わる」
3     ：間違えても現行の音は止まらない
```

| 操作 | 例 | Hermes の入力 |
| --- | --- | --- |
| A/B 曲ロード | 曲を切り替える | 「load_song songs/techno1.strudel A」 |
| クロスフェード | 曲を跨ぐ | 「xfade B 4」 |
| トラックミュート | 一部を消す | 「mute A kick」 |
| BPM 変更 | テンポ | 「set_bpm 80」 |
| フェーダー | A/B 音量 | 「mixer_fader A 0.6」 |
| EQ（Lo/Mid/Hi） | 「暗く」「ファンキー」 | 「mixer_eq lo +6」 |
| フィルター | 「揺らぎ」等 | 「mixer_filter lo 1200」 |

---

## 実装

### 変更ファイル

- Mod: `src/api.rs`
  - `/mixer/fader` `POST`
  - `/mixer/eq` `POST`
  - `/mixer/filter` `POST`
  - `/deck/mute` `POST`
- Mod: `src/mcp.rs`
  - `strudel_mixer_fader(deck, level)` / `strudel_mixer_eq(band, db)` / `strudel_mixer_filter(band, hz)` / `strudel_deck_mute(deck, track, muted)` / `strudel_deck_unmute(deck, track)`
- Mod: `src/engine.rs`
  - `Command::MixerFader { deck, level }`
  - `Command::MixerEq { band, db }`
  - `Command::MixerFilter { band, hz }`
  - `Command::SetTrackMute` の拡張（track=None はデッキ全体）
- Mod: `src/mixer.rs`
  - 各操作が **バー量子化適用** であることを明確化
- Add: `src/ctl.sh` / `src/ctl.py`（任意）— Hermes からではなくターミナルからも打てる
- Add: `VIDEO.md` / `REEL.md`（展示回し方の短文）

---

## 「デモ用プリセット案」

Hermes の skill または固定プロンプトで、これらのプリセット操作を提供：

```bash
mixer_eq lo +6          # 暗く
mixer_eq hi -4          # こもる
mixer_eq mid +5         # ファンキー
mixer_filter lo 900     # ローカットする
xfade B 4               # 曲切替
mute A kick             # kick 消す
set_bpm 95              # 少し落ち着く
```

---

## 検証・品質

- API e2e: `curl POST /mixer/eq` → `GET /status` で反映値を確認
- MCP bridge: `strudel_mixer_eq` JSON-RPC call → 音が変化する
- 展示前提: 本体プロセス落ちても MCP bridge は 503 を出し、Hermes が「本体を起動して」旨を案内する文言
    
---

## 関係タスク

- Task 20 から hermes_push.sh を取り込みつつ MCP ツールを拡張する形
- Task 22 展示 README 内に「Hermes チャットで触るモード」のセクション新設
- Task 25（任意）Viz: TUI 下部に mixer_eq/xfade 状態を表示するなら、通知 SSE の data に eq/fader 値を追加するのみで済む

---

## 参考イメージ

```
Hermes chat
  → mcp_servers.strudel.*
  → rmcp stdio bridge
  → HTTP 127.0.0.1:17878
  → Scheduler / Mixer / Deck
  → 展示スピーカー出力
```
