---
name: techno-duck
description: >-
  Use when writing a techno loop in strudel-rs with kick sidechain: duck
  pad and bass on the same orbit, short duckattack (0.03–0.05), no
  per-track compressor (it is master last-write).
---

# Techno duck / sidechain (strudel-rs)

## When to use

- The request is **techno at ~126 BPM** with a **pumping pad** and an FM or low bass.
- You need the **kick in front** of both the pad **and** the sub.
- You are **not** writing a dry four-on-the-floor with no duck, and you are **not** treating `.compressor` as a track insert.

## Pattern

```
// @title skill-techno-duck
// @genre techno
setcpm(126/4)
// kick
$: s("bd*4").gain(0.9).duckorbit(2).duckattack(0.04).duckdepth(0.85)
// drums
$: s("[~ hh]*4, [~ sd]*2").gain(0.35)
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

Same mix as the corrected `songs/techno1.strudel`.

| Piece | Role |
| --- | --- |
| `setcpm(126/4)` | 126 BPM — fine for this grid |
| `s("bd*4")` on its own `$:` | Duck **driver**. Keep the kick off orbit 2. |
| `.duckorbit(2)` | Triggers the envelope on orbit 2 (`dsp.rs` `DuckState`) |
| `.duckattack(0.04)` | Recover time **0.03–0.05** (not 0.12) |
| bass **and** pad `.orbit(2)` | Both get the duck. Bass below ~500 Hz must be on this orbit or it fights the kick. |
| no `.compressor(...)` | Pattern method is **mixer master**, last-write (`engine.rs`) |

## Why it sounds that way

Four-on-the-floor at 126 plus a held pad is techno only if the **kick punches a hole**. `DuckState` drops the target orbit **instantly**, then ramps gain back to 1.0 over `duckattack` seconds. That number is recover time, not a fade-in of the duck.

- `0.12` s at 126 BPM is ~¼ of a beat — a loose whoosh; the pad/bass stay down too long and the next kick feels soft.
- `0.03–0.05` s snaps back before the next 16th, so the pump is tight.

**Duck only the orbit you name.** `techno1` used to put the pad on orbit 2 and leave the FM bass on the default orbit (1). The bass never saw the envelope, so it kept energy under ~500 Hz and masked the kick. Put **every** low/sustained layer you want sidechained on that orbit.

**`.compressor` on a track is not a track compressor.** `deck.rs` stashes `pending_compressor`; `engine.rs` does last-write-wins onto `mixer.set_compressor`. The kick, hats, and both decks go through it. A bass-line `.compressor("-18:3:6:.003:.12")` therefore **squashes the kick**. Omit it for this recipe. If you ever set it, know it is the master bus.

Kick stays on orbit 1 (default) so it does not duck itself.

## Try it in this app

```bash
strudel-rs play songs/skill-techno-duck.strudel --seconds 12
strudel-rs play songs/techno1.strudel --seconds 12
strudel-rs dj songs/techno1.strudel songs/ambient1.strudel
```

No device: `cargo test --test e2e skill_techno_duck -- --nocapture`.

## Rules (do not skip)

1. Duck **pad and bass** (any low layer) on the same orbit as `.duckorbit(N)`.
2. `.duckattack` in **0.03–0.05** (short vs 126 BPM).
3. Do not put `.compressor` on a track unless you intend **master** glue — it is not per-track.

## Do not

- `.duckattack(0.12)` for this tempo — pump is too slow.
- FM/sub bass on orbit 1 while only the pad is on orbit 2.
- `.compressor(...)` on the bass “to control the low end”.
- `stack()` / `.cpm(126)`.
