---
name: dnb-reese-mid-stab
description: >-
  Use when writing a Drum and Bass loop in strudel-rs with the bundled
  mid Reese sample and hollow-fifth stab: 174 BPM, square C2 sub,
  reese-mid at C4:minor, stab-fm_fifth degrees 4/7. Not saw-Reese.
  Do not DJ-pair with 124 house.
---

# DnB: square sub + reese-mid + hollow-fifth stab (strudel-rs)

## When to use

- The request is **DnB at 174 BPM** with the **bundled mid Reese sample** and a **hollow fifth stab**.
- You need a **half-time break** in front of a **split Reese**: square synth sub + `reese-mid` (not a saw).
- You are **not** writing the older square+saw recipe ([drum-and-bass](../drum-and-bass/SKILL.md)).
- You are **not** writing 124 house (`[~ cp]*2`) or 126 techno. Shared Transport: do not pair this file with those tempos.

## Pattern

```
// @title skill-dnb-reese-mid-stab
// @genre drum and bass
setcpm(174/4)
// drums
$: s("[bd <~ sd> ~ sd ~ <bd ~> <bd sd> <bd ~>, hh*4, [~@5 oh ~@2]]*2").gain(0.7).lpf(4000)
// sub
$: note("0 3 0 <0 -1>").scale("C2:minor").s("square").lpf(120).gain(0.42)
// mid
$: note("0 3 0 <0 -1>").scale("C4:minor").s("reese-mid").gain(0.38)
// stab
$: note("~ 4 ~ <7 4>").scale("C4:minor").s("stab-fm_fifth").gain(0.22).cut(1)
```

Playable copy: `songs/skill-dnb-reese-mid-stab.strudel`. Do not “improve” the degrees or the drum grid. Full-range Reese with sub is `reese-dark`, not a swap for this mid glue — [factory-pcm-usage](../factory-pcm-usage/SKILL.md).

| Piece | Role |
| --- | --- |
| `setcpm(174/4)` | **174 BPM**. Not 124 house, not 126 techno. |
| 8-slot break + mini `*2` | One cycle of a break, tiled twice so it fills the bar |
| drums `.gain(0.7)` | **Above** the sub (0.42) so kick/snare sit in front |
| square + `C2:minor` + `lpf(120)` | Synth sub only — not a sample |
| `reese-mid` + `C4:minor` | Flat stem `samples/reese-mid.wav`; native 800–1200 Hz stays |
| `stab-fm_fifth` degrees `4` / `7` | Flat stem; hollow C–G transposed (still no third) |
| `.cut(1)` | Steal the previous stab one-shot |
| no `.compressor` / no clap | Compressor is mixer last-write. No house `cp` on this grid |

## Why mid is C4, not C2

`note().s("sample")` sets playback rate to `target_hz / SAMPLE_ROOT_HZ` (`deck.rs`). `SAMPLE_ROOT_HZ` is **261.63 Hz (C4)** even though the constant comment says C3.

`samples/reese-mid.wav` is a **C3** mid-band Reese (band-pass **800–1200 Hz**, no sub). Ratio 1.0 already sounds as the recorded C3 band.

| Scale | Degree 0 target | Ratio | What you hear |
| --- | --- | --- | --- |
| `.scale("C4:minor")` | C4 = 261.63 Hz | 1.0 | Native 800–1200 Hz mid glue |
| `.scale("C2:minor")` | C2 ≈ 65.4 Hz | ~0.25 | Band dumped to ~200–300 Hz |

Use **C4:minor** on `reese-mid`. `C2:minor` dumps the band. Do not put this sample on the sub octave to “match” the square.

## Why the sub stays square

The sub is a **synth waveform**, not a sample. `s("square")` resolves in `sound.rs` as `Wave::Square` before the bank is consulted. The oscillator runs at the scale frequency (C2 ≈ 65 Hz) and `lpf(120)` keeps mids out.

`reese-mid` has **no sub**. It cannot replace the square. One `$:` cannot be both: the sample’s native band is 800–1200; the square’s job is the basement.

Sub and mid share degrees `0 3 0 <0 -1>` (C–F–C–C/Bb in C minor: degree 3 is the fourth, not Eb) but **different scales/octaves**:

| Track | Scale | Degree 0 | Timbre |
| --- | --- | --- | --- |
| sub | `C2:minor` | C2 (real oscillator Hz) | square + `lpf(120)` gain 0.42 |
| mid | `C4:minor` | written C4, ratio 1.0 → recorded C3 band | `reese-mid` gain 0.38 |

Do not retune the mid to `C2:minor`. Do not swap the square for `reese-mid`.

## Why stab 4 / 7 is still no-third

`samples/stab-fm_fifth.wav` is a C3 hollow fifth: **C and G only** (ratios 1 and 3/2). No major third in the wav.

`note().s("stab-fm_fifth")` only **transposes** that recording (`target_hz / SAMPLE_ROOT_HZ`). It does not add chord tones. Deck scheduling still plays **one pitch per event**; `expand_chord` is not called.

C minor intervals are `[0, 2, 3, 5, 7, 8, 10]`. Degrees `~ 4 ~ <7 4>`:

| Degree | Interval | Written (C4:minor) | Ratio vs C4 | Heard interval |
| --- | --- | --- | --- | --- |
| 4 | P5 | G | ×1.5 | G–D (still a fifth) |
| 7 | octave | C | ×2 | C–G one octave up |

Moving by a fifth or an octave keeps a hollow fifth. It does **not** invent E (major third) or Eb (minor third). Do not rewrite `4` / `7` to `2` or `3`. Do not write `note("c3'maj")` — the suffix is root only anyway.

`.cut(1)` puts the stab on cut group 1; `Deck::alloc_voice` drops earlier voices in that group. The one-shot must not overlap itself.

## Why `*2`, not `.fast(2)`

Mini `*2` (`mini.rs` `Node::Fast`) copies the inner span twice across the bar, so the break occupies `[0, 1)`.

The method `.fast(2)` only multiplies `PatternCode.speed`. `deck.rs` then divides each event’s start by that speed and **does not re-query the next cycle**. A full-bar break with `.fast(2)` is squeezed into the first half; the second half is empty. Tile with mini `*2`.

## Why 174 ≠ 124

Both decks share one `Transport`. A DJ pair must use the **same** `setcpm` or the second file’s tempo is discarded.

This file is **174**. House clap / four-on-the-floor / minor-scale-loop are **124**. Sidechain-ducking / ambient1 are **126**. Pairing 174 with 124 (or 126) drops the other side’s BPM. Play this file **solo**. Do not DJ-pair it with [house-clap-backbeat](../house-clap-backbeat/SKILL.md).

## Sample keys (flat stems)

`load_dir` keys a loose WAV as `file_stem()` lowercased (`sample.rs`). Write the stem exactly. Hyphen and underscore are different characters.

| Disk | Sound key | Wrong key (silent) |
| --- | --- | --- |
| `samples/reese-mid.wav` | `reese-mid` | `reese_mid` |
| `samples/stab-fm_fifth.wav` | `stab-fm_fifth` | `stab-fm-fifth` |

`stab-fm_fifth` follows `{family}-{character}_{detail}` — underscore before `fifth`. `reese-mid` is family-character only (hyphen, no underscore). Unknown names fail resolve and the current performance continues.

Drums stay the bundled folder keys `bd` / `sd` / `hh` / `oh`. There is no `db` sample.

**Chord suffixes:** `note("c3'maj")` still plays the **root only**. This song uses degrees.

No `.compressor` on a `$:` — that writes the **mixer master** (last-write) and will squash the kick. No house clap (`cp`) on this break.

## Try it in this app

```bash
strudel-rs play songs/skill-dnb-reese-mid-stab.strudel --seconds 12
strudel-rs play songs/skill-dnb-reese-mid-stab.strudel --headless --seconds 8
```

Play **solo**. Do not `dj` this file with a 124 house or 126 techno song.

No device: `cargo test --test e2e skill_dnb_reese -- --nocapture`.

Live TUI: `/a load skill-dnb-reese-mid-stab`.

## Variations (still this syntax)

| Goal | Change |
| --- | --- |
| Busier hats | `hh*4` → `hh*8` inside the same `*2` group |
| Darker kit | `.lpf(2500)` on the drum `$:` |
| New root, same split | sub `.scale("A2:minor")` **and** mid/stab `.scale("A4:minor")` (keep the octave split) |

Do not move `reese-mid` to octave 2. Do not replace the square with the sample.

## Do not

- Put `reese-mid` on `C2:minor` — that dumps 800–1200 Hz.
- Replace the square sub with a sample, or name one `$:` `square` “Reese”.
- Write `reese_mid` or `stab-fm-fifth` — those keys do not resolve.
- Rewrite stab degrees `4` / `7` (they only transpose the hollow fifth).
- Use `.fast(2)` when you want the break to fill the bar.
- Add house `cp` / `[~ cp]*2` on this grid.
- Put `.compressor` on a track.
- Pair this file with 124 house or 126 techno (shared clock; the other tempo is discarded).
- `stack()` / `.cpm(174)`.
