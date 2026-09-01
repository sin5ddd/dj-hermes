---
name: strudel-genre-techno-duck
description: >-
  Use when writing a techno loop in strudel-rs with kick sidechain: duck
  pad and bass on the same orbit, short duckattack (0.03–0.05), techno
  grid bd*4 + offbeat hats, no per-track compressor (it is master last-write).
version: 4.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, techno, sidechain, duck]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-four-on-the-floor
      - strudel-genre-house
---

# Sidechain ducking (strudel-rs, techno)

## When to use

- The request is **techno at ~126 BPM** with a **pumping pad** and an FM or low bass.
- You need the **kick in front** of both the pad **and** the sub.
- You are **not** writing a dry four-on-the-floor with no duck, and you are **not** treating `.compressor` as a track insert.
- You are **not** writing a house backbeat unless you label that variant **house**.

## Pattern

```
// @title skill-sidechain-ducking
// @genre techno
setcpm(126/4)
// kick
$: s("bd*4").gain(0.9).duckorbit(2).duckattack(0.04).duckdepth(0.85)
// hats
$: s("[~ hh]*4").gain(0.4)
// bass
$: note("0 0 2 <4 6>").scale("C2:minor")
  .s("sine").fm(3).fmh(1.5).lpf(500).gain(0.55)
  .attack(0.005).decay(0.1).sustain(0.3).release(0.08)
  .orbit(2)
// pad
$: note("0 2 4 7").scale("C3:minor")
  .s("sawtooth").lpf(900).orbit(2).gain(0.35)
  .attack(0.05).decay(0.2).sustain(0.6).release(0.2)
```

Playable copy: `songs/techno-duck-01.strudel`. Techno grid (no house snare) and the bass on the ducked orbit.

| Piece | Role |
| --- | --- |
| `setcpm(126/4)` | 126 BPM |
| `s("bd*4")` on its own `$:` | Duck **driver**. Hats must not carry `duckorbit` (each hat would retrigger the envelope). |
| `.duckorbit(2)` | Triggers `DuckState` on orbit 2 (`dsp.rs`) |
| `.duckattack(0.04)` | Recover time **0.03–0.05** at this BPM (not 0.12) |
| bass **and** pad `.orbit(2)` | Both get the duck. Unducked FM bass under ~500 Hz is the failure mode. |
| `s("[~ hh]*4")` | Techno offbeat hats. **No** `[~ sd]*2` — that is a house backbeat. |
| no `.compressor(...)` | Pattern method is **mixer master**, last-write (`engine.rs`) |

## Why it sounds that way

Four-on-the-floor at 126 plus a held pad is techno only if the **kick punches a hole**. `DuckState` (`dsp.rs`) drops the target orbit **instantly**, then ramps gain back to 1.0 over `duckattack` seconds. That number is recover time, not a fade-in of the duck.

At 126 BPM a quarter is ~0.476 s and a 16th is ~0.119 s:

- `0.12` s is about one 16th — a loose whoosh; pad and bass stay down too long and the next kick feels soft.
- `0.03–0.05` s snaps back before the next 16th, so the pump is tight.

**Duck the low orbit as well as the mids.** `duckorbit(N)` only scales the orbit you name (`deck.rs` applies `DuckState::advance` per orbit after voice sum). A pad on orbit 2 with the FM bass left on the default orbit (1) means the bass never sees the envelope and keeps fighting the kick. Put **every** low/sustained layer you want sidechained on that orbit.

Kick stays on orbit 1 (default) so it does not duck itself.

**Techno grid, not house.** The dance pulse here is `bd*4` plus `[~ hh]*4` (kick in front, hats on the offbeats). `[~ sd]*2` puts snares on 2 and 4 — a **house** backbeat. Use that only when the request is house.

**`.compressor` on a `$:` is not a track insert.** `deck.rs` stashes `pending_compressor`; `engine.rs` does last-write-wins onto `mixer.set_compressor`. The kick, hats, and both decks go through it after faders and EQ. A bass-line `.compressor("-18:3:6:.003:.12")` therefore **squashes the kick**. Omit it for this recipe. If you want master glue, add `.compressor(...)` as the last method on a late `$:` and know it is the master bus, not that track.

Both decks share one `Transport`. This file is 126 BPM — pair it with `songs/electro-01.strudel` (also `setcpm(126/4)`). Do **not** pair it with 70 BPM ambient or the 174 DnB skill.

## Try it in this app

```bash
strudel-rs play songs/techno-duck-01.strudel --seconds 12
strudel-rs play songs/techno-duck-01.strudel --headless --seconds 8

# Dual deck — both files are setcpm(126/4)
strudel-rs dj songs/techno-duck-01.strudel songs/electro-01.strudel
```

No device: `cargo test --test e2e techno_duck_01 -- --nocapture`.

Live TUI: `/a load techno-duck-01`. Then `/x 4` to crossfade toward B (equal-power, bar-quantized).

## Variations (still this syntax)

| Goal | Change |
| --- | --- |
| Split low / mid orbits | bass `.orbit(2)`, pad `.orbit(3)`, kick `.duckorbit("2:3")` |
| Deeper pump | `.duckdepth(0.95)` (still `0.03–0.05` attack) |
| House backbeat | add `[~ sd]*2` on the hats `$:` **and label the song house** |
| Master glue | `.compressor("-18:3:6:.003:.12")` on the **last** `$:` — mixer last-write, not a pad insert |

## Rules (do not skip)

1. Duck **pad and bass** (any low layer) on the same orbit as `.duckorbit(N)`.
2. `.duckattack` in **0.03–0.05** at ~126 BPM.
3. Techno drums are `bd*4` + `[~ hh]*4`. `[~ sd]*2` only if you label **house**.
4. Do not put `.compressor` on the bass — it is master last-write.

## Do not

- `.duckattack(0.12)` for this tempo — pump is too slow.
- FM/sub bass on orbit 1 while only the pad is on orbit 2.
- `.compressor(...)` on the bass “to control the low end”.
- Call `bd*4` + `[~ sd]*2` techno — that is a house backbeat.
- Pair this file with a 174 BPM DnB song (shared clock; the other tempo is discarded).
- `stack()` / `.cpm(126)`.
