#!/usr/bin/env python3
"""Validate the fuzz graph's exact admitted AWS-LC package versions."""

from __future__ import annotations

import re
import sys


ANSI_ESCAPE = re.compile(r"\x1b\[[0-?]*[ -/]*[@-~]")
PACKAGE = re.compile(r"^(aws-lc-(?:rs|sys) v\S+?)(?: \(\*\))?$")
EXPECTED = frozenset(("aws-lc-rs v1.17.3", "aws-lc-sys v0.43.0"))


def observed_packages(lines: list[str]) -> frozenset[str]:
    packages: set[str] = set()
    for raw_line in lines:
        line = ANSI_ESCAPE.sub("", raw_line).strip()
        match = PACKAGE.fullmatch(line)
        if match is not None:
            packages.add(match.group(1))
    return frozenset(packages)


def main() -> int:
    observed = observed_packages(sys.stdin.readlines())
    if observed == EXPECTED:
        return 0

    print(
        "fuzz harness: AWS-LC graph differs from the admitted exact versions",
        file=sys.stderr,
    )
    for package in sorted(observed):
        print(package, file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
