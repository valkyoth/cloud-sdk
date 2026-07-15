#!/usr/bin/env python3
"""Require source evidence for every active prepared Hetzner operation."""

from __future__ import annotations

import argparse
import re
from pathlib import Path

from check_api_matrix_coverage import parse_operations

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_MATRIX = ROOT / "docs" / "API_MATRIX.md"
PREPARED = ROOT / "crates" / "cloud-sdk-hetzner" / "src" / "prepared"
DEFAULT_ENDPOINTS = PREPARED / "endpoints"
DEFAULT_BODIES = PREPARED / "bodies"
DEFAULT_BODY_LOCK = ROOT / "docs" / "PREPARED_BODY_OPERATIONS.txt"
EXPECTED_ACTIVE = 208
EXPECTED_BODIES = 91
MAX_SOURCE_BYTES = 2 * 1024 * 1024
OPERATION = re.compile(r"[a-z][a-z0-9_]+")
STRING = re.compile(r'"([a-z][a-z0-9_]+)"')


def read_sources(path: Path, *, include_root: bool) -> str:
    """Read a bounded, deterministic Rust source set."""
    files = sorted(path.glob("*.rs"))
    if include_root:
        root_file = path.with_suffix(".rs")
        if root_file.is_file():
            files.insert(0, root_file)
    if not files:
        raise ValueError(f"no Rust sources found under {path}")
    payloads: list[str] = []
    total = 0
    for source in files:
        payload = source.read_bytes()
        total += len(payload)
        if total > MAX_SOURCE_BYTES:
            raise ValueError("prepared source evidence exceeds local size limit")
        payloads.append(payload.decode("utf-8"))
    return "\n".join(payloads)


def read_body_lock(path: Path) -> set[str]:
    """Read unique operation IDs from the reviewed request-body lock."""
    operations: set[str] = set()
    for number, raw in enumerate(path.read_text(encoding="ascii").splitlines(), 1):
        value = raw.strip()
        if not value or value.startswith("#"):
            continue
        if OPERATION.fullmatch(value) is None:
            raise ValueError(f"invalid body operation at line {number}")
        if value in operations:
            raise ValueError(f"duplicate body operation at line {number}")
        operations.add(value)
    if not operations:
        raise ValueError("request-body operation lock is empty")
    return operations


def quoted_operations(source: str, admitted: set[str]) -> set[str]:
    """Return admitted operation IDs present as exact Rust string literals."""
    return {value for value in STRING.findall(source) if value in admitted}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--matrix", type=Path, default=DEFAULT_MATRIX)
    parser.add_argument("--endpoints", type=Path, default=DEFAULT_ENDPOINTS)
    parser.add_argument("--bodies", type=Path, default=DEFAULT_BODIES)
    parser.add_argument("--body-lock", type=Path, default=DEFAULT_BODY_LOCK)
    parser.add_argument("--expected-active", type=int, default=EXPECTED_ACTIVE)
    parser.add_argument("--expected-bodies", type=int, default=EXPECTED_BODIES)
    args = parser.parse_args()

    try:
        matrix = parse_operations(args.matrix)
        active = {
            operation.operation
            for operation in matrix
            if operation.deprecated == "no"
        }
        deprecated = {
            operation.operation
            for operation in matrix
            if operation.deprecated == "yes"
        }
        if len(active) != args.expected_active:
            raise ValueError("active operation count changed unexpectedly")
        body_lock = read_body_lock(args.body_lock)
        if len(body_lock) != args.expected_bodies:
            raise ValueError("request-body operation count changed unexpectedly")
        if not body_lock <= active:
            raise ValueError("request-body lock contains inactive operations")
        endpoint_source = read_sources(args.endpoints, include_root=True)
        body_source = read_sources(args.bodies, include_root=False)
    except (OSError, UnicodeError, ValueError) as error:
        raise SystemExit(f"prepared operation coverage: {error}") from error

    endpoint_evidence = quoted_operations(endpoint_source, active | deprecated)
    body_evidence = quoted_operations(body_source, active | deprecated)
    missing_endpoints = sorted(active - endpoint_evidence)
    missing_bodies = sorted(body_lock - body_evidence)
    deferred_evidence = sorted(deprecated & (endpoint_evidence | body_evidence))
    if missing_endpoints:
        raise SystemExit(
            "prepared operation coverage: missing endpoint adapters: "
            + ", ".join(missing_endpoints)
        )
    if missing_bodies:
        raise SystemExit(
            "prepared operation coverage: missing body adapters: "
            + ", ".join(missing_bodies)
        )
    if deferred_evidence:
        raise SystemExit(
            "prepared operation coverage: deprecated adapters are forbidden: "
            + ", ".join(deferred_evidence)
        )
    print(
        "Prepared operation coverage: "
        f"{len(active)} endpoints and {len(body_lock)} request bodies checked."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
