#!/usr/bin/env python3
"""Require exact reviewed versions for direct third-party dependencies."""

from __future__ import annotations

from pathlib import Path
import re
import sys
import tomllib


ROOT = Path(__file__).resolve().parent.parent
EXACT_VERSION = re.compile(r"^=[0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?$")


def pin_problems(dependencies: dict) -> list[str]:
    problems = []
    for name, specification in sorted(dependencies.items()):
        if name.startswith("cloud-sdk"):
            continue
        if isinstance(specification, str):
            requirement = specification
        elif isinstance(specification, dict):
            requirement = specification.get("version")
            if requirement is None and "path" in specification:
                continue
        else:
            requirement = None
        if not isinstance(requirement, str) or not EXACT_VERSION.fullmatch(
            requirement
        ):
            problems.append(
                f"{name}: direct third-party requirement must be an exact =X.Y.Z pin"
            )
    return problems


def main() -> int:
    try:
        manifest = tomllib.loads((ROOT / "Cargo.toml").read_text("ascii"))
        dependencies = manifest["workspace"]["dependencies"]
    except (OSError, UnicodeError, KeyError, tomllib.TOMLDecodeError) as error:
        print(f"dependency pins: {error}", file=sys.stderr)
        return 1
    problems = pin_problems(dependencies)
    if problems:
        for problem in problems:
            print(f"dependency pins: {problem}", file=sys.stderr)
        return 1
    print("Every direct third-party workspace dependency has an exact reviewed pin.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
