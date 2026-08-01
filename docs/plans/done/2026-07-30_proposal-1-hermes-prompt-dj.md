# Proposal 1 — TUI/REPL から Hermes を呼び出してプロンプト作曲 / DJ（Proposal 方向修正）

> 方向が逆だった。`strudel-rs` の TUI / REPL から Hermes へ**プロンプトを投げ**、
> **Hermes が返した code / song / mixer 操作**を反映する。
> つまり展示では **「Hermes が作曲家」** に見える形にする。

---

## Goal

来場者は **キーワードを1行入力するだけ** でコードが変わり、音が変わる。
内部処理は「strudel-rs の TUI → Hermes HTTP/MCP → 生成 → strudel-rs 反映」。

---

## Exhibit flow（来場者に見せる流れ）

```
来場者: 「暗いtechnoちょうだい」
  └─ TUI のプロンプト入力（または選択メニュー）
      └─ Hermes client call（OpenAI/ローカル LLM + ツール定義）
          └─ Hermes が返した strudel コード or mixer 操作
              └─ strudel-rs が即時反映（バー量子化）
                  └─ 音が変わる / TUI ハイライトが変化
```

---

## ユーザーが触る最小 UI（展示の設計）

- TUI モードで **プロンプト入力行** を常時表示
- あるいは **「Hermes に作曲させる」ボタン**（キーボード: `h` 押下でプロンプト入力）
- 入力例：
  - 「暗い techno」
  - 「ambient に xfade B 8」
  - 「BPM 90 / lo +6」
- Hermes 実行中は **spinner表示**、応答後は **成功/失敗文言** を1行表示

---

## 実装方針（strudel-rs 側）

### Hermes クライアント側（呼び出し元）

Hermes を `mcp_servers.strudel` の向きではなく、**strudel-rs が Hermes MCP client としてふるまう**形になる。

- `HermesClient` は stdio で Hermes の MCP session を開始する。
- strudel-rs からは tool を定義せず、**Hermes 内のツール/プロンプト** を利用する。
- MCP client → Hermes を介して、Hermes 内の LLM にプロンプト → 返答を strudel 操作にマッピングする Hermes 側ルールが必要。

**実装案（局所解）:**

```rust
// src/hermes.rs
pub struct HermesClient {
    tx: Sender<Command>,
    // Hermes MCP は client として使うため、Hermes が外部から tools を登録する必要がある
}
```

別アプローチ: **Hermes の HTTP API を利用する**
- Hermes の `/v1/chat/completions` 相当（使用モデル/プロンプト）を使う
- system prompt:
  ```
  You are a Strudel DJ assistant.
  Reply ONLY with strudel-rs JSON commands or strudel code.
  Allowed operations: code / load / xfade / set_bpm / mixer_eq / mixer_filter / mixer_fader.
  Reply format: { "ops": [ { "tool": "strudel_set_code", "args": { ...} }, ... ] }
  ```
- strudel-rs が Hermes の completions API を叩き、返ってきた JSON ops を Command に流す

どちらかというと **Hermes HTTP 経由** が実装が小さいので推奨。

---

## 変更ファイル

- Add: `src/hermes_client.rs`
  - Hermes HTTP API クライアント（`POST /v1/chat/completions` 想定）
  - system prompt 内蔵（DJ assistant 役割固定）
  - JSON ops → `Command` 列に変換
- Mod: `src/main.rs` / `src/repl.rs`
  - REPL に「プロンプト入力コマンド」を追加
    - `:prompt <text>` または
    - `:hermes "暗い techno"` または
    - REPL 中の特定キー（`h`）でプロンプト入力行に切替
- Mod: `src/engine.rs`
  - `Command::HermesPrompt { text }` を追加
  - Scheduler がこのコマンドを受け取り（または専用スレッドで）、Hermes API を叩いて ops を pending に積む

---

## Hermes の system prompt（固定プロンプト案）

```yaml
You: DJ copilot for Strudel live-coding DJ set.
User input: short Japanese prompt about music / mix / BPM / mood.

Allowed tool calls: ONLY generate JSON ops.

Ops schema example:
{"ops": [
  {"tool": "strudel_set_code", "args": {"code": "note(\"c2 eb2 g2\").s(\"sawtooth\").lpf(400).gain(0.7)", "deck": "A"}},
  {"tool": "strudel_set_bpm", "args": {"bpm": 90}},
  {"tool": "strudel_mixer_eq", "args": {"band": "lo", "db": 6}},
  {"tool": "strudel_xfade", "args": {"to": "B", "bars": 4}}
]}

Rules:
- Never execute ops — only serialize tool calls into JSON.
- If user says 暗く, do mixer_eq lo +6 and hp 1200.
- If user says ファンキー, do mixer_eq mid +5, bpm +5.
- If user says ambient, load_song songs/ambient1.strudel B.
- If user says BPM下げ, do strudel_set_bpm.
- Use minimal ops. Prefer one operation when possible.
- Track names (kick, hat, bass) map directly.
```

---

## 展示用キーワードと Hermes の対応（規定）

| 発話 | Hermes が生成する ops |
| --- | --- |
| 「暗く」「暗い」 | `mixer_eq lo +6`, `mixer_filter lo 900`, `mixer_fader A 0.4` |
| 「明るく」 | `mixer_eq hi +6`, `mixer_eq lo -3` |
| 「ファンキー」 | `mixer_eq mid +6`, `set_bpm +5` |
| 「ゆっくり」 | `set_bpm 80` |
| 「ambient に」 | `xfade B 8`（前提: deck B = ambient1） |
| 「techno に」 | `xfade A 4`（前提: deck A = techno1） |
| 「BPM 90」 | `set_bpm 90` |
| 「kick 切って」 | `mute A kick` |
| 「元に戻して」 | `mixer_eq all 0`, `mixer_fader A 0.8`, `mixer_fader B 0.8` |

---

## 検証

- REPL で `:h "暗い techno"` → Hermes API が strudel コマンド列を返す → 音が変わる
- ネットワーク断: Hermes API が落ちてる → 「Hermes 未起動」を表示し strudel-rs 単体継続
- パース失敗: Hermes の返却 JSON が未対応 schema → 無視して現行継続 + 警告1行
- フォールバック: 展示当日に Hermes が使えなければ手動 REPL も有効

---

## dependency

- Hermes 側に **HTTP API 呼び出し（OpenAI 互換）** があること
- Hermes 側が CORS/認証なしで `127.0.0.1` から叩けること
- 展示では Hermes のモデルを **事前に選択・固定**（プロンプト注入前に起動しておく）

---

## 展示インパクト

| 来場者アクション | 見える変化 |
| --- | --- |
| 「暗くして」 | TUI の mixer_eq 値がアニメーション + 音がくぐもる |
| 「ambient に」 | crossfade が進行 + TUI の playhead が追従 |
| 「BPM 90」 | BPM 値が変わる + 次の小節から速くなる |

**「AIに話しかけるとDJミックスが変わる」** は vol.6 でもほぼ確実に唯一無二。

---
