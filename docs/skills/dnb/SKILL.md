---
name: dnb
description: >-
  Use when writing a Drum and Bass loop in strudel-rs: 174 BPM, break in
  front of the sub, split Reese (square sub + saw mid). Never use sample db.
---

# Drum and Bass (strudel-rs)

## When to use

- The request is **DnB / jungle / breakbeat at ~170–180 BPM**.
- You need a **half-time break** (kick/snare not four-on-the-floor) plus a **Reese**.
- You are **not** writing house (`bd*4`) or a single-oscillator “Reese”.

## Pattern

```
// @title skill-dnb
// @genre drum and bass
setcpm(174/4)
// drums
$: s("bd <~ sd> ~ sd ~ <bd ~> <bd sd> <bd ~>, hh*4, [~@5 oh ~@2]").fast(2).gain(0.6).lpf(4000)
// sub
$: note("0 3 0 <0 -1>").scale("C2:minor")
  .s("square").lpf(120).gain(0.45)
  .attack(0.01).decay(0.5).release(0.4)
// mid
$: note("0 3 0 <0 -1>").scale("C2:minor")
  .s("sawtooth").lpf(1000).gain(0.32)
  .attack(0.01).decay(0.4).release(0.3)
```

| Piece | Role |
| --- | --- |
| `setcpm(174/4)` | **174 BPM** (cycles × 4). This is the skill tempo. |
| break + `.fast(2)` | Pattern is half-time; `*2` plays it twice per bar |
| drums `.gain(0.6)` | **Above** the sub (0.45) so kick/snare sit in front |
| square + `lpf(120)` | Sub only — no mids |
| saw + `lpf(1000)` | Mid Reese (keep 800–1200) |
| `bd` / `sd` / `hh` / `oh` | Bundled samples only |

Same mix rules as the corrected `songs/dnb16.strudel` (that file also has a quiet stab).

## Why it sounds that way

DnB reads as **fast grid + slow bass**. `setcpm(174/4)` is 174 quarter-notes per minute. The drum string is one cycle of a break; `.fast(2)` doubles it so snares land twice per bar (amen-ish half-time against 174).

**Kick and snare must be louder than the sub.** `songs/dnb16.strudel` previously used drums `.gain(0.25)` against sub `0.7`. Unknown sounds are dropped (`deck.rs` `resolve_sound` miss → no voice), so they cannot “cut through” a loud square. If the break is quieter than the sub, it disappears.

**Never write `db`.** Bundled keys are `bd`, `sd`, `hh`, `oh` (`samples/`). `db` is not a sample and not a waveform. The old dnb16 ending `<db ~>` scheduled an atom that never spawned — that slot was silence. Use `<bd ~>` (or `~`) instead.

**Reese is two tracks, not one square.** A lone `square` + low LPF is a sub. The mid growl is a **saw** with `lpf` **800–1200**. Same degrees, different band. Do not try to make one oscillator do both.

## Try it in this app

```bash
strudel-rs play songs/skill-dnb.strudel --seconds 12
strudel-rs play songs/dnb16.strudel --seconds 12
```

No device: `cargo test --test e2e skill_dnb -- --nocapture`.

## Rules (do not skip)

1. `setcpm(174/4)`
2. Drum `.gain` **greater than** sub `.gain`
3. Never `db` (silent)
4. Reese = **square sub + saw mid** (not a single square)

## Do not

- `bd*4` at 174 — that is techno/house, not a break.
- One `$:` `square` named “Reese”.
- Bank-less `cp` or `db`.
- `stack()` / `.cpm(174)`.
