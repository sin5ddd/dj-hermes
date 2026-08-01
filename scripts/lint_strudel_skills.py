#!/usr/bin/env python3
"""Lint exhibit Strudel skills for strudel-rs-only song format.

Scans fenced code blocks in docs/profile/dj-hermes/skills/**/SKILL.md and
fails on patterns that small models copy into strudel_save_song content.

Usage:
  python scripts/lint_strudel_skills.py
  python scripts/lint_strudel_skills.py --root docs/profile/dj-hermes/skills
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
    (re.compile(r"\bsine\.range\s*\("), "sine.range is not supported in strudel-rs"),
    (re.compile(r"\barrange\s*\("), "arrange(...) is not strudel-rs save format"),
]

# Inside method calls like .lpf("<a b>") — dynamic mini args not supported.
DYNAMIC_METHOD_ARG = re.compile(
    r"\.(lpf|hpf|lpq|gain|room|delay|cutoff|vib)\s*\(\s*\"<"
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
    if name.startswith("strudel-"):
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
        default=Path("docs/profile/dj-hermes/skills"),
        help="skills root directory",
    )
    args = ap.parse_args()
    root: Path = args.root
    if not root.is_dir():
        print(f"error: skills root not found: {root}", file=sys.stderr)
        return 2

    files = find_skill_files(root)
    if not files:
        print(f"error: no SKILL.md under {root}", file=sys.stderr)
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
