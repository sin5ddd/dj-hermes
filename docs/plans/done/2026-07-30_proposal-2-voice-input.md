# Proposal 2 — 音声入力で DJ 操作（Hermes Whisper → 意図解析 → MCP → strudel-rs）

> 展示時の「華」になる機能。来場者が話しかけるとミキサーが返事する。
> 技術的には Hermes 標準機能のWhisper + LLM + strudel MCP で追加ライブラリ不要。

---

## Goal

Hermes が音声をテキスト化し、**「もっと暗くして」** のような自然な発話を
MCP ツール列に変換して strudel-rs へ送る。ブース体験は「AIに話しかけるとDJミックスが変わる」。

---

## 前提

- Proposal 1 の MCP ツール拡張 **完了前提**
- Hermes 標準: Whisper（STT） `text_to_speech` provider などは使わず STT のみ
- 展示では **ノイズ多め + 距離1m以内で小声でも反応** させるため VAD 必須

---

## 入力フロー

```
[来場者発話]
    │
    ▼
[Hermes Whisper / 録音]
    │
    ▼
[Transcript]
    │
    ▼
[Hermes LLM 意図抽出]  ← 短い system prompt + 工具定義
    │
    ▼
[MCP tools/strudel_* 列]
    │
    ▼
[strudel-rs] → 展示スピーカー出力
    │
    ▼
[Hermes TTS 応答] 「暗くしました」など最小
```

---

## Hermes 側の промпト案（展示専用 prompt / memory）

```yaml
role: dj-host
 When user speech is transcribed, decide strudel_* tool(s).
 Short Japanese reply required ("承知" / "暗くしました").
```

VAD は Hermes が **中ボタン長押し / または無音トリガ** を取る前提で良い。
展示では「押さなくても小さい声で検知」より **「押して話す」** の方がブースが騒がしくても安全。

---

## 展示演出

1. スピーカーが流れる中、来場者に **マイクを渡す**
2. 「ライブ暗くして」等と言う
3. Hermes TUI で `🎤 Listening...` 表示し、数秒後に応答
4. 音が変わる

もし失敗した場合の保険として、**バックアップで用意するプレースホルダ発話**も提示：
- 「暗くして」→ mixer_eq lo +6
- 「ファンキーに」→ mixer_eq mid +6
- 「揺らぎ」→ mixer_filter lo 800 + slight vibrato preset（track code change）
- 「BPM落ちて」→ set_bpm 95
- 「曲切替」→ xfade B 4

---

## 未解決点・制約

- Hermes Whisper は日本語 16kHz 前提で悪くないが、HM受話表示が小さい場合は風音対策で **指向性マイク** を推奨
- 発話は最大 7 秒に切る。それ以上は LLM に余計な解釈が入るため誤操作率が上がる
- 誤操作寄りなら、**What would you like to do?** の確認文言を Hermes に入れる

---

## 依存

- Proposal 1 の MCP ツール拡張 **必須**
- Hermes 標準 STT を有効化するだけ。追加 model 等は不要

---

## ファイル

- Add: `docs/plans/done/2026-07-30_proposal-2-voice-input.md`
- Optional: `VIDEO.md` 更新で「音声操作デモ動画」のリンク文

---

## 検証

- 展示環境で **風・人声混じり** 状態で実際に 20 回試す
- LLM の誤操作率が 10% 超なら system prompt へ「不明時は strudel_hush しないで」制約追加

---

## risks

| リスク | 影響 | 対策 |
| --- | --- | --- |
| 誤操作 | 会場で意図せぬ BPM 変更等 | system prompt で destructive ops は事前確認必須 |
| WHISPER 低音 | 展示で通らない | 展示用 external mic + VAD 調整 |
| Hermes 側のシステムプロンプト肥大 | プロンプト長制限 | operations セットを固定にする |
