#!/usr/bin/env python3
"""Regression tests for deterministic release provenance comparison."""

from __future__ import annotations

import importlib.util
from pathlib import Path
import subprocess
import sys
import tempfile

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


def test_checkout_directory_name_is_canonicalized_explicitly() -> None:
    first = document("2026-01-01T00:00:00Z", "urn:first", ("a",))
    second = document("2026-01-01T00:00:00Z", "urn:first", ("a",))
    first["name"] = "first"
    second["name"] = "second"
    assert checker.canonical_sbom(first, "cloud-sdk") == checker.canonical_sbom(
        second, "cloud-sdk"
    )


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


def test_policy_names_all_six_publishable_packages() -> None:
    assert checker.packages_from_policy() == (
        "cloud-sdk-sanitization",
        "cloud-sdk",
        "cloud-sdk-reqwest",
        "cloud-sdk-testkit",
        "cloud-sdk-hetzner",
        "cloud-sdk-cratesio",
    )


def test_every_publishable_package_has_explicit_patch_policy() -> None:
    assert set(checker.packages_from_policy()) == set(checker.PACKAGE_PATCHES)


def repository() -> tuple[Path, str]:
    root = Path(tempfile.mkdtemp())
    subprocess.run(["git", "init", "--quiet"], cwd=root, check=True)
    subprocess.run(
        ["git", "config", "user.email", "provenance@example.invalid"],
        cwd=root,
        check=True,
    )
    subprocess.run(
        ["git", "config", "user.name", "Provenance Test"], cwd=root, check=True
    )
    (root / "evidence.txt").write_text("reviewed\n", encoding="ascii")
    (root / "release-governance.toml").write_text(
        '[packages]\npublishable = ["cloud-sdk-sanitization", "cloud-sdk", '
        '"cloud-sdk-reqwest", "cloud-sdk-testkit", "cloud-sdk-hetzner", '
        '"cloud-sdk-cratesio"]\n',
        encoding="ascii",
    )
    subprocess.run(
        ["git", "add", "evidence.txt", "release-governance.toml"],
        cwd=root,
        check=True,
    )
    subprocess.run(["git", "commit", "--quiet", "-m", "fixture"], cwd=root, check=True)
    head = checker.capture(["git", "rev-parse", "HEAD"], root=root)
    return root, head


def test_committed_file_ignores_mutable_worktree() -> None:
    root, head = repository()
    (root / "evidence.txt").write_text("changed\n", encoding="ascii")
    assert checker.committed_file(root, head, "evidence.txt") == b"reviewed\n"


def test_source_tree_is_bound_to_captured_commit() -> None:
    root, head = repository()
    expected = checker.capture(["git", "rev-parse", "HEAD^{tree}"], root=root)
    assert checker.source_tree_at(root, head) == expected


def test_source_change_is_rejected() -> None:
    root, head = repository()
    (root / "evidence.txt").write_text("changed\n", encoding="ascii")
    assert_fails(
        "source worktree changed during reproduction",
        lambda: checker.assert_source_unchanged(root, head),
    )


def test_package_policy_is_read_from_captured_commit() -> None:
    root, head = repository()
    (root / "release-governance.toml").write_text(
        '[packages]\npublishable = ["unreviewed"]\n', encoding="ascii"
    )
    assert checker.packages_from_policy(root, head) == checker.packages_from_policy()


def main() -> None:
    tests = (
        test_volatile_sbom_fields_do_not_change_identity,
        test_dependency_change_changes_sbom_identity,
        test_checkout_directory_name_is_canonicalized_explicitly,
        test_comparison_rejects_missing_artifact,
        test_comparison_rejects_changed_artifact,
        test_policy_names_all_six_publishable_packages,
        test_every_publishable_package_has_explicit_patch_policy,
        test_committed_file_ignores_mutable_worktree,
        test_source_tree_is_bound_to_captured_commit,
        test_source_change_is_rejected,
        test_package_policy_is_read_from_captured_commit,
    )
    for test in tests:
        test()
    print(f"{len(tests)} release provenance regression tests passed.")


if __name__ == "__main__":
    main()
