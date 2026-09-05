---
name: strudel-genre-four-on-the-floor
description: >-
  Use when writing a techno four-on-the-floor loop in strudel-rs
  (kick on every beat, hats on the offbeats, kick in front).
  7–8 $: tracks, 4-bar phrase, no clap.
version: 5.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, techno, four-on-the-floor]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-house
      - strudel-genre-techno-duck
      - strudel-minor-scale-loop
---

# Four-on-the-floor (strudel-rs)

## When to use

- The request is a **techno** pulse: kick on every quarter, hats on the offbeats, **kick in front**.
- Drums stay on **one** `$:` (bundled `bd` / `hh`). New apply / preset is a **7-track bed**, not drums alone.
- You are **not** writing a house clap backbeat (`[~ cp]*2` on 2 and 4). That is [strudel-genre-house](../strudel-genre-house/SKILL.md).
- You are **not** writing 2-step (`bd ~ bd ~`) or a kick-less pad. **No clap** on this grid.

## Pattern (pulse identity)

The dance pulse is this one-track drum string. Keep it as the grid; do not grow a clap.

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

No snare and **no clap**: the kick stays the front of the grid.

### House backbeat (not this file)

House 2/4 is **`[~ cp]*2`** (bundled clap), not `[~ sd]*2`, and not stacked with `sd`. Do not add that layer to this techno grid. Recipe: [strudel-genre-house](../strudel-genre-house/SKILL.md).

## Pattern (full song)

7 tracks (one bass, perc not required). 4th-bar drum fill on the same `$:` as the pulse. Pitched tracks use a 4-bar `.scale("<…>")`.

```
// @title skill-four-on-the-floor
// @genre techno
setcpm(124/4)
// drums
$: s("bd*4, [~ hh]*4, <~ ~ ~ [bd sd bd sd]>").gain(0.62)
// bass
$: note("0 0 2 <4 0 3 0>").scale("<C2:minor C2:minor G2:phrygian C2:minor>")
  .s("sawtooth").lpf(400).gain(0.44)
  .attack(0.001).decay(0.08).sustain(0.2).release(0.05)
// lead
$: note("~ 7 6 <4 9 3 7>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("square").lpf(2800).gain(0.15)
// hook
$: note("~ 4 ~ <7 4 4 7>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("plk:s5").gain(0.18).cut(1)
// arp
$: note("0 3 0 7  3 0 5 ~").scale("<C5:minor C5:minor G5:phrygian C5:minor>")
  .s("plk:ac").gain(0.12).cut(1)
// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("<C3:minor C3:minor G3:phrygian C3:minor>")
  .s("triangle").lpf(1200).gain(0.22)
// pad
$: note("[0,4]").scale("<C3:minor C3:minor G3:phrygian C3:minor>")
  .s("sawtooth").lpf(800).orbit(2).gain(0.16)
  .attack(0.08).decay(0.2).sustain(0.7).release(0.4)
```

`songs/four-on-the-floor/01.strudel` matches the full-song fence.

Bass is a synth sub at **C2** (`sawtooth` + `lpf(400)`). Do not stack another sub (`bs:su` / `bs:hf` / `bs:dk` / a second `square`+low lpf). PCM hook/arp stay at **C4/C5**. Chords `[0,2,4]`, pad `[0,4]`. Lead / hook / arp rest on different slots.

## Why it sounds that way

`mini.rs` maps one cycle to **one bar** `[0, 1)`. `*n` copies the inner pattern *n* times across that span. Commas run layers in **parallel** over the same bar.

**Techno** (`bd*4, [~ hh]*4`):

| Event | Start in bar | Beat (1-based) |
| --- | --- | --- |
| `bd` | 0.00, 0.25, 0.50, 0.75 | 1, 2, 3, 4 |
| `hh` | 0.125, 0.375, 0.625, 0.875 | 1&, 2&, 3&, 4& |

Kicks on every quarter are the dance pulse. Offbeat hats fill the eighths. There is no snare on 2/4, so the kick is not covered by a backbeat.

`bd*4` is four copies of one atom. `[~ hh]` is rest-then-hat in a quarter-bar; `*4` tiles it four times.

The full-song drums add `<~ ~ ~ [bd sd bd sd]>`: bars 1–3 stay the pulse; bar 4 is a fill. That is still kick-front techno, not a house clap.

**House** (`bd*4, [~ cp]*2, [~ hh]*4`) adds claps at 0.25 and 0.75. That is a backbeat, not techno four-on-the-floor. See [strudel-genre-house](../strudel-genre-house/SKILL.md). Do not stack `sd` on those hits.

Both decks share one `Transport`. A DJ pair must use the **same** `setcpm`. This example is 124 BPM; `songs/house/01.strudel` uses `setcpm(124/4)` for that reason.

## Try it in this app

```bash
# From the repo root (needs samples/bd, hh)
strudel-rs play songs/four-on-the-floor/01.strudel --seconds 12

# Dual deck — both files are setcpm(124/4); do not pair a different BPM
strudel-rs dj songs/house/01.strudel songs/four-on-the-floor/01.strudel
```

Load `songs/four-on-the-floor/01.strudel`, or `strudel_apply_song` the full-song fence.

No audio device: `cargo test --test e2e four_on_the_floor -- --nocapture` renders through `Engine::process`.

Live TUI: `/a load four-on-the-floor-01` (or the `songs/` path). HTTP: `POST /song/load` with that file, or send the same `setcpm` + `$:` text as apply-song content.

## Variations (still this syntax)

| Goal | Change |
| --- | --- |
| Faster hats | `[~ hh]*4` → `hh*8` |
| House clap backbeat | do **not** add it here — use [strudel-genre-house](../strudel-genre-house/SKILL.md) |
| Darker kit | `.lpf(4000)` on the drum `$:` |

## Checklist

- [ ] Pulse identity remains `bd*4, [~ hh]*4` (no clap)
- [ ] New apply is **7 `$:`** (drums, bass, lead, hook, arp, chords, pad)
- [ ] 4-bar phrase on pitched tracks; drums 1-bar + 4th-bar fill is OK
- [ ] Synth bass at `C2:`. PCM at `C4:` / `C5:`. Chords `[0,2,4]`, pad `[0,4]`
- [ ] `songs/four-on-the-floor/01.strudel` matches the full-song fence

## Do not

- Call `bd*4, [~ cp]*2` techno — that is a house backbeat (see [strudel-genre-house](../strudel-genre-house/SKILL.md)).
- Put `[~ cp]*2` on this kick-front grid. Techno: **no clap**.
- Ship drums-only for a new apply.
- Pair this file with a song at another `setcpm` (shared clock; the other tempo is discarded).
- Write `in_bank=no` PCM (`ld:ac`, `pf:al`, `dr:*`, `ps:*`). Allowed long PCM: `ld:ss`, `pf:ff`.
- Stack subs (`bs:su` / `bs:hf` / `bs:dk` / a second `square`+low lpf).
- `stack("bd*4", …)` or `.cpm(124)` or `.lfo()` — not song format.
- `kit:bd` — bank does not go on the left. Catalog one-shots use `bd:hf` (see strudel-pcm-catalog). Default kit remains `s("bd")`.
