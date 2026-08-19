#!/usr/bin/env python3
"""Regression tests for deterministic release provenance comparison."""

from __future__ import annotations

import importlib.util
from pathlib import Path
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "check_release_provenance.py"


def load_checker():
    spec = importlib.util.spec_from_file_location("release_provenance", SCRIPT)
    if spec is None or spec.loader is None:
        raise RuntimeError("could not load release provenance checker")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


checker = load_checker()


def assert_fails(expected: str, operation) -> None:
    try:
        operation()
    except checker.ProvenanceError as error:
        assert expected in str(error), error
        return
    raise AssertionError("expected provenance failure")


def document(created: str, namespace: str, package_order: tuple[str, ...]) -> dict:
    return {
        "SPDXID": "SPDXRef-DOCUMENT",
        "creationInfo": {"created": created, "creators": ["Tool: test"]},
        "documentNamespace": namespace,
        "files": [],
        "name": "test",
        "packages": [
            {"SPDXID": package, "name": package} for package in package_order
        ],
        "relationships": [],
        "spdxVersion": "SPDX-2.3",
    }


def test_volatile_sbom_fields_do_not_change_identity() -> None:
    first = document("2026-01-01T00:00:00Z", "urn:first", ("b", "a"))
    second = document("2026-08-19T00:00:00Z", "urn:second", ("a", "b"))
    assert checker.canonical_sbom(first) == checker.canonical_sbom(second)


def test_dependency_change_changes_sbom_identity() -> None:
    first = document("2026-01-01T00:00:00Z", "urn:first", ("a",))
    second = document("2026-01-01T00:00:00Z", "urn:first", ("b",))
    assert checker.canonical_sbom(first) != checker.canonical_sbom(second)


def test_comparison_rejects_missing_artifact() -> None:
    assert_fails(
        "differs between clean clones",
        lambda: checker.compare("packages", {"a": "1"}, {}),
    )


def test_comparison_rejects_changed_artifact() -> None:
    assert_fails(
        "changed=['a']",
        lambda: checker.compare("packages", {"a": "1"}, {"a": "2"}),
    )


def test_policy_names_all_five_publishable_packages() -> None:
    assert checker.packages_from_policy() == (
        "cloud-sdk-sanitization",
        "cloud-sdk",
        "cloud-sdk-reqwest",
        "cloud-sdk-testkit",
        "cloud-sdk-hetzner",
    )


def test_every_publishable_package_has_explicit_patch_policy() -> None:
    assert set(checker.packages_from_policy()) == set(checker.PACKAGE_PATCHES)


def main() -> None:
    tests = (
        test_volatile_sbom_fields_do_not_change_identity,
        test_dependency_change_changes_sbom_identity,
        test_comparison_rejects_missing_artifact,
        test_comparison_rejects_changed_artifact,
        test_policy_names_all_five_publishable_packages,
        test_every_publishable_package_has_explicit_patch_policy,
    )
    for test in tests:
        test()
    print(f"{len(tests)} release provenance regression tests passed.")


if __name__ == "__main__":
    main()
