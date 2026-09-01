---
name: strudel-minor-scale-loop
description: >-
  Use when writing a short minor-key bass and triad loop in strudel-rs
  (scale degrees, not chord-suffix tokens).
version: 4.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, harmony, scale, minor]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-four-on-the-floor
      - strudel-mood-bright-dark
---

# Minor-scale loop (strudel-rs)

## When to use

- The request is a **simple harmonic loop**, minor bassline, or “sad / dark / Aeolian” pad+bass.
- You need pitches that stay **in key** when you later change the root.
- You are **not** writing a four-chord pop progression (use `.scale("<…>")` in a later skill) or a drum-only beat.

## Pattern

```
// @title skill-minor-scale-loop
// @details C minor degrees: bass arpeggio + root-position triad
setcpm(124/4)
// bass
$: note("0 2 4 0").scale("C2:minor").s("sawtooth").lpf(400).gain(0.55)
// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("C3:minor").s("triangle").lpf(1400).gain(0.28)
```

| Piece | Role |
| --- | --- |
| `setcpm(124/4)` | 124 BPM — **same** as the four-on-the-floor skill (shared Transport) |
| `note("0 2 4 0")` | Four degrees, one per beat |
| `.scale("C2:minor")` | Degree 0 = C2 (bass octave) |
| `.s("sawtooth").lpf(400)` | Low saw; no drum samples required |
| `[0,2,4]` | **Parallel** degrees = C–Eb–G together |
| `.scale("C3:minor")` | Same mode, one octave up |

Named-note equivalent (fixed key, no `.scale`):

```
$: note("c2 eb2 g2 c2").s("sawtooth").lpf(400).gain(0.55)
$: note("[c3,eb3,g3] ~ [c3,eb3,g3] ~").s("triangle").lpf(1400).gain(0.28)
```

Prefer degrees + `.scale` so a live root swap (`C2:minor` → `A2:minor`) keeps the shape.

## Why it sounds that way

`scale.rs` maps **0-based** degrees through the mode’s semitone list. `C2:minor` / Aeolian is `[0, 2, 3, 5, 7, 8, 10]`:

| Degree | Interval | Note at C2 | Note at C3 |
| --- | --- | --- | --- |
| 0 | unison | C2 | C3 |
| 2 | minor 3rd (3 st) | Eb2 | Eb3 |
| 4 | perfect 5th (7 st) | G2 | G3 |

Degree **2** is Eb, not E. That minor third is why the loop reads as minor, not major (`C:major` degree 2 is E, +4 st). `[0,2,4]` is the root-position minor triad.

Comma inside brackets is **simultaneous** (`mini.rs` `Parallel`): three events share the same start and duration, so the deck starts three voices. Space-separated `0 2 4 0` is a **sequence**: one pitch per quarter of the bar (arpeggio, not a block chord).

`[0,2,4] ~ [0,2,4] ~` is four equal slots: stab, rest, stab, rest — half-note feel against the walking bass.

**Chord suffixes:** `expand_chord("c3'min")` in `code.rs` knows `maj` / `min` / `maj7` / `min7` / `dim` / `aug` / `sus2` / `sus4`, but **deck scheduling does not expand them**. `note("c3'min")` plays **C3 only** (`note_to_midi_i32` strips `'…`). Use `[0,2,4]` or `[c3,eb3,g3]` to hear a triad.

Saw + low LPF keeps the bass out of the triad’s midrange. Triangle + higher LPF is a soft mid stab. Both are built-in waveforms (`sound.rs`); this file plays with an empty sample bank.

## Try it in this app

```bash
# Apply the inline recipe with strudel_apply_song, or pair existing 124 files:
strudel-rs dj songs/house-01.strudel songs/four-on-the-floor-01.strudel
```

No audio device: `cargo test --test e2e four_on_the_floor -- --nocapture`.

Live TUI: `/b load four-on-the-floor-01`. Then `/x 4` to crossfade toward B (equal-power, bar-quantized).

## Variations (still this syntax)

| Goal | Change |
| --- | --- |
| Brighter / major | `.scale("C2:major")` on both tracks (degree 2 becomes E) |
| New root, same shape | `.scale("A2:minor")` / `.scale("A3:minor")` |
| Darker bass | `.lpf(250)` or degrees `-1 0 2 0` (Bb below C in C minor) |
| Hold the triad | `note("[0,2,4]")` with `.attack(0.05).release(0.3)` |

## Do not

- `stack()` / `.cpm()` / a `note(...)` line without `$:` in a `.strudel` file.
- `note("c3'min")` when you want three notes — it is the root only.
- `.scale("C:foo")` — unknown modes error (`scale.rs`).
- Assume default octave 4 is a bass (`C:minor` is C4). Use `C2:`.
