---
name: house-clap-backbeat
description: >-
  Use when writing a house loop in strudel-rs: clap on 2 and 4
  ([~ cp]*2), not a snare and not stacked with sd, plus the C-minor
  FM pluck at C4:minor. Not kick-front techno.
---

# House clap backbeat (strudel-rs)

## When to use

- The request is **house**: four-on-the-floor kick, **clap on 2 and 4**, offbeat hats, ~124 BPM.
- You need the bundled house clap (`s("cp")` → `samples/cp/00.wav`) and/or the C3 FM pluck (`s("lead-fm_pluck")`).
- You are **not** writing kick-front techno (`bd*4, [~ hh]*4` with no clap) — that is [four-on-the-floor](../four-on-the-floor/SKILL.md).
- You are **not** stacking `sd` on the same 2/4 hits. You are **not** using `stab-fm_fifth` or `reese-mid` (next DnB skill).

## Pattern

```
// @title skill-house-clap-backbeat
// @genre house
setcpm(124/4)
// drums
$: s("bd*4, [~ cp]*2, [~ hh]*4").gain(0.65)
// pluck
$: note("4 ~ 7 4  2 0 ~ -1").scale("C4:minor").s("lead-fm_pluck").gain(0.4).cut(1)
```

Playable copy: `songs/skill-house-clap-backbeat.strudel`. Do not “improve” the degrees. Supersaw lead is a different stem (`lead-supersaw` at `C4:minor`) — [factory-pcm-usage](../factory-pcm-usage/SKILL.md).

| Piece | Role |
| --- | --- |
| `setcpm(124/4)` | **124 BPM** (house). Same clock as the 124 techno / minor-scale skills. |
| `bd*4` | Kick on 1–4 |
| `[~ cp]*2` | House clap on **2 and 4** (`samples/cp/00.wav`) |
| `[~ hh]*4` | Closed hat on each **and** |
| `note("4 ~ 7 4  2 0 ~ -1")` | Eighths: G–rest–**C**–G–Eb–C–**rest**–Bb |
| `.scale("C4:minor")` | Degrees against C4. The wav is C3; `C3:minor` dumps the line into the bass register |
| `.s("lead-fm_pluck")` | Flat file `samples/lead-fm_pluck.wav` (underscore before `pluck`) |
| `.cut(1)` | Steal the previous pluck (~0.4 s one-shot must not overlap itself) |
| no `.compressor` / no `.duckorbit` | Compressor is mixer last-write. No duck in this recipe |

## Why `[~ cp]*2`, not `[~ sd]*2`

`[~ x]*2` tiles rest-then-hit across the bar, so hits land at 0.25 and 0.75 (beats 2 and 4). That grid is the house backbeat.

`s("cp")` is the bundled **dry house clap** (`samples/cp/00.wav`). `s("sd")` is a **snare** (`samples/sd/00.wav`). They occupy the same 2/4 slots and **mask each other** — do not write `[~ cp, sd]*2` or `bd*4, [~ cp]*2, [~ sd]*2`. Do not treat `sd` as the house clap.

Do **not** put this clap layer on a techno drum string. Kick-front techno is `bd*4, [~ hh]*4` with no 2/4 backbeat ([four-on-the-floor](../four-on-the-floor/SKILL.md)).

`s("cp")` resolves as a SampleBank folder key, not a flat `cp.wav`:

| Disk | Sound key | How |
| --- | --- | --- |
| `samples/cp/00.wav` | `cp` | `load_dir` reads `dir/<name>/*.wav` (`sample.rs`). Default variation is sort order → `00.wav` |
| `samples/cp.wav` | `cp` | Would also work as a single flat file. **Not in this repo.** If both existed, folder vars win |
| `samples/lead-fm_pluck.wav` | `lead-fm_pluck` | Flat stem. Write the underscore; `lead-fm-pluck` does not resolve |

`cp` is not a waveform (`sound.rs`), so it falls through to the bank. Unknown names fail resolve and the current performance continues.

## Why C4:minor on a C3 sample

`note().s("sample")` sets playback rate to `target_hz / SAMPLE_ROOT_HZ` (`deck.rs`). `SAMPLE_ROOT_HZ` is **261.63 Hz (C4)** even though the constant comment says C3.

`lead-fm_pluck.wav` is a **C3** one-shot (~131 Hz / MIDI 48). Ratio 1.0 already sounds as C3.

| Scale | Degree 0 target | Ratio | What you hear |
| --- | --- | --- | --- |
| `.scale("C4:minor")` | C4 = 261.63 Hz | 1.0 | Recorded C3 — mid pluck |
| `.scale("C3:minor")` | C3 ≈ 130.8 Hz | ~0.5 | ~65 Hz (C2) — bass register |

Use **C4:minor**. `C3:minor` is the dump.

C minor (Aeolian) intervals are `[0, 2, 3, 5, 7, 8, 10]`. Degrees `4 ~ 7 4  2 0 ~ -1`:

| Degree | Interval | Note (written C4:minor) | Role |
| --- | --- | --- | --- |
| 4 | P5 | G | pickup |
| ~ | | | rest |
| 7 | octave | C | **lands with the clap on beat 2** (house hook) |
| 4 | P5 | G | |
| 2 | m3 | Eb | |
| 0 | unison | C | |
| ~ | | | rest on beat 4 (clap is alone) |
| −1 | b7 below root | Bb | after the beat-4 clap; intentional |

That is G–C–G–Eb–C–Bb (5–1–5–b3–1–b7). Do not rewrite it.

Eight atoms = eighths. At 124 BPM an eighth is ~0.242 s. The pluck one-shot is ~0.4 s, so a new note starts before the previous wav ends. `.cut(1)` puts the lead on cut group 1; `Deck::alloc_voice` drops earlier voices in that group (`deck.rs`). Without it the tails stack.

**Chord suffixes:** `note("c3'maj")` still plays the **root only** (`deck.rs` does not call `expand_chord`). This song uses degrees, not suffixes.

## Why 124, not techno

House in this tree is ~120–128. This file is **124**. Techno kick-front examples may also sit at 124 for a shared clock, but they must **not** grow a clap. A 130 acid / 126 duck / 174 DnB file is a different tempo — do not pair them.

Both decks share one `Transport`. A DJ pair must use the **same** `setcpm`. Pair this file with `songs/skill-minor-scale-loop.strudel` or `songs/skill-four-on-the-floor.strudel` (both `setcpm(124/4)`). Do not pair `songs/house16.strudel` (122) or a techno file that is not 124.

No `.compressor` on a `$:` — that writes the **mixer master** (last-write) and will squash the kick. No duck work in this skill.

## Try it in this app

```bash
strudel-rs play songs/skill-house-clap-backbeat.strudel --seconds 12
strudel-rs play songs/skill-house-clap-backbeat.strudel --headless --seconds 8

# Dual deck — both files are setcpm(124/4). Do not pair a different BPM.
strudel-rs dj songs/skill-house-clap-backbeat.strudel songs/skill-minor-scale-loop.strudel
```

No device: `cargo test --test e2e skill_house_clap -- --nocapture`.

Live TUI: `/a load skill-house-clap-backbeat`.

## Variations (still this syntax)

| Goal | Change |
| --- | --- |
| Faster hats | `[~ hh]*4` → `hh*8` (keep `[~ cp]*2`) |
| Darker kit | `.lpf(4000)` on the drum `$:` |
| New root, same shape | `.scale("A4:minor")` (keep the octave-4 root so the C3 wav stays mid) |

Do not add `[~ sd]*2`. Do not drop the clap onto the techno skill song.

## Do not

- Write `[~ sd]*2` or stack `sd` with `cp` on 2/4 — they mask each other.
- Put `[~ cp]*2` on kick-front techno (`bd*4, [~ hh]*4`).
- Use `.scale("C3:minor")` with this wav — it plays in the bass register.
- Write `lead-fm-pluck` (hyphen) — the key is `lead-fm_pluck`.
- Use `stab-fm_fifth` / `reese-mid` here.
- `note("c3'maj")` when you want a chord — suffix is root only.
- Put `.compressor` on a track. Do not add `.duckorbit`.
- Pair this file with another `setcpm` (shared clock; the other tempo is discarded).
- `stack()` / `.cpm(124)`.
