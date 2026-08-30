# Demo sample kit

Default drum one-shots for `s("bd sd hh")` etc.

**配置・命名・bank / フルネームの正本:** [LAYOUT.md](./LAYOUT.md)

Quick layout:

```
samples/
  <name>/00.wav   # variation index 0 (n=0)
  <name>/01.wav   # optional n=1, ...
  <name>.wav      # also accepted as a single variation
```

Bundled extras (same format): `cp/00.wav` (house 2/4 clap), `lead-fm_pluck.wav`, `lead-fm_bell.wav` (C3 inharmonic bell), `keys-fm_ep.wav` (C3 EP, 2×/3× tines), `perc-fm_metal.wav` (unpitched metal), `stab-fm_fifth.wav` (hollow C+G), `stab-fm_major.wav` (C–E–G), `reese-mid.wav`, `pad-fm_fifth.wav` (C3 fifth pad), `bass-fm_sub.wav` (C2 sub), `bass-fm_house.wav` (C3), `reese-dark.wav` (full-range Reese with sub), `lead-supersaw.wav` (C3, ~8 s), `fx-riser_noise.wav`, `fx-uplifter.wav`, `fx-impact_dnb.wav`, `fx-sub_drop.wav` (unpitched FX).

WAV files are stored with **Git LFS**. After clone, or if a `.wav` is a 3-line pointer file:

```bash
git lfs install   # once per machine (clean/smudge hooks)
git lfs pull
```

## Refresh from Sonic Pi (maintainers)

Upstream ships FLAC; this repo only commits 16-bit mono 48 kHz WAV **under subfolders** (e.g. `bd/00.wav`).

```bash
# example
ffmpeg -y -i bd_haus.flac -ac 1 -ar 48000 -sample_fmt s16 samples/bd/00.wav
```

See `LICENSE.md` for mapping and CC0 provenance.

## Adding a kit

Extra WAVs use the same one-level layout (`samples/<name>.wav` or `samples/<name>/00.wav`) and are **Git LFS** (not gitignored). Prefer CC0 or original recordings. Do **not** vendor Dirt-Samples.

Scratch bounce / debug audio stays out of git (`/out/`, `/recordings/`).
