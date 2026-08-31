#!/usr/bin/env python3
"""Keep release review evidence consolidated into five bounded digests."""

from __future__ import annotations

from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[1]
DOCS = ROOT / "docs"
VERSION = re.compile(r"^## v(\d+\.\d+\.\d+)$", re.MULTILINE)
CATEGORIES = {
    "DEPENDENCY_REVIEW": {
        "0.24.0",
        *(f"0.{minor}.0" for minor in range(32, 96)),
        "1.0.0",
        "1.1.0",
    },
    "MIGRATION": {*(f"0.{minor}.0" for minor in range(27, 96)), "1.0.0"},
    "PUBLIC_API_REVIEW": {
        "0.27.0",
        *(f"0.{minor}.0" for minor in range(32, 96)),
        "1.0.0",
    },
    "REJECTED_ABSTRACTIONS": {
        *(f"0.{minor}.0" for minor in range(62, 96)),
        "1.0.0",
    },
    "THREAT_MODEL_DELTA": {
        *(f"0.{minor}.0" for minor in range(62, 96)),
        "1.0.0",
    },
}


def future_versions(versions: set[str]) -> set[str]:
    future = sorted(
        int(version.split(".")[1])
        for version in versions
        if version.startswith("0.") and int(version.split(".")[1]) >= 96
    )
    if not future:
        return set()
    expected = list(range(96, future[-1] + 1))
    if future != expected:
        raise ValueError(f"review digest future versions are not contiguous: {future}")
    return {f"0.{minor}.0" for minor in future}


def validate_digest(prefix: str, historical: set[str]) -> set[str]:
    path = DOCS / f"{prefix}.md"
    text = path.read_text(encoding="ascii")
    if len(text.splitlines()) > 500:
        raise ValueError(f"{path.name} exceeds 500 lines")
    versions = VERSION.findall(text)
    if len(versions) != len(set(versions)):
        raise ValueError(f"{path.name} contains duplicate version headings")
    actual = set(versions)
    missing = sorted(historical - actual)
    if missing:
        raise ValueError(f"{path.name} lacks historical versions {missing}")
    future = future_versions(actual)
    unexpected = actual - historical - future
    if unexpected:
        raise ValueError(f"{path.name} contains unexpected versions {sorted(unexpected)}")
    for version in historical - {"0.95.0", "1.0.0", "1.1.0"}:
        source = f"/v{version}/docs/{prefix}_{version}.md"
        if source not in text:
            raise ValueError(f"{path.name} lacks signed-tag source for v{version}")
    return future


def main() -> int:
    try:
        future_sets = {
            prefix: validate_digest(prefix, historical)
            for prefix, historical in CATEGORIES.items()
        }
        expected_future = next(iter(future_sets.values()))
        if any(value != expected_future for value in future_sets.values()):
            raise ValueError("review digests do not cover the same future releases")
        stale = sorted(
            path.name
            for prefix in CATEGORIES
            for path in DOCS.glob(f"{prefix}_*.md")
        )
        if stale:
            raise ValueError(f"version-named review documents remain: {stale}")
    except (OSError, UnicodeError, ValueError) as error:
        print(f"review digests: {error}", file=sys.stderr)
        return 1
    print("5 consolidated review digests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
