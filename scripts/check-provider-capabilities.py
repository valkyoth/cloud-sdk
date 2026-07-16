#!/usr/bin/env python3
"""Validate that provider capability claims remain precise and bounded."""

from __future__ import annotations

from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parent.parent
DEFAULT_README = ROOT / "crates" / "cloud-sdk-hetzner" / "README.md"
HEADER = "| Capability | Current coverage | Planned completion |"
EXPECTED = (
    ("Request models", "Complete for all 208 non-deprecated operations", "Current"),
    ("Path/query encoding", "Complete for all 208 non-deprecated operations", "Current"),
    (
        "Body serialization",
        "Complete for all 91 non-deprecated operations with request bodies",
        "Current",
    ),
    (
        "Success response models",
        "Complete checked envelope and resource-identity coverage for all 208 "
        "non-deprecated operations",
        "Provider-complete resource fields before `1.0.0`",
    ),
    (
        "Error response models",
        "Complete checked typed API error decoding for all active operations",
        "Current",
    ),
    (
        "End-to-end client",
        "Not available",
        "`v0.41.0`, after provider-neutral contract hardening",
    ),
)


def parse_table(text: str) -> tuple[tuple[str, str, str], ...]:
    lines = text.splitlines()
    try:
        start = lines.index(HEADER)
    except ValueError as error:
        raise ValueError("capability table header is missing or changed") from error
    if start + 1 >= len(lines) or lines[start + 1] != "| --- | --- | --- |":
        raise ValueError("capability table separator is missing or changed")

    rows: list[tuple[str, str, str]] = []
    for line in lines[start + 2 :]:
        if not line.startswith("|"):
            break
        cells = tuple(cell.strip() for cell in line.strip("|").split("|"))
        if len(cells) != 3:
            raise ValueError("capability table row has the wrong column count")
        rows.append(cells)
    return tuple(rows)


def validate(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    if "| Hetzner API area | Supported |" in text:
        raise ValueError("ambiguous Supported capability column is forbidden")
    rows = parse_table(text)
    if rows != EXPECTED:
        raise ValueError("capability table differs from the reviewed roadmap contract")


def main() -> int:
    path = Path(sys.argv[1]) if len(sys.argv) == 2 else DEFAULT_README
    if len(sys.argv) > 2:
        print("usage: check-provider-capabilities.py [README]", file=sys.stderr)
        return 2
    try:
        validate(path)
    except (OSError, UnicodeError, ValueError) as error:
        print(f"provider capabilities: {error}", file=sys.stderr)
        return 1
    print("Provider capability claims match the reviewed roadmap contract.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
