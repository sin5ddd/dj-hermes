---
name: strudel-genre-house
description: >-
  Use when writing a house loop in strudel-rs: clap on 2 and 4
  ([~ cp]*2), not a snare and not stacked with sd, plus the C-minor
  FM pluck at C4:minor as the hook. 7–8 $: tracks, 4-bar phrase.
  Not kick-front techno.
version: 5.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [strudel-rs, music, genre, house, clap]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-four-on-the-floor
      - strudel-minor-scale-loop
      - strudel-mood-bright-dark
---

# House clap backbeat (strudel-rs)

## When to use

- The request is **house**: four-on-the-floor kick, **clap on 2 and 4**, offbeat hats, ~124 BPM.
- You need the bundled house clap (`s("cp")` → `samples/cp/00.wav`) and/or the C3 FM pluck (`s("plk:lp")`) as the **hook**.
- New apply / preset: **7 `$:` tracks** (one bass, no perc required). Do not ship a 2-track loop.
- You are **not** writing kick-front techno (`bd*4, [~ hh]*4` with no clap) — that is [strudel-genre-four-on-the-floor](../strudel-genre-four-on-the-floor/SKILL.md).
- You are **not** stacking `sd` on the same 2/4 hits. You are **not** using `plk:s5` or `bs:rm` (next DnB skill). You are **not** adding `bs:su`.

## Pattern

```
// @title skill-house-clap-backbeat
// @genre house
setcpm(124/4)
// drums
$: s("bd:hf*4, [~ cp]*2, [~ hh:hs]*4, <~ ~ ~ [~@3 bd:hf ~@4]>").gain(0.58)
// bass
$: note("0 0 4 <0 2 4 0>").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("bs:hf").gain(0.42)
// lead
$: note("~ 7 4 <7 9 4 2>").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("ld:ss").gain(0.16).cut(1)
// hook
$: note("4 ~ 7 4  2 0 ~ -1").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("plk:lp").gain(0.22).cut(1)
// arp
$: note("~ 0 4 7  ~ 4 0 2").scale("<C5:minor C5:minor G5:dorian C5:minor>")
  .s("plk:hd").gain(0.14).cut(1)
// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("<C3:minor C3:minor G3:dorian C3:minor>")
  .s("triangle").lpf(1400).gain(0.24)
// pad
$: note("[0,4]").scale("<C3:minor C3:minor G3:dorian C3:minor>")
  .s("sine").fm(1.2).fmh(1).fmdec(0.8).fmsus(0.4)
  .room(0.3).orbit(2).gain(0.16)
```

This fence is the new target (7 tracks: one bass, so perc is not required). Do not “improve” the hook degrees `4 ~ 7 4  2 0 ~ -1` or the clap grid `[~ cp]*2`. Do not add `bs:su`.

`songs/house/01.strudel` matches this fence.

| Piece | Role |
| --- | --- |
| `setcpm(124/4)` | **124 BPM** (house). Same clock as the 124 techno / minor-scale skills. |
| `bd:hf*4` | Kick on 1–4 (`part:slug`; default `bd` also fine) |
| `[~ cp]*2` | House clap on **2 and 4** (`samples/cp/00.wav`) |
| `[~ hh:hs]*4` | Closed hat on each **and** |
| `<~ ~ ~ [~@3 bd:hf ~@4]>` | 4th-bar fill; drums may stay 1-bar + fill |
| `bs:hf` + `C4:minor` | PCM floor. Pitched PCM uses **C4**. One bass only — do not stack `bs:su` |
| `ld:ss` | Bundled supersaw lead (`in_bank=yes`). Long PCM other than `ld:ss` / `pf:ff` is out |
| `note("4 ~ 7 4  2 0 ~ -1")` | Hook eighths: G–rest–**C**–G–Eb–C–**rest**–Bb. Do not rewrite |
| `.scale("<C4:minor C4:minor G4:dorian C4:minor>")` | 4-bar phrase on pitched tracks |
| `.s("plk:lp")` | `samples/plk/lp.wav` (`part:slug`) |
| `.cut(1)` | Steal the previous one-shot (~0.4 s must not overlap itself) |
| chords `[0,2,4]` / pad `[0,4]` | Triad vs fifth. Pad on orbit 2 |
| no `.compressor` / no `.duckorbit` | Compressor is mixer last-write. No duck in this recipe |

## Why `[~ cp]*2`, not `[~ sd]*2`

`[~ x]*2` tiles rest-then-hit across the bar, so hits land at 0.25 and 0.75 (beats 2 and 4). That grid is the house backbeat.

`s("cp")` is the bundled **dry house clap** (`samples/cp/00.wav`). `s("sd")` is a **snare** (`samples/sd/00.wav`). They occupy the same 2/4 slots and **mask each other** — do not write `[~ cp, sd]*2` or `bd*4, [~ cp]*2, [~ sd]*2`. Do not treat `sd` as the house clap.

Do **not** put this clap layer on a techno drum string. Kick-front techno is `bd*4, [~ hh]*4` with no 2/4 backbeat ([strudel-genre-four-on-the-floor](../strudel-genre-four-on-the-floor/SKILL.md)).

`s("cp")` resolves as a SampleBank folder key, not a flat `cp.wav`:

| Disk | Sound key | How |
| --- | --- | --- |
| `samples/cp/00.wav` | `cp` | `load_dir` reads `dir/<name>/*.wav` (`sample.rs`). Default variation is sort order → `00.wav` |
| `samples/cp.wav` | `cp` | Would also work as a single flat file. **Not in this repo.** If both existed, folder vars win |
| `samples/plk/lp.wav` | `plk:lp` | `part:slug`. Old flat `lead-fm_pluck` / `lead-fm-pluck` do not resolve |

`cp` is not a waveform (`sound.rs`), so it falls through to the bank. Unknown names fail resolve and the current performance continues.

## Why C4:minor on a C3 sample

`note().s("sample")` sets playback rate to `target_hz / SAMPLE_ROOT_HZ` (`deck.rs`). `SAMPLE_ROOT_HZ` is **261.63 Hz (C4)** even though the constant comment says C3.

`plk/lp.wav` is a **C3** one-shot (~131 Hz / MIDI 48). Ratio 1.0 already sounds as C3. The same rule applies to `bs:hf` (PCM bass stays on **C4**, not C2).

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

Eight atoms = eighths. At 124 BPM an eighth is ~0.242 s. The pluck one-shot is ~0.4 s, so a new note starts before the previous wav ends. `.cut(1)` puts the hook on cut group 1; `Deck::alloc_voice` drops earlier voices in that group (`deck.rs`). Without it the tails stack. Lead and arp also use `.cut(1)` so their one-shots do not overlap themselves.

Pitched tracks share `.scale("<C4:minor C4:minor G4:dorian C4:minor>")` (bass/lead/hook at C4, arp at C5, chords/pad at C3). The degree string stays fixed; bar 3 is G dorian. Do not drop the `<>` back to a 1-bar `.scale("C4:minor")` for a new apply.

Lead / hook / arp are call-and-response (rests on different eighths). Do not fill all 16ths on every melody track at once.

**Chord suffixes:** `note("c3'maj")` still plays the **root only** (`deck.rs` does not call `expand_chord`). This song uses degrees, not suffixes. Chords are `[0,2,4]` (max 3 notes). Pad is `[0,4]`.

## Why 124, not techno

House in this tree is ~120–128. This file is **124**. Techno kick-front examples may also sit at 124 for a shared clock, but they must **not** grow a clap. A 130 acid / 126 duck / 174 DnB file is a different tempo — do not pair them.

Both decks share one `Transport`. A DJ pair must use the **same** `setcpm`. Pair this file with `songs/four-on-the-floor/01.strudel` (also `setcpm(124/4)`). Do not pair a techno file that is not 124.

No `.compressor` on a `$:` — that writes the **mixer master** (last-write) and will squash the kick. No duck work in this skill.

## Try it in this app

```bash
strudel-rs play songs/house/01.strudel --seconds 12
strudel-rs play songs/house/01.strudel --headless --seconds 8

# Dual deck — both files are setcpm(124/4). Do not pair a different BPM.
strudel-rs dj songs/house/01.strudel songs/four-on-the-floor/01.strudel
```

Load `songs/house/01.strudel`, or `strudel_apply_song` the Pattern fence.

No device: `cargo test --test e2e house_01 -- --nocapture`.

Live TUI: `/a house 01`.

## Variations (still this syntax)

| Goal | Change |
| --- | --- |
| Faster hats | `[~ hh:hs]*4` → `hh:hs*8` (keep `[~ cp]*2`) |
| Darker kit | `.lpf(4000)` on the drum `$:` |
| New root, same shape | keep the 4-bar `<>` and the hook degrees; change C4/G4 together (octave-4 so the C3 wav stays mid) |

Do not add `[~ sd]*2`. Do not drop the clap onto the techno skill song. Do not add a second sub (`bs:su` / `square`+`lpf(120)`).

## Checklist

- [ ] `setcpm(124/4)` + **7 `$:`** (drums, bass, lead, hook, arp, chords, pad). 7 is OK; do not add perc or `bs:su` to make 8
- [ ] 4-bar phrase on pitched tracks (`.scale("<C4:minor C4:minor G4:dorian C4:minor>")` and octave variants)
- [ ] Clap grid `[~ cp]*2`. Hook degrees `4 ~ 7 4  2 0 ~ -1`. Do not rewrite either
- [ ] PCM pitched at `C4:`. One bass (`bs:hf`). Chords `[0,2,4]`, pad `[0,4]`
- [ ] `songs/house/01.strudel` matches this fence

## Do not

- Write `[~ sd]*2` or stack `sd` with `cp` on 2/4 — they mask each other.
- Put `[~ cp]*2` on kick-front techno (`bd*4, [~ hh]*4`).
- Use `.scale("C3:minor")` with this wav — it plays in the bass register.
- Write `lead-fm-pluck` or `lead-fm_pluck` — the key is `plk:lp`.
- Use `plk:s5` / `bs:rm` here. Do not add `bs:su`.
- Write `in_bank=no` PCM (`ld:ac`, `pf:al`, `dr:*`, `ps:*`). Allowed long PCM: `ld:ss`, `pf:ff`.
- Ship a 2-track loop for a new apply.
- `note("c3'maj")` when you want a chord — suffix is root only.
- Put `.compressor` on a track. Do not add `.duckorbit`.
- Pair this file with another `setcpm` (shared clock; the other tempo is discarded).
- `stack()` / `.cpm(124)` / `.lfo()` / `kit:bd`.
