#!/usr/bin/env python3
"""Tests for the per-crate release policy helper."""

from __future__ import annotations

import argparse
import contextlib
import importlib.util
import io
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "release_crates.py"


def load_release_crates():
    spec = importlib.util.spec_from_file_location("release_crates", SCRIPT)
    if spec is None or spec.loader is None:
        raise RuntimeError("could not load release_crates.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


release_crates = load_release_crates()


def package(name: str, version: str, deps: tuple[str, ...] = ()) -> dict:
    return {
        "name": name,
        "version": version,
        "dependencies": [{"name": dep} for dep in deps],
    }


def base_plan() -> dict:
    return {
        "version": "0.4.0",
        "crates": {
            name: {
                "previous_version": "0.3.0",
                "version": "0.3.0",
                "change": "unchanged",
                "publish": False,
                "reason": "test",
            }
            for name in release_crates.PUBLISH_ORDER
        },
    }


def base_packages() -> dict[str, dict]:
    packages = {
        name: package(name, "0.3.0") for name in release_crates.PUBLISH_ORDER
    }
    packages["cloud-sdk-hetzner"]["dependencies"] = [{"name": "cloud-sdk"}]
    packages["cloud-sdk-hetzner-reqwest"]["dependencies"] = [
        {"name": "cloud-sdk-hetzner"}
    ]
    packages["cloud-sdk-hetzner-sanitization"]["dependencies"] = [
        {"name": "cloud-sdk-hetzner"}
    ]
    packages["cloud-sdk-hetzner-testkit"]["dependencies"] = [
        {"name": "cloud-sdk-hetzner"}
    ]
    return packages


def assert_fails(expected: str, func, *args) -> None:
    try:
        func(*args)
    except RuntimeError as exc:
        if expected not in str(exc):
            raise AssertionError(f"expected {expected!r} in {exc!r}") from exc
        return
    raise AssertionError("expected failure")


def test_current_plan_accepts_unchanged_crates() -> None:
    release_crates.verify_publish_order(base_packages(), base_plan())


def test_facade_code_changes_must_use_milestone_version() -> None:
    entry = {
        "previous_version": "0.3.0",
        "version": "0.5.0",
        "change": "code",
        "publish": True,
        "reason": "test",
    }
    assert_fails(
        "must always match release version 0.4.0",
        release_crates.validate_plan_entry,
        "cloud-sdk",
        entry,
        "0.4.0",
    )


def test_facade_must_always_match_release_version() -> None:
    entry = {
        "previous_version": "0.3.0",
        "version": "0.3.0",
        "change": "unchanged",
        "publish": False,
        "reason": "test",
    }
    assert_fails(
        "must always match release version 0.4.0",
        release_crates.validate_plan_entry,
        "cloud-sdk",
        entry,
        "0.4.0",
    )


def test_facade_must_publish_for_every_release() -> None:
    entry = {
        "previous_version": "0.3.0",
        "version": "0.4.0",
        "change": "metadata",
        "publish": False,
        "reason": "test",
    }
    assert_fails(
        "must publish for every release",
        release_crates.validate_plan_entry,
        "cloud-sdk",
        entry,
        "0.4.0",
    )


def test_provider_code_changes_use_next_independent_minor() -> None:
    entry = {
        "previous_version": "0.7.0",
        "version": "0.8.0",
        "change": "code",
        "publish": True,
        "reason": "test",
    }
    release_crates.validate_plan_entry("cloud-sdk-hetzner", entry, "0.19.0")


def test_provider_code_changes_reject_release_counter_jump() -> None:
    entry = {
        "previous_version": "0.7.0",
        "version": "0.19.0",
        "change": "code",
        "publish": True,
        "reason": "test",
    }
    assert_fails(
        "independent crate version must be 0.8.0",
        release_crates.validate_plan_entry,
        "cloud-sdk-hetzner",
        entry,
        "0.19.0",
    )


def test_initial_release_accepts_none_previous_version() -> None:
    entry = {
        "previous_version": "none",
        "version": "0.1.0",
        "change": "code",
        "publish": True,
        "reason": "test",
    }
    release_crates.validate_plan_entry("cloud-sdk-hetzner", entry, "0.1.0")


def test_dependency_only_changes_must_patch_bump() -> None:
    entry = {
        "previous_version": "0.3.0",
        "version": "0.4.0",
        "change": "dependency",
        "publish": True,
        "reason": "test",
    }
    assert_fails(
        "dependency-only bumps",
        release_crates.validate_plan_entry,
        "cloud-sdk",
        entry,
        "0.4.0",
    )


def test_unchanged_crates_are_not_published() -> None:
    entry = {
        "previous_version": "0.3.0",
        "version": "0.3.0",
        "change": "unchanged",
        "publish": True,
        "reason": "test",
    }
    assert_fails(
        "unchanged but publish is true",
        release_crates.validate_plan_entry,
        "cloud-sdk-hetzner",
        entry,
        "0.4.0",
    )


def test_metadata_changes_use_milestone_version() -> None:
    entry = {
        "previous_version": "0.3.0",
        "version": "0.4.0",
        "change": "metadata",
        "publish": True,
        "reason": "test",
    }
    release_crates.validate_plan_entry("cloud-sdk", entry, "0.4.0")


def test_publish_plan_skips_unchanged_crates() -> None:
    plan = base_plan()
    plan["crates"]["cloud-sdk-hetzner"] = {
        "previous_version": "0.3.0",
        "version": "0.4.0",
        "change": "code",
        "publish": True,
        "reason": "test",
    }
    assert release_crates.publish_plan(plan) == ("cloud-sdk-hetzner",)


def test_required_release_tag_rejects_mismatched_head() -> None:
    original = release_crates.try_capture
    values = iter(("a" * 40, "b" * 40))
    release_crates.try_capture = lambda _command: next(values)
    try:
        try:
            with contextlib.redirect_stderr(io.StringIO()):
                release_crates.check_release_tag("0.11.0", require_tag=True)
        except SystemExit:
            return
        raise AssertionError("mismatched required tag was accepted")
    finally:
        release_crates.try_capture = original


def test_required_release_tag_rejects_unsigned_tag() -> None:
    original = release_crates.try_capture
    values = iter(("a" * 40, "a" * 40, "commit", None))
    release_crates.try_capture = lambda _command: next(values)
    try:
        try:
            with contextlib.redirect_stderr(io.StringIO()):
                release_crates.check_release_tag("0.11.0", require_tag=True)
        except SystemExit:
            return
        raise AssertionError("unsigned required tag was accepted")
    finally:
        release_crates.try_capture = original


def test_publish_command_has_no_bypass_flags() -> None:
    commands: list[list[str]] = []
    original = release_crates.run
    release_crates.run = lambda command, *, dry_run: commands.append(command)
    try:
        release_crates.publish(
            "cloud-sdk", argparse.Namespace(dry_run=False)
        )
    finally:
        release_crates.run = original
    assert commands == [["cargo", "publish", "-p", "cloud-sdk"]]


def run_tests() -> None:
    tests = (
        test_current_plan_accepts_unchanged_crates,
        test_facade_code_changes_must_use_milestone_version,
        test_facade_must_always_match_release_version,
        test_facade_must_publish_for_every_release,
        test_provider_code_changes_use_next_independent_minor,
        test_provider_code_changes_reject_release_counter_jump,
        test_initial_release_accepts_none_previous_version,
        test_dependency_only_changes_must_patch_bump,
        test_unchanged_crates_are_not_published,
        test_metadata_changes_use_milestone_version,
        test_publish_plan_skips_unchanged_crates,
        test_required_release_tag_rejects_mismatched_head,
        test_required_release_tag_rejects_unsigned_tag,
        test_publish_command_has_no_bypass_flags,
    )
    for test in tests:
        test()


if __name__ == "__main__":
    run_tests()
