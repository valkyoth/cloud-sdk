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
    module.git_success = lambda *arguments: not (
        arguments[:2] == ("cat-file", "-e")
        and arguments[2].endswith(":security/mutation/v0.100.0.json")
        and arguments[2].startswith("a" * 40)
    )
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


def test_git_state_failures(module) -> None:
    source = "a" * 40
    evidence = ("security/mutation/v0.100.0.json",)
    original = module.git_success
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


def main() -> None:
    module = load_checker()
    test_path_allowlist(module)
    test_git_state_failures(module)
    print("Controlled-mutation release-binding regressions passed.")


if __name__ == "__main__":
    main()
