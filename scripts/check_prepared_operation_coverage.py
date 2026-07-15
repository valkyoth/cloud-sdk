#!/usr/bin/env python3
"""Require AST-bound evidence for every active prepared Hetzner operation."""

from __future__ import annotations

import argparse
import subprocess
from collections import Counter
from pathlib import Path

from check_api_matrix_coverage import parse_operations

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_MATRIX = ROOT / "docs" / "API_MATRIX.md"
PREPARED = ROOT / "crates" / "cloud-sdk-hetzner" / "src" / "prepared"
DEFAULT_ENDPOINTS = PREPARED / "endpoints"
DEFAULT_BODIES = PREPARED / "bodies"
DEFAULT_BODY_LOCK = ROOT / "docs" / "PREPARED_BODY_OPERATIONS.txt"
CHECKER_MANIFEST = ROOT / "tools" / "prepared-coverage-check" / "Cargo.toml"
EXPECTED_ACTIVE = 208
EXPECTED_BODIES = 91
ENDPOINT_ALIASES = {"get_image": 2, "list_images": 2}


def valid_operation(value: str) -> bool:
    """Return whether a value is one conservative operation identifier."""
    return bool(value) and value[0].islower() and all(
        character.islower() or character.isdigit() or character == "_"
        for character in value
    )


def read_body_lock(path: Path) -> set[str]:
    """Read unique operation IDs from the reviewed request-body lock."""
    operations: set[str] = set()
    for number, raw in enumerate(path.read_text(encoding="ascii").splitlines(), 1):
        value = raw.strip()
        if not value or value.startswith("#"):
            continue
        if not valid_operation(value):
            raise ValueError(f"invalid body operation at line {number}")
        if value in operations:
            raise ValueError(f"duplicate body operation at line {number}")
        operations.add(value)
    if not operations:
        raise ValueError("request-body operation lock is empty")
    return operations


def ast_registries(endpoints: Path, bodies: Path) -> tuple[list[str], list[str]]:
    """Obtain adapter keys from Rust items parsed by the isolated syn checker."""
    prepared_directory = endpoints.parent
    prepared_root = prepared_directory.with_suffix(".rs")
    crate_root = prepared_root.parent / "lib.rs"
    command = [
        "cargo",
        "run",
        "--quiet",
        "--locked",
        "--manifest-path",
        str(CHECKER_MANIFEST),
        "--",
        str(crate_root),
        str(prepared_root),
        str(endpoints.with_suffix(".rs")),
        str(bodies.with_suffix(".rs")),
        str(endpoints),
        str(bodies),
    ]
    result = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        message = result.stderr.strip() or "Rust AST checker failed without diagnostics"
        raise ValueError(message)
    registries: dict[str, list[str]] = {"endpoint": [], "body": []}
    for line in result.stdout.splitlines():
        try:
            kind, operation = line.split("\t", 1)
        except ValueError as error:
            raise ValueError("malformed Rust AST checker output") from error
        if kind not in registries or not valid_operation(operation):
            raise ValueError("invalid Rust AST checker output")
        registries[kind].append(operation)
    return registries["endpoint"], registries["body"]


def validate_registry(
    name: str,
    evidence: list[str],
    admitted: set[str],
    aliases: dict[str, int] | None = None,
) -> set[str]:
    """Require exact, unique adapter evidence except reviewed public aliases."""
    counts = Counter(evidence)
    unknown = sorted(counts.keys() - admitted)
    if unknown:
        raise ValueError(f"unknown {name} adapter keys: " + ", ".join(unknown))
    expected_counts = {
        operation: count
        for operation, count in (aliases or {}).items()
        if operation in admitted
    }
    ambiguous = sorted(
        operation
        for operation, count in counts.items()
        if count != expected_counts.get(operation, 1)
    )
    if ambiguous:
        raise ValueError(f"ambiguous {name} adapter keys: " + ", ".join(ambiguous))
    missing_aliases = sorted(set(expected_counts) - counts.keys())
    if missing_aliases:
        raise ValueError(f"missing reviewed {name} aliases: " + ", ".join(missing_aliases))
    return set(counts)


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
            operation.operation for operation in matrix if operation.deprecated == "no"
        }
        deprecated = {
            operation.operation for operation in matrix if operation.deprecated == "yes"
        }
        if len(active) != args.expected_active:
            raise ValueError("active operation count changed unexpectedly")
        body_lock = read_body_lock(args.body_lock)
        if len(body_lock) != args.expected_bodies:
            raise ValueError("request-body operation count changed unexpectedly")
        if not body_lock <= active:
            raise ValueError("request-body lock contains inactive operations")
        endpoint_registry, body_registry = ast_registries(args.endpoints, args.bodies)
        admitted = active | deprecated
        endpoint_evidence = validate_registry(
            "endpoint", endpoint_registry, admitted, ENDPOINT_ALIASES
        )
        body_evidence = validate_registry("body", body_registry, admitted)
    except (OSError, UnicodeError, ValueError) as error:
        raise SystemExit(f"prepared operation coverage: {error}") from error

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
