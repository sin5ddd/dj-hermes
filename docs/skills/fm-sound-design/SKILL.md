---
name: fm-sound-design
description: >-
  Use when writing time-varying lead or bass with live 2-op FM in
  strudel-rs (.fm / .fmh / FM env). Percussion and one-shots are PCM
  samples, not live FM. Not a genre loop, not a 4-op TOML factory.
---

# FM sound design (strudel-rs)

Live `.fm` / `.fmh` is **only** for **time-varying lead and bass synths**. Drums and one-shots are **PCM** (`s("bd")`, `s("cp")`, `s("lead-fm_pluck")`, `s("stab-fm_fifth")`, …). Do not build pluck / bell / metal-hit as live 2-op one-shots.

Playable copy: `songs/skill-fm-sound-design.strudel` (`setcpm(124/4)`). How to trigger issue #21 batch 1 factory stems (`C4:…` / unpitched FX): [factory-pcm-usage](../factory-pcm-usage/SKILL.md).

## When to use

- You need an **evolving** lead / pad / growl bass that this engine synthesizes.
- You need to know **how the 2-op path runs** (`synth.rs`) so the signed-off numbers make sense.
- You are **not** writing a house / techno / DnB grid recipe (those skills own the drums).
- You are **not** recreating a `rust-fm-synthe` TOML patch with `.fm(4)`. Factory files are **samples**.
- You are **not** replacing the lead with the retired live EP recipe, or with PCM `keys-fm_ep` (tine harmonics landed in #37; that wav is a one-shot, not this lead).

## Two FM worlds (keep them distinct)

| World | What it is | How you trigger it here |
| --- | --- | --- |
| **Live 2-op** | One **carrier** + one **sine modulator**. Index, ratio, ADS envelope on the index. | `.s("sine")` or `.s("sawtooth")` + `.fm` / `.fmh` / `.fmdec` / `.fmsus` — **lead and bass only** |
| **Offline 4-op factory** | [sin5ddd/rust-fm-synthe](https://github.com/sin5ddd/rust-fm-synthe) renders WAV one-shots | `s("lead-fm_pluck")`, `s("stab-fm_fifth")`, `s("stab-fm_major")`, `s("cp")`, … The engine **samples** the wav |

`.fm(4)` does **not** reproduce a 4-op TOML patch. There is no algorithm graph, no `fmh2`…`fmh8`, no second modulator.

## How live FM actually works

`Voice::osc_sample_modulated` + `Voice::fm_env_level` in `synth.rs`. Methods are `parse_num` **scalars** (`code.rs`).

```
fm_idx    = fm * fm_env_level()
mod_sig   = sin(2π · mod_phase)          // modulator is always a sine
mod_phase += (fmh * freq) / sr           // fmh clamped to min 0.01
inst_freq = max(freq + fm_idx * freq * mod_sig, 0.1)
carrier   += inst_freq / sr
```

| Knob | Engine meaning | Musical use |
| --- | --- | --- |
| `.fm(index)` | Peak deviation (Hz) = `fm_idx * carrier_hz` at env 1 | **Index 2–4** is everyday. **8+ breaks up fast.** Do not put `.fm(8)` on bass. |
| `.fmh(ratio)` | Modulator advances at `fmh * freq`. Default **1.0** | **Signed-off recipes use integer `fmh`** (`1` growl/pad, `2` lead) = harmonics. `3.5` / `11` = bell / metal **timbre** — those hits are **PCM**, not live FM one-shots. Do **not** copy `techno1`'s `.fmh(1.5)` into this rule. |
| `.fmdec` / `.fmsus` | Index env: attack → decay toward sustain. **No FM release.** Defaults 0.001 / 0.1 / **0.0** | `fmsus(0)` is percussive (do not use live FM for that). **Leave sustain** on evolving leads / pads. |
| `.noise(0..1)` | Pink **mix** into the osc | Not a second operator |

Carrier is `.s("sine"|"triangle"|"sawtooth"|"square"|wt_*)`. The modulator waveform is not selectable. `.s("pink")` is a noise *source* and does not use `inst_freq` as a carrier — do not FM that.

Amp ADSR is a **different** envelope (defaults 0.01 / 0.1 / 0.7 / 0.1). It scales the voice after FM.

| Idiom | Works? |
| --- | --- |
| Scalar `.fm` / `.fmh` / `.fmattack` / `.fmdecay` / `.fmsustain` (`fmatt` / `fmdec` / `fmsus`) | **Yes** |
| `.fm("3 5")` / patterned `fmh` / `fmh2` / `fmenv` | **No** |
| Live FM on a sample (`s("lead-fm_pluck").fm(4)`) | Plays the **wav**; SampleVoice does not run this 2-op path |

## Signed-off live recipes (synthesist + Beatmaker)

Carrier is **sine** unless noted. Put these numbers **as-is**. Mix: the growl already fills the mids — **do not** stack a square sub, `reese-mid`, pluck, or stab under it.

### Lead — C4 and above, evolving (not a one-shot)

```
$: note("4 2 0 2").scale("C4:minor")
  .s("sine").fm(3).fmh(2).fmatt(0.01).fmdec(0.3).fmsus(0.25)
  .lpf(1800).lpenv(2).gain(0.3)
```

Degrees `4 2 0 2` in C minor = **5–♭3–1–♭3** (G–Eb–C–Eb). Integer `fmh(2)` = harmonics. `.fm(3)` + `.fmatt(0.01)` + `.fmdec(0.3)` + `.fmsus(0.25)` keeps the index moving through the note (lead, not a hit). `.lpf(1800).lpenv(2)` is the per-note filter sweep on that parked base.

The live EP recipe (`.fm(2).fmh(1).fmdec(0.6).fmsus(0.15)` on `0 ~ 4 2`) is **retired**. PCM `keys-fm_ep` is a tine one-shot (#37), not this lead.

### Pad — C4 and above; leave `fmsus`, add `.room`

```
$: note("[0,4]").scale("C4:minor")
  .s("sine").fm(1.2).fmh(1).fmdec(0.8).fmsus(0.4)
  .room(0.3).orbit(2)
  .gain(0.16)
```

`.room` is per-orbit, last-write (`deck.rs`). Pad on **orbit 2** so the bass / lead stay dry. Do not drop `.fmsus(0.4)` — sustain is what makes it a pad.

### Growl bass — do not stack on square / `reese-mid`

```
$: note("0 0 3 0").scale("C2:minor")
  .s("sawtooth").fm(4).fmh(1).fmdec(0.25).fmsus(0.2)
  .lpf(400).lpenv(3)
  .gain(0.38)
```

`.fm(4).fmh(1)` + `lpf(400)` already fills the mids. `.lpenv(3)` is the per-note filter sweep on that parked base (same env law as the 303 skill: `cutoff = base * 2^(lpenv * level)`). **Do not** put `.fm(8)` on bass (it breaks up). **Do not** add a square sub or `reese-mid` on another `$:` — that is a different mix ([dnb-reese-mid-stab](../dnb-reese-mid-stab/SKILL.md)).

`songs/techno1.strudel` has another live FM bass (`.s("sine").fm(3).fmh(1.5).lpf(500)` at 126). That is this 2-op world, **not** a signed-off recipe here. Do not fold `.fmh(1.5)` into the integer-ratio rule above.

## PCM one-shots (not live FM)

Pluck, bell, and metal-hit were dropped from live 2-op. Trigger the factory / kit wavs. `SAMPLE_ROOT_HZ` is **261.63 Hz (C4)** (the comment in `sample.rs` says C3). Bundled FM wavs are **C3** recordings — write **`C4:…`**.

This demo does **not** stack those one-shots on the growl (mids fill up). Keep them PCM when you need them elsewhere.

| Use | Sound key | How to write | Do not |
| --- | --- | --- | --- |
| Drums / clap | `bd` `sd` `hh` `oh` `cp` | `s("bd*4")` etc. | Live `.fm` on a kick |
| プラック | `lead-fm_pluck` | `note("…").scale("C4:minor").s("lead-fm_pluck")`. `.cut(1)` **only if monophonic** | `lead-fm-pluck`; `C3:minor`; live `.fm` pluck; stacking it on this demo |
| ベル / メタルヒット | `stab-fm_fifth` / factory bell-metal wavs | Factory/PCM, **low gain**, not chords | Live `.fmh(3.5)` / `.fmh(11)` hits |
| Hollow fifth | `stab-fm_fifth` | Transposes C–G (no third). Sparse degrees, low gain. Dark-side stab if you must — **prefer none** on this mix | Inventing a third; `stab-fm-fifth` |
| Major stab | `stab-fm_major` | Already a **C–E–G** triad wav: `note("0 ~ 0 ~").s("stab-fm_major")` | On this **minor** demo (E vs Eb); `note("[0,2,4]")` **triples** it |
| EP one-shot | `keys-fm_ep` | PCM tine (#37). Not the live lead | Using it as the evolving lead; live EP recipe |

`expand_chord("c3'maj")` is not called by the deck. `note("c3'maj")` is the **root only**.

Inharmonic `fmh` (3.5, 11) is why those factory wavs sound like bell / metal. That is **lore**, not a live one-shot recipe.

## Playable song

Minor growl bass + evolving lead + pad only. Drums are PCM. **No** pluck, **no** `stab-fm_major`. One clock: **124 BPM**.

```
// @title skill-fm-sound-design
// @details live FM growl+lead+pad; drums are PCM
setcpm(124/4)
// drums
$: s("bd*4").gain(0.3)
// bass
$: note("0 0 3 0").scale("C2:minor")
  .s("sawtooth").fm(4).fmh(1).fmdec(0.25).fmsus(0.2)
  .lpf(400).lpenv(3)
  .gain(0.38)
// lead
$: note("4 2 0 2").scale("C4:minor")
  .s("sine").fm(3).fmh(2).fmatt(0.01).fmdec(0.3).fmsus(0.25)
  .lpf(1800).lpenv(2).gain(0.3)
// pad
$: note("[0,4]").scale("C4:minor")
  .s("sine").fm(1.2).fmh(1).fmdec(0.8).fmsus(0.4)
  .room(0.3).orbit(2)
  .gain(0.16)
```

`bd*4` is a **pulse**, not a signed-off house/techno grid (no `[~ cp]*2`, no `[~ hh]*4`, no 174 break).

This file is **124** — same clock as house / mood / minor-scale. A leftover **120** file is isolated; **do not DJ-pair 120 with 124** (shared Transport discards the other tempo). Do not pair with 126 techno or 174 DnB.

No `.compressor` (mixer master, last-write). No `.duckorbit`. No square sub. No `reese-mid`. No pluck. No stab.

## Try it in this app

```bash
strudel-rs play songs/skill-fm-sound-design.strudel --seconds 12
strudel-rs play songs/skill-fm-sound-design.strudel --headless --seconds 8
```

No device: `cargo test --test e2e skill_fm_sound -- --nocapture`.

Live TUI: `/a load skill-fm-sound-design`.

## Variations (still this syntax)

| Goal | Change |
| --- | --- |
| Lead only | mute or drop the pad `$:` |
| Darker pad | keep `.fmsus(0.4)` and `.room`; do not zero sustain |
| House bed | do **not** invent a grid — [house-clap-backbeat](../house-clap-backbeat/SKILL.md) is the **124** house grid. This pulse stays `bd*4` |
| Dark stab | prefer **none**. If you must, `stab-fm_fifth` (hollow fifth, no third) — never `stab-fm_major` on this minor demo |

Do not “vary” by turning the lead into a live FM pluck (`fmsus(0)` + short decay) or a live bell (`.fmh(3.5)`). Those are PCM. Do not bring back the retired live EP.

## Rules (do not skip)

1. Live `.fm` / `.fmh` = **evolving lead and bass only**. Hits are PCM.
2. Signed-off numbers stay as written. Integer `fmh` on those recipes (`1` growl/pad, `2` lead). Index 2–4 everyday; **no `.fm(8)` on bass**. Do not copy `techno1`'s `.fmh(1.5)` into that rule.
3. Growl fills the mids — **no** square sub, **no** `reese-mid`, **no** pluck, **no** stab on this demo.
4. Factory wavs: `C4` on C3 recordings. `stab-fm_major` is `note("0 ~ 0 ~")` only — and **not** on this minor file. `.cut(1)` on pluck only if monophonic.
5. This file is **124**. Leftover **120** files stay isolated. Do not DJ-pair 120 with 124.

## Do not

- Teach pluck / bell / metal-hit as live 2-op one-shots.
- Invent 4-operator algorithms, `fmh2`, or “this `.fm(4)` is the factory pluck”.
- Stack square, `reese-mid`, pluck, or stab under the growl.
- Put `.fm(8)` on bass.
- Put `stab-fm_major` on this C-minor demo (baked E vs the lead's Eb).
- Play `stab-fm_major` as `note("[0,2,4]")` (triples the baked triad).
- Revive the live EP recipe, or substitute PCM `keys-fm_ep` for the evolving lead.
- Copy `techno1`'s `.fmh(1.5)` into these integer-ratio recipes.
- Write `C3:minor` on `lead-fm_pluck` / `stab-fm_*` (`SAMPLE_ROOT_HZ` is C4).
- Write `lead-fm-pluck` or `stab-fm-fifth` (wrong stem).
- Put `.compressor` or `.duckorbit` on these recipes.
- Borrow a house/techno/DnB drum string and change it. Pulse here is `bd*4` only.
- Pair a leftover 120 file with this 124 file (shared clock). Do not pair 126 / 174 either.
- `stack()` / `.cpm(124)` / `.fm("3 5")`.
