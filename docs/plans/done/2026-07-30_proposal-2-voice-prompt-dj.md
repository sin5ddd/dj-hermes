# Proposal 2 — 音声入力でプロンプトDJ（Proposal 1 の入力口を音声にする）

> Proposal 1 をさらに華やかにする展示演出。
> 来場者の**音声発話**を Hermes Whisper → LLM → strudel-rs ops へ流し、
> 「話しかけるだけでDJミックスが変わる」体験を作る。

---

## Goal

TUI/REPL 上の **Hermes プロンプト入力**をキーボードから **マイク入力** に置き換える。
内部ルートは Proposal 1 と同じで、**Hermes HTTP 経由で strudel-rs ops を生成**する流れは共通。

---

## Exhibit flow（合体）

```
来場者「もっと暗くして」
  → Hermes（Whisper STT）
    → Hermes（system prompt / DJ copilot で strudel ops 生成）
      → Hermes HTTP → strudel-rs 内部 client
        → Command pending（バー量子化）
          → 音 / TUI playhead / mixer 値 が変化
            → Hermes TTS（応答）または TUI 文言
```

---

## ブース演出例

| 発話 | 見える変化 | 返答 |
| --- | --- | --- |
| 「暗くして」 | bass + low EQ +6、音がくぐもる | 「暗くしました」 |
| 「明るくして」 | hi +6、hihat が前面に | 「明るくしました」 |
| 「ambient に」 | crossfade B 8 秒 | 「ambient に切替えます」 |
| 「BPM 90」 | BPM バー変わりから 90 | 「BPM 90 にしました」 |

---

## 実装方針

### Hermes 側（最小実装）

Hermes 標準機能だけで成立する想定：

1. **Whisper を有効化** — Hermes の STT provider を使う。押下方式（ボタン長押し）で開始。
2. **system prompt / memory** に DJ copilot 役割を定義：
   - ユーザー発話を `strudel_*` ops JSON に変換する
   - 日本語で応答文言を返す
3. **Hermes chat は常時 strudel-rs HTTP に接続** — または strudel-rs が Hermes にポーリングする

**実装上の分岐:**

#### A. strudel-rs が Hermes をポーリングする（推奨）

- `src/hermes_poll.rs`
-  Hermes の SSE /events 相当を購読、`🎤 Transcript` イベントを検知
- 発話が来たら Hermes が生成する ops の pending を待つ
- Hermes 側は transcript → ops JSON を生成する役割

#### B. Hermes が strudel-rs HTTP を叩く（双方向）

- Hermes が MCP client として strudel-rs を叩く
- `strudel_set_bpm` 等を Hermes が直接 tool call
- strudel-rs がサーバ側なので MCP server は既存のままで成立

**展示安定性で B の方が良い。** 理由：
- strudel-rs 本体が API server なので「Hermes が strudel を叩く」形
- Hermes の MCP client ランタイムを使うだけ
- Hermes 側の config.yaml だけで接続でき、コード変更が最小

---

## 変更ファイル

- Add: `src/hermes_bridge.rs`（任意）
  - Hermes から strudel-rs に対する MCP/HTTP 操作の仲介
  - 展示当日の `hermes_bridge` プロセスを 1 つ上げておく
- Add: `VAD.md`（展示用マイク/VAD 設定メモ）
- Add: `VOICE.md`（展示会話シナリオ + fallback）
- Mod: `PROMPTS.md`（DJ copilot の system prompt + 対応表）

---

## Hermes 側 config / prompt（展示当日用）

`mcp_servers.strudel` は既存のままとし、**用途は 2 種類**:

| Actor | 方向 | 用途 |
| --- | --- | --- |
| 人間（REPL） | strudel-rs → Hermes | キーボードでプロンプト入力 |
| Hermes（STT応答） | Hermes → strudel-rs | 音声での自動操作 |

両方を同じ API `17878` に向ける。

---

## 展示用の insurance（保険設計）

| 障害 | フォールバック |
| --- | --- |
| Hermes クラッシュ | TUI から手動 REPL コマンドが打てる状態を維持 |
| Whisper 誤認識 | 「何て言いました？」確認文言 → strudel に送らない |
| ネットワーク断 | キーボード入力のみで継続 |
| 騒音で聞き取れない | 担当者が手動で ops 投入 |

---

## 検証

1. Hermes が起動済み + strudel-rs play 起動済み
2. Hermes で `DJ: 暗くして` → Hermes が `mixer_eq lo +6` を tool call
3. strudel-rs の TUI で mixer 値がアニメーション
4. 実際の音が変化することを展示スピーカーで確認
5. 10 パターン × 3 回 = 30 回の事前 dry-run で誤操作率を記録
6. 誤操作率 10% 超なら system prompt で destructive ops は確認文言必須に変更

---

## 依存

- Proposal 1（**TUI から Hermes プロンプトで作曲/MIX**）**必須**
- Hermes 側 MCP client runtime が利用可能
- Whisper STT が Hermes config で有効

---
