---
name: strudel-genre-acid
description: >-
  Use when writing a TB-303-style acid line in strudel-rs: per-note
  filter envelope (lpenv), high resonance, saw or square, monophonic
  16ths. Not a parked lpf plus amp ADSR, and not sidechain ducking.
version: 4.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, acid, 303, filter-envelope]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-four-on-the-floor
      - strudel-genre-techno-duck
---

# TB-303 filter envelope (strudel-rs, acid techno)

## When to use

- The request is **acid / TB-303**: a resonant saw or square that **opens on each note and decays into the body**.
- You need the **filter envelope**, not a louder amp click and not a static `.lpf(800)`.
- Drums are **techno** four-on-the-floor (`bd*4` + offbeat hats). You are **not** adding a house snare unless you label the song **house** (or an explicit hybrid).
- You are **not** writing sidechain duck or putting `.compressor` on the bass.

`songs/acid-01.strudel` is the playable 303 filter-envelope loop. A parked `.lpf` plus amp ADSR is only a starting point — this skill is the missing envelope piece.

## Pattern

```
// @title skill-acid-303-filter-envelope
// @genre acid techno
setcpm(130/4)
// drums — techno kick-front (no house snare)
$: s("bd*4, [~ hh]*4").gain(0.65)
// acid — 16ths, monophonic, filter env retriggers per onset
$: note("0 0 3 0  7 3 2 0  4 4 3 0  -1 3 0 <2 5>")
  .scale("C2:minor")
  .s("sawtooth")
  .lpf("260 260 720 260  760 260 260 260  260 680 260 260  740 720 260 260")
  .lpq(14)
  .lpenv(3)
  .lpattack(0.001)
  .lpdecay(0.09)
  .lpsustain(0.05)
  .cut(1)
  .gain(0.48)
  .attack(0.001)
  .decay(0.1)
  .sustain(0.12)
  .release(0.04)
```

Playable copy: `songs/acid-01.strudel`.

| Piece | Role |
| --- | --- |
| `setcpm(130/4)` | 130 BPM (acid techno). |
| `s("bd*4, [~ hh]*4")` | Techno grid, **kick in front**. No `[~ sd]*2`. `.gain(0.65)` keeps the kick above the resonant line. |
| `note("…")` 16 tokens + `.scale("C2:minor")` | Monophonic 16ths. Degrees, not `c3'min` (suffix = root only). |
| `.s("sawtooth")` | 303 source. `.s("square")` is the other hardware waveform. |
| `.lpf("260 …")` | **Base** cutoff per 16th. Accent bases stay in **600–800 Hz**. Not the envelope. |
| `.lpq(14)` | High resonance (scream). 12–16; above that is harsh. |
| `.lpenv(3)` | Envelope **depth in octaves**. Cutoff = `base * 2^(lpenv * env)`. |
| `.lpattack(0.001)` / `.lpdecay(0.09)` / `.lpsustain(0.05)` | Per-**note** AD toward a closed body. |
| `.cut(1)` | Steal the previous voice (mono). |
| amp `.attack` / `.decay` / `.sustain` / `.release` | Gate the oscillator. **Not** a substitute for `lpenv`. |
| no `.compressor` / no `.duckorbit` | Compressor is mixer last-write. This recipe is the filter, not duck. |

## Why it sounds that way

A TB-303 accent is a **cutoff sweep**, not a volume envelope. Hardware: a lowpass with high resonance; each gate (and extra on accent) opens the cutoff, then the filter decay closes it while the note is still sounding. Parked `.lpf(800).lpq(16)` plus a short amp ADSR only shapes loudness. The spectrum stays the same.

This engine **does** a true per-note filter env (`synth.rs` `Voice::advance_lp_env` + `effective_lpf_hz`):

1. Each onset allocates a `Voice`. `lpenv` / `lpa` / `lpd` / `lps` live on that voice (`ModParams` in `code.rs`).
2. Filter env is **attack → decay toward sustain**. Level goes 0→1 over `lpattack`, then toward `lpsustain` over `lpdecay`.
3. Cutoff is `base_hz * 2^(lpenv * level)`, clamped 20–20000 Hz. Depth is **octaves**, not Hertz.
4. Coeffs are rebuilt every sample while `lpenv != 0` (`synth.rs`). That is intra-note motion, not a per-cycle snapshot.

At 130 BPM a 16th is ~0.115 s. `.lpdecay(0.09)` with `.lpsustain(0.05)` closes most of the sweep inside that 16th. Example: base `260`, `lpenv(3)` peaks near `260 * 2^3 ≈ 2.1 kHz`, then falls to about `260 * 2^(3*0.05) ≈ 290 Hz`. An accent base of `760` peaks near `760 * 8 ≈ 6.1 kHz` — enough scream without a thin digital-saw top. **Do not** use `lpf(900)` + `lpenv(3.5)`: that peaks near 10 kHz and the saw goes gappy.

**Amp ADSR is a different envelope.** It scales the oscillator after the osc, before the biquad. A short amp decay with static `lpf` is a pluck. The 303 needs the **biquad cutoff** to move.

### What this engine can and cannot do

| Idiom | Works? | Notes |
| --- | --- | --- |
| Scalar `.lpenv` + `.lpattack` + `.lpdecay` + `.lpsustain` | **Yes** | Per-note. Same depth on every hit. |
| `.lpf("a b c …")` + `lpenv` | **Yes** | Pattern snapshots **base** Hz at the note’s bar phase (`deck.rs`). Then `lpenv` multiplies. **This is the accent.** |
| `.lpf("[260 260 720 260]*4")` | **Yes** | Four 16th bases tiled across the bar (regular accent; keep the high step ≤ 800). |
| `.lpenv("4 1 4 1")` / patterned `lpdecay` | **No** | Those methods are `parse_num` scalars only. |
| `.lprelease` / `.lpr` | Parsed, **unused** | `advance_lp_env` has no release stage. Omit it. |
| `.lpf(sine.rangex(400, 4000))` | LFO, **not** 303 | Continuous LFO **wins over** `lpenv` (`effective_lpf_hz`). Cycle wobble, not per-note decay. |
| Static `.lpf(800)` + amp ADSR | Plays | **Not** a filter env. Do not call it 303. |
| `.lpf("<400 1200>")` alone | Per-**cycle** step | Alternates a parked cutoff. No intra-note decay. |

Closest working accent: raise **base** `.lpf` on the accented 16ths **into 600–800 Hz**; keep one scalar `.lpenv(3)`. Do not fake accent with `.gain` or amp decay. Do not teach `900` + `lpenv(3.5)`.

Both decks share one `Transport`. This file is 130 BPM. Play it **solo** — stacking another 303 on the other deck doubles the acid, it is not a mix. Do **not** pair this file with a different `setcpm` (the other tempo is discarded).

## Try it in this app

```bash
strudel-rs play songs/acid-01.strudel --seconds 12
strudel-rs play songs/acid-01.strudel --headless --seconds 8
```

No device: `cargo test --test e2e acid_01 -- --nocapture`.

Live TUI: `/a load acid-01`.

## Variations (still this syntax)

| Goal | Change |
| --- | --- |
| Square 303 | `.s("square")` (keep `lpenv` / `lpq`) |
| Regular accent | `.lpf("[260 260 720 260]*4")` (accent still ≤ 800) |
| Darker body | base numbers `180–220`, or `.lpenv(2.5)` |
| Longer scream | `.lpdecay(0.14)` (starts to smear at 16ths) |
| House backbeat | add `[~ sd]*2` on drums **and label the song house** |
| More open | raise body bases; keep accent bases in **600–800** and `.lpenv(3)` |

## Rules (do not skip)

1. The 303 sound is **`.lpenv` + high `.lpq` + low/mid `.lpf` base**. Amp ADSR is only the gate.
2. Accent = patterned **base** `.lpf` in **600–800 Hz**, not `900` + `lpenv(3.5)`, not patterned `lpenv`, and not `.gain`.
3. Techno drums are `bd*4` + `[~ hh]*4`. `[~ sd]*2` only if you label **house** (or say hybrid).
4. Do not put `.compressor` on the acid line. Do not use duck for this recipe.
5. Shared clock is **130**. Play solo — another 303 on B is not a mix pair.

## Do not

- Ship a parked `.lpf(800).lpq(16)` + amp ADSR and call it a filter envelope.
- Use accent base `900` with `.lpenv(3.5)` — peak sits near 10 kHz; the digital saw goes thin.
- Present two 303 files as a DJ mix pair. BPM may match; two 303s stacked is not a mix.
- Combine `.lpf(sine.rangex(…))` with `.lpenv` — the LFO replaces the env.
- Write `.lpenv("4 1 4 1")` — parse error / not a pattern.
- Rely on `.lprelease` — it does not run.
- `note("c3'min")` for a triad — suffix is root only; a chord is `note("[0,2,4]")` (and a 303 line should stay mono).
- Pair this file with a different `setcpm` (shared clock; the other tempo is discarded).
- `stack()` / `.cpm(130)`.
