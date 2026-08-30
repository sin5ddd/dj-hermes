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

These five files are original renders from
[rust-fm-synthe](https://github.com/sin5ddd/rust-fm-synthe) (4-op FM, offline).
They are dedicated to the public domain under **CC0 1.0 Universal**.
They are **not** from Sonic Pi / Freesound.

| Path | Role |
|------|------|
| `cp/00.wav` | Dry house 2/4 clap (unpitched HP noise-ish FM). Not a snare stand-in. |
| `lead-fm_pluck.wav` | C3 (MIDI 48) short FM pluck |
| `stab-fm_fifth.wav` | C3 hollow fifth: C and G only (ratios 1 and 3/2). No major third. |
| `stab-fm_major.wav` | C3 major triad: C–E–G (ratios 1, 5/4, 3/2). Bright counterpart to the hollow fifth. |
| `reese-mid.wav` | C3 mid Reese glue, band-pass 800–1200 Hz, no sub |

## Custom samples

You may add your own WAV files under this tree (same layout). Prefer CC0 or
original recordings you own. Do **not** vendor Dirt-Samples (license unclear;
see https://github.com/tidalcycles/Dirt-Samples/issues/19).
