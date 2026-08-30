---
name: factory-pcm-usage
description: >-
  Use when triggering issue #21 batch 1 factory PCM stems in
  strudel-rs: how to play each sample (C4:… for native pitch,
  unpitched FX never note()). Not a genre recipe.
---

# How to play factory PCM samples (batch 1)

This skill is **how each factory one-shot is triggered** — keys, register, and
what the wav already contains. It is **not** a house / techno / DnB loop recipe.
Those skills own the drum grids. The 124 house song below is only a beatmaker
check that the signed-off stems sit together.

Existing stems stay on their own skills. Do not rewrite those recipes:

| Stem | Skill |
| --- | --- |
| `reese-mid` | [dnb-reese-mid-stab](../dnb-reese-mid-stab/SKILL.md) |
| `lead-fm_pluck` | [house-clap-backbeat](../house-clap-backbeat/SKILL.md) |
| Live 2-op `.fm` | [fm-sound-design](../fm-sound-design/SKILL.md) (PCM hits are not that path) |

This batch adds **no** `bd/` bank and **no** third-party drum kit (issue #21
license). Drums stay the bundled folder keys `bd` / `cp` / `hh` / `sd` / `oh`.

## When to use

- You need the signed-off `.s(...)` / `.scale(...)` for a **batch 1** factory wav.
- You need to know **why C4 plays native pitch**, **why FX have no `note()`**,
  **why `pad-fm_fifth` cannot brighten**, **why `reese-dark` ≠ `reese-mid`**.
- You are **not** inventing a genre grid. Do not “improve” the beatmaker song.

## Engine: `SAMPLE_ROOT_HZ` is C4

`note().s("sample")` sets playback rate to `target_hz / SAMPLE_ROOT_HZ`
(`deck.rs`). `SAMPLE_ROOT_HZ` is **261.63 Hz (C4)** even though the constant
comment in `sample.rs` says C3.

Ratio **1.0** plays the recording at its **native** pitch. All pitched one-shots
in this batch were recorded at **C3 or C2**. Writing `.scale("C4:…")` on
degree 0 targets C4, so the ratio is 1.0 and you hear C2 or C3 as recorded.
Writing `C3:…` targets C3 (~130.8 Hz) → ratio **0.5** → an **extra octave down**.

| Written scale (degree 0) | Target Hz | Ratio | Heard (C3 wav) | Heard (C2 wav) |
| --- | --- | --- | --- | --- |
| `.scale("C4:minor")` | 261.63 (C4) | 1.0 | native C3 | native C2 |
| `.scale("C3:minor")` | ~130.8 (C3) | 0.5 | C2 (dumped) | C1 (dumped) |
| `.scale("C2:minor")` | ~65.4 (C2) | 0.25 | C1 | C0 |

Always write **`C4:…`** for native pitch on this batch. Do not “match” the
recorded octave in the scale string.

Bare `s("name")` (no `note()`) sets `is_note = false` (`code.rs`). Then
`pitch_ratio` is **1.0** (`deck.rs`) — the wav plays as recorded, unpitched.

## Pitched stems (always `C4:…` for native pitch)

`load_dir` keys a loose WAV as `file_stem()` lowercased (`sample.rs`). Write the
stem exactly. Hyphen and underscore are different characters.

### `bass-fm_house` — recorded C2, house floor under the kick

```
setcpm(124/4)
$: note("0 0 4 0").scale("C4:minor").s("bass-fm_house").gain(0.45)
```

Native **C2** (~65 Hz). Tight house floor. `C4:minor` yields that C2. Do **not**
put it at `C3:…` — musically that would be a mid-bass **on top of the kick**,
and the engine would dump an extra octave anyway (you hear C1, not C3).
Key is `bass-fm_house` (underscore before `house`). Not `bass-fm-house`.

### `bass-fm_sub` — recorded C2 (~65 Hz), floor only

```
setcpm(124/4)
$: note("0 0 4 0").scale("C4:minor").s("bass-fm_sub").gain(0.5)
```

Clean sine sub. `C4:…` yields native C2. **Floor only.** Do **not** stack with
`reese-dark` or a square sub (double basement). Not in the pad+reese demo.

### `reese-dark` — dark Reese **with** ~65 Hz sub

```
setcpm(124/4)
$: note("0 3 0 <0 -1>").scale("C4:minor").s("reese-dark").gain(0.35)
```

Full-range dark Reese (sub + mid). **Not** a band-swap for `reese-mid`.
`reese-mid` is 800–1200 Hz glue with **no** sub and still needs a square C2
([dnb-reese-mid-stab](../dnb-reese-mid-stab/SKILL.md)). `reese-dark` already
owns ~65 Hz. Do **not** stack with square sub or `bass-fm_sub`.
Key is `reese-dark` (hyphen). Not `reese_dark`.

### `pad-fm_fifth` — hollow C3+G3, **no third**

```
setcpm(124/4)
$: note("0 ~ 0 ~").scale("C4:minor").s("pad-fm_fifth").gain(0.25)
```

Sustained fifth pad (C and G only). `note()` only **transposes** that recording.
It cannot invent a major (or minor) third. Do **not** use this to brighten.
Do **not** play it as `note("[0,2,4]")` — that stacks three hollow fifths
(root / third / fifth), still no E or Eb inside the wav.
Key is `pad-fm_fifth` (underscore before `fifth`). Not `pad-fm-fifth`.
Not a swap for `stab-fm_fifth` (that one is a short stab).

### `lead-supersaw` — recorded C3, melody at `C4:minor`

Same register rule as `lead-fm_pluck`. Do **not** stack this on the same mids
as the pad+reese demo (it fills up). Separate song at the same 124 clock.

```
setcpm(124/4)
$: note("4 ~ 7 4").scale("C4:minor").s("lead-supersaw").gain(0.28).cut(1)
```

The wav is a long hold (~8 s). `.cut(1)` steals the previous shot so the
melody stays monophonic. Key is `lead-supersaw` (hyphen, no underscore).
Not `lead_supersaw`. Do not rewrite the [house-clap-backbeat](../house-clap-backbeat/SKILL.md)
pluck line to this stem.

## Unpitched FX — `s("name")` only, **never `note()`**

These four are gestures, not pitched instruments. Bare `s("…")`. No `.scale`.

| Disk | Sound key | Role |
| --- | --- | --- |
| `samples/fx-uplifter.wav` | `fx-uplifter` | Uplifter (pitch + filter open, ~2.8 s) |
| `samples/fx-riser_noise.wav` | `fx-riser_noise` | Noise riser (~3.2 s). Underscore before `noise` |
| `samples/fx-impact_dnb.wav` | `fx-impact_dnb` | DnB impact (~0.5 s). Underscore before `dnb` |
| `samples/fx-sub_drop.wav` | `fx-sub_drop` | Sub drop (~50 Hz, ~1.1 s). Underscore before `drop` |

```
setcpm(124/4)
$: s("fx-uplifter").gain(0.3)
$: s("fx-riser_noise").gain(0.28)
$: s("fx-impact_dnb").gain(0.35)
$: s("fx-sub_drop").gain(0.35)
```

`fx-sub_drop` is already ~50 Hz. It still gets **no** `note()`. Kick-lead-in,
not a bass note. Wrong keys (`fx-riser-noise`, `fx_uplifter`) fail resolve;
the current performance continues.

## Why C4 vs recorded C2 / C3

The engine does not know the recording key. It always divides by **C4**.

A C2 house bass recorded at ~65 Hz is **already** the floor. Degree 0 at
`C4:minor` leaves the rate at 1.0, so you hear that C2 under the kick.
Writing `C3:minor` (or `C2:minor`) is “I want the written octave” — the
ratio drops to 0.5 or 0.25 and the floor disappears an octave (or two).

A C3 pad / Reese / supersaw works the same way: `C4:…` = native C3.
`C3:…` dumps them into the bass register. That is why `lead-fm_pluck` and
`reese-mid` already use `C4:minor` — same constant, same rule.

## Why FX have no `note()`

`s("fx-uplifter")` is not a note head (`code.rs` `is_note = false`).
`deck.rs` then uses `pitch_ratio = 1.0`. The riser / impact / drop already
has its pitch motion **baked into the wav**.

`note("0").scale("C4:minor").s("fx-sub_drop")` would retune a ~50 Hz drop
by `target / C4`. That stretches the gesture and moves the basement. Leave
FX unpitched even when the recording is low.

## Why `pad-fm_fifth` cannot brighten

The wav is **C3+G3** (ratios 1 and 3/2). No third in the file.

`note().s("pad-fm_fifth")` only changes playback rate. Deck scheduling still
plays **one pitch per mini event**; `expand_chord` is not called.
`note("c3'maj")` is the **root only**.

`note("[0,2,4]")` fires three events (degrees 0, 2, 4). Each event is still
the hollow fifth, transposed. You hear stacked C–G / Eb–Bb / G–D — **not**
a major triad and **not** a brighter pad. Brightening in this engine is a
mode / voicing / sample swap ([mood-bright-dark](../mood-bright-dark/SKILL.md)),
not this stem.

## Why `reese-dark` ≠ `reese-mid`

| | `reese-mid` | `reese-dark` |
| --- | --- | --- |
| Band | 800–1200 Hz, **no sub** | Full-range, **with ~65 Hz sub** |
| Scale | `C4:minor` (native C3 mid) | `C4:minor` (native C3 + baked sub) |
| Needs a synth sub? | Yes — square at `C2:minor` | **No** |
| Swap? | Not a dark version of the other | Not a mid-band replacement |

Do not put `reese-dark` on the DnB mid track and drop the square. Do not put
`reese-mid` under this house demo. Do not stack `reese-dark` with
`bass-fm_sub` or `s("square")` + `lpf(120)`.

## Sample keys (flat stems)

One-level load only. Flat file `samples/<stem>.wav` → key `<stem>` lowercased.
`{family}-{character}_{detail}`: hyphen between family and character, underscore
before the detail. Family-character-only names are hyphen only.

| Disk | Sound key | Silent (wrong) |
| --- | --- | --- |
| `samples/bass-fm_house.wav` | `bass-fm_house` | `bass-fm-house` |
| `samples/bass-fm_sub.wav` | `bass-fm_sub` | `bass-fm-sub` |
| `samples/reese-dark.wav` | `reese-dark` | `reese_dark` |
| `samples/pad-fm_fifth.wav` | `pad-fm_fifth` | `pad-fm-fifth` |
| `samples/lead-supersaw.wav` | `lead-supersaw` | `lead_supersaw` |
| `samples/fx-riser_noise.wav` | `fx-riser_noise` | `fx-riser-noise` |
| `samples/fx-impact_dnb.wav` | `fx-impact_dnb` | `fx-impact-dnb` |
| `samples/fx-sub_drop.wav` | `fx-sub_drop` | `fx-sub-drop` |
| `samples/fx-uplifter.wav` | `fx-uplifter` | `fx_uplifter` |

No `.bank(...)` on these names. This batch did not add a `bd/` kit.

## Playable songs (do not “improve”)

Beatmaker 124 house grid — signed-off degrees and gains. Pad + dark Reese
together; house floor under the kick; one unpitched uplifter.

```
// @title skill-factory-pcm-usage
setcpm(124/4)
$: s("bd*4, [~ cp]*2, [~ hh]*4").gain(0.65)
$: note("0 0 4 0").scale("C4:minor").s("bass-fm_house").gain(0.45)
$: note("0 ~ 0 ~").scale("C4:minor").s("pad-fm_fifth").gain(0.25)
$: note("0 3 0 <0 -1>").scale("C4:minor").s("reese-dark").gain(0.35)
$: s("fx-uplifter").gain(0.3)
```

Copy: `songs/skill-factory-pcm-usage.strudel`.

### Lead only (same clock, not stacked on that mid bed)

```
// @title skill-factory-pcm-lead
setcpm(124/4)
$: s("bd*4, [~ cp]*2, [~ hh]*4").gain(0.65)
$: note("0 0 4 0").scale("C4:minor").s("bass-fm_house").gain(0.45)
$: note("4 ~ 7 4").scale("C4:minor").s("lead-supersaw").gain(0.28).cut(1)
```

Copy: `songs/skill-factory-pcm-lead.strudel`. Shared `setcpm(124/4)` so the
pair can `dj`. Do not add pad + `reese-dark` on this file.

## Try it in this app

```bash
strudel-rs play songs/skill-factory-pcm-usage.strudel --seconds 12
strudel-rs play songs/skill-factory-pcm-usage.strudel --headless --seconds 8

strudel-rs play songs/skill-factory-pcm-lead.strudel --seconds 12
strudel-rs play songs/skill-factory-pcm-lead.strudel --headless --seconds 8

# Dual deck — both files are setcpm(124/4)
strudel-rs dj songs/skill-factory-pcm-usage.strudel songs/skill-factory-pcm-lead.strudel
```

No device: `cargo test --test e2e skill_factory_pcm -- --nocapture`.

Live TUI: `/a load skill-factory-pcm-usage`.

## Do not

- Write `C3:…` or `C2:…` on these pitched stems — that dumps octaves.
- Put `bass-fm_house` at C3 (on top of the kick) or hear it as a mid-bass.
- Stack `reese-dark` with `bass-fm_sub` or square sub.
- Treat `reese-dark` as a darker `reese-mid` (or the reverse).
- Use `pad-fm_fifth` to brighten, or play it as `[0,2,4]`.
- Put `note()` / `.scale` on `fx-uplifter`, `fx-riser_noise`, `fx-impact_dnb`,
  or `fx-sub_drop` (including the ~50 Hz drop).
- Stack `lead-supersaw` on the pad+reese demo.
- Rewrite `reese-mid` / `lead-fm_pluck` / live 2-op recipes to these stems.
- Add a `bd/` bank or a third-party drum kit from this batch.
- Write `bass-fm-house`, `pad-fm-fifth`, `reese_dark`, `fx-riser-noise`.
- Put `.compressor` on a `$:` (mixer master, last-write).
- Pair these 124 files with 174 DnB or 126 techno (shared clock).
- `stack()` / `.cpm(124)`.
