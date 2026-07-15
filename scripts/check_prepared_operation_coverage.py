#!/usr/bin/env python3
"""Require source evidence for every active prepared Hetzner operation."""

from __future__ import annotations

import argparse
import re
from collections import Counter
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
LINE_COMMENT = re.compile(r"//[^\n]*")
BLOCK_COMMENT = re.compile(r"/\*.*?\*/", re.DOTALL)
TEST_CFG = re.compile(r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]")
ANY_CFG = re.compile(r"#\s*\[\s*cfg\b")
ENDPOINT_ALIASES = {"get_image": 2, "list_images": 2}


def production_source(source: str) -> str:
    """Remove comments and reject conditionally compiled evidence."""
    without_blocks = BLOCK_COMMENT.sub("", source)
    without_comments = LINE_COMMENT.sub("", without_blocks)
    if TEST_CFG.search(without_comments) or ANY_CFG.search(without_comments):
        raise ValueError("conditionally compiled prepared evidence is forbidden")
    return without_comments


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
        payloads.append(production_source(payload.decode("utf-8")))
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


def matching_delimiter(source: str, start: int, opening: str, closing: str) -> int:
    """Find a balanced Rust delimiter while ignoring ordinary string contents."""
    depth = 0
    index = start
    in_string = False
    while index < len(source):
        character = source[index]
        if in_string:
            if character == "\\":
                index += 2
                continue
            if character == '"':
                in_string = False
        elif character == '"':
            in_string = True
        elif character == opening:
            depth += 1
        elif character == closing:
            depth -= 1
            if depth == 0:
                return index
        index += 1
    raise ValueError(f"unclosed {opening} in prepared evidence source")


def split_arguments(arguments: str) -> list[str]:
    """Split one macro invocation at top-level commas."""
    parts: list[str] = []
    start = 0
    depths = {"(": 0, "[": 0, "{": 0}
    pairs = {")": "(", "]": "[", "}": "{"}
    index = 0
    in_string = False
    while index < len(arguments):
        character = arguments[index]
        if in_string:
            if character == "\\":
                index += 2
                continue
            if character == '"':
                in_string = False
        elif character == '"':
            in_string = True
        elif character in depths:
            depths[character] += 1
        elif character in pairs:
            depths[pairs[character]] -= 1
        elif character == "," and all(depth == 0 for depth in depths.values()):
            parts.append(arguments[start:index].strip())
            start = index + 1
        index += 1
    tail = arguments[start:].strip()
    if tail:
        parts.append(tail)
    return parts


def macro_arguments(source: str, name: str) -> list[list[str]]:
    """Return arguments from concrete invocations of one adapter macro."""
    invocations: list[list[str]] = []
    pattern = re.compile(rf"\b{re.escape(name)}!\s*\(")
    for match in pattern.finditer(source):
        opening = source.find("(", match.start())
        closing = matching_delimiter(source, opening, "(", ")")
        invocations.append(split_arguments(source[opening + 1 : closing]))
    return invocations


def implementation_blocks(source: str, trait: str) -> list[str]:
    """Return concrete trait implementation bodies, excluding macro templates."""
    if trait == "EndpointWire":
        pattern = re.compile(r"\bimpl\s+EndpointWire\s+for\b")
    else:
        pattern = re.compile(r"\bimpl\s+(?:crate::prepared::)?BodyWire\s+for\b")
    blocks: list[str] = []
    for match in pattern.finditer(source):
        opening = source.find("{", match.end())
        if opening < 0:
            raise ValueError(f"{trait} implementation has no body")
        closing = matching_delimiter(source, opening, "{", "}")
        blocks.append(source[opening + 1 : closing])
    return blocks


def method_body(source: str, name: str) -> str | None:
    """Return one named method body from a concrete trait implementation."""
    pattern = re.compile(rf"\bfn\s+{re.escape(name)}\s*\(")
    match = pattern.search(source)
    if match is None:
        return None
    parameters = source.find("(", match.start())
    parameters_end = matching_delimiter(source, parameters, "(", ")")
    opening = source.find("{", parameters_end)
    if opening < 0:
        raise ValueError(f"{name} method has no body")
    closing = matching_delimiter(source, opening, "{", "}")
    return source[opening + 1 : closing]


def operation_literals(source: str) -> list[str]:
    """Return syntactically valid operation-key literals from a bounded fragment."""
    return [value for value in STRING.findall(source) if OPERATION.fullmatch(value)]


def endpoint_registry(source: str) -> list[str]:
    """Derive endpoint keys only from adapters that implement EndpointWire."""
    operations: list[str] = []
    for arguments in macro_arguments(source, "endpoint_wire"):
        if len(arguments) != 6:
            raise ValueError("endpoint_wire declaration shape changed")
        keys = operation_literals(arguments[3])
        if not keys:
            raise ValueError("endpoint_wire declaration has no operation keys")
        operations.extend(keys)
    for implementation in implementation_blocks(source, "EndpointWire"):
        body = method_body(implementation, "operation_key")
        if body is None:
            raise ValueError("EndpointWire implementation has no operation_key method")
        operations.extend(operation_literals(body))
    return operations


def body_registry(source: str) -> list[str]:
    """Derive body keys only from adapters that implement BodyWire."""
    operations: list[str] = []
    for name, key_index, expected in (("body_wire", 2, 4), ("body_component", 1, 3)):
        for arguments in macro_arguments(source, name):
            if len(arguments) != expected:
                raise ValueError(f"{name} declaration shape changed")
            keys = operation_literals(arguments[key_index])
            if len(keys) != 1:
                raise ValueError(f"{name} declaration must bind one operation key")
            operations.extend(keys)
    for implementation in implementation_blocks(source, "BodyWire"):
        operation_key = method_body(implementation, "operation_key")
        if operation_key is None:
            raise ValueError("BodyWire implementation has no operation_key method")
        implementation_keys = set(operation_literals(operation_key))
        accepted = method_body(implementation, "accepts_operation")
        if accepted is not None:
            implementation_keys.update(operation_literals(accepted))
        operations.extend(sorted(implementation_keys))
    return operations


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
    duplicates = sorted(
        operation
        for operation, count in counts.items()
        if count != expected_counts.get(operation, 1)
    )
    if duplicates:
        raise ValueError(f"ambiguous {name} adapter keys: " + ", ".join(duplicates))
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
        admitted = active | deprecated
        endpoint_evidence = validate_registry(
            "endpoint", endpoint_registry(endpoint_source), admitted, ENDPOINT_ALIASES
        )
        body_evidence = validate_registry("body", body_registry(body_source), admitted)
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
