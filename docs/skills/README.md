# Musicality skills (strudel-rs)

Assistant-agnostic recipes for this engine: **to make music of type X, write Strudel like Y**.

These are Cursor-format `SKILL.md` files (`name` + `description` frontmatter). They are **not** the Hermes exhibit copies under `docs/profile/dj-hermes/skills/` — those stay booth/MCP-specific. This tree documents how **this repo** turns notation into rhythm, harmony, genre, and DJ mix.

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
| `.s(...)` | `bd`/`sd`/`hh`/`oh`/`cp` samples; `lead-fm_pluck`; `sawtooth`/`square`/`sine`/`triangle`; `wt_organ` / `wt_bright` |
| `.lpf` / `.lpq` | Dark bass vs acid (high Q) vs open hats |
| ADSR | Pluck vs pad |
| `.room` / `.delay` | Space (orbit-shared FX, ids 1–4 per deck) |
| `.duckorbit` | Kick ducks **that orbit** (put pad **and** bass on it). `duckattack` is recover time |

Bundled one-shots: `samples/bd`, `sd`, `hh`, `oh`, `cp` (`samples/cp/00.wav`), plus `lead-fm_pluck`. `db` is not a sample. Unknown names fail resolve (performance continues). `stack()`, `.cpm()`, and a bare `s("...")` line without `$:` are not song format.

## DJ / mix

- Two decks, one `Transport`.
- Mixer faders + per-deck Hi/Mid/Lo EQ (shelves at 6 kHz / 1 kHz / 200 Hz) + master LPF/HPF.
- Crossfade: `gainA = cos(θ)`, `gainB = sin(θ)` for `θ` in `0 … π/2` (`mixer.rs`). Starts on a bar boundary; `hush` is immediate.
- `.compressor(...)` on a `$:` is **mixer master**, last-write (`engine.rs`) — not a track insert. It will squash the kick.
- Try a pair **at the same BPM**: `strudel-rs dj songs/techno1.strudel songs/ambient1.strudel` (both `setcpm(126/4)`) then `/x 4`. A second file at another `setcpm` does not keep its own tempo. Do not pair 174 DnB with 126 techno.

## Skills in this tree

| Skill | When | Example song |
| --- | --- | --- |
| [four-on-the-floor](./four-on-the-floor/SKILL.md) | Techno kick+offbeat hats (`bd*4, [~ hh]*4`); no clap | `songs/skill-four-on-the-floor.strudel` |
| [house-clap-backbeat](./house-clap-backbeat/SKILL.md) | House clap on 2/4 (`[~ cp]*2`, not stacked with `sd`) + C4:minor pluck | `songs/skill-house-clap-backbeat.strudel` |
| [minor-scale-loop](./minor-scale-loop/SKILL.md) | Short minor bass + triad | `songs/skill-minor-scale-loop.strudel` |
| [drum-and-bass](./drum-and-bass/SKILL.md) | 174 BPM break, drums above sub, square+saw Reese | `songs/skill-drum-and-bass.strudel` |
| [dnb-reese-mid-stab](./dnb-reese-mid-stab/SKILL.md) | 174 BPM break, square C2 sub + `reese-mid` at C4 + hollow-fifth stab | `songs/skill-dnb-reese-mid-stab.strudel` |
| [sidechain-ducking](./sidechain-ducking/SKILL.md) | Techno kick ducks pad **and** bass; short recover; no track compressor | `songs/skill-sidechain-ducking.strudel` |
| [dnb](./dnb/SKILL.md) | Same mix idea as drum-and-bass (shorter path name) | `songs/skill-dnb.strudel` |
| [techno-duck](./techno-duck/SKILL.md) | Same duck idea as sidechain-ducking | `songs/skill-techno-duck.strudel` |
| [acid-303-filter-envelope](./acid-303-filter-envelope/SKILL.md) | TB-303: per-note `lpenv`, high `lpq`, not static `lpf` + amp ADSR | `songs/skill-acid-303-filter-envelope.strudel` |
| [mood-bright-dark](./mood-bright-dark/SKILL.md) | Make a loop brighter/darker (mode, voicing, register, sample, filter) — not a genre change | `songs/skill-mood-dark.strudel` + `songs/skill-mood-bright.strudel` |

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
```

Headless hosts without an audio device: `cargo test --test e2e` renders through `Engine::process` (no ALSA).

## Adding a skill

1. New directory `docs/skills/<name>/SKILL.md` with YAML `name` and `description` (when to use it).
2. Include: when, the exact `$:` pattern, **why it sounds that way** (cite mini/scale/mixer/duck behavior), and a play/dj command.
3. Add a playable `songs/skill-<name>.strudel` (or point at an existing demo). Every `songs/*.strudel` is parsed by `tests/e2e.rs`.
4. Fence only syntax this parser accepts. Lint: `python scripts/lint_strudel_skills.py`.
