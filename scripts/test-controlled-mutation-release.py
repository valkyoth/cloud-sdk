#!/usr/bin/env python3
"""Regression tests for controlled-mutation release source binding."""

from __future__ import annotations

import importlib.util
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "scripts" / "check-controlled-mutation-release.py"


def load_checker():
    specification = importlib.util.spec_from_file_location("mutation_release", CHECKER)
    assert specification is not None and specification.loader is not None
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def expect_failure(module, message: str, source: str, paths: tuple[str, ...]) -> None:
    try:
        module.validate_binding(source, paths)
    except module.ReleaseEvidenceError as error:
        assert message in str(error), error
        return
    raise AssertionError("invalid release evidence binding was accepted")


def test_path_allowlist(module) -> None:
    original = module.git_success
    original_blob = module.regular_blob_oid
    module.git_success = lambda *arguments: not (
        arguments[:2] == ("cat-file", "-e")
        and arguments[2].endswith(":security/mutation/v0.100.0.json")
        and arguments[2].startswith("a" * 40)
    )
    module.regular_blob_oid = lambda _revision, _path: "b" * 40
    source = "a" * 40
    evidence = "security/mutation/v0.100.0.json"
    try:
        module.validate_binding(source, (evidence,))
        module.validate_binding(
            source,
            (
                evidence,
                "security/pentest/v0.100.0.md",
                "release-notes/RELEASE_NOTES_0.100.0.md",
            ),
        )
        for forbidden in (
            "Cargo.toml",
            "Cargo.lock",
            "crates/cloud-sdk/src/lib.rs",
            "controlled-mutation-policy.json",
            ".github/workflows/ci.yml",
            "docs/CONTROLLED_MUTATION.md",
        ):
            expect_failure(
                module,
                "qualified source changed",
                source,
                (evidence, forbidden),
            )
        expect_failure(
            module,
            "does not follow qualified source",
            source,
            ("security/pentest/v0.100.0.md",),
        )
    finally:
        module.git_success = original
        module.regular_blob_oid = original_blob


def test_git_state_failures(module) -> None:
    source = "a" * 40
    evidence = ("security/mutation/v0.100.0.json",)
    original = module.git_success
    original_blob = module.regular_blob_oid
    module.regular_blob_oid = lambda _revision, _path: "b" * 40
    cases = (
        (
            lambda *arguments: arguments[:2] != ("cat-file", "-e")
            or not arguments[2].endswith("^{commit}"),
            "qualified source commit was not found",
        ),
        (
            lambda *arguments: arguments[:2] != ("merge-base", "--is-ancestor"),
            "not an ancestor",
        ),
        (
            lambda *arguments: True,
            "evidence must follow",
        ),
    )
    try:
        for fake, message in cases:
            module.git_success = fake
            expect_failure(module, message, source, evidence)
    finally:
        module.git_success = original
        module.regular_blob_oid = original_blob


def expect_tree_failure(module, raw: bytes, path: str) -> None:
    try:
        module.parse_tree_entry(raw, path)
    except module.ReleaseEvidenceError:
        return
    raise AssertionError("non-regular committed path was accepted")


def test_committed_tree_modes(module) -> None:
    path = "security/mutation/v0.100.0.json"
    object_id = b"a" * 40
    accepted = b"100644 blob " + object_id + b"\t" + path.encode("ascii") + b"\0"
    assert module.parse_tree_entry(accepted, path) == "a" * 40
    for raw in (
        b"",
        b"120000 blob " + object_id + b"\t" + path.encode("ascii") + b"\0",
        b"100755 blob " + object_id + b"\t" + path.encode("ascii") + b"\0",
        b"040000 tree " + object_id + b"\t" + path.encode("ascii") + b"\0",
        b"100644 blob " + object_id + b"\tother.json\0",
        accepted + accepted,
    ):
        expect_tree_failure(module, raw, path)


def test_required_controls_must_be_regular(module) -> None:
    source = "a" * 40
    evidence = ("security/mutation/v0.100.0.json",)
    original_git = module.git_success
    original_blob = module.regular_blob_oid
    module.git_success = lambda *arguments: not (
        arguments[:2] == ("cat-file", "-e")
        and arguments[2].endswith(":security/mutation/v0.100.0.json")
        and arguments[2].startswith(source)
    )

    def reject_policy(_revision: str, path: str) -> str:
        if path == "controlled-mutation-policy.json":
            raise module.ReleaseEvidenceError("committed path must be a regular file")
        return "b" * 40

    module.regular_blob_oid = reject_policy
    try:
        expect_failure(module, "lacks mutation controls", source, evidence)
    finally:
        module.git_success = original_git
        module.regular_blob_oid = original_blob


def test_repository_blob_reader(module) -> None:
    checker = module.load_checker()
    policy = module.committed_json(
        checker,
        "HEAD",
        "controlled-mutation-policy.json",
        module.MAX_POLICY_BYTES,
    )
    checker.validate_policy(policy)
    try:
        module.committed_json(
            checker,
            "HEAD",
            "controlled-mutation-policy.json",
            1,
        )
    except module.ReleaseEvidenceError:
        pass
    else:
        raise AssertionError("oversized committed blob was accepted")


def main() -> None:
    module = load_checker()
    test_path_allowlist(module)
    test_git_state_failures(module)
    test_committed_tree_modes(module)
    test_required_controls_must_be_regular(module)
    test_repository_blob_reader(module)
    print("Controlled-mutation release-binding regressions passed.")


if __name__ == "__main__":
    main()
