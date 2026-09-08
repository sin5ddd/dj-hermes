---
name: strudel-genre-acid
description: >-
  Use when writing a TB-303-style acid line in dj-hermes: per-note
  filter envelope (lpenv), high resonance, saw or square, monophonic
  16ths as the hook. Other slots (drums kit, bass filter/LFO, lead,
  arp, chords, pad) may differ freely. Not a parked lpf plus amp ADSR,
  not a chromatic key walk of the same bed, and not sidechain ducking.
  7 $: tracks, play solo at 130.
version: 6.1.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [dj-hermes, music, genre, acid, 303, filter-envelope]
    related_skills:
      - strudel-composition
      - strudel-sound-design
      - strudel-data-format
      - strudel-genre-four-on-the-floor
      - strudel-genre-techno-duck
---

# TB-303 filter envelope (dj-hermes, acid techno)

## When to use

- The request is **acid / TB-303**: a resonant saw or square that **opens on each note and decays into the body**.
- You need the **filter envelope**, not a louder amp click and not a static `.lpf(800)`. The 303 sits on **`// hook`** — do not add a second 303.
- **Only the 303 line is locked.** Drums kit, sub bass filter, lead, arp, chords, and pad should **not** copy the previous song. A new kit (`bd:9p` / `bd:ez` / `bd:hf` …) is enough to separate 02 from 01.
- You are **not** adding a house snare or clap unless you label the song **house** (or an explicit hybrid).
- New apply is **7 `$:` tracks** (one bass under the 303, perc as arp). Play **solo** at 130 BPM.
- You are **not** writing sidechain duck or putting `.compressor` on the bass.
- Factory 02–30 must **not** be the same bed walking C→D→Eb→E. Jump the key (fifth up, fourth down). Vary the **sub** `.lpf` or give it `sine.rangex` LFO. Never put LFO on the 303 (it replaces `lpenv`).

`songs/acid/01.strudel` is this 7-track bed. A parked `.lpf` plus amp ADSR is only a starting point — this skill is the missing envelope piece.

## Pattern

```
// @title skill-acid-303-filter-envelope
// @genre acid techno
setcpm(130/4)
// drums
$: s("bd*4, [~ hh]*4, <~ ~ ~ [~@3 bd ~@4]>").gain(0.62)
// bass
$: note("0 ~ 0 <0 0 3 0>").scale("<C2:minor C2:minor G2:phrygian C2:minor>")
  .s("sine").lpf(180).gain(0.32)
// hook
$: note("0 0 3 0  7 3 2 0  4 4 3 0  -1 3 0 <2 5>")
  .scale("C2:minor")
  .s("sawtooth")
  .lpf("260 260 720 260  760 260 260 260  260 680 260 260  740 720 260 260")
  .lpq(14)
  .lpenv(3)
  .lpattack(0.001)
  .lpdecay(0.09)
  .lpsustain(0.05)
  .cut(1)
  .gain(0.44)
  .attack(0.001)
  .decay(0.1)
  .sustain(0.12)
  .release(0.04)
// lead
$: note("~ 7 ~ <9 7 4 12>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("plk:lp").gain(0.12).cut(1)
// arp
$: s("<~ perc:mh ~ perc:tm>").gain(0.14)
// chords
$: note("[0,4] ~ ~ [0,4]").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("ep:mt").gain(0.16)
// pad
$: note("0").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("pf:ff").gain(0.12).room(0.25).orbit(2)
```

Keep the **exact** 303 note string, patterned `.lpf`, `.lpq(14)`, `.lpenv(3)`, lp attack/decay/sustain, `.cut(1)`, and amp ADSR on `// hook`. Do not add a second 303. The 303 stays on fixed `.scale("C2:minor")` — the patterned lpf is already 16ths; do not put `.scale("<…>")` on that line. Other pitched tracks may use the 4-bar scale.

7 tracks is OK. Play solo, 130 BPM.

`songs/acid/01.strudel` matches this fence (303 line and techno grid). New apply picks non-303 `.s()` from the palette below.

## Timbre palette (pick per new apply)

The Pattern fence is one example of grid, degrees, and slots. Keep the **303 on `// hook`**. For a **new** apply, pick **one** sound per other slot. Do not copy the fence `.s()` every time. Do not add a second 303 on lead or bass. Slug meanings: strudel-pcm-catalog INDEX. Long one-shots (`ld:` / `dr:` / `pf:` / `ps:`, `plk:fp` / `plk:sp`) need `.cut(1)` or `s("<x ~ ~ ~>")`.

| Slot | Keep | Pick one | Forbidden |
| --- | --- | --- | --- |
| drums | `bd*4` + `[~ hh]*4` or `hh*8` (no clap) | distinct kit each apply: `bd:tc` / `bd:9p` / `bd:ez` / `bd:hf` / `bd:8t` / `bd:ng` / `bd:mt` / `bd:dc` / `bd:lf` + `hh:cl` / `hh:ch` / `hh:hs` / `hh:dk` / `hh:ns` / `hh:pd` / `hh:tt` | `[~ cp]*2`, `[~ sd]*2` unless labelled house, same kit as the last acid song |
| bass | sine sub **or** `bs:su` (one only). **Change the filter** (`lpf` 100–280 or `sine.rangex(…).slow(n)`). Synth at `C2:`, PCM at `C4:` | `sine`+lpf, `bs:su`+lpf | second 303, stacked subs, LFO on the **hook** |
| hook | 303: `sawtooth` or `square` + `lpenv` / `lpq` + the note string | waveform swap only (Variations) | a second 303, `ld:ac` as a second acid, `.lpf(sine.rangex)` |
| lead | not a 303. **PCM or live FM only** — no bare `sawtooth` / `square` / `sine` | `plk:pk`, `plk:lp`, `ld:fm`, `ld:sf`, `plk:fc`, `ld:mt`; or `.s("sine").fm(3).fmh(2).fmatt(0.01).fmdec(0.3).fmsus(0.25).lpf(1800).lpenv(2)` | `ld:si` / `ld:sw` / `ld:pu`（素のサイン／ソー／パルス）、`plk:mx`, kawaii bells, `ld:ac` as a second 303 |
| arp | sparse perc | `perc:mh`, `perc:tm`, `perc:st`, `perc:tk`, `perc:cb`, `perc:sk` | a second 303 |
| chords | `[0,4]` | `ep:mt`, `plk:sf`, `plk:s5`, `ep:ky`, `plk:an` | `triangle`, major stab `plk:sm` / `plk:s3` on this minor bed |
| pad | | `pf:pu`, `dr:pd` / `dr:fg` with `<>`, `pf:cs`, `pf:fo`, `pf:ff`+`note("0")` | `ps:mx`, house Rhodes |

| Piece | Role |
| --- | --- |
| `setcpm(130/4)` | 130 BPM (acid techno). Play **solo**. |
| `s("bd*4, [~ hh]*4, …")` | Techno grid, **kick in front**. No clap. No `[~ sd]*2`. 4th-bar fill is OK. |
| `// bass` sine `lpf(180)` | Sub under the 303. Synth at **C2**. Not a second 303. |
| `note("…")` 16 tokens + `.scale("C2:minor")` | Monophonic 16ths on **hook**. Degrees, not `c3'min` (suffix = root only). |
| `.s("sawtooth")` | 303 source. `.s("square")` is the other hardware waveform. |
| `.lpf("260 …")` | **Base** cutoff per 16th. Accent bases stay in **600–800 Hz**. Not the envelope. |
| `.lpq(14)` | High resonance (scream). 12–16; above that is harsh. |
| `.lpenv(3)` | Envelope **depth in octaves**. Cutoff = `base * 2^(lpenv * env)`. |
| `.lpattack(0.001)` / `.lpdecay(0.09)` / `.lpsustain(0.05)` | Per-**note** AD toward a closed body. |
| `.cut(1)` | Steal the previous voice (mono). |
| amp `.attack` / `.decay` / `.sustain` / `.release` | Gate the oscillator. **Not** a substitute for `lpenv`. |
| no `.compressor` / no `.duckorbit` | Compressor is mixer last-write. This recipe is the filter, not duck. |

## Why it sounds that way

A TB-303 accent is a **cutoff sweep**, not a volume envelope. Hardware: a lowpass with high resonance; each gate (and extra on accent) opens the cutoff, then the filter decay closes it while the note is still sounding. Parked `.lpf(800).lpq(16)` plus a short amp ADSR only shapes loudness. The spectrum stays the same.

This engine **does** a true per-note filter env (`synth.rs` `Voice::advance_lp_env` + `effective_lpf_hz`):

1. Each onset allocates a `Voice`. `lpenv` / `lpa` / `lpd` / `lps` live on that voice (`ModParams` in `code.rs`).
2. Filter env is **attack → decay toward sustain**. Level goes 0→1 over `lpattack`, then toward `lpsustain` over `lpdecay`.
3. Cutoff is `base_hz * 2^(lpenv * level)`, clamped 20–20000 Hz. Depth is **octaves**, not Hertz.
4. Coeffs are rebuilt every sample while `lpenv != 0` (`synth.rs`). That is intra-note motion, not a per-cycle snapshot.

At 130 BPM a 16th is ~0.115 s. `.lpdecay(0.09)` with `.lpsustain(0.05)` closes most of the sweep inside that 16th. Example: base `260`, `lpenv(3)` peaks near `260 * 2^3 ≈ 2.1 kHz`, then falls to about `260 * 2^(3*0.05) ≈ 290 Hz`. An accent base of `760` peaks near `760 * 8 ≈ 6.1 kHz` — enough scream without a thin digital-saw top. **Do not** use `lpf(900)` + `lpenv(3.5)`: that peaks near 10 kHz and the saw goes gappy.

**Amp ADSR is a different envelope.** It scales the oscillator after the osc, before the biquad. A short amp decay with static `lpf` is a pluck. The 303 needs the **biquad cutoff** to move.

**Why the 303 scale stays scalar.** The 16-step `.lpf("260 …")` already walks the bar. `.scale("<C2:minor …>")` on that same `$:` would retune the 16ths every bar while the accent bases stayed locked to the original degrees — the scream would land on the wrong notes. Keep `.scale("C2:minor")` on the hook. Bass / lead / chords / pad take the 4-bar `<>`.

The sine bass is a **sub**, not a second 303: no `lpenv`, `lpf(180)`, sparse degrees. Do not stack `bs:su` / `bs:hf` / `bs:dk` on top of it.

### What this engine can and cannot do

| Idiom | Works? | Notes |
| --- | --- | --- |
| Scalar `.lpenv` + `.lpattack` + `.lpdecay` + `.lpsustain` | **Yes** | Per-note. Same depth on every hit. |
| `.lpf("a b c …")` + `lpenv` | **Yes** | Pattern snapshots **base** Hz at the note’s bar phase (`deck.rs`). Then `lpenv` multiplies. **This is the accent.** |
| `.lpf("[260 260 720 260]*4")` | **Yes** | Four 16th bases tiled across the bar (regular accent; keep the high step ≤ 800). |
| `.lpenv("4 1 4 1")` / patterned `lpdecay` | **No** | Those methods are `parse_num` scalars only. |
| `.lprelease` / `.lpr` | Parsed, **unused** | `advance_lp_env` has no release stage. Omit it. |
| `.lpf(sine.rangex(400, 4000))` | LFO, **not** 303 | Continuous LFO **wins over** `lpenv` (`effective_lpf_hz`). Cycle wobble, not per-note decay. |
| Static `.lpf(800)` + amp ADSR | Plays | **Not** a filter env. Do not call it 303. |
| `.lpf("<400 1200>")` alone | Per-**cycle** step | Alternates a parked cutoff. No intra-note decay. |

Closest working accent: raise **base** `.lpf` on the accented 16ths **into 600–800 Hz**; keep one scalar `.lpenv(3)`. Do not fake accent with `.gain` or amp decay. Do not teach `900` + `lpenv(3.5)`.

Both decks share one `Transport`. This file is 130 BPM. Play it **solo** — stacking another 303 on the other deck doubles the acid, it is not a mix. Do **not** pair this file with a different `setcpm` (the other tempo is discarded).

## Try it in this app

```bash
dj-hermes play songs/acid/01.strudel --seconds 12
dj-hermes play songs/acid/01.strudel --headless --seconds 8
```

Load `songs/acid/01.strudel`, or `dj_hermes_apply_song` the Pattern fence.

No device: `cargo test --test e2e acid_01 -- --nocapture`.

Live TUI: `/a load acid-01`.

## Variations (still this syntax)

| Goal | Change |
| --- | --- |
| Square 303 | `.s("square")` on **hook** (keep `lpenv` / `lpq`) |
| Regular accent | `.lpf("[260 260 720 260]*4")` (accent still ≤ 800) |
| Darker body | base numbers `180–220`, or `.lpenv(2.5)` |
| Longer scream | `.lpdecay(0.14)` (starts to smear at 16ths) |
| Darker / opener sub | bass `.lpf(100)` … `.lpf(280)` |
| Sub LFO | bass `.lpf(sine.rangex(70, 200).slow(8))` (not on hook) |
| Other kit | swap `bd:` / `hh:` slugs; optional `[~ hh]*4` → `hh*8` |
| Key jump | fifth up or fourth down — not C D Eb E in song order |
| House backbeat | do **not** add it here — use [strudel-genre-house](../strudel-genre-house/SKILL.md) |
| More open 303 | raise body bases; keep accent bases in **600–800** and `.lpenv(3)` |

Do not add a second 303 on lead or bass.

## Rules (do not skip)

1. The 303 sound is **`.lpenv` + high `.lpq` + low/mid `.lpf` base**. Amp ADSR is only the gate. It lives on **`// hook`**.
2. Accent = patterned **base** `.lpf` in **600–800 Hz**, not `900` + `lpenv(3.5)`, not patterned `lpenv`, and not `.gain`.
3. Drums stay one `$:` and **no clap**. Kit and hat density may change. `[~ sd]*2` only if you label **house** (or say hybrid).
4. Do not put `.compressor` on the acid line. Do not use duck for this recipe.
5. Shared clock is **130**. Play solo — another 303 on B is not a mix pair.
6. Do not put `.scale("<…>")` on the 303 line.
7. Do not ship 02–09 as the same 303 bed with only a rising key. Jump the key. Change the sub filter and the drum kit.

## Checklist

- [ ] `setcpm(130/4)` + **7 `$:`** (drums, bass, hook=303, lead, arp, chords, pad). 7 is OK
- [ ] Exact 303 note string, lpf pattern, `lpq(14)`, `lpenv(3)`, lp ADSR, `cut(1)`, amp ADSR on hook
- [ ] 303 scale stays a **scalar** (`C2:minor` or a jumped root). Other pitched tracks use 4-bar `.scale("<…>")`
- [ ] No second 303. No clap. Sub filter is not a copy of `.lpf(180)` on every song. No LFO on hook
- [ ] Non-303 `.s()` from the Timbre palette — drums kit and lead/pad differ from the last acid song
- [ ] Play **solo** at 130. `songs/acid/01.strudel` matches this fence’s 303 line and grid

## Do not

- Ship a parked `.lpf(800).lpq(16)` + amp ADSR and call it a filter envelope.
- Use accent base `900` with `.lpenv(3.5)` — peak sits near 10 kHz; the digital saw goes thin.
- Present two 303 files as a DJ mix pair. BPM may match; two 303s stacked is not a mix.
- Add a second 303 on another `$:`.
- Combine `.lpf(sine.rangex(…))` with `.lpenv` — the LFO replaces the env. Sub bass LFO is fine; 303 LFO is not.
- Walk factory songs chromatically (C D Eb E F) with the same kit and the same `.lpf(180)`.
- Write `.lpenv("4 1 4 1")` — parse error / not a pattern.
- Rely on `.lprelease` — it does not run.
- `note("c3'min")` for a triad — suffix is root only; a chord is `note("[0,2,4]")` (and a 303 line should stay mono).
- Put `.scale("<…>")` on the 303 hook line.
- Copy the fence `.s()` on every new apply for non-303 slots.
- 長い PCM（`ld:` / `dr:` / `pf:` / `ps:`、約 8–17 秒）を毎小節撃たない。
- Ship a 2-track loop for a new apply.
- Pair this file with a different `setcpm` (shared clock; the other tempo is discarded).
- `stack()` / `.cpm(130)` / `.lfo()` / `kit:bd`.
