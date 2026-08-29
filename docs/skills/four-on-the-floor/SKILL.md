---
name: four-on-the-floor
description: >-
  Use when writing a techno four-on-the-floor loop in strudel-rs
  (kick on every beat, hats on the offbeats, kick in front).
---

# Four-on-the-floor (strudel-rs)

## When to use

- The request is a **techno** pulse: kick on every quarter, hats on the offbeats, **kick in front**.
- You need a **one-track drum** `$:` (bundled `bd` / `hh`).
- You are **not** writing a house backbeat (snare on 2 and 4) unless you label that variant **house**.
- You are **not** writing 2-step (`bd ~ bd ~`) or a kick-less pad.

## Pattern (techno)

```
// @title skill-four-on-the-floor
// @genre techno
setcpm(124/4)
// drums
$: s("bd*4, [~ hh]*4").gain(0.65)
```

Keep drums on **one** `$:` (comma layers). Do not use `stack()`.

| Piece | Role |
| --- | --- |
| `setcpm(124/4)` | 124 BPM (`cycles * 4`; `src/song.rs`) |
| `bd*4` | Kick on beats 1–4 |
| `[~ hh]*4` | Closed hat on each **and** (offbeat) |
| `.gain(0.65)` | Leave headroom for a bass track later |

No snare: the kick stays the front of the grid.

### House backbeat (house only — not techno)

Snare on 2 and 4 is a **house** backbeat. Use it only when the request is house (see `songs/house16.strudel` / `songs/smoke.strudel`):

```
// @genre house
$: s("bd*4, [~ sd]*2, [~ hh]*4").gain(0.65)
```

## Why it sounds that way

`mini.rs` maps one cycle to **one bar** `[0, 1)`. `*n` copies the inner pattern *n* times across that span. Commas run layers in **parallel** over the same bar.

**Techno** (`bd*4, [~ hh]*4`):

| Event | Start in bar | Beat (1-based) |
| --- | --- | --- |
| `bd` | 0.00, 0.25, 0.50, 0.75 | 1, 2, 3, 4 |
| `hh` | 0.125, 0.375, 0.625, 0.875 | 1&, 2&, 3&, 4& |

Kicks on every quarter are the dance pulse. Offbeat hats fill the eighths. There is no snare on 2/4, so the kick is not covered by a backbeat.

`bd*4` is four copies of one atom. `[~ hh]` is rest-then-hat in a quarter-bar; `*4` tiles it four times.

**House** (`bd*4, [~ sd]*2, [~ hh]*4`) adds snares at 0.25 and 0.75. That is a backbeat, not techno four-on-the-floor.

Both decks share one `Transport`. A DJ pair must use the **same** `setcpm`. This example is 124 BPM; `songs/skill-minor-scale-loop.strudel` uses `setcpm(124/4)` for that reason.

## Try it in this app

```bash
# From the repo root (needs samples/bd, hh)
strudel-rs play songs/skill-four-on-the-floor.strudel --seconds 12

# Dual deck — both files are setcpm(124/4); do not pair a different BPM
strudel-rs dj songs/skill-four-on-the-floor.strudel songs/skill-minor-scale-loop.strudel
```

No audio device: `cargo test --test e2e skill_four_on_the_floor -- --nocapture` renders through `Engine::process`.

Live TUI: `/a load skill-four-on-the-floor` (or the `songs/` path). HTTP: `POST /song/load` with that file, or send the same `setcpm` + `$:` text as apply-song content.

## Variations (still this syntax)

| Goal | Change |
| --- | --- |
| Faster hats | `[~ hh]*4` → `hh*8` |
| House backbeat | add `[~ sd]*2` and label **house** |
| Darker kit | `.lpf(4000)` on the drum `$:` |

## Do not

- Call `bd*4, [~ sd]*2` techno — that is a house backbeat.
- Pair this file with a song at another `setcpm` (shared clock; the other tempo is discarded).
- `stack("bd*4", …)` or `.cpm(124)` — not song format.
- `bd:00` or `kit:bd` — colon is not a sample selector here.
- Bank-less `cp` — not in the default sample set.
