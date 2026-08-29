---
name: four-on-the-floor
description: >-
  Use when writing a house, techno, or disco drum loop in strudel-rs
  (kick on every beat, snare on 2 and 4, hats on the offbeats).
---

# Four-on-the-floor (strudel-rs)

## When to use

- The request is a **basic beat**, house/techno/disco pulse, or “kick on every quarter”.
- You need a **one-track drum** `$:` that this engine can play (bundled `bd` / `sd` / `hh`).
- You are **not** writing 2-step (`bd ~ bd ~`), a DnB break, or a kick-less pad.

## Pattern

```
// @title skill-four-on-the-floor
// @genre house
setcpm(124/4)
// drums
$: s("bd*4, [~ sd]*2, [~ hh]*4").gain(0.65)
```

Same skeleton as `songs/smoke.strudel` and `songs/house16.strudel`. Keep drums on **one** `$:` (comma layers). Do not use `stack()`.

| Piece | Role |
| --- | --- |
| `setcpm(124/4)` | 124 BPM (`cycles * 4`; `src/song.rs`) |
| `bd*4` | Kick on beats 1–4 |
| `[~ sd]*2` | Snare on beats 2 and 4 |
| `[~ hh]*4` | Closed hat on each **and** (offbeat) |
| `.gain(0.65)` | Leave headroom for a bass track later |

Optional open hat (still one track), as in `songs/house16.strudel`:

```
$: s("bd*4, [~ sd]*2, [~ hh]*4, [~ oh]*2").gain(0.55)
```

## Why it sounds that way

`mini.rs` maps one cycle to **one bar** `[0, 1)`. `*n` copies the inner pattern *n* times across that span. Commas run layers in **parallel** over the same bar.

| Event | Start in bar | Beat (1-based) |
| --- | --- | --- |
| `bd` | 0.00, 0.25, 0.50, 0.75 | 1, 2, 3, 4 |
| `sd` | 0.25, 0.75 | 2, 4 |
| `hh` | 0.125, 0.375, 0.625, 0.875 | 1&, 2&, 3&, 4& |

That is the classic four-on-the-floor + backbeat + offbeat-hat grid. Kicks on every quarter read as dance pulse; snares on 2/4 mark the backbeat; hats on the “and” fill the eighth-note offbeats.

`bd*4` is four copies of one atom, not “four different kicks”. `[~ sd]` is a two-slot cell (rest, then snare); `*2` places that cell in each half-bar, so the snare lands at 25% and 75%. Same idea for hats: `[~ hh]` is rest-then-hat in a quarter-bar, `*4` tiles it four times.

Tempo 120–128 BPM is house/techno range. The **grid** is what reads as the genre; `setcpm` only sets how fast that grid goes by.

## Try it in this app

```bash
# From the repo root (needs samples/bd, sd, hh)
strudel-rs play songs/skill-four-on-the-floor.strudel --seconds 12

# Dual deck against a harmonic loop
strudel-rs dj songs/skill-four-on-the-floor.strudel songs/skill-minor-scale-loop.strudel
```

No audio device: `cargo test --test e2e skill_four_on_the_floor -- --nocapture` renders through `Engine::process`.

Live TUI: `/a load skill-four-on-the-floor` (or the `songs/` path). HTTP: `POST /song/load` with that file, or send the same `setcpm` + `$:` text as apply-song content.

## Variations (still this syntax)

| Goal | Change |
| --- | --- |
| Faster hats | `[~ hh]*4` → `hh*8` |
| Occasional open hat | `[~ <sd oh>]*2` (see `songs/techno16.strudel`) |
| Darker kit | `.lpf(4000)` on the drum `$:` |
| Separate ducking kick | Extra `$:` `s("bd*4").duckorbit(2)` — only when a pad is on orbit 2 |

## Do not

- `stack("bd*4", …)` or `.cpm(124)` — not song format.
- `bd:00` or `kit:bd` — colon is not a sample selector here.
- Bank-less `cp` — not in the default sample set.
- Treat `c3'maj` as a drum hit — that is a note token, not a sample name.
