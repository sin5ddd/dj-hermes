#!/usr/bin/env python3
"""Copy rust-fm-synthe dist WAVs into samples/<part>/<slug>.wav and emit INDEX.

Usage (from repo root):
  python scripts/import_fm_pcm.py
  python scripts/import_fm_pcm.py --all
  python scripts/import_fm_pcm.py --dry-run
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
import tomllib
from pathlib import Path

from fm_pcm_slugs import (
    BUNDLED_FLATS,
    DEFAULT_PARTS,
    ENTRIES,
    LONG_PARTS,
    emit_toml,
    validate,
)
import shutil

REPO = Path(__file__).resolve().parent.parent
SKILL_DIR = REPO / "docs" / "skills" / "strudel-pcm-catalog"
ID_RE = re.compile(r"^[a-z0-9-]+$")
PART_RE = re.compile(r"^[a-z]+$")
SLUG_RE = re.compile(r"^[a-z0-9]{2,4}$")


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument(
        "--src",
        type=Path,
        default=REPO.parent / "rust-fm-synthe",
        help="rust-fm-synthe repo root (presets/ + dist/)",
    )
    p.add_argument("--all", action="store_true", help="also copy long holds (ld/dr/pf/ps)")
    p.add_argument("--dry-run", action="store_true")
    p.add_argument("--skip-wav", action="store_true", help="only write INDEX + slugs.toml")
    return p.parse_args()


def load_preset_meta(presets_dir: Path) -> dict[str, dict]:
    out: dict[str, dict] = {}
    for path in presets_dir.rglob("*.toml"):
        data = tomllib.loads(path.read_text(encoding="utf-8"))
        out[path.stem] = {
            "name": str(data.get("name") or path.stem),
            "description": str(data.get("description") or ""),
            "default_note": data.get("default_note"),
            "default_duration": data.get("default_duration"),
            "folder": path.parent.name,
        }
    return out


def relocate_bundled(dry_run: bool) -> tuple[int, int]:
    """Move samples/<old>.wav onto samples/<part>/<slug>.wav."""
    moved = 0
    missing = 0
    samples = REPO / "samples"
    for old, part, slug, _ow, _n, _d in BUNDLED_FLATS:
        src = samples / f"{old}.wav"
        dest = safe_dest(part, slug)
        if not src.is_file():
            if dest.is_file():
                continue
            missing += 1
            print(f"bundled missing {src.name}", file=sys.stderr)
            continue
        if dry_run:
            print(f"move {src.relative_to(REPO)} -> {dest.relative_to(REPO)}")
            moved += 1
            continue
        dest.parent.mkdir(parents=True, exist_ok=True)
        if dest.resolve() != src.resolve():
            shutil.copy2(src, dest)
            src.unlink()
        moved += 1
    return moved, missing


def ffmpeg_convert(src: Path, dest: Path) -> None:
    dest.parent.mkdir(parents=True, exist_ok=True)
    cmd = [
        "ffmpeg",
        "-y",
        "-i",
        str(src),
        "-ac",
        "1",
        "-ar",
        "48000",
        "-sample_fmt",
        "s16",
        str(dest),
    ]
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode != 0:
        raise RuntimeError(f"ffmpeg failed for {src.name}: {r.stderr[-400:]}")


def safe_dest(part: str, slug: str) -> Path:
    if not PART_RE.match(part) or not SLUG_RE.match(slug):
        raise ValueError(f"bad part/slug {part}/{slug}")
    dest = (REPO / "samples" / part / f"{slug}.wav").resolve()
    samples_root = (REPO / "samples").resolve()
    if not dest.is_relative_to(samples_root):
        raise ValueError(f"refusing to write outside samples/: {dest}")
    return dest


def md_cell(s: str) -> str:
    return s.replace("|", "\\|").replace("\n", " ").strip()


def write_index(rows: list[dict], path: Path) -> None:
    parts: dict[str, list[dict]] = {}
    for row in rows:
        parts.setdefault(row["part"], []).append(row)
    lines = [
        "# rust-fm-synthe PCM index",
        "",
        "strudel-rs の呼び出しは `s(\"<part>:<slug>\")`。"
        " 音程楽器は `note(...).scale(\"C4:…\").s(\"<part>:<slug>\")`。"
        " `in_bank` が no の行はファイルが `samples/` に無い（長尺は既定オフ、または skip）。",
        "",
        "生成: `python scripts/import_fm_pcm.py`",
        "",
    ]
    for part in sorted(parts):
        lines.append(f"## `{part}`")
        lines.append("")
        lines.append(
            "| call | in_bank | id | name | description | note | dur |"
        )
        lines.append("| --- | --- | --- | --- | --- | --- | --- |")
        for row in parts[part]:
            note = row["default_note"]
            dur = row["default_duration"]
            note_s = "" if note is None else str(note)
            dur_s = "" if dur is None else str(dur)
            alias = f" alias `{row['alias']}`" if row["alias"] else ""
            lines.append(
                "| `{call}` | {bank} | `{id}` | {name} | {desc}{alias} | {note} | {dur} |".format(
                    call=row["call"],
                    bank=row["in_bank"],
                    id=row["id"],
                    name=md_cell(row["name"]),
                    desc=md_cell(row["description"]),
                    alias=alias,
                    note=note_s,
                    dur=dur_s,
                )
            )
        lines.append("")
    path.write_text("\n".join(lines), encoding="utf-8")


def main() -> int:
    args = parse_args()
    err = validate()
    if err:
        print("\n".join(err), file=sys.stderr)
        return 1

    src = args.src.resolve()
    dist = src / "dist"
    presets = src / "presets"
    if not dist.is_dir() or not presets.is_dir():
        print(f"missing dist/ or presets/ under {src}", file=sys.stderr)
        return 1

    meta = load_preset_meta(presets)
    missing_meta = [i for i, *_ in ENTRIES if i not in meta]
    if missing_meta:
        print("presets missing: " + ", ".join(missing_meta[:20]), file=sys.stderr)
        return 1

    allow_parts = set(DEFAULT_PARTS)
    if args.all:
        allow_parts |= set(LONG_PARTS)

    SKILL_DIR.mkdir(parents=True, exist_ok=True)
    slugs_path = SKILL_DIR / "slugs.toml"
    if not args.dry_run:
        slugs_path.write_text(emit_toml(), encoding="utf-8")

    if not args.skip_wav:
        b_moved, b_miss = relocate_bundled(args.dry_run)
        print(f"bundled_move={b_moved} bundled_missing={b_miss}")

    copied = 0
    skipped_exists = 0
    skipped_long = 0
    skipped_alias = 0
    rows: list[dict] = []

    for pid, part, slug, skip, alias in ENTRIES:
        wav = dist / f"{pid}.wav"
        if not wav.is_file():
            print(f"missing wav {wav}", file=sys.stderr)
            return 1
        if not ID_RE.match(pid):
            print(f"bad id {pid}", file=sys.stderr)
            return 1

        in_bank = "no"
        dest = safe_dest(part, slug)
        do_copy = (not skip) and part in allow_parts
        if skip:
            skipped_alias += 1
        elif part not in allow_parts:
            skipped_long += 1
        elif dest.exists():
            skipped_exists += 1
            in_bank = "yes"
        elif args.skip_wav or args.dry_run:
            in_bank = "no"
        else:
            ffmpeg_convert(wav, dest)
            copied += 1
            in_bank = "yes"
        if dest.exists():
            in_bank = "yes"

        m = meta[pid]
        call = f"{part}:{slug}"
        rows.append(
            {
                "id": pid,
                "part": part,
                "slug": slug,
                "call": call,
                "in_bank": in_bank,
                "name": m["name"],
                "description": m["description"],
                "default_note": m["default_note"],
                "default_duration": m["default_duration"],
                "alias": alias,
            }
        )

    have = {(r["part"], r["slug"]) for r in rows}
    for old, part, slug, _ow, name, desc in BUNDLED_FLATS:
        if (part, slug) in have:
            continue
        dest = safe_dest(part, slug)
        rows.append(
            {
                "id": old,
                "part": part,
                "slug": slug,
                "call": f"{part}:{slug}",
                "in_bank": "yes" if dest.exists() else "no",
                "name": name,
                "description": desc,
                "default_note": None,
                "default_duration": None,
                "alias": "",
            }
        )
        have.add((part, slug))

    index_path = SKILL_DIR / "INDEX.md"
    if not args.dry_run:
        write_index(rows, index_path)

    print(
        f"copied={copied} exists={skipped_exists} "
        f"alias_skip={skipped_alias} long_skip={skipped_long} "
        f"index={index_path.relative_to(REPO)}"
    )
    if args.dry_run:
        print("dry-run: no files written")
    return 0


if __name__ == "__main__":
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    raise SystemExit(main())
