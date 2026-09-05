---
name: strudel-genre-dnb
description: >-
  Use when writing a Drum and Bass loop in strudel-rs: 174 BPM, break in
  front of the sub, split Reese (square sub + saw mid as bass + bass-mid).
  Never use sample db. 8 $: tracks, play solo at 174.
version: 5.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, dnb, reese]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-dnb-reese-mid-stab
---

# Drum and Bass (strudel-rs)

## When to use

- The request is **DnB / jungle / breakbeat at ~170–180 BPM**.
- You need a **half-time break** (not four-on-the-floor) plus a **Reese**.
- New apply is **8 `$:` tracks**: drums + bass + bass-mid + lead + hook + arp + chords + pad. Play **solo** at 174. Do not DJ-pair 124.
- You are **not** writing house (`bd*4` + snare on 2/4) or a single-oscillator “Reese”.
- Sampled mid Reese (`bs:rm`) + hollow-fifth stab: [strudel-genre-dnb-reese-mid-stab](../strudel-genre-dnb-reese-mid-stab/SKILL.md) (still 174; do not pair with 124 house).

## Pattern

```
// @title skill-drum-and-bass
// @genre drum and bass
setcpm(174/4)
// drums
$: s("[bd <~ sd> ~ sd ~ <bd ~> <bd sd> <bd ~>, hh*4, [~@5 oh ~@2]]*2").gain(0.7).lpf(4000)
// bass
$: note("0 3 0 <0 -1 0 3>").scale("C2:minor")
  .s("square").lpf(120).gain(0.42)
  .attack(0.01).decay(0.5).release(0.4)
// bass-mid
$: note("0 3 0 <0 -1 0 3>").scale("C2:minor")
  .s("sawtooth").lpf(1000).gain(0.32)
  .attack(0.01).decay(0.4).release(0.3)
// lead
$: note("~ 7 ~ <9 7 4 11>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("square").lpf(2600).gain(0.14)
// hook
$: note("~ 4 ~ <7 4 4 7>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("plk:dt").gain(0.16).cut(1)
// arp
$: s("<~ perc:st ~ perc:tm>").gain(0.16)
// chords
$: note("[0,4] ~ [0,4] ~").scale("<C3:minor C3:minor G3:phrygian C3:minor>")
  .s("triangle").lpf(1500).gain(0.16)
// pad
$: note("[0,4]").scale("<C3:minor C3:minor G3:phrygian C3:minor>")
  .s("sine").fm(1.2).fmh(1).fmdec(0.8).fmsus(0.4)
  .room(0.22).orbit(2).gain(0.12)
```

Keep the break `*2` (not `.fast(2)`). Drums `.gain(0.7)` stays **above** the sub (0.42). Square at **C2** is `// bass`; saw mid is `// bass-mid`. Never `db`.

`songs/dnb-01.strudel` is still a thin demo (break + two Reese layers). Apply the fence above, not the on-disk file.

Keep drums on **one** `$:` (comma layers). Do not use `stack()`.

| Piece | Role |
| --- | --- |
| `setcpm(174/4)` | **174 BPM** (`cycles * 4` in `src/song.rs`). This is the skill tempo. Play **solo**. |
| 8-slot break + mini `*2` | One cycle of a break, tiled twice so it fills the bar as 16ths |
| drums `.gain(0.7)` | **Above** the sub (0.42) so kick/snare sit in front |
| square + `lpf(120)` + `C2:minor` | `// bass` — sub only, no mids |
| saw + `lpf(1000)` + `C2:minor` | `// bass-mid` — mid Reese (keep **800–1200**) |
| `bd` / `sd` / `hh` / `oh` | Bundled samples only (`samples/`). Never `db` |
| lead / hook 4-bar scale | Melody call-and-response. Hook is `plk:dt`, not `plk:s5` (that is the other DnB skill) |
| chords / pad `[0,4]` | Two-note blocks so the Reese keeps the mid. Pad on orbit 2 |

## Why it sounds that way

DnB reads as **fast grid + slow bass**. `setcpm(174/4)` is 174 quarter-notes per minute. The drum string is one cycle of a break (kick/snare placements, not `bd*4`). Mini `*2` (`mini.rs` `Node::Fast`) copies that span twice across the bar, so snares land twice per bar (amen-ish half-time against 174).

**Do not write `.fast(2)` and expect the same thing.** The method (`.fast` in `code.rs`) only multiplies `PatternCode.speed`. `deck.rs` then divides each event’s start by that speed and **does not re-query the next cycle**. A full-bar break with `.fast(2)` is squeezed into the first half of the bar; the second half is empty. Mini `*2` is the operator that actually tiles.

**Kick and snare must be louder than the sub.** Unknown sounds are dropped (`deck.rs` `resolve_sound` miss → no voice), so they cannot “cut through” a loud square. If the break is quieter than the sub (the old trap was drums `.gain(0.25)` under sub `0.7`), the kit disappears.

**Never write `db`.** Bundled keys are `bd`, `sd`, `hh`, `oh` (`samples/`). `db` is not a sample and not a waveform. An atom that fails resolve is silence. Use `<bd ~>` (or `~`) instead of `<db ~>`.

**Reese is two tracks, not one oscillator.** A lone `square` + low LPF is a sub. The mid growl is a **saw** with `lpf` **800–1200**. Same degrees, different band. Comment names are `// bass` and `// bass-mid`. Do not smear sub and mid into one `square` or one `saw`. Do not add a third sub (`bs:su` / `bs:hf` / `bs:dk`).

Lead and hook share a 4-bar `.scale("<C4:minor C4:minor G4:phrygian C4:minor>")` and rest on different slots. Bass / bass-mid keep scalar `C2:minor` so the Reese does not jump octaves with the melody progression. Arp is sparse perc, not another mid oscillator.

Both decks share one `Transport`. This file is 174 BPM. Do **not** pair it with a 126 techno file or 124 house — the second `setcpm` is discarded. Play solo. Do not DJ-pair 124.

## Try it in this app

```bash
# From the repo root (needs samples/bd, sd, hh, oh)
strudel-rs play songs/dnb-01.strudel --seconds 12
strudel-rs play songs/dnb-01.strudel --headless --seconds 8
```

The on-disk song is still the thin demo. For a full bed, `strudel_apply_song` the Pattern fence. Play **solo** at 174.

No audio device: `cargo test --test e2e dnb_01 -- --nocapture` renders through `Engine::process`.

Live TUI: `/a load dnb-01` (or the `songs/` path). HTTP: `POST /song/load` with that file, or send the same `setcpm` + `$:` text as apply-song content.

## Variations (still this syntax)

| Goal | Change |
| --- | --- |
| Busier hats | `hh*4` → `hh*8` inside the same `*2` group |
| Darker kit | `.lpf(2500)` on the drum `$:` |
| New root, same Reese | `.scale("A2:minor")` on bass **and** bass-mid |
| Sampled mid + stab | use [strudel-genre-dnb-reese-mid-stab](../strudel-genre-dnb-reese-mid-stab/SKILL.md) — do not mix `bs:rm` into this saw recipe |

## Rules (do not skip)

1. `setcpm(174/4)`. Play **solo**. Do not DJ-pair 124.
2. Drum `.gain` **greater than** sub `.gain`
3. Never `db` (silent)
4. Reese = **square sub + saw mid** (`// bass` + `// bass-mid`, not a single oscillator)
5. Tile the break with mini `*2`, not method `.fast(2)`
6. **8 tracks**. No 9th.

## Checklist

- [ ] `setcpm(174/4)` + **8 `$:`** (drums, bass, bass-mid, lead, hook, arp, chords, pad)
- [ ] Break `*2`, drums gain **above** sub. Never `db`. Never `.fast(2)`
- [ ] Square `C2` + `lpf(120)` as bass; saw `C2` + `lpf(1000)` as bass-mid
- [ ] 4-bar phrase on lead / hook / chords / pad. Melody call-and-response
- [ ] Play **solo** at 174. `songs/dnb-01.strudel` is still a thin demo; this fence is the new target

## Do not

- `bd*4` at 174 — that is techno/house, not a break.
- One `$:` `square` named “Reese”.
- Bank-less `cp` or `db`.
- `.fast(2)` when you want the break to fill the bar.
- DJ-pair this file with 124 house or 126 techno (shared clock; the other tempo is discarded).
- Write `in_bank=no` PCM (`ld:ac`, `pf:al`, `dr:*`, `ps:*`). Allowed long PCM: `ld:ss`, `pf:ff`.
- Stack a third sub (`bs:su` / `bs:hf` / `bs:dk`) on the square+saw split.
- Ship the 3-track on-disk demo as a new apply.
- `stack()` / `.cpm(174)` / `.lfo()` / `kit:bd`.
