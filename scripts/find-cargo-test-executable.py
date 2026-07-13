#!/usr/bin/env python3
"""Select one test executable from Cargo's JSON message stream."""

from __future__ import annotations

import json
from pathlib import Path
import sys


def find_executable(messages: Path, target_name: str) -> Path:
    matches: set[Path] = set()
    with messages.open(encoding="utf-8") as stream:
        for line_number, line in enumerate(stream, start=1):
            try:
                message = json.loads(line)
            except json.JSONDecodeError as error:
                raise SystemExit(
                    f"cargo executable: invalid JSON message at line {line_number}"
                ) from error
            target = message.get("target", {})
            executable = message.get("executable")
            if (
                message.get("reason") == "compiler-artifact"
                and target.get("name") == target_name
                and "test" in target.get("kind", [])
                and isinstance(executable, str)
            ):
                matches.add(Path(executable))
    if len(matches) != 1:
        raise SystemExit(
            f"cargo executable: expected one {target_name!r} test executable, "
            f"found {len(matches)}"
        )
    return next(iter(matches))


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit("usage: find-cargo-test-executable.py MESSAGES TARGET")
    print(find_executable(Path(sys.argv[1]), sys.argv[2]))


if __name__ == "__main__":
    main()
