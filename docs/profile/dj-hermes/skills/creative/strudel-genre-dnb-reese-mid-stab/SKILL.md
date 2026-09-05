---
name: strudel-genre-dnb-reese-mid-stab
description: >-
  Use when writing a Drum and Bass loop in strudel-rs with the bundled
  mid Reese sample and hollow-fifth stab: 174 BPM, square C2 sub,
  bs:rm at C4:minor, plk:s5 degrees 4/7 as the hook. Not saw-Reese.
  8 $: tracks. Do not DJ-pair with 124 house.
version: 5.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, dnb, reese, stab]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-dnb
---

# DnB: square sub + bs:rm + hollow-fifth stab (strudel-rs)

## When to use

- The request is **DnB at 174 BPM** with the **bundled mid Reese sample** and a **hollow fifth stab**.
- You need a **half-time break** in front of a **split Reese**: square synth sub + `bs:rm` (not a saw).
- New apply is **8 `$:` tracks**: drums + bass + bass-mid + hook + lead + arp + chords + pad. Play **solo** at 174.
- You are **not** writing the older square+saw recipe ([strudel-genre-dnb](../strudel-genre-dnb/SKILL.md)).
- You are **not** writing 124 house (`[~ cp]*2`) or 126 techno. Shared Transport: do not pair this file with those tempos.

## Pattern

```
// @title skill-dnb-reese-mid-stab
// @genre drum and bass
setcpm(174/4)
// drums
$: s("[bd <~ sd> ~ sd ~ <bd ~> <bd sd> <bd ~>, hh*4, [~@5 oh ~@2]]*2").gain(0.7).lpf(4000)
// bass
$: note("0 3 0 <0 -1>").scale("C2:minor").s("square").lpf(120).gain(0.42)
// bass-mid
$: note("0 3 0 <0 -1>").scale("C4:minor").s("bs:rm").gain(0.36)
// hook
$: note("~ 4 ~ <7 4>").scale("C4:minor").s("plk:s5").gain(0.2).cut(1)
// lead
$: note("~ 11 7 <12 9 7 4>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("square").lpf(2800).gain(0.12)
// arp
$: note("~ 0 7 12  7 0 ~ 4").scale("<C5:minor C5:minor G5:phrygian C5:minor>")
  .s("plk:dt").gain(0.12).cut(1)
// chords
$: note("[0,4] ~ ~ [0,4]").scale("<C3:minor C3:minor G3:phrygian C3:minor>")
  .s("triangle").lpf(1400).gain(0.14)
// pad
$: note("[0,4]").scale("<C3:minor C3:minor G3:phrygian C3:minor>")
  .s("sine").fm(1.2).fmh(1).fmdec(0.8).fmsus(0.4)
  .room(0.2).orbit(2).gain(0.12)
```

Keep square at **C2**, `bs:rm` at **C4**, and `plk:s5` degrees `~ 4 ~ <7 4>` as **`// hook`**. Do not retune `bs:rm` to C2. Do not rewrite stab 4/7.

`songs/dnb-reese/01.strudel` is still a thin demo (break + sub + mid + stab). Apply the fence above, not the on-disk file. Full-range Reese with sub is `bs:dk`, not a swap for this mid glue — [strudel-sound-design](../strudel-sound-design/SKILL.md). Do not stack `bs:dk` on this split.

| Piece | Role |
| --- | --- |
| `setcpm(174/4)` | **174 BPM**. Not 124 house, not 126 techno. Play **solo**. |
| 8-slot break + mini `*2` | One cycle of a break, tiled twice so it fills the bar |
| drums `.gain(0.7)` | **Above** the sub (0.42) so kick/snare sit in front |
| square + `C2:minor` + `lpf(120)` | `// bass` — synth sub only, not a sample |
| `bs:rm` + `C4:minor` | `// bass-mid` — `samples/bs/rm.wav`; native 800–1200 Hz stays |
| `plk:s5` degrees `4` / `7` | `// hook` — `samples/plk/s5.wav`; hollow C–G transposed (still no third) |
| `.cut(1)` | Steal the previous stab one-shot |
| no `.compressor` / no clap | Compressor is mixer last-write. No house `cp` on this grid |

## Why mid is C4, not C2

`note().s("sample")` sets playback rate to `target_hz / SAMPLE_ROOT_HZ` (`deck.rs`). `SAMPLE_ROOT_HZ` is **261.63 Hz (C4)** even though the constant comment says C3.

`samples/bs/rm.wav` is a **C3** mid-band Reese (band-pass **800–1200 Hz**, no sub). Ratio 1.0 already sounds as the recorded C3 band.

| Scale | Degree 0 target | Ratio | What you hear |
| --- | --- | --- | --- |
| `.scale("C4:minor")` | C4 = 261.63 Hz | 1.0 | Native 800–1200 Hz mid glue |
| `.scale("C2:minor")` | C2 ≈ 65.4 Hz | ~0.25 | Band dumped to ~200–300 Hz |

Use **C4:minor** on `bs:rm`. `C2:minor` dumps the band. Do not put this sample on the sub octave to “match” the square.

## Why the sub stays square

The sub is a **synth waveform**, not a sample. `s("square")` resolves in `sound.rs` as `Wave::Square` before the bank is consulted. The oscillator runs at the scale frequency (C2 ≈ 65 Hz) and `lpf(120)` keeps mids out.

`bs:rm` has **no sub**. It cannot replace the square. One `$:` cannot be both: the sample’s native band is 800–1200; the square’s job is the basement.

Sub and mid share degrees `0 3 0 <0 -1>` (C–F–C–C/Bb in C minor: degree 3 is the fourth, not Eb) but **different scales/octaves**:

| Track | Scale | Degree 0 | Timbre |
| --- | --- | --- | --- |
| bass | `C2:minor` | C2 (real oscillator Hz) | square + `lpf(120)` gain 0.42 |
| bass-mid | `C4:minor` | written C4, ratio 1.0 → recorded C3 band | `bs:rm` gain 0.36 |

Do not retune the mid to `C2:minor`. Do not swap the square for `bs:rm`. Comment names are `// bass` and `// bass-mid`.

## Why stab 4 / 7 is still no-third

`samples/plk/s5.wav` is a C3 hollow fifth: **C and G only** (ratios 1 and 3/2). No major third in the wav.

`note().s("plk:s5")` only **transposes** that recording (`target_hz / SAMPLE_ROOT_HZ`). It does not add chord tones. Deck scheduling still plays **one pitch per event**; `expand_chord` is not called.

C minor intervals are `[0, 2, 3, 5, 7, 8, 10]`. Degrees `~ 4 ~ <7 4>` on **`// hook`**:

| Degree | Interval | Written (C4:minor) | Ratio vs C4 | Heard interval |
| --- | --- | --- | --- | --- |
| 4 | P5 | G | ×1.5 | G–D (still a fifth) |
| 7 | octave | C | ×2 | C–G one octave up |

Moving by a fifth or an octave keeps a hollow fifth. It does **not** invent E (major third) or Eb (minor third). Do not rewrite `4` / `7` to `2` or `3`. Do not write `note("c3'maj")` — the suffix is root only anyway.

`.cut(1)` puts the stab on cut group 1; `Deck::alloc_voice` drops earlier voices in that group. The one-shot must not overlap itself.

Lead and arp take the 4-bar `.scale("<…>")` and rest on different slots from the hook. Do not fill all 16ths on every melody track at once. Chords and pad are `[0,4]` so they do not add a third on top of the hollow stab.

## Why `*2`, not `.fast(2)`

Mini `*2` (`mini.rs` `Node::Fast`) copies the inner span twice across the bar, so the break occupies `[0, 1)`.

The method `.fast(2)` only multiplies `PatternCode.speed`. `deck.rs` then divides each event’s start by that speed and **does not re-query the next cycle**. A full-bar break with `.fast(2)` is squeezed into the first half; the second half is empty. Tile with mini `*2`.

## Why 174 ≠ 124

Both decks share one `Transport`. A DJ pair must use the **same** `setcpm` or the second file’s tempo is discarded.

This file is **174**. House clap / four-on-the-floor are **124**. Techno-duck / electro are **126**. Pairing 174 with 124 (or 126) drops the other side’s BPM. Play this file **solo**. Do not DJ-pair it with [strudel-genre-house](../strudel-genre-house/SKILL.md).

## Sample keys (`part:slug`)

These one-shots live under `samples/<part>/<slug>.wav`. Call them as `s("part:slug")`. Old flat stems (`reese-mid`, `stab-fm_fifth`) are gone.

| Disk | Sound key | Wrong key (silent) |
| --- | --- | --- |
| `samples/bs/rm.wav` | `bs:rm` | `reese-mid` / `reese_mid` |
| `samples/plk/s5.wav` | `plk:s5` | `stab-fm_fifth` / `stab-fm-fifth` |

Unknown names fail resolve and the current performance continues.

Drums stay the bundled folder keys `bd` / `sd` / `hh` / `oh`. There is no `db` sample.

**Chord suffixes:** `note("c3'maj")` still plays the **root only**. This song uses degrees.

No `.compressor` on a `$:` — that writes the **mixer master** (last-write) and will squash the kick. No house clap (`cp`) on this break.

## Try it in this app

```bash
strudel-rs play songs/dnb-reese/01.strudel --seconds 12
strudel-rs play songs/dnb-reese/01.strudel --headless --seconds 8
```

Play **solo**. Do not `dj` this file with a 124 house or 126 techno song.

The on-disk song is still the thin demo. For a full bed, `strudel_apply_song` the Pattern fence.

No device: `cargo test --test e2e dnb_reese_01 -- --nocapture`.

Live TUI: `/a load dnb-reese-01`.

## Variations (still this syntax)

| Goal | Change |
| --- | --- |
| Busier hats | `hh*4` → `hh*8` inside the same `*2` group |
| Darker kit | `.lpf(2500)` on the drum `$:` |
| New root, same split | bass `.scale("A2:minor")` **and** bass-mid/hook `.scale("A4:minor")` (keep the octave split) |

Do not move `bs:rm` to octave 2. Do not replace the square with the sample. Do not rewrite hook degrees `4` / `7`.

## Checklist

- [ ] `setcpm(174/4)` + **8 `$:`** (drums, bass, bass-mid, hook, lead, arp, chords, pad)
- [ ] Square `C2` + `lpf(120)` as bass; `bs:rm` at **`C4:minor`** as bass-mid
- [ ] Hook `plk:s5` degrees `~ 4 ~ <7 4>` — do not rewrite 4/7
- [ ] Break `*2`, drums gain above sub. Never `db`. Never `.fast(2)`
- [ ] 4-bar phrase on lead / arp / chords / pad
- [ ] Play **solo** at 174. `songs/dnb-reese/01.strudel` is still a thin demo; this fence is the new target

## Do not

- Put `bs:rm` on `C2:minor` — that dumps 800–1200 Hz.
- Replace the square sub with a sample, or name one `$:` `square` “Reese”.
- Write `reese_mid` or `stab-fm-fifth` — those keys do not resolve.
- Rewrite stab degrees `4` / `7` (they only transpose the hollow fifth).
- Use `.fast(2)` when you want the break to fill the bar.
- Add house `cp` / `[~ cp]*2` on this grid.
- Put `.compressor` on a track.
- Pair this file with 124 house or 126 techno (shared clock; the other tempo is discarded).
- Write `in_bank=no` PCM (`ld:ac`, `pf:al`, `dr:*`, `ps:*`). Allowed long PCM: `ld:ss`, `pf:ff`.
- Stack `bs:su` / `bs:dk` on this split. `bs:dk` is a different recipe.
- Ship the 4-track on-disk demo as a new apply.
- `stack()` / `.cpm(174)` / `.lfo()` / `kit:bd`.
