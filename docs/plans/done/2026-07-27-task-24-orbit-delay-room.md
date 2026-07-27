# Task 24: orbit 共有 delay / room + Mixer チャンネル EQ 連動

> 正本: [../2026-07-27_020000-strudel-rs-final.md](../2026-07-27_020000-strudel-rs-final.md)  
> 状態: **完了**（2026-07-27）

## Objective

1. Task 23 の **Deck 内 orbit バス**に、global FX として簡易 **delay** と **room（リバーブ）** を接続する。
2. DJ モード TUI の Hi/Mid/Lo EQ（モック）を **Mixer チャンネル EQ** と連動させる。

## 実装内容

| 領域 | 内容 | 主なファイル |
| --- | --- | --- |
| DSP | Peaking/LowShelf/HighShelf、DelayLine、SimpleReverb、OrbitFx | `src/dsp.rs` |
| パーサ | `.delay` / `.delaytime` / `.delayfeedback` / `.room` / `.roomsize`（コロン記法可） | `src/code.rs` |
| Deck | orbit×4 の OrbitFx、spawn 時 last-write、process で dry+wet | `src/deck.rs` |
| Mixer | per-deck 3-band EQ（Hi shelf / Mid peak / Lo shelf、±12 dB） | `src/mixer.rs` |
| Engine | `Command::SetDeckEq`（即時） | `src/engine.rs` |
| live UI | ドラッグ → SetDeckEq、engine 同期 | `src/live_ui.rs` |
| デモ | ambient1 に room/delay を追加 | `songs/ambient1.strudel` |

## API 要点

### delay / room（パターン）

- orbit は **デッキ単位 1..4**（A/B 非共有）
- 同 orbit 上は **last-write-wins**（ヒットで delay/room が正のときのみ更新）
- `delayfeedback` は DSP 側で **max 0.95** にクランプ
- dry は常に通し、delay/room は wet 加算
- 非対応: IR、`delayspeed`、`roomlp` 等

### チャンネル EQ（演奏操作）

- スライダー 0..=1、**0.5 = 0 dB**、端で ±12 dB
- band: 0=Hi (6 kHz shelf), 1=Mid (1 kHz peak), 2=Lo (200 Hz shelf)
- 適用順: channel EQ → fader → master LPF/HPF → compressor

## 完了条件（実績）

- [x] delay / delaytime / delayfeedback / room / roomsize（別名・コロン記法）
- [x] orbit 共有 delay/room（Deck 内）
- [x] feedback &lt; 1 強制
- [x] ノート後にテールが残る unit テスト
- [x] TUI Hi/Mid/Lo → Mixer 実音連動
- [x] cargo test 緑

## テスト

`cargo test`（unit: dsp / code / deck / mixer ほか）
