#!/usr/bin/env python3
"""Validate repository-local links without fetching external resources."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from urllib.parse import unquote, urlsplit

MARKDOWN_LINK = re.compile(r"!?\[[^\]]*\]\(([^)]+)\)")
HTML_LINK = re.compile(r"(?:href|src)=[\"']([^\"']+)[\"']")
FENCE = re.compile(r"^\s*(```|~~~)")


def documentation_files(root: Path) -> list[Path]:
    files = [
        root / name
        for name in ("README.md", "CHANGELOG.md", "CONTRIBUTING.md", "SECURITY.md")
    ]
    files.extend((root / "docs").glob("*.md"))
    files.extend((root / "release-notes").glob("*.md"))
    files.extend((root / "security").glob("**/*.md"))
    files.extend((root / "crates").glob("*/README.md"))
    return sorted({path for path in files if path.is_file()})


def destinations(path: Path) -> list[str]:
    found: list[str] = []
    fenced = False
    for line in path.read_text(encoding="utf-8").splitlines():
        if FENCE.match(line):
            fenced = not fenced
            continue
        if fenced:
            continue
        found.extend(match.group(1) for match in MARKDOWN_LINK.finditer(line))
        found.extend(match.group(1) for match in HTML_LINK.finditer(line))
    return found


def normalized_destination(raw: str) -> str:
    value = raw.strip()
    if value.startswith("<") and value.endswith(">"):
        return value[1:-1]
    if " " in value:
        return value.split(" ", 1)[0]
    return value


def local_target(root: Path, source: Path, raw: str) -> Path | None:
    destination = normalized_destination(raw)
    parsed = urlsplit(destination)
    if parsed.scheme or parsed.netloc or not parsed.path:
        return None
    relative = Path(unquote(parsed.path))
    candidate = root / str(relative).lstrip("/") if relative.is_absolute() else source.parent / relative
    target = candidate.resolve()
    try:
        target.relative_to(root)
    except ValueError as error:
        raise ValueError("local link escapes repository root") from error
    return target


def check(root: Path) -> list[str]:
    failures: list[str] = []
    for source in documentation_files(root):
        for destination in destinations(source):
            try:
                target = local_target(root, source, destination)
            except ValueError as error:
                failures.append(f"invalid local link: {source.relative_to(root)} -> {destination}: {error}")
                continue
            if target is not None and not target.exists():
                failures.append(
                    f"missing local link target: {source.relative_to(root)} -> {destination}"
                )
    return failures


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, required=True)
    arguments = parser.parse_args()
    root = arguments.root.resolve()
    failures = check(root)
    if failures:
        print("\n".join(failures), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
