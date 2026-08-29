---
name: drum-and-bass
description: >-
  Use when writing a Drum and Bass loop in strudel-rs: 174 BPM, break in
  front of the sub, split Reese (square sub + saw mid). Never use sample db.
---

# Drum and Bass (strudel-rs)

## When to use

- The request is **DnB / jungle / breakbeat at ~170–180 BPM**.
- You need a **half-time break** (not four-on-the-floor) plus a **Reese**.
- You are **not** writing house (`bd*4` + snare on 2/4) or a single-oscillator “Reese”.

## Pattern

```
// @title skill-drum-and-bass
// @genre drum and bass
setcpm(174/4)
// drums
$: s("[bd <~ sd> ~ sd ~ <bd ~> <bd sd> <bd ~>, hh*4, [~@5 oh ~@2]]*2").gain(0.7).lpf(4000)
// sub
$: note("0 3 0 <0 -1>")
  .scale("C2:minor")
  .s("square").lpf(120).gain(0.42)
  .attack(0.01).decay(0.5).release(0.4)
// mid
$: note("0 3 0 <0 -1>")
  .scale("C2:minor")
  .s("sawtooth").lpf(1000).gain(0.3)
  .attack(0.01).decay(0.4).release(0.3)
```

Keep drums on **one** `$:` (comma layers). Do not use `stack()`.

| Piece | Role |
| --- | --- |
| `setcpm(174/4)` | **174 BPM** (`cycles * 4` in `src/song.rs`). This is the skill tempo. |
| 8-slot break + mini `*2` | One cycle of a break, tiled twice so it fills the bar as 16ths |
| drums `.gain(0.7)` | **Above** the sub (0.42) so kick/snare sit in front |
| square + `lpf(120)` | Sub only — no mids |
| saw + `lpf(1000)` | Mid Reese (keep **800–1200**) |
| `bd` / `sd` / `hh` / `oh` | Bundled samples only (`samples/`) |

Playable copy: `songs/skill-drum-and-bass.strudel`. Same mix idea as `songs/dnb16.strudel`, without the silent `db` atom and without a starved drum gain.

## Why it sounds that way

DnB reads as **fast grid + slow bass**. `setcpm(174/4)` is 174 quarter-notes per minute. The drum string is one cycle of a break (kick/snare placements, not `bd*4`). Mini `*2` (`mini.rs` `Node::Fast`) copies that span twice across the bar, so snares land twice per bar (amen-ish half-time against 174).

**Do not write `.fast(2)` and expect the same thing.** The method (`.fast` in `code.rs`) only multiplies `PatternCode.speed`. `deck.rs` then divides each event’s start by that speed and **does not re-query the next cycle**. A full-bar break with `.fast(2)` is squeezed into the first half of the bar; the second half is empty. Mini `*2` is the operator that actually tiles.

**Kick and snare must be louder than the sub.** Unknown sounds are dropped (`deck.rs` `resolve_sound` miss → no voice), so they cannot “cut through” a loud square. If the break is quieter than the sub (the old `dnb16` trap was drums `.gain(0.25)` under sub `0.7`), the kit disappears.

**Never write `db`.** Bundled keys are `bd`, `sd`, `hh`, `oh` (`samples/`). `db` is not a sample and not a waveform. An atom that fails resolve is silence. Use `<bd ~>` (or `~`) instead of `<db ~>`.

**Reese is two tracks, not one oscillator.** A lone `square` + low LPF is a sub. The mid growl is a **saw** with `lpf` **800–1200**. Same degrees, different band. Do not smear sub and mid into one `square` or one `saw`.

Both decks share one `Transport`. This file is 174 BPM. Do **not** pair it with a 126 techno file — the second `setcpm` is discarded.

## Try it in this app

```bash
# From the repo root (needs samples/bd, sd, hh, oh)
strudel-rs play songs/skill-drum-and-bass.strudel --seconds 12
strudel-rs play songs/skill-drum-and-bass.strudel --headless --seconds 8
```

No audio device: `cargo test --test e2e skill_drum_and_bass -- --nocapture` renders through `Engine::process`.

Live TUI: `/a load skill-drum-and-bass` (or the `songs/` path). HTTP: `POST /song/load` with that file, or send the same `setcpm` + `$:` text as apply-song content.

## Variations (still this syntax)

| Goal | Change |
| --- | --- |
| Busier hats | `hh*4` → `hh*8` inside the same `*2` group |
| Darker kit | `.lpf(2500)` on the drum `$:` |
| New root, same Reese | `.scale("A2:minor")` on sub **and** mid |
| Quiet stab | add `note("~ 4 ~ <7 9>").scale("C3:minor").s("square").lpf(2000).gain(0.15)` |

## Rules (do not skip)

1. `setcpm(174/4)`
2. Drum `.gain` **greater than** sub `.gain`
3. Never `db` (silent)
4. Reese = **square sub + saw mid** (not a single oscillator)
5. Tile the break with mini `*2`, not method `.fast(2)`

## Do not

- `bd*4` at 174 — that is techno/house, not a break.
- One `$:` `square` named “Reese”.
- Bank-less `cp` or `db`.
- `.fast(2)` when you want the break to fill the bar.
- Pair this file with a song at another `setcpm` (shared clock; the other tempo is discarded).
- `stack()` / `.cpm(174)`.
