#!/usr/bin/env python3
"""Regression tests for release-state revalidation."""

from __future__ import annotations

import subprocess
import tempfile
from pathlib import Path

from release_state import verify_release_state


def run(root: Path, *command: str) -> None:
    subprocess.run(command, cwd=root, check=True, stdout=subprocess.DEVNULL)


def assert_rejected(root: Path, version: str, head: str, expected: str) -> None:
    try:
        verify_release_state(root, version, head, dry_run=False)
    except (RuntimeError, subprocess.CalledProcessError) as error:
        assert expected in str(error), error
        return
    raise AssertionError("modified release state was accepted")


with tempfile.TemporaryDirectory() as directory:
    root = Path(directory)
    run(root, "git", "init", "-q")
    run(root, "git", "config", "user.email", "release-state@example.invalid")
    run(root, "git", "config", "user.name", "Release State Test")
    (root / "README.md").write_text("release\n", encoding="ascii")
    run(root, "git", "add", "README.md")
    run(root, "git", "commit", "-q", "-m", "release")
    head = subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=root, text=True
    ).strip()

    assert_rejected(root, "0.23.0", "0" * 40, "HEAD changed")
    (root / "DIRTY").write_text("dirty\n", encoding="ascii")
    assert_rejected(root, "0.23.0", head, "working tree changed")
    (root / "DIRTY").unlink()
    (root / "SECOND").write_text("second\n", encoding="ascii")
    run(root, "git", "add", "SECOND")
    run(root, "git", "commit", "-q", "-m", "second")
    approved_head = subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=root, text=True
    ).strip()
    run(root, "git", "tag", "-a", "v0.23.0", "-m", "old", head)
    assert_rejected(root, "0.23.0", approved_head, "approved commit")
    run(root, "git", "tag", "-d", "v0.23.0")
    run(root, "git", "tag", "v0.23.0")
    assert_rejected(root, "0.23.0", approved_head, "annotated tag")

print("4 release-state mutation tests passed.")
