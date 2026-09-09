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

Bundled extras (same format, `s("part:slug")`): `cp/00.wav` (house 2/4 clap), `plk/lp.wav` (`plk:lp`), `plk/bl.wav` (`plk:bl`), `ep/ky.wav` (`ep:ky`), `perc/fm.wav` (`perc:fm`), `plk/s5.wav` (`plk:s5`), `plk/s3.wav` (`plk:s3`), `bs/rm.wav` (`bs:rm`), `pf/ff.wav` (`pf:ff`), `bs/su.wav` / `bs/hf.wav` (`bs:su` / `bs:hf`, C2, write `C4:…`), `bs/dk.wav` (`bs:dk`), `ld/ss.wav` (`ld:ss`), `fx/nr.wav` / `fx/up.wav` / `fx/id.wav` / `fx/sd.wav` (unpitched FX), `vc/pa.wav` etc. (`vc:pa` … `vc:yeah`, C4 vocal chops, ~3.2 s).

WAV files are stored with **Git LFS**. After clone, or if a `.wav` is a 3-line pointer file:

```bash
git lfs install   # once per machine (clean/smudge hooks)
git lfs pull
```

If a `.wav` on disk is still a pointer, the engine follows it into `.git/lfs/objects`. `git lfs pull` is still required so the objects exist locally. Prefer `git lfs checkout` so the working tree has real WAVs.

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
