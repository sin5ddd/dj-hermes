#!/usr/bin/env python3
"""Gate bundled songs by integrated LUFS (ITU-R BS.1770-4).

Renders offline via `strudel-rs render`, measures with pyloudnorm, writes
eval/lufs/runs/<timestamp>/. Real-time playback is not used.
"""
from __future__ import annotations

import argparse
import json
import math
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SONGS = ROOT / "songs"
DEFAULT_RUNS = ROOT / "eval" / "lufs" / "runs"
DEFAULT_MIN_LUFS = -28.0
DEFAULT_BARS = 16
DEFAULT_WARMUP = 1


def collect_songs(root: Path) -> list[Path]:
    """Files in `root` plus one directory level (same as song::collect_song_files)."""
    root = root.resolve()
    if root.is_file():
        if root.suffix.lower() == ".strudel":
            return [root]
        raise SystemExit(f"not a .strudel file: {root}")
    if not root.is_dir():
        raise SystemExit(f"songs dir not found: {root}")
    out: list[Path] = []
    out.extend(sorted(p for p in root.glob("*.strudel") if p.is_file()))
    for sub in sorted(p for p in root.iterdir() if p.is_dir()):
        out.extend(sorted(p for p in sub.glob("*.strudel") if p.is_file()))
    return out


def find_render_prefix(root: Path, explicit: str | None) -> list[str]:
    if explicit:
        name = Path(explicit).name.lower()
        if name not in ("strudel-rs", "strudel-rs.exe"):
            raise SystemExit("--bin must be a strudel-rs executable")
        return [str(Path(explicit))]
    exe = "strudel-rs.exe" if os.name == "nt" else "strudel-rs"
    release = root / "target" / "release" / exe
    if release.is_file():
        return [str(release)]
    return ["cargo", "run", "--release", "--quiet", "--"]


def relative_song_id(songs_root: Path, path: Path) -> str:
    try:
        rel = path.resolve().relative_to(songs_root.resolve())
    except ValueError:
        rel = path.name
    return rel.as_posix()


def genre_of(songs_root: Path, path: Path) -> str:
    parent = path.parent.resolve()
    root = songs_root.resolve()
    if parent == root:
        return ""
    return parent.name


def measure_lufs(wav_path: Path) -> tuple[float, float, float]:
    """Return (integrated_lufs, peak, rms)."""
    try:
        import numpy as np
        import pyloudnorm as pyln
        import soundfile as sf
    except ImportError as e:
        raise SystemExit(
            "missing LUFS deps. From repo root:\n"
            "  pip install -r scripts/requirements-eval.txt\n"
            f"({e})"
        ) from e

    data, rate = sf.read(str(wav_path), always_2d=True)
    peak = float(np.max(np.abs(data))) if data.size else 0.0
    rms = float(np.sqrt(np.mean(np.square(data)))) if data.size else 0.0
    if data.size == 0 or peak < 1e-12:
        return (float("-inf"), peak, rms)
    meter = pyln.Meter(int(rate))
    try:
        lufs = float(meter.integrated_loudness(data))
    except Exception:
        return (float("-inf"), peak, rms)
    if not math.isfinite(lufs):
        lufs = float("-inf")
    return (lufs, peak, rms)


def fmt_lufs(v: float | None) -> str:
    if v is None or not math.isfinite(v):
        return "-inf"
    return f"{v:.2f}"


def write_reports(run_dir: Path, payload: dict) -> None:
    run_dir.mkdir(parents=True, exist_ok=True)
    (run_dir / "summary.json").write_text(
        json.dumps(payload, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    (run_dir / "songs.json").write_text(
        json.dumps(payload["songs"], indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )

    below = [s for s in payload["songs"] if s["status"] == "below"]
    errors = [s for s in payload["songs"] if s["status"] == "error"]

    lines = [
        f"# LUFS gate {payload['started_utc']}",
        "",
        f"- min_lufs: {payload['min_lufs']}",
        f"- bars: {payload['bars']} (warmup {payload['warmup_bars']})",
        f"- songs: {payload['song_count']}",
        f"- below: {payload['below_count']}",
        f"- errors: {payload['error_count']}",
        "",
    ]
    if below:
        lines.append("## Below threshold")
        lines.append("")
        lines.append("| LUFS | path | genre | peak |")
        lines.append("| ---: | --- | --- | ---: |")
        for s in below:
            lines.append(
                f"| {fmt_lufs(s['lufs'])} | `{s['id']}` | {s['genre']} | {s['peak']:.3f} |"
            )
        lines.append("")
    if errors:
        lines.append("## Errors")
        lines.append("")
        for s in errors:
            lines.append(f"- `{s['id']}`: {s.get('error', '')}")
        lines.append("")

    lines.append("## All (by genre)")
    lines.append("")
    by_genre: dict[str, list] = {}
    for s in payload["songs"]:
        by_genre.setdefault(s["genre"] or "(root)", []).append(s)
    for genre in sorted(by_genre):
        lines.append(f"### {genre}")
        lines.append("")
        lines.append("| file | LUFS | peak | status |")
        lines.append("| --- | ---: | ---: | --- |")
        for s in by_genre[genre]:
            lines.append(
                f"| `{s['id']}` | {fmt_lufs(s['lufs'])} | {s['peak']:.3f} | {s['status']} |"
            )
        lines.append("")
    (run_dir / "summary.md").write_text("\n".join(lines), encoding="utf-8")

    q = [
        f"# Below {payload['min_lufs']} LUFS",
        "",
        "Human next step: keep / recreate / hold.",
        "",
    ]
    if not below:
        q.append("None.")
        q.append("")
    else:
        q.append("| LUFS | path | genre | peak |")
        q.append("| ---: | --- | --- | ---: |")
        for s in below:
            q.append(
                f"| {fmt_lufs(s['lufs'])} | `{s['id']}` | {s['genre']} | {s['peak']:.3f} |"
            )
        q.append("")
    (run_dir / "below-threshold.md").write_text("\n".join(q), encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Render songs and flag those below an integrated LUFS floor."
    )
    parser.add_argument(
        "--songs",
        type=Path,
        default=DEFAULT_SONGS,
        help="songs/ dir, a genre dir, or one .strudel file",
    )
    parser.add_argument("--min-lufs", type=float, default=DEFAULT_MIN_LUFS)
    parser.add_argument("--bars", type=int, default=DEFAULT_BARS)
    parser.add_argument("--warmup-bars", type=int, default=DEFAULT_WARMUP)
    parser.add_argument("--bin", default=None, help="strudel-rs binary (else release, else cargo)")
    parser.add_argument("--samples-dir", type=Path, default=None)
    parser.add_argument(
        "--out",
        type=Path,
        default=None,
        help="run directory (default eval/lufs/runs/<timestamp>)",
    )
    parser.add_argument("--keep-wav", action="store_true")
    parser.add_argument(
        "--no-fail",
        action="store_true",
        help="always exit 0 (still write the report)",
    )
    args = parser.parse_args()
    if args.bars < 1:
        raise SystemExit("--bars must be >= 1")

    catalog_root = DEFAULT_SONGS if DEFAULT_SONGS.is_dir() else (
        args.songs if args.songs.is_dir() else args.songs.parent
    )
    resolved = args.songs.resolve()
    songs_default = DEFAULT_SONGS.resolve()
    if resolved == songs_default or songs_default in resolved.parents:
        catalog_root = DEFAULT_SONGS

    paths = collect_songs(args.songs)
    if not paths:
        raise SystemExit(f"no .strudel files under {args.songs}")

    stamp = datetime.now(timezone.utc).strftime("%Y%m%d-%H%M%S")
    run_dir = args.out if args.out is not None else DEFAULT_RUNS / stamp
    wav_dir = run_dir / "wav"
    wav_dir.mkdir(parents=True, exist_ok=True)

    prefix = find_render_prefix(ROOT, args.bin)
    records: list[dict] = []

    for i, path in enumerate(paths, start=1):
        sid = relative_song_id(catalog_root, path)
        genre = genre_of(catalog_root, path)
        wav_path = wav_dir / Path(sid).with_suffix(".wav")
        rec = {
            "id": sid,
            "path": str(path),
            "genre": genre,
            "lufs": None,
            "peak": 0.0,
            "rms": 0.0,
            "status": "error",
            "error": None,
        }
        print(f"[{i}/{len(paths)}] render {sid}", flush=True)
        cmd = prefix + [
            "render",
            str(path),
            "--out",
            str(wav_path),
            "--bars",
            str(args.bars),
            "--warmup-bars",
            str(args.warmup_bars),
        ]
        if args.samples_dir is not None:
            cmd += ["--samples-dir", str(args.samples_dir)]
        try:
            # argv list (no shell). Binary is cargo or a strudel-rs executable.
            proc = subprocess.run(
                cmd,
                cwd=ROOT,
                capture_output=True,
                text=True,
            )
        except FileNotFoundError as e:
            rec["error"] = f"render not found: {e}"
            records.append(rec)
            continue
        if proc.returncode != 0:
            err = (proc.stderr or proc.stdout or "").strip()
            rec["error"] = f"render exit {proc.returncode}: {err[-400:]}"
            records.append(rec)
            continue
        try:
            lufs, peak, rms = measure_lufs(wav_path)
        except SystemExit:
            raise
        except Exception as e:
            rec["error"] = f"measure: {e}"
            records.append(rec)
            continue
        rec["lufs"] = round(lufs, 3) if math.isfinite(lufs) else None
        rec["peak"] = round(peak, 4)
        rec["rms"] = round(rms, 4)
        rec["error"] = None
        measured = rec["lufs"]
        if measured is None or measured < args.min_lufs:
            rec["status"] = "below"
        else:
            rec["status"] = "pass"
        records.append(rec)
        if not args.keep_wav:
            try:
                wav_path.unlink(missing_ok=True)
            except OSError:
                pass

    if not args.keep_wav:
        try:
            wav_dir.rmdir()
        except OSError:
            pass

    below_count = sum(1 for r in records if r["status"] == "below")
    error_count = sum(1 for r in records if r["status"] == "error")
    payload = {
        "started_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "min_lufs": args.min_lufs,
        "bars": args.bars,
        "warmup_bars": args.warmup_bars,
        "songs_root": str(args.songs),
        "song_count": len(records),
        "below_count": below_count,
        "error_count": error_count,
        "render_prefix": prefix,
        "songs": records,
    }
    write_reports(run_dir, payload)
    print(f"report: {run_dir}", flush=True)
    print(
        f"below={below_count} errors={error_count} / {len(records)}  min_lufs={args.min_lufs}",
        flush=True,
    )
    if args.no_fail:
        return 0
    return 1 if (below_count or error_count) else 0


if __name__ == "__main__":
    sys.exit(main())
