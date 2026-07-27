# Task 23: Synths / Effects 拡張（ティア B + orbit / duck / wt / compressor）

> 正本: [../2026-07-27_020000-strudel-rs-final.md](../2026-07-27_020000-strudel-rs-final.md)  
> 状態: **完了**（2026-07-27）

正本プラン上は Task 23 の詳細ステップ節が無かったため、実装結果の要約をここに残す。

## Objective

ティア B シンセ/FX に加え、ユーザー要求で **Deck 内 orbit・duck・wavetable・マスター compressor** まで実装。

## 実装内容

| 領域 | 内容 | 主なファイル |
| --- | --- | --- |
| DSP 共通 | RBJ biquad、DuckState、Compressor | src/dsp.rs |
| パーサ | vib/fm/hpf/bpf/lpq/lpenv/penv/bank/clip/cut/orbit/duck/compressor 等 | src/code.rs |
| シンセ | FM 1op、vib、noise mix、wt_*、biquad チェーン | src/synth.rs |
| サンプル | bank 接頭辞、SampleVoice に ADSR/フィルタ | src/sample.rs / src/sound.rs |
| Deck | orbit×4 合算、duck リトリガ、cut steal | src/deck.rs |
| Mixer | マスター compressor（last-write from patterns） | src/mixer.rs / src/engine.rs |

## API 要点

- orbit は **デッキ単位 1..4**（A/B 非共有）
- duck: トリガ時に対象 orbit を depth まで下げ、attack 秒で 1.0 へ線形回復
- wt_sine / wt_bright / wt_organ（手続き生成 1 周期）
- delay/room は Task 24 で実装済み → [task-24](./2026-07-27-task-24-orbit-delay-room.md)

## 完了条件（実績）

- [x] orbit / duckorbit / duckattack / duckdepth
- [x] vib / fm / fmh
- [x] hpf / bpf / lpq + biquad LPF
- [x] lpenv / penv 簡易
- [x] bank / clip / cut
- [x] wt_*
- [x] Mixer compressor
- [x] cargo test 緑

## テスト

cargo test（unit: code / dsp / synth / deck / mixer / sound）
