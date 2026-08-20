#!/usr/bin/env python3
"""Regression tests for stable-candidate equivalence normalization."""

from __future__ import annotations

from pathlib import Path
import tempfile

import check_stable_candidate as candidate


def manifest(version: str, *, external: str = "1.0.0") -> bytes:
    return f"""
[workspace]

[workspace.package]
version = "{version}"

[workspace.dependencies]
cloud-sdk = {{ version = "{version}", path = "crates/cloud-sdk" }}
external = "{external}"
""".encode("ascii")


def lock(first_party: str, *, external: str = "2.0.0") -> bytes:
    return f"""
version = 4

[[package]]
name = "cloud-sdk"
version = "{first_party}"
dependencies = ["external {external}"]

[[package]]
name = "external"
version = "{external}"
""".encode("ascii")


def test_manifest_versions_are_normalized() -> None:
    before = candidate.normalize_manifest("Cargo.toml", manifest("0.100.0"))
    after = candidate.normalize_manifest("Cargo.toml", manifest("1.0.0"))
    assert before == after


def test_external_manifest_changes_remain_visible() -> None:
    before = candidate.normalize_manifest("Cargo.toml", manifest("0.100.0"))
    after = candidate.normalize_manifest(
        "Cargo.toml", manifest("1.0.0", external="1.1.0")
    )
    assert before != after


def test_lock_versions_are_normalized() -> None:
    assert candidate.normalize_lock(lock("0.100.0")) == candidate.normalize_lock(
        lock("1.0.0")
    )


def test_external_lock_changes_remain_visible() -> None:
    assert candidate.normalize_lock(lock("0.100.0")) != candidate.normalize_lock(
        lock("1.0.0", external="2.1.0")
    )


def test_untracked_runtime_file_is_rejected() -> None:
    original_root = candidate.ROOT
    original_git = candidate.git
    try:
        with tempfile.TemporaryDirectory() as directory:
            candidate.ROOT = Path(directory)

            def fake_git(*arguments: str) -> str:
                if arguments[0] == "diff":
                    return ""
                return "crates/cloud-sdk/src/untracked.rs\n"

            candidate.git = fake_git
            try:
                candidate.changed_package_files()
            except candidate.StableCandidateError:
                pass
            else:
                raise AssertionError("untracked runtime source was accepted")
    finally:
        candidate.ROOT = original_root
        candidate.git = original_git


def main() -> None:
    test_manifest_versions_are_normalized()
    test_external_manifest_changes_remain_visible()
    test_lock_versions_are_normalized()
    test_external_lock_changes_remain_visible()
    test_untracked_runtime_file_is_rejected()
    print("5 stable-candidate equivalence regression groups passed.")


if __name__ == "__main__":
    main()
