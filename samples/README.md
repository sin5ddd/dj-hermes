# Demo sample kit

Default drum one-shots for `s("bd sd hh")` etc.

**配置・命名・bank / フルネームの正本:** [LAYOUT.md](./LAYOUT.md)

Quick layout:

```
samples/
  <name>/00.wav   # variation index 0 (n=0)
  <name>/01.wav   # optional n=1, ...
  <name>.wav      # also accepted as a single variation (user kits; gitignored at repo root)
```

## Refresh from Sonic Pi (maintainers)

Upstream ships FLAC; this repo only commits 16-bit mono 48 kHz WAV **under subfolders** (e.g. `bd/00.wav`).

```bash
# example
ffmpeg -y -i bd_haus.flac -ac 1 -ar 48000 -sample_fmt s16 samples/bd/00.wav
```

See `LICENSE.md` for mapping and CC0 provenance.

## User kits (not in git)

- Place extra WAVs as **flat files** under `samples/` (e.g. `tr808-hard_bd.wav`, `pad-ambient_drone01.wav`, `piano-acoustic_soft.wav`).
- `samples/*.wav` at the **directory root** is **gitignored**.
- Bundled kit folders (`bd/`, `sd/`, …) stay tracked.
- Do **not** vendor Dirt-Samples. Prefer CC0 or original recordings.
- Keep a local provenance note if you redistribute third-party audio outside this repo.
