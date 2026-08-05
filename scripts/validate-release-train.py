#!/usr/bin/env python3
"""Validate release-train metadata without loading Cargo package metadata."""

from __future__ import annotations

import sys
from pathlib import Path

sys.dont_write_bytecode = True

import release_train

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - release host guard.
    print("Python 3.11+ is required because this script uses tomllib.", file=sys.stderr)
    raise


ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    with (ROOT / "release-crates.toml").open("rb") as handle:
        plan = release_train.validate_release_context(tomllib.load(handle)["release"])
    release_train.validate_repository_train(plan)
    print(
        f"release train {plan['version']} is valid with stage={plan['stage']} "
        f"baseline=v{plan['baseline']} and "
        f"review-baseline=v{plan['review_baseline']}"
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except RuntimeError as error:
        print(f"release train: {error}", file=sys.stderr)
        raise SystemExit(1) from None
