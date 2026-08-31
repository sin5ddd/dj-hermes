---
name: strudel-mood-bright-dark
description: >-
  Use when asked to make a strudel-rs loop brighter or darker
  (mode, voicing width, register, sample swap, filter/EQ) — not a
  genre change and not a new BPM.
version: 4.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, mood, bright, dark, scale]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-house
      - strudel-genre-four-on-the-floor
---

# Brighter / darker (strudel-rs)

## When to use

- The request is **mood**: “make it brighter”, “make it darker”, “less sad”, “more open”.
- You already have (or are writing) a loop and need a **transform**, not a new genre recipe.
- You are **not** switching house → techno → DnB. Tempo and drum grid stay put. A pair that will be mixed must share one `setcpm`.

Playable pair (same 124 house grid): `songs/skill-mood-dark.strudel` and `songs/skill-mood-bright.strudel`.

## The five moves

Work top-down. Each move actually changes what this engine plays. Skip a layer that does nothing here (see [What does not work](#what-does-not-work)).

| # | Layer | Darker | Brighter |
| --- | --- | --- | --- |
| 1 | Mode / quality | `.scale("…:minor")` (degree 2 = minor 3rd) | `.scale("…:major")` (degree 2 = major 3rd) |
| 2 | Voicing width | close `[0,2,4]` | spread `[0,4,9]` or add octave `7` |
| 3 | Register | lower synth octave (`C2`) | higher synth octave (`C3`); samples stay `C4` |
| 4 | Sample swap | `square` sub + `reese-mid` at **C4** | `lead-fm_pluck` at **C4** (or a more open synth) |
| 5 | Filter / EQ | parked low `.lpf` on a **synth**; mixer Hi cut | higher `.lpf`; mixer Hi boost |

Do not invent a third inside `stab-fm_fifth`. Do not write `note("c3'maj")` and expect a chord.

## Darker (starting point)

Same drum grid as [strudel-genre-house](../strudel-genre-house/SKILL.md). Harmonic/timbre contrast lives on bass + chords.

```
// @title skill-mood-dark
setcpm(124/4)
// drums
$: s("bd*4, [~ cp]*2, [~ hh]*4").gain(0.6)
// bass
$: note("0 2 4 0").scale("C2:minor").s("square").lpf(140).gain(0.4)
// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("C4:minor").s("reese-mid").gain(0.22)
```

## Brighter (same material)

Same `setcpm`, same drum `$:` string, same bass rhythm `0 2 4 0`, same chord rhythm `[…] ~ […] ~`.

```
// @title skill-mood-bright
setcpm(124/4)
// drums
$: s("bd*4, [~ cp]*2, [~ hh]*4").gain(0.6)
// bass
$: note("0 2 4 0").scale("C3:major").s("sawtooth").lpf(1400).gain(0.36)
// chords
$: note("[0,4,9] ~ [0,4,9] ~").scale("C4:major").s("lead-fm_pluck").gain(0.34)
```

| Kept | Changed |
| --- | --- |
| `setcpm(124/4)` | one Transport — required for `dj` |
| `bd*4, [~ cp]*2, [~ hh]*4` | not techno (`bd*4, [~ hh]*4`), not 174 DnB |
| `0 2 4 0` / `[chord] ~ [chord] ~` | mode, voicing, octave, sound, cutoff |

## 1. Mode / quality

Degrees are **0-based** and relative to the scale root (`scale.rs`). `C4:major` vs `C4:minor`:

| Degree | `C:major` `[0, 2, 4, 5, 7, 9, 11]` | `C:minor` `[0, 2, 3, 5, 7, 8, 10]` |
| --- | --- | --- |
| 0 | C | C |
| 2 | **E** (+4 st) | **Eb** (+3 st) |
| 4 | G | G |

Switching `minor` → `major` on the same degrees is the quality change. Degree 2 is the third; that is what reads bright vs dark. Degree 4 is the fifth in both modes — it does not carry major/minor.

Modes that sit between (still this parser):

| Want | Mode | What moves |
| --- | --- | --- |
| Darker than minor | `phrygian` | degree 1 is Db (b2) |
| Brighter than minor, not major | `dorian` | degree 5 is A (nat 6), not Ab |
| Darker than major | `mixolydian` | degree 6 is Bb (b7) |
| Brighter than major | `lydian` | degree 3 is F# (#4) |

`.scale("<A2:minor D:dorian G:mixolydian C:major>")` changes key/mode **per bar**. It is not a substitute for rewriting one loop brighter.

## 2. Voicing width

Deck scheduling (`deck.rs`) resolves **one pitch per mini event**. Real chords are **parallel degrees in one span**. There is no voicing helper.

| Pattern | Voices | C minor | C major |
| --- | --- | --- | --- |
| `[0,2,4]` | close triad | C–Eb–G | C–E–G |
| `[0,4,7]` | root + fifth + octave | C–G–C (no third) | same shape, still no third |
| `[0,4,9]` | spread (root, fifth, tenth) | C–G–Eb+oct | C–G–**E+oct** |
| `[0,2,4,7]` | close triad + octave | thicker, still close | thicker, still close |

`[0,4,9]` is wider because degree **9** = 7+2 = octave + the third. In major that is the major tenth (E above C). In minor it is still a minor tenth (Eb) — spread does not invent a major third.

The 9th is degree **8** (octave + degree 1 → D in C major). Degree **11** is 7+4 = G an octave up, so `[0,4,7,11]` is C–G–C–G, not a 9th. Wider-than-spread: `[0,4,7,8]`.

Space-separated `0 2 4` is an **arpeggio** (one pitch per slot), not a block chord.

## 3. Register

Scale octave is the **written root**. `C:major` with no digit is **C4**.

| Source | Darker | Brighter | Why |
| --- | --- | --- | --- |
| Synth (`square` / `sawtooth` / `triangle`) | `C2:…` | `C3:…` | oscillator Hz follow the scale |
| `reese-mid` / `lead-fm_pluck` | keep `C4:…` | keep `C4:…` | see below |

`note().s("sample")` sets playback rate to `target_hz / SAMPLE_ROOT_HZ` (`deck.rs`). `SAMPLE_ROOT_HZ` is **261.63 Hz (C4)** (the comment in `sample.rs` still says C3).

Both bundled pitched one-shots are **recorded at C3**. Ratio 1.0 already sounds as that C3 recording.

| Scale on those wavs | Target | Ratio | Heard |
| --- | --- | --- | --- |
| `.scale("C4:minor")` / `C4:major` | C4 | 1.0 | recorded C3 — mid |
| `.scale("C3:…")` | C3 ≈ 130.8 | ~0.5 | ~65 Hz — **bass dump** |
| `.scale("C2:…")` on `reese-mid` | C2 ≈ 65.4 | ~0.25 | 800–1200 Hz band drops to ~200–300 |

Do **not** “darken” `reese-mid` or `lead-fm_pluck` by lowering the scale octave. That does not park a filter; it retunes the whole recording down. Raise a **synth** (`C2` → `C3`) when you want a higher register. Optional extra on the pluck: `C5:major` (ratio 2) sits an octave above the recording — thinner; not this pair.

Square sub stays a **synth**. `s("square")` resolves in `sound.rs` before the bank. Do not replace it with a sample to “match” the mid.

## 4. Sample swap

| Key | Disk | Mood role | Constraint |
| --- | --- | --- | --- |
| `lead-fm_pluck` | `samples/lead-fm_pluck.wav` | brighter one-shot (underscore before `pluck`) | melody at `C4:major` / `C4:minor`. `.cut(1)` on **monophonic** hits only |
| `reese-mid` | `samples/reese-mid.wav` | darker mid glue (800–1200 Hz, no sub) | **C4 only**; hyphen, not `reese_mid` |
| `square` | synth | darker basement | real Hz at `C2`; `lpf` around 120–180 |
| `sawtooth` | synth | more open mid-bass | raise `.lpf` when brightening |
| `cp` | `samples/cp/00.wav` | house 2/4 clap | keep on this 124 grid; do not put it on techno |
| `stab-fm_major` | key `stab-fm_major` (C3+E3+G3 in the wav) | brighter triad one-shot | `note("0 ~ 0 ~")` only — [do not stack degrees](#stab-fm_major-is-already-a-triad). Not this pair |
| `stab-fm_fifth` | `samples/stab-fm_fifth.wav` | hollow C–G **only** | **cannot** carry major/minor — [gap](#gap-no-third-in-the-stab) |

Write stems exactly. `lead-fm-pluck` and `stab-fm-fifth` do not resolve (silent; performance continues).

This pair is **not** the 174 DnB recipe. `reese-mid` here is a dark mid at 124 house. Do not copy the DnB break, and do not DJ-pair these files with `songs/skill-dnb-reese-mid-stab.strudel`.

## 5. Filter / EQ layer

Two different layers. Do not mix them up.

**Song rewrite — per-voice `.lpf`:** higher cutoff = brighter spectrum on that `$:` (`synth.rs` biquad). Parked low `.lpf(140)` on the square is why the dark bass stays a sub. `.lpf(1400)` on the bright saw lets the major third speak.

Amp ADSR (`.attack` / `.decay` / `.sustain` / `.release`) scales loudness after the oscillator. A shorter decay is a pluck envelope. It is **not** a darker spectrum. Do not “darken” by only shortening the amp.

`reese-mid` is already band-passed 800–1200 Hz. An `.lpf` above that band does almost nothing; an `.lpf` below ~800 starts to eat the glue. Darkness from this sample is the **swap**, not a parked cutoff.

**Live DJ — mixer**, not a rewrite:

- Per-deck 3-band EQ (`mixer.rs`): Hi shelf 6 kHz, Mid peak 1 kHz, Lo shelf 200 Hz. Boost Hi on the bright deck; cut Hi (or Lo) on the dark deck.
- Master LPF/HPF on the mixer: a low master LPF darkens **both** decks.
- `.compressor(...)` on a `$:` is **mixer master, last-write** (`engine.rs`). It will squash the kick. It is not a mood tool.

TUI: `/eq a hi 0.8` (brighter A) vs `/eq b hi 0.2` (darker B). That is the booth bright/dark. The files stay as written.

## What does not work

### `note("c3'maj")` / `'min` is the root only

`expand_chord` in `code.rs` knows `maj` / `min` / `maj7` / `min7` / `dim` / `aug` / `sus2` / `sus4`, but **the deck never calls it**. `note("c3'maj")` plays **C3**. Use `[0,2,4]` or `[0,4,9]`.

### Gap: no third in the hollow stab

`samples/stab-fm_fifth.wav` is a C3 hollow fifth: **C and G only** (ratios 1 and 3/2). `note().s("stab-fm_fifth")` only transposes that recording. There is no major or minor third in the wav, and this engine will not invent one.

Do not write `note("c3'maj").s("stab-fm_fifth")` (root only, still no third). Do not rewrite stab degrees to `2` and call it a major triad. For a triad you write the degrees (`[0,2,4]` / `[0,4,9]`) on a synth, `reese-mid`, or `lead-fm_pluck` — or trigger `stab-fm_major` as a **single** degree (below).

### `stab-fm_major` is already a triad

Key `stab-fm_major`: recorded **C3+E3+G3**. The third is in the wav. Trigger with a **single** degree so the recording plays once:

```
$: note("0 ~ 0 ~").scale("C4:major").s("stab-fm_major").gain(0.22)
```

Do **not** write `note("[0,2,4]").s("stab-fm_major")`. Deck scheduling starts one voice per parallel degree; each voice plays the whole triad already in the sample, so you hear three stacked triads. `.cut(1)` on that parallel form would also steal sibling voices.

This pair does not use the major stab (no new WAV in this skill). Hollow `stab-fm_fifth` still cannot carry major/minor.

### `.cut(1)` on a parallel chord

`.cut(1)` is for **monophonic** one-shots (a single-note pluck or stab hit). `Deck::alloc_voice` drops earlier voices in that cut group. On `[0,4,9]` the three degrees allocate in one span: the second voice kills the first, the third kills the second — you hear one note, not a chord.

Keep `.cut(1)` on lines like house `note("4 ~ 7 4  2 0 ~ -1")` (eighths shorter than the ~0.4 s wav). Do not put it on `[0,2,4]` / `[0,4,9]`.

### Other dead ends

- Different `setcpm` on the pair — the second tempo is discarded (shared Transport).
- House clap on techno (`bd*4, [~ hh]*4`) or 174 with 124 — genre/clock, not mood.
- `[~ sd]*2` stacked with `cp` — they mask (house skill).
- Amp ADSR as a stand-in for `.lpf`.
- Track `.compressor` as “darker”.
- `stack()` / `.cpm(124)` / a `note(...)` line without `$:`.

## Why the pair sounds different

`C2:minor` degree 2 is **Eb2**. `C3:major` degree 2 is **E3**. That is quality **and** a synth-octave lift.

Close `[0,2,4]` on `C4:minor` + `reese-mid` is three mid-band voices on C–Eb–G (written C4, ratio 1.0 → recorded C3 cluster). Spread `[0,4,9]` on `C4:major` + `lead-fm_pluck` is C–G–E+oct (heard a major tenth above the recording). No `.cut(1)` on that chord: cut steals earlier voices in the same group, so the triad dies one note at a time. At 124, `[…] ~ […] ~` is four quarter slots (~0.48 s); the ~0.4 s one-shot ends in the rest before the next stab.

Square + `lpf(140)` vs saw + `lpf(1400)` is the synth-spectrum move. Drums stay `[~ cp]*2` at 124 so the ear compares harmony/timbre, not the kit.

## Try it in this app

```bash
strudel-rs play songs/skill-mood-dark.strudel --seconds 12
strudel-rs play songs/skill-mood-bright.strudel --seconds 12

strudel-rs play songs/skill-mood-dark.strudel --headless --seconds 8
strudel-rs play songs/skill-mood-bright.strudel --headless --seconds 8

# Dual deck — both files are setcpm(124/4). Do not pair a different BPM.
strudel-rs dj songs/skill-mood-dark.strudel songs/skill-mood-bright.strudel
```

No device: `cargo test --test e2e skill_mood -- --nocapture`.

Live TUI: `/a load skill-mood-dark` then `/b load skill-mood-bright`. `/x 4` crossfades toward B (equal-power, bar-quantized). Optional booth EQ: `/eq a hi 0.2` / `/eq b hi 0.8`.

## Variations (still this syntax)

| Goal | Change |
| --- | --- |
| Brighten without raising the bass | keep `.scale("C2:major")` on the square/saw; only flip the chord to `[0,4,9]` + `C4:major` |
| Darken without the Reese sample | `[0,2,4]` on `triangle` + `.lpf(800)` at `C3:minor` (synth-only; empty bank still plays) |
| Wider than `[0,4,9]` | `[0,4,7,8]` (C major: C–G–C–**D**, the 9th is degree **8**; degree 11 is G+oct) |
| Live only | mixer Hi / master LPF — do not rewrite the file |

Do not add `stab-fm_fifth` to “make a major stab”. Do not play `stab-fm_major` as `[0,2,4]`. Do not drop `[~ cp]*2` onto [strudel-genre-four-on-the-floor](../strudel-genre-four-on-the-floor/SKILL.md).

## Do not

- `note("c3'maj")` / `'min` when you want a chord.
- Treat `stab-fm_fifth` as a major/minor carrier (wav has no third).
- Play `stab-fm_major` as `note("[0,2,4]")` — the wav is already C–E–G; use `note("0 ~ 0 ~")`.
- Put `.cut(1)` on parallel-degree chords (`[0,2,4]`, `[0,4,9]`).
- Put `reese-mid` or `lead-fm_pluck` on `C3` / `C2` (band/register dump).
- Write `lead-fm-pluck`, `reese_mid`, `stab-fm-fifth`, or `stab-fm-major`.
- Put house `cp` on techno `bd*4, [~ hh]*4`.
- Pair this 124 pair with 174 DnB or any other `setcpm`.
- Use amp ADSR or `.compressor` as the bright/dark control.
- `stack()` / `.cpm(124)`.
