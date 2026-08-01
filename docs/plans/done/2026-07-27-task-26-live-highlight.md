# Task 26: ミニ記法ライブハイライト

> 正本: [../2026-07-27_020000-strudel-rs-final.md](../2026-07-27_020000-strudel-rs-final.md)

状態: **完了**（正本 Progress の [x] から切り出し）

---

### Task 26（任意）: ミニ記法ライブハイライト TUI

**Objective:** Strudel の Mini Notation Highlighting 相当を端末で提供する。曲ソース全文を表示し、Transport 位置に応じて今アクティブなミニ記法 atom を ANSI 反転で強調する。

**前提:** Task 6（events）、Task 12（Engine / play）。

**Files:**
- Create: `src/highlight.rs` — HighlightModel / active_spans / render_ansi
- Modify: `src/mini.rs` — `Span`、トークン・Atom・Event にソース位置
- Modify: `src/code.rs` — `mini_src` / `mini_base`
- Modify: `src/song.rs` — `Song.source`、絶対 mini_base
- Modify: `src/engine.rs` — `playhead: Arc<AtomicU64>`（UI 用）
- Modify: `src/main.rs` — `play` 既定ハイライト / `--headless`（互換で `--highlight` / `--hl` も可）
- Modify: `Cargo.toml` — `crossterm`

**設計:**
- アクティブ判定は **UI スレッドが AST を再評価**（audio は global_sample の Atomic store のみ）
- MVP 対象は `note`/`s`/`sound` の第一引数ミニ記法 atom のみ（メソッド引数は非対応）
- 依存は crossterm（ratatui は使わない）
- **既定はハイライト TUI**。旧メタログ表示は `--headless`

**起動:**
```bash
strudel-rs play songs/smoke.strudel
strudel-rs play --seconds 15
strudel-rs play songs/smoke.strudel --headless
```

**検証:**
- 単体: span がソース部分文字列と一致、bar_pos で active 集合が変化
- 手動: kick `bd*4` / hat `hh*8` / snare `sd` / bass ノートが点灯

**非目標:** エディタ埋め込み、`.bank`/`.color` パターニング、Punchcard（Task 25）。

---

---
