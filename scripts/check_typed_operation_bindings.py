#!/usr/bin/env python3
"""Prove complete typed bindings for every active pre-Robot operation."""

from __future__ import annotations

import sys
import subprocess

import check_prepared_operation_coverage as prepared
import check_response_operation_coverage as responses
import generate_operation_associations as associations
import generate_typed_operation_bindings as bindings

EXPECTED_ACTIVE = 208
EXPECTED_DEPRECATED = 13
EXPECTED_REJECTING_BODY_VARIANTS = {
    "detach_server_iso",
    "disable_server_backup",
    "disable_server_rescue",
    "enable_server_backup",
    "poweroff_server",
    "poweron_server",
    "reboot_server",
    "remove_server_from_placement_group",
    "request_server_console",
    "reset_server",
    "reset_server_password",
    "shutdown_server",
}
EVIDENCE_COLUMNS = bindings.COLUMNS


def executable_evidence() -> list[dict[str, str]]:
    """Read policy values emitted from compiled Rust descriptors."""
    result = subprocess.run(
        [
            "cargo",
            "run",
            "--quiet",
            "--locked",
            "-p",
            "cloud-sdk-hetzner",
            "--example",
            "typed_binding_evidence",
        ],
        cwd=bindings.ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise ValueError("compiled Rust binding evidence could not be emitted")
    rows = []
    for number, line in enumerate(result.stdout.splitlines(), 1):
        values = line.split("\t")
        if len(values) != len(EVIDENCE_COLUMNS):
            raise ValueError(f"compiled Rust evidence row {number} is malformed")
        rows.append(dict(zip(EVIDENCE_COLUMNS, values, strict=True)))
    return rows


def fingerprint_sets() -> tuple[set[str], set[str]]:
    """Return exact active and deprecated operation sets from fingerprints."""
    active: set[str] = set()
    deprecated: set[str] = set()
    for row in associations.read_tsv(associations.FINGERPRINTS):
        operation = row["operation_id"]
        destination = active if row["deprecated"] == "no" else deprecated
        if row["deprecated"] not in {"no", "yes"} or operation in active | deprecated:
            raise ValueError("fingerprint operation state is invalid or duplicated")
        destination.add(operation)
    if len(active) != EXPECTED_ACTIVE or len(deprecated) != EXPECTED_DEPRECATED:
        raise ValueError("active or deprecated operation count changed unexpectedly")
    return active, deprecated


def validate() -> None:
    """Cross-check the generated contract and every independent executable registry."""
    active, deprecated = fingerprint_sets()
    expected_rows = bindings.binding_rows()
    actual_rows = bindings.read_manifest()
    if actual_rows != expected_rows:
        raise ValueError("typed operation binding manifest differs from source locks")
    if executable_evidence() != actual_rows:
        raise ValueError("compiled Rust bindings differ from the reviewed manifest")
    manifest_operations = {row["operation_id"] for row in actual_rows}
    if manifest_operations != active or manifest_operations & deprecated:
        raise ValueError("typed binding active/deprecated partition is invalid")

    association_operations = {
        row["operation_id"] for row in associations.read_associations()
    }
    response_operations = responses.response_operations(associations.RESPONSES)
    matrix_operations = responses.active_operations(
        responses.MATRIX.read_text(encoding="utf-8")
    )
    body_operations = associations.read_bodies(associations.BODIES)
    if association_operations != active:
        raise ValueError("compile-time marker source does not exactly cover active operations")
    if response_operations != active or matrix_operations != active:
        raise ValueError("response binding sources do not exactly cover active operations")

    endpoint_registry, body_registry = prepared.ast_registries(
        prepared.DEFAULT_MANIFEST, prepared.DEFAULT_ENDPOINTS, prepared.DEFAULT_BODIES
    )
    admitted = active | deprecated
    endpoint_operations = prepared.validate_registry(
        "endpoint", endpoint_registry, admitted, prepared.ENDPOINT_ALIASES
    )
    ast_body_operations = prepared.validate_registry("body", body_registry, admitted)
    if endpoint_operations != active:
        raise ValueError("prepared endpoints do not exactly cover active operations")
    if not body_operations <= ast_body_operations:
        raise ValueError("prepared bodies do not cover every required body binding")
    rejecting_body_variants = ast_body_operations - body_operations
    if rejecting_body_variants != EXPECTED_REJECTING_BODY_VARIANTS:
        raise ValueError("reviewed fail-closed body variant set changed")
    policies = {row["operation_id"]: row["body_policy"] for row in actual_rows}
    if any(policies[operation] != "forbidden" for operation in rejecting_body_variants):
        raise ValueError("an extra prepared body variant is not fail-closed by policy")
    if deprecated & (endpoint_operations | ast_body_operations):
        raise ValueError("deprecated operation has an executable prepared binding")

    generated_markers = associations.formatted_render()
    if associations.OUTPUT.read_text(encoding="ascii") != generated_markers:
        raise ValueError("generated compile-time operation markers are stale")


def main() -> int:
    try:
        validate()
    except (OSError, UnicodeError, ValueError) as error:
        print(f"typed operation bindings: {error}", file=sys.stderr)
        return 1
    print(
        f"Typed operation bindings: {EXPECTED_ACTIVE} active operations and "
        f"{EXPECTED_DEPRECATED} deprecated exclusions checked."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
