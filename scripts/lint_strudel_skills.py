#!/usr/bin/env python3
"""Lint Strudel skills for strudel-rs-only song format.

Scans fenced code blocks in SKILL.md files and fails on patterns that
must not be copied into .strudel / apply-song content.

Usage:
  python scripts/lint_strudel_skills.py
  python scripts/lint_strudel_skills.py --root docs/skills
  python scripts/lint_strudel_skills.py --root docs/skills --root docs/profile/dj-hermes/skills
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

# Forbidden inside ``` code fences (models copy these into save content).
# Note: cat(...) IS valid strudel-rs song format (multi-bar); do not forbid it.
FORBIDDEN = [
    (re.compile(r"\bstack\s*\("), "stack(...) is not strudel-rs file format"),
    (re.compile(r"\)\s*\.cpm\s*\("), ").cpm(...) is not strudel-rs (use setcpm)"),
    (re.compile(r"(?<![a-zA-Z_])\.cpm\s*\("), ".cpm(...) is not strudel-rs (use setcpm)"),
    (re.compile(r"\barrange\s*\("), "arrange(...) is not strudel-rs save format"),
    # Unimplemented / easy-to-copy-wrong in small models (examples must stay playable).
    (re.compile(r"\.lfo\s*\("), ".lfo(...) is not implemented in strudel-rs"),
    # .add / .sub / .ply, dyn mini args, sine.range(x) are implemented — do not forbid.
    (re.compile(r'(?<![a-zA-Z_])cp(?![a-zA-Z_])'), "sample 'cp' is not in the default bank (use sd/oh)"),
]

# Method args that still cannot be mini patterns (vib etc. remain scalar-only for now).
DYNAMIC_METHOD_ARG = re.compile(
    r"\.(lpq|room|delay|vib|attack|decay|sustain|release)\s*\(\s*\"<"
)

REQUIRES_DOLLAR = re.compile(r"\$:")
REQUIRES_SETCPM = re.compile(r"setcpm\s*\(")

FENCE = re.compile(r"^```")


def find_skill_files(root: Path) -> list[Path]:
    return sorted(root.rglob("SKILL.md"))


def iter_code_fence_lines(text: str) -> list[tuple[int, str]]:
    """Return (lineno, line) for lines inside ``` fences (content only)."""
    out: list[tuple[int, str]] = []
    in_fence = False
    for i, line in enumerate(text.splitlines(), 1):
        if FENCE.match(line.strip()):
            in_fence = not in_fence
            continue
        if in_fence:
            out.append((i, line))
    return out


def lint_file(path: Path) -> list[str]:
    text = path.read_text(encoding="utf-8")
    errors: list[str] = []

    for i, line in iter_code_fence_lines(text):
        for rx, msg in FORBIDDEN:
            if rx.search(line):
                errors.append(f"{path}:{i}: {msg}: {line.strip()[:120]}")
        if DYNAMIC_METHOD_ARG.search(line):
            errors.append(
                f"{path}:{i}: dynamic mini-notation method arg not supported: {line.strip()[:120]}"
            )

    name = path.parent.name
    posix = path.as_posix()
    if name.startswith("strudel-") or "/docs/skills/" in f"/{posix}":
        if not REQUIRES_DOLLAR.search(text):
            errors.append(f"{path}: missing `$:` track example (strudel-rs save format)")
        if not REQUIRES_SETCPM.search(text):
            errors.append(f"{path}: missing setcpm(...) example (strudel-rs tempo)")
    return errors


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--root",
        type=Path,
        action="append",
        dest="roots",
        help="skills root directory (repeatable). Default: docs/skills and Hermes exhibit skills.",
    )
    args = ap.parse_args()
    roots: list[Path] = args.roots or [
        Path("docs/skills"),
        Path("docs/profile/dj-hermes/skills"),
    ]

    files: list[Path] = []
    for root in roots:
        if not root.is_dir():
            print(f"error: skills root not found: {root}", file=sys.stderr)
            return 2
        files.extend(find_skill_files(root))
    files = sorted(set(files))
    if not files:
        print(f"error: no SKILL.md under {roots}", file=sys.stderr)
        return 2

    all_errors: list[str] = []
    for f in files:
        all_errors.extend(lint_file(f))

    if all_errors:
        print(f"FAIL: {len(all_errors)} issue(s) in {len(files)} skill file(s)\n")
        for e in all_errors:
            print(e)
        return 1

    print(f"OK: {len(files)} skill file(s), no forbidden patterns in code fences")
    return 0


if __name__ == "__main__":
    sys.exit(main())
