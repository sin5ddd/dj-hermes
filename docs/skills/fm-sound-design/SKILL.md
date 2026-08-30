---
name: fm-sound-design
description: >-
  Use when designing live 2-op FM timbres in strudel-rs (.fm / .fmh /
  fm envelope on one carrier + one sine modulator). Not a genre loop,
  and not rust-fm-synthe’s 4-op TOML factory (those WAVs are sampled).
---

# FM sound design (strudel-rs, live 2-op)

## When to use

- The request is **timbre**: pluck, bell, growl, pad, stab — built with **this engine’s** `.fm` / `.fmh` / FM envelope.
- You need to know **how the 2-op path actually runs** (`synth.rs`), not DX7 lore and not a 4-operator algorithm.
- You are **not** writing a house / techno / DnB grid recipe. Those stay in their own skills.
- You are **not** recreating a `rust-fm-synthe` TOML patch with `.fm(4)`. Factory one-shots are **samples**.

Playable copy (live recipes): `songs/skill-fm-sound-design.strudel`.
Sampled factory contrast (same 120 BPM): `songs/skill-fm-sound-design-sampled.strudel`.

## Two FM worlds (keep them distinct)

| World | What it is | How you trigger it here |
| --- | --- | --- |
| **Live 2-op** (this skill) | One **carrier** + one **sine modulator**. Index, ratio, and an ADS envelope on the index. | `.s("sine")` (or triangle / saw / square / `wt_*`) + `.fm` / `.fmh` / `.fmattack` / `.fmdecay` / `.fmsustain` |
| **Offline 4-op factory** | [sin5ddd/rust-fm-synthe](https://github.com/sin5ddd/rust-fm-synthe) renders WAV one-shots (`lead-fm_pluck`, `stab-fm_fifth`, `stab-fm_major`, `reese-mid`, `cp`, …) | `s("lead-fm_pluck")` etc. The engine **samples** the wav. `.fm` is not applied to that recording |

`.fm(4)` does **not** reproduce a 4-op TOML patch. There is no algorithm graph, no `fmh2`…`fmh8`, no second modulator.

## How live FM actually works

All of this is `Voice::osc_sample_modulated` + `Voice::fm_env_level` in `synth.rs`. Methods parse in `code.rs` (`parse_num` scalars only).

### Instantaneous frequency

```
fm_idx    = fm * fm_env_level()
mod_sig   = sin(2π · mod_phase)          // modulator is always a sine
mod_phase += (fmh * freq) / sr           // fmh clamped to min 0.01
inst_freq = max(freq + fm_idx * freq * mod_sig, 0.1)
carrier   += inst_freq / sr              // then Wave / wavetable at that phase
```

- `.fm(index)` is a **frequency-deviation ratio**: peak deviation (Hz) = `fm_idx * carrier_hz` when the env is 1. `.fm(4)` on a 220 Hz note is ±880 Hz at full env — not a DX7 phase-modulation index in radians.
- `.fmh(ratio)` is **modulator harmonicity**: the modulator advances at `fmh * freq`. Default **1.0**. Values below 0.01 are clamped at render (`fmh.max(0.01)`). Parse does not clamp.
- Integer `fmh` (1, 2, 3, …) keeps sidebands **harmonic** (same family as the carrier). Non-integer `fmh` (1.4, 3.5, …) is **inharmonic** (bell / clang).
- Higher index → **brighter / more sidebands**. Lower index → closer to the bare carrier.
- Negative `inst_freq` is clamped to 0.1 Hz (through-zero is not a feature here).

The **carrier** is whatever `.s(...)` resolved: `sine` / `triangle` / `sawtooth` / `square` / `wt_*`. The **modulator waveform is not selectable** — it is a sine. `sawtooth` + FM is already rich; start with `sine` unless you want that extra grit.

`.s("white"|"pink"|"brown")` is a noise oscillator. That path does not advance a carrier phase with `inst_freq`. Do not use a noise *source* as an FM carrier.

### Index envelope (ADS, no release)

Defaults (`ModParams` in `code.rs`): **attack 0.001**, **decay 0.1**, **sustain 0.0**.

`fm_env_level`:

1. If `|fm| < 1e-6` → 0 (FM off; modulator is not advanced).
2. Attack: level `0 → 1` over `.fmattack` / `.fmatt` (seconds).
3. Decay: level `1 → fmsustain` over `.fmdecay` / `.fmdec`.
4. Then it **holds sustain**. There is **no FM release stage**.

Amp ADSR (`.attack` / `.decay` / `.sustain` / `.release`) is a **different** envelope. It scales the oscillator after FM, before the biquad. Short amp decay with a long FM decay still silences the voice; long amp sustain with `fmsustain(0)` is a dull body after the bright attack.

Amp defaults (if you omit them): attack 0.01, decay 0.1, sustain 0.7, release 0.1.

### `.noise` is not a second operator

`.noise(0..1)` mixes **pink** into the oscillator output (`pure * (1-mix) + pink * mix`). It is not an operator, not a modulator, and not `s("pink")`.

### What this engine cannot do

| Idiom | Works? | Notes |
| --- | --- | --- |
| Scalar `.fm` / `.fmh` / `.fmattack` / `.fmdecay` / `.fmsustain` | **Yes** | Aliases: `fmatt`, `fmdec`, `fmsus` |
| `.fm("3 5")` / patterned `fmh` | **No** | `parse_num` only |
| `fmh2`…`fmh8`, `fmenv`, 4-op / 8-op algorithms | **No** | Not parsed |
| Modulator waveform other than sine | **No** | Hard-coded `sin` |
| FM on a sample (`s("lead-fm_pluck").fm(4)`) | Plays the **wav** | `note().s("sample")` is SampleVoice; live FM lives on synth voices |
| `.lprelease` as an FM stage | Unused | Filter env has no release either |

## Per-use recipes (live 2-op)

Numbers below are **starting points** for this engine, not factory-patch transcriptions. A synthesist should correct them if the render is wrong. Each recipe states carrier, index / harmonicity / FM env, amp ADSR, and optional filter.

Playable copies are the `$:` tracks in `songs/skill-fm-sound-design.strudel` (`setcpm(120/4)`).

### Pluck

```
$: note("4 ~ 7 4  ~ 0 ~ -1").scale("C4:minor")
  .s("sine").fm(4).fmh(2)
  .fmattack(0.001).fmdecay(0.07).fmsustain(0)
  .attack(0.001).decay(0.15).sustain(0).release(0.04)
  .cut(1).gain(0.28)
```

| Knob | Why |
| --- | --- |
| `sine` | Clean carrier so the index *is* the brightness |
| `.fm(4)` (3–6) | Enough sidebands for a pick / hammer transient |
| `.fmh(2)` (or 1) | Integer → tonal, not clangy |
| short `.fmdecay` + `fmsustain(0)` | Brightness only at the attack (the “pluck”) |
| amp `.decay(0.15)` + sustain 0 | One-shot body; default sustain 0.7 would pad it |
| `.cut(1)` | Steal the previous voice so one-shots do not pile up |

### Bell / inharmonic

```
$: note("~ ~ 7 11").scale("C4:minor")
  .s("sine").fm(2.8).fmh(3.5)
  .fmattack(0.001).fmdecay(0.45).fmsustain(0)
  .attack(0.002).decay(0.7).sustain(0).release(0.25)
  .gain(0.2)
```

| Knob | Why |
| --- | --- |
| `.fmh(3.5)` (or 1.4) | Non-integer ratio → inharmonic partials (bell, not a keyed sine) |
| slightly **lower** index than pluck | Less scream; the inharmonicity does the work |
| longer FM + amp decay, sustain 0 | Ring that dies; a high `fmsustain` would keep the clang on |

### Metallic (inharmonic + pink mix)

```
$: note("~ ~ ~ ~  ~ ~ ~ 4").scale("C5:minor")
  .s("sine").fm(4).fmh(7)
  .fmattack(0.001).fmdecay(0.2).fmsustain(0)
  .noise(0.12)
  .attack(0.001).decay(0.25).sustain(0).release(0.08)
  .cut(1).gain(0.16)
```

| Knob | Why |
| --- | --- |
| higher `.fmh` (5–8) | Farther from the carrier family → more metal than bell |
| `.noise(0.12)` | Pink grit in the osc, **not** a second operator |
| short-medium decays | A hit, not a pad |

### Bass growl

```
$: note("0 0 3 0").scale("C2:minor")
  .s("sine").fm(2.5).fmh(1)
  .fmattack(0.001).fmdecay(0.22).fmsustain(0.5)
  .lpf(420)
  .attack(0.008).decay(0.18).sustain(0.45).release(0.08)
  .gain(0.4)
```

| Knob | Why |
| --- | --- |
| `sine` or `triangle` | Soft carrier; `sawtooth` + FM gets fizzy under a parked LPF |
| `.fmh(1)` | Integer 1 → harmonic growl. `fmh(1.5)` (see `techno1`) is a slightly more metallic cousin |
| moderate `.fm` (2–3) | Body without a 4-op stack. Do not write `.fm(4)` and call it a factory Reese |
| `.fmsustain(0.5)` | Index stays partly open so the note keeps growl after the attack |
| `.lpf(420)` parked | Parks extra sidebands in the bass band. This is **not** a 303 `lpenv` |

`songs/techno1.strudel` already uses live FM bass: `.s("sine").fm(3).fmh(1.5).lpf(500)` at 126. That is this world, not a sample.

### Pad / soft

```
$: note("[0,4]").scale("C3:minor")
  .s("sine").fm(1.2).fmh(1)
  .fmattack(0.3).fmdecay(0.4).fmsustain(0.55)
  .attack(0.2).decay(0.3).sustain(0.55).release(0.25)
  .lpf(1800)
  .room(0.3).orbit(2)
  .gain(0.18)
```

| Knob | Why |
| --- | --- |
| low `.fm` (~1–1.5) | Mild brightness; high index on a long note is fatiguing |
| slow `.fmattack` | Brightness fades *in* with the amp attack |
| more `.fmsustain` | Index does not fall to 0 (that would become a dull sine hold) |
| `.room` on **orbit 2** | Room is per-orbit, last-write (`deck.rs`). Other tracks stay on orbit 1 so they stay dry |
| optional `.lpf` | Soft top; do not use `lpenv` unless you want a per-note filter sweep |

### Stab (live 2-op)

```
$: note("~ 4 ~ 0").scale("C4:minor")
  .s("sine").fm(5).fmh(2)
  .fmattack(0.001).fmdecay(0.06).fmsustain(0)
  .attack(0.001).decay(0.1).sustain(0.05).release(0.04)
  .lpf(2400)
  .cut(1).gain(0.22)
```

| Knob | Why |
| --- | --- |
| higher index, integer `fmh` | Punchy harmonic bark, then gone |
| very short FM + amp | A stab, not a pad |
| `.lpf(2400)` | Optional ceiling; `lpenv` is optional if you want the bark to close |
| `.cut(1)` | One-shot group |

This is **not** `s("stab-fm_fifth")` or `s("stab-fm_major")`. Those are factory wavs (next section).

## Offline 4-op factory (sampled)

`sound.rs` resolves waveforms first, then the SampleBank. Factory keys are **flat stems** (`sample.rs` `file_stem()`, lowercased). Hyphen ≠ underscore.

`note().s("sample")` sets rate to `target_hz / SAMPLE_ROOT_HZ` (`deck.rs`). `SAMPLE_ROOT_HZ` is **261.63 Hz (C4)** even though the constant comment says C3. The bundled FM wavs are **C3** recordings. **Write `C4:…`** so ratio 1.0 plays the recorded C3. `C3:…` dumps an octave.

| Disk | Sound key | Recording | How to write | Do not |
| --- | --- | --- | --- | --- |
| `samples/lead-fm_pluck.wav` | `lead-fm_pluck` | C3 short pluck | `note("…").scale("C4:minor").s("lead-fm_pluck").cut(1)` | `lead-fm-pluck`; `C3:minor` |
| `samples/stab-fm_fifth.wav` | `stab-fm_fifth` | Hollow **C–G** (no third) | `note("~ 4 ~ 7").scale("C4:minor")` — transposes the fifth | `stab-fm-fifth`; `note("[0,2,4]")` does not invent a third |
| `samples/stab-fm_major.wav` | `stab-fm_major` | Already a **C–E–G** triad wav | `note("0 ~ 0 ~").s("stab-fm_major")` | `note("[0,2,4]")` **triples** the triad (3 events × 3 notes) |
| `samples/reese-mid.wav` | `reese-mid` | C3 mid Reese, 800–1200 Hz | `C4:minor` (see [dnb-reese-mid-stab](../dnb-reese-mid-stab/SKILL.md)) | `reese_mid`; `C2:minor` |
| `samples/cp/00.wav` | `cp` | Dry house clap | house `[~ cp]*2` only | stacking with `sd` |

House pluck line (signed-off, 124): `note("4 ~ 7 4  2 0 ~ -1").scale("C4:minor").s("lead-fm_pluck").cut(1)` — [house-clap-backbeat](../house-clap-backbeat/SKILL.md).

`expand_chord("c3'maj")` is **not** called by the deck. `note("c3'maj")` is the **root only**.

Playable factory demo (120, same clock as the live file): `songs/skill-fm-sound-design-sampled.strudel`. Play each file **solo**. Pairing them stacks two full arrangements.

```
// @title skill-fm-sound-design-sampled
// @details rust-fm-synthe one-shots — sampled, not live .fm
setcpm(120/4)
// pluck
$: note("4 ~ 7 4  ~ 0 ~ -1").scale("C4:minor").s("lead-fm_pluck").gain(0.32).cut(1)
// major
$: note("0 ~ 0 ~").scale("C4:major").s("stab-fm_major").gain(0.2).cut(2)
// fifth
$: note("~ 4 ~ 7").scale("C4:minor").s("stab-fm_fifth").gain(0.18).cut(3)
```

## Playable live song

```
// @title skill-fm-sound-design
// @details live 2-op FM recipes (not rust-fm-synthe one-shots)
setcpm(120/4)
// pulse
$: s("bd*4").gain(0.28)
// bass
$: note("0 0 3 0").scale("C2:minor")
  .s("sine").fm(2.5).fmh(1)
  .fmattack(0.001).fmdecay(0.22).fmsustain(0.5)
  .lpf(420)
  .attack(0.008).decay(0.18).sustain(0.45).release(0.08)
  .gain(0.4)
// pluck
$: note("4 ~ 7 4  ~ 0 ~ -1").scale("C4:minor")
  .s("sine").fm(4).fmh(2)
  .fmattack(0.001).fmdecay(0.07).fmsustain(0)
  .attack(0.001).decay(0.15).sustain(0).release(0.04)
  .cut(1).gain(0.28)
// bell
$: note("~ ~ 7 11").scale("C4:minor")
  .s("sine").fm(2.8).fmh(3.5)
  .fmattack(0.001).fmdecay(0.45).fmsustain(0)
  .attack(0.002).decay(0.7).sustain(0).release(0.25)
  .gain(0.2)
// metallic
$: note("~ ~ ~ ~  ~ ~ ~ 4").scale("C5:minor")
  .s("sine").fm(4).fmh(7)
  .fmattack(0.001).fmdecay(0.2).fmsustain(0)
  .noise(0.12)
  .attack(0.001).decay(0.25).sustain(0).release(0.08)
  .cut(1).gain(0.16)
// pad
$: note("[0,4]").scale("C3:minor")
  .s("sine").fm(1.2).fmh(1)
  .fmattack(0.3).fmdecay(0.4).fmsustain(0.55)
  .attack(0.2).decay(0.3).sustain(0.55).release(0.25)
  .lpf(1800)
  .room(0.3).orbit(2)
  .gain(0.18)
// stab
$: note("~ 4 ~ 0").scale("C4:minor")
  .s("sine").fm(5).fmh(2)
  .fmattack(0.001).fmdecay(0.06).fmsustain(0)
  .attack(0.001).decay(0.1).sustain(0.05).release(0.04)
  .lpf(2400)
  .cut(1).gain(0.22)
```

`setcpm(120/4)` is **120 BPM**. The `bd*4` line is a **pulse**, not a signed-off house/techno grid (no `[~ cp]*2`, no `[~ hh]*4`, no 174 break). Play **solo**. Do not pair with 124 house, 126 techno, or 174 DnB (shared Transport discards the other tempo).

No `.compressor` (mixer master, last-write). No `.duckorbit` (that is the duck skill).

## Try it in this app

```bash
strudel-rs play songs/skill-fm-sound-design.strudel --seconds 12
strudel-rs play songs/skill-fm-sound-design.strudel --headless --seconds 8
strudel-rs play songs/skill-fm-sound-design-sampled.strudel --seconds 12
strudel-rs play songs/skill-fm-sound-design-sampled.strudel --headless --seconds 8
```

Play each **solo**. Same `setcpm(120/4)` so the clock *would* align — pairing stacks two full arrangements, it is not a mix.

No device: `cargo test --test e2e skill_fm_sound -- --nocapture`.

Live TUI: `/a load skill-fm-sound-design`.

## Variations (still this syntax)

| Goal | Change |
| --- | --- |
| Softer pluck | `.fm(3)` or `.fmh(1)` |
| More bell | `.fmh(1.4)` or longer `.fmdecay(0.6)` |
| Growlier bass | `.fm(3)` or `.s("triangle")`; keep `fmh(1)` and the LPF |
| Techno-ish bass | `.fmh(1.5)` as in `techno1` (still live 2-op) |
| Darker pad | `.fm(0.8)` or `.lpf(1200)`; keep room on orbit 2 |
| House bed under live FM | do **not** invent a grid — use [house-clap-backbeat](../house-clap-backbeat/SKILL.md) at **124** (different clock) |

## Rules (do not skip)

1. Live FM is **one carrier + one sine modulator**. Document `synth.rs`, not a 4-op algorithm.
2. Index = brightness. Integer `fmh` = harmonic. Non-integer `fmh` = inharmonic. Short `fmdecay` + `fmsustain(0)` = pluck/stab attack.
3. `.noise` is pink mix, not an operator.
4. Factory wavs are **samples**. `C4` on C3 recordings. `stab-fm_major` is already a triad — `note("0 ~ 0 ~")`, not `[0,2,4]`.
5. FM methods are **scalars**. This file is **120**. Play solo.

## Do not

- Invent 4-operator algorithms, `fmh2`, or “this `.fm(4)` is the factory pluck”.
- Call `s("lead-fm_pluck")` a live 2-op recipe (or the reverse).
- Play `stab-fm_major` as `note("[0,2,4]")` (triples the baked triad).
- Play `stab-fm_fifth` as a major/minor triad (the wav has no third).
- Write `C3:minor` on `lead-fm_pluck` / `stab-fm_*` / `reese-mid` (`SAMPLE_ROOT_HZ` is C4).
- Write `lead-fm-pluck` or `stab-fm-fifth` (wrong stem).
- Put `.compressor` or `.duckorbit` on these recipes.
- Borrow a house/techno/DnB drum string and change it. Pulse here is `bd*4` only.
- Pair this 120 file with 124 / 126 / 174 (shared clock).
- `stack()` / `.cpm(120)` / `.fm("3 5")`.
