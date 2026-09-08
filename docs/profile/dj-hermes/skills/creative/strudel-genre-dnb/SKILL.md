---
name: strudel-genre-dnb
description: >-
  Use when writing a Drum and Bass loop in dj-hermes: 174 BPM, break in
  front of the sub, split Reese (square sub + saw mid as bass + bass-mid).
  Never use sample db. 8 $: tracks, play solo at 174.
version: 5.1.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [dj-hermes, music, genre, dnb, reese]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-dnb-reese-mid-stab
---

# Drum and Bass (dj-hermes)

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
  .s("ld:ss").gain(0.14).cut(1)
// hook
$: note("~ 4 ~ <7 4 4 7>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("plk:dt").gain(0.16).cut(1)
// arp
$: s("<~ perc:st ~ perc:tm>").gain(0.16)
// chords
$: note("[0,4] ~ [0,4] ~").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("ep:mt").gain(0.16)
// pad
$: note("0").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("pf:ff").gain(0.12).room(0.22).orbit(2)
```

Keep the break `*2` (not `.fast(2)`). Drums `.gain(0.7)` stays **above** the sub (0.42). Square at **C2** is `// bass`; saw mid is `// bass-mid`. Never `db`.

`songs/dnb/01.strudel` matches this fence (break and Reese split). New apply picks non-Reese `.s()` from the palette below.

## Timbre palette (pick per new apply)

The Pattern fence is one example of grid, degrees, and slots. Keep the **Reese split** (`square` sub + `sawtooth` mid, same degrees). For a **new** apply, pick **one** sound per other slot. Do not copy the fence `.s()` every time. Do not reuse the same `.s()` on two pitched tracks except this Reese split. Slug meanings: strudel-pcm-catalog INDEX. Long one-shots (`ld:` / `dr:` / `pf:` / `ps:`, `plk:fp` / `plk:sp`) need `.cut(1)` or `s("<x ~ ~ ~>")`.

| Slot | Keep | Pick one | Forbidden |
| --- | --- | --- | --- |
| drums | break `*2` (not `bd*4`) | `bd:dn` / `bd:jg`, `sd:dn` / `sd:jg`, `hh:dn`, `oh:dn` | `db`, house `cp`, `bd*4` |
| bass | `square`+`lpf(120)` at `C2:` | (identity) | replacing the square with a sample |
| bass-mid | `sawtooth`+`lpf(1000)` at `C2:` | (identity; sampled mid is the other DnB skill) | mixing `bs:rm` into this saw recipe |
| lead | | `ld:ds`, `plk:nn`, `ld:dp` | `ld:ss` on every song, `plk:mx` |
| hook | | `plk:dt`, `plk:nn` | `plk:s5` (other DnB skill), Rhodes |
| arp | sparse perc | `perc:st`, `perc:tm` | a third mid oscillator |
| chords | `[0,4]` | `plk:s5`, `ep:mt` | `triangle`, maj7 city-pop |
| pad | | `pf:fo`, `dr:rd` with `<>`, `pf:ff`+`note("0")` | music box, Rhodes, `ps:mx` |

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
| chords `ep:mt` / pad `pf:ff` | Hollow-ish blocks so the Reese keeps the mid. Pad is `note("0")` on orbit 2 |

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
dj-hermes play songs/dnb/01.strudel --seconds 12
dj-hermes play songs/dnb/01.strudel --headless --seconds 8
```

Load `songs/dnb/01.strudel`, or `dj_hermes_apply_song` the Pattern fence. Play **solo** at 174.

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
- [ ] Non-Reese `.s()` from the Timbre palette (`ld:ss` is not the only lead)
- [ ] Play **solo** at 174. `songs/dnb/01.strudel` matches this fence’s break and Reese split

## Do not

- `bd*4` at 174 — that is techno/house, not a break.
- One `$:` `square` named “Reese”.
- Bank-less `cp` or `db`.
- `.fast(2)` when you want the break to fill the bar.
- DJ-pair this file with 124 house or 126 techno (shared clock; the other tempo is discarded).
- Copy the fence `.s()` on every new apply for lead / hook / pad.
- 長い PCM（`ld:` / `dr:` / `pf:` / `ps:`、約 8–17 秒）を毎小節撃たない。
- Stack a third sub (`bs:su` / `bs:hf` / `bs:dk`) on the square+saw split.
- Ship a 3-track loop for a new apply.
- `stack()` / `.cpm(174)` / `.lfo()` / `kit:bd`.
