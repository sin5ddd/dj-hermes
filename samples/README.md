# Demo sample kit

Default drum one-shots for `s("bd sd hh")` etc. Layout:

```
samples/
  <name>/00.wav   # variation index 0 (n=0)
  <name>/01.wav   # optional n=1, ...
  <name>.wav      # also accepted as a single variation
```

## Refresh from Sonic Pi (maintainers)

Upstream ships FLAC; this repo only commits 16-bit mono 48 kHz WAV.

```bash
# example
ffmpeg -y -i bd_haus.flac -ac 1 -ar 48000 -sample_fmt s16 samples/bd/00.wav
```

See `LICENSE.md` for mapping and CC0 provenance.

## Demo hardware kits

Drop custom WAVs into the same folders (or add new names) without code changes.
Keep `LICENSE.md` updated when redistributing third-party audio.
