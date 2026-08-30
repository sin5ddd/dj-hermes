# Sample licenses

All bundled audio under `samples/` is dedicated to the public domain under
**CC0 1.0 Universal**.

The six Sonic Pi drum files below are converted (16-bit mono PCM WAV, 48 kHz) from
[Sonic Pi](https://github.com/sonic-pi-net/sonic-pi) bundled samples
(`etc/samples/`), which Sonic Pi documents as CC0:

- https://github.com/sonic-pi-net/sonic-pi/blob/dev/LICENSE.md
- https://github.com/sonic-pi-net/sonic-pi/blob/dev/etc/samples/README.md

Sonic Pi obtained most of these from [Freesound](https://freesound.org)
under CC0. Conversion does not change the license.

## File mapping

| Path | Sonic Pi source | Freesound / note |
|------|-----------------|------------------|
| `bd/00.wav` | `bd_haus.flac` | https://freesound.org/people/Rodrigo%20The%20Mad/sounds/137722/ |
| `bd/01.wav` | `bd_tek.flac` | https://freesound.org/people/DWSD/sounds/171104/ |
| `sd/00.wav` | `sn_dolf.flac` | https://freesound.org/people/Dolfeus/sounds/57534/ |
| `sd/01.wav` | `sn_zome.flac` | https://freesound.org/people/Dolfeus/sounds/55232/ |
| `hh/00.wav` | `drum_cymbal_closed.flac` | https://www.freesound.org/people/menegass/sounds/100053/ |
| `oh/00.wav` | `drum_cymbal_open.flac` | https://www.freesound.org/people/menegass/sounds/100055/ |

## Bundled FM one-shots (CC0)

These seventeen files are original renders from
[rust-fm-synthe](https://github.com/sin5ddd/rust-fm-synthe) (4-op FM, offline).
They are dedicated to the public domain under **CC0 1.0 Universal**.
They are **not** from Sonic Pi / Freesound.

Pitched stems target **C3 (~131 Hz)** except the two bass one-shots
(`bass-fm_sub`, `bass-fm_house`), which are native **C2 (~65 Hz)** — same
`C4:…` playback convention as each other (house floor sits with the kick).
FX stems use the factory `default_note` / `default_duration` (no `--note` retune).

| Path | Role |
|------|------|
| `cp/00.wav` | Dry house 2/4 clap (unpitched HP noise-ish FM). Not a snare stand-in. |
| `lead-fm_pluck.wav` | C3 (MIDI 48) short FM pluck |
| `lead-fm_bell.wav` | C3 (MIDI 48) bell / glass. Inharmonic (ratio 3.5). Short-medium decay, not a pad. |
| `keys-fm_ep.wav` | C3 (MIDI 48) electric-piano-ish FM. Harmonic 2×/3× tines plus `.fm(2).fmh(1)` attack bite. Tonal one-shot. |
| `perc-fm_metal.wav` | Unpitched metallic hit (`.fm(8).fmh(11)`). Short. Not a chord stab or kick. |
| `stab-fm_fifth.wav` | C3 hollow fifth: C and G only (ratios 1 and 3/2). No major third. |
| `stab-fm_major.wav` | C3 major triad: C–E–G (ratios 1, 5/4, 3/2). Bright counterpart to the hollow fifth. |
| `reese-mid.wav` | C3 mid Reese glue, band-pass 800–1200 Hz, no sub |
| `pad-fm_fifth.wav` | C3 (MIDI 48) fifth pad: sustained C+G, thin harmonics (~8.2 s). Not a stab. |
| `bass-fm_sub.wav` | C2 (~65 Hz) clean sine sub (factory ratio 0.5). Almost no click or grit. Same native-C2 / `C4:…` convention as `bass-fm_house`. |
| `bass-fm_house.wav` | C2 (~65 Hz) tight house floor bass. Short decay, sidechain-friendly. Not Eb (factory +3.5 st scoop). Sits with the kick, not on top of it. |
| `reese-dark.wav` | C3 (MIDI 48) dark full-range Reese (sub + mid). Not the 800–1200 Hz `reese-mid` glue. |
| `lead-supersaw.wav` | C3 (~131 Hz) classic supersaw lead. ~8.2 s hold (4 bars @ 120). Factory HP 220 Hz hid C3; this stem lets the C3 fundamental through. |
| `fx-riser_noise.wav` | Unpitched noise riser (~3.2 s). Factory default_note/duration. |
| `fx-uplifter.wav` | Unpitched uplifter: pitch + filter open (~2.8 s). Factory duration. |
| `fx-impact_dnb.wav` | Unpitched DnB impact: tight mid hit + short grit (~0.5 s). |
| `fx-sub_drop.wav` | Unpitched sub drop: large pitch fall (~1.1 s). Kick-lead-in, not a bass note. |

## Custom samples

You may add your own WAV files under this tree (same layout). Prefer CC0 or
original recordings you own. Do **not** vendor Dirt-Samples (license unclear;
see https://github.com/tidalcycles/Dirt-Samples/issues/19).
