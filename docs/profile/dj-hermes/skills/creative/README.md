# Musicality skills (strudel-rs)

Assistant-agnostic recipes for this engine: **to make music of type X, write Strudel like Y**.

These are Hermes-format `SKILL.md` files (`name`, `description` starting with “Use when”, `version`, `author`, `license`, `metadata.hermes`). Folders are named `strudel-*` (genre recipes `strudel-genre-*`). This tree documents how **this repo** turns notation into rhythm, harmony, genre, and DJ mix.

Engine-accurate recipes (formerly unprefixed folders such as `four-on-the-floor`) were merged into the matching `strudel-*` skill, or renamed when there was no overlap. Song files under `songs/skill-*.strudel` keep their original names. This directory is the canonical skill tree and the copy source for the live `dj-hermes` profile. `strudel-live-edit` also lives here (natural-language live edits); it is not a song-recipe skill.

Do not invent syntax from the public Strudel REPL. Only patterns that parse and play here belong in a skill.

## How notation becomes sound

```
.strudel text
    │  song.rs — setcpm / $: tracks / @tags
    ▼
PatternCode (code.rs) — note()/s() + method chain
    │
    ├─ mini.rs     — string → timed events in one cycle (1 bar)
    ├─ scale.rs    — integer degrees → Hz (optional)
    ├─ sound.rs    — waveform / wavetable / SampleBank
    └─ synth.rs    — voice (osc + ADSR + per-voice FX)
         │
         ▼
Deck A / Deck B  ← same Transport (shared BPM, 1 bar = 4 beats)
         │
         ▼
Mixer — faders, 3-band EQ, master filter, compressor, equal-power xfade
         │
         ▼
AudioBackend (cpal) or Engine::process (NullBackend / tests)
```

| Layer | What it decides | Code |
| --- | --- | --- |
| Song | Tempo, track list, metadata | `src/song.rs` |
| Mini-notation | *When* hits fire inside a bar | `src/mini.rs` |
| Scale / notes | *Which pitches* | `src/scale.rs`, `src/code.rs` |
| Sound | *What timbre* | `src/sound.rs`, `samples/` |
| Deck | Schedule voices for one song | `src/deck.rs` |
| Transport | Shared clock; both decks lock | `src/transport.rs` |
| Mixer / DJ | How A and B are heard together | `src/mixer.rs`, `src/engine.rs` |

Parse failures do not stop the current performance; the engine reports the error and keeps playing.

## Rhythm (mini-notation)

One **cycle = one bar = four beats**. `events()` in `mini.rs` places each atom in `[0, 1)` of that bar.

| Syntax | Engine meaning | Musical result |
| --- | --- | --- |
| `bd sd hh oh` | Space = sequence; equal time unless `@` | Four hits, one per beat |
| `bd,sd` / `[bd,sd]` | Comma = **parallel** (same span) | Unison / layered hits |
| `bd*4` | Fast: *n* copies of the inner span | Four kicks on the beat |
| `bd/2` | Slow: fire every *n* cycles | Half-time |
| `[~ sd]` | Brackets = subdivide that span | Rest then snare in that cell |
| `<a b>` | One child per cycle (not unison) | Alternate bars |
| `a@2 b` | Elongate: *a* gets twice *b*'s weight | Uneven grid |
| `~` | Rest (no event) | Silence |

`setcpm(N)` is cycles per minute. Engine BPM is `N * 4`. `setcpm(124/4)` is 124 BPM. `setcps(x)` is `BPM = x * 240`. Both decks share **one** master tempo: a DJ pair must use the same `setcpm` or the second file’s tempo is discarded.

## Harmony

Integer atoms plus `.scale("RootOct:mode")` are **0-based degrees** relative to the root (`scale.rs`). Negative degrees wrap below the root (`C2:major` `-1` → B1).

| Mode string | Semitones from root |
| --- | --- |
| `major` / `ionian` | 0 2 4 5 7 9 11 |
| `minor` / `aeolian` | 0 2 **3** 5 7 8 10 |
| `dorian` | 0 2 3 5 7 9 10 |
| `phrygian` | 0 1 3 5 7 8 10 |
| `lydian` | 0 2 4 6 7 9 11 |
| `mixolydian` | 0 2 4 5 7 9 10 |
| `locrian` | 0 1 3 5 6 8 10 |
| `major:pentatonic` / `pentatonic` | 0 2 4 7 9 |
| `minor:pentatonic` | 0 3 5 7 10 |
| `chromatic` | 0–11 |

Root without an octave defaults to **octave 4** (`C:major` → C4). Bass lines should set the octave (`C2:minor`).

**Chords that actually sound as chords:** parallel degrees or named notes in the same span — `note("[0,2,4]")`, `note("[c3,eb3,g3]")`. Deck scheduling (`deck.rs`) resolves **one pitch per mini event**. `expand_chord("c3'maj")` exists in `code.rs` and is unit-tested, but the deck does **not** call it: `note("c3'maj")` plays the **root only**.

Per-bar changes of key/mode: `.scale("<A2:minor D:dorian G:mixolydian C:major>")` (whitespace-separated `Root:mode` inside `<>`).

`.add(n)` / `.sub(n)` shift degrees (or named notes in semitones).

## Timbre and genre

Genre in this repo is mostly **tempo + drum grid + register + filter**, not a hidden style engine.

| Knob | Typical use |
| --- | --- |
| `setcpm` | House ~120–128, techno ~126–130, DnB ~170+, ambient can sit on a shared DJ BPM |
| Drum string | Four-on-the-floor vs 2-step vs break |
| `.s(...)` | `bd`/`sd`/`hh`/`oh`/`cp` samples; factory FM wavs (`plk:lp`, `plk:s5`/`plk:s3`, `bs:hf`, `bs:dk`, `pf:ff`, `ld:ss`, `fx:up`/`fx:nr`/`fx:id`/`fx:sd`); `sawtooth`/`square`/`sine`/`triangle` + live `.fm`; `wt_organ` / `wt_bright` |
| `.lpf` / `.lpq` | Dark bass vs acid (high Q) vs open hats |
| ADSR | Pluck vs pad |
| `.room` / `.delay` | Space (orbit-shared FX, ids 1–4 per deck) |
| `.duckorbit` | Kick ducks **that orbit** (put pad **and** bass on it). `duckattack` is recover time |

Bundled one-shots: `samples/bd`, `sd`, `hh`, `oh`, `cp` (`samples/cp/00.wav`), plus factory FM wavs as `part:slug` (`plk:lp`, `plk:s5`, `plk:s3`, `bs:rm`, `bs:hf`, `bs:su`, `bs:dk`, `pf:ff`, `ld:ss`, `fx:up` / `fx:nr` / `fx:id` / `fx:sd` — how to play each: [strudel-sound-design](./strudel-sound-design/SKILL.md) (Factory PCM batch 1) and [strudel-pcm-catalog](./strudel-pcm-catalog/SKILL.md)). Live 2-op FM is `.s("sine").fm(…)` — not those wavs. `db` is not a sample. Unknown names fail resolve (performance continues). `stack()`, `.cpm()`, and a bare `s("...")` line without `$:` are not song format.

## DJ / mix

- Two decks, one `Transport`.
- Mixer faders + per-deck Hi/Mid/Lo EQ (shelves at 6 kHz / 1 kHz / 200 Hz) + master LPF/HPF.
- Crossfade: `gainA = cos(θ)`, `gainB = sin(θ)` for `θ` in `0 … π/2` (`mixer.rs`). Starts on a bar boundary; `hush` is immediate.
- `.compressor(...)` on a `$:` is **mixer master**, last-write (`engine.rs`) — not a track insert. It will squash the kick.
- Try a pair **at the same BPM**: `strudel-rs dj songs/techno1.strudel songs/ambient1.strudel` (both `setcpm(126/4)`) then `/x 4`. A second file at another `setcpm` does not keep its own tempo. Do not pair 174 DnB with 126 techno.

## Skills in this tree

Cross-cutting:

| Skill | When | Example song |
| --- | --- | --- |
| [strudel-composition](./strudel-composition/SKILL.md) | Mini-notation + `$:` tracks | — |
| [strudel-data-format](./strudel-data-format/SKILL.md) | `.strudel` save/load shape | — |
| [strudel-sound-design](./strudel-sound-design/SKILL.md) | Synths, FX, live 2-op FM, factory PCM stems | `songs/skill-fm-sound-design.strudel`, `songs/skill-factory-pcm-usage.strudel` |
| [strudel-pcm-catalog](./strudel-pcm-catalog/SKILL.md) | rust-fm-synthe `part:slug`（`bd:8b`, `hh:cl`）。意味は INDEX | `songs/skill-pcm-catalog.strudel` |
| [strudel-minor-scale-loop](./strudel-minor-scale-loop/SKILL.md) | Short minor bass + triad | `songs/skill-minor-scale-loop.strudel` |
| [strudel-mood-bright-dark](./strudel-mood-bright-dark/SKILL.md) | Brighter/darker (mode, voicing, register, sample, filter) — not a genre change | `songs/skill-mood-dark.strudel` + `songs/skill-mood-bright.strudel` |
| [strudel-live-edit](./strudel-live-edit/SKILL.md) | Natural language → one-track / one-method live edits | — |
| [strudel-dj-mix](./strudel-dj-mix/SKILL.md) | A/B mix macros (`strudel_mix` one call: long / cut / fill / switch / hold) | — |

Genre recipes (`strudel-genre-*`):

| Skill | When | Example song |
| --- | --- | --- |
| [strudel-genre-four-on-the-floor](./strudel-genre-four-on-the-floor/SKILL.md) | Techno kick+offbeat hats (`bd*4, [~ hh]*4`); no clap | `songs/skill-four-on-the-floor.strudel` |
| [strudel-genre-house](./strudel-genre-house/SKILL.md) | House clap on 2/4 (`[~ cp]*2`, not stacked with `sd`) + C4:minor pluck | `songs/skill-house-clap-backbeat.strudel` |
| [strudel-genre-techno-duck](./strudel-genre-techno-duck/SKILL.md) | Techno kick ducks pad **and** bass; short recover; no track compressor | `songs/skill-sidechain-ducking.strudel` (`songs/skill-techno-duck.strudel` is the older sibling) |
| [strudel-genre-acid](./strudel-genre-acid/SKILL.md) | TB-303: per-note `lpenv`, high `lpq`, not static `lpf` + amp ADSR | `songs/skill-acid-303-filter-envelope.strudel` |
| [strudel-genre-dnb](./strudel-genre-dnb/SKILL.md) | 174 BPM break, drums above sub, square+saw Reese; mini `*2` not `.fast(2)` | `songs/skill-drum-and-bass.strudel` (`songs/skill-dnb.strudel` is the shorter sibling) |
| [strudel-genre-dnb-reese-mid-stab](./strudel-genre-dnb-reese-mid-stab/SKILL.md) | 174 BPM break, square C2 sub + `bs:rm` at C4 + hollow-fifth stab | `songs/skill-dnb-reese-mid-stab.strudel` |
| [strudel-genre-ambient](./strudel-genre-ambient/SKILL.md) | Ambient | — |
| [strudel-genre-chill](./strudel-genre-chill/SKILL.md) | Chill / downtempo | — |
| [strudel-genre-chill-pop](./strudel-genre-chill-pop/SKILL.md) | Chill Pop | — |
| [strudel-genre-dubstep](./strudel-genre-dubstep/SKILL.md) | Dubstep | — |
| [strudel-genre-electro](./strudel-genre-electro/SKILL.md) | Electro | — |
| [strudel-genre-future-bass](./strudel-genre-future-bass/SKILL.md) | Future Bass | — |
| [strudel-genre-lofi-hiphop](./strudel-genre-lofi-hiphop/SKILL.md) | Lo-fi hip hop | — |
| [strudel-genre-minimal-techno](./strudel-genre-minimal-techno/SKILL.md) | Minimal Techno | — |
| [strudel-genre-progressive-house](./strudel-genre-progressive-house/SKILL.md) | Progressive House | — |

`songs/acid16.strudel` is the static-cutoff live loop (same 130 BPM). Clock would align; both files are 303 lines, so A/B would double the acid — not a mix reason. Do not treat `acid16` as the envelope recipe.

Other demos: `songs/house16.strudel`, `garage16.strudel`, `ambient1.strudel`, `dnb16.strudel`, `techno1.strudel`.

## How to try any example

From the repo root (needs `songs/` and `samples/`):

```bash
# Highlight TUI (q / Esc to quit)
strudel-rs play songs/skill-four-on-the-floor.strudel --seconds 12

# No TTY / CI
strudel-rs play songs/skill-minor-scale-loop.strudel --headless --seconds 8

# Dual deck — both files must share one setcpm (here 124/4)
strudel-rs dj songs/skill-four-on-the-floor.strudel songs/skill-minor-scale-loop.strudel

strudel-rs play songs/skill-drum-and-bass.strudel --seconds 12
strudel-rs play songs/skill-sidechain-ducking.strudel --headless --seconds 8

# Dual deck — both files must share one setcpm (here 126/4). Do not pair with 174 DnB.
strudel-rs dj songs/skill-sidechain-ducking.strudel songs/ambient1.strudel

strudel-rs play songs/skill-acid-303-filter-envelope.strudel --seconds 12
strudel-rs play songs/skill-acid-303-filter-envelope.strudel --headless --seconds 8

strudel-rs play songs/skill-house-clap-backbeat.strudel --seconds 12
strudel-rs play songs/skill-house-clap-backbeat.strudel --headless --seconds 8

# Dual deck — both files must share one setcpm (here 124/4)
strudel-rs dj songs/skill-house-clap-backbeat.strudel songs/skill-minor-scale-loop.strudel

# 174 DnB with sampled mid Reese — play solo. Do not pair with 124 house.
strudel-rs play songs/skill-dnb-reese-mid-stab.strudel --seconds 12
strudel-rs play songs/skill-dnb-reese-mid-stab.strudel --headless --seconds 8

# Mood pair — same 124 house grid; contrast is harmonic/timbre. Do not pair with 174.
strudel-rs play songs/skill-mood-dark.strudel --seconds 12
strudel-rs play songs/skill-mood-bright.strudel --seconds 12
strudel-rs play songs/skill-mood-dark.strudel --headless --seconds 8
strudel-rs play songs/skill-mood-bright.strudel --headless --seconds 8
strudel-rs dj songs/skill-mood-dark.strudel songs/skill-mood-bright.strudel

# Live FM growl+lead+pad at 124 (PCM drums). Same clock as house/mood.
# Do not DJ-pair a leftover 120 file with this 124 file.
strudel-rs play songs/skill-fm-sound-design.strudel --seconds 12
strudel-rs play songs/skill-fm-sound-design.strudel --headless --seconds 8

# rust-fm-synthe catalog — house floor kick + tight bass + uplifter (`bd:hf`)
strudel-rs play songs/skill-pcm-catalog.strudel --seconds 12
strudel-rs play songs/skill-pcm-catalog.strudel --headless --seconds 8

# Factory PCM batch 1 — floor (drums+bass+pad), separate Reese bed, supersaw lead.
strudel-rs play songs/skill-factory-pcm-usage.strudel --seconds 12
strudel-rs play songs/skill-factory-pcm-usage.strudel --headless --seconds 8
strudel-rs play songs/skill-factory-pcm-reese.strudel --seconds 12
strudel-rs play songs/skill-factory-pcm-reese.strudel --headless --seconds 8
strudel-rs play songs/skill-factory-pcm-lead.strudel --seconds 12
strudel-rs play songs/skill-factory-pcm-lead.strudel --headless --seconds 8
strudel-rs dj songs/skill-factory-pcm-usage.strudel songs/skill-factory-pcm-lead.strudel
```

Headless hosts without an audio device: `cargo test --test e2e` renders through `Engine::process` (no ALSA).

## Adding a skill

1. New directory `docs/profile/dj-hermes/skills/creative/strudel-<name>/SKILL.md` (genre recipes: `strudel-genre-<name>`). Hermes YAML: `name`, `description` starting with “Use when”, `version`, `author`, `license`, `metadata.hermes` (`tags`, `related_skills`).
2. Include: when, the exact `$:` pattern, **why it sounds that way** (cite mini/scale/mixer/duck behavior), and a play/dj command.
3. Add a playable `songs/skill-<name>.strudel` (or point at an existing demo). Every `songs/*.strudel` is parsed by `tests/e2e.rs`.
4. Fence only syntax this parser accepts (`setcpm` + `$:`. No `stack()` / `.cpm()`).
