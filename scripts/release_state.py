#!/usr/bin/env python3
"""Revalidate immutable release state immediately before publication."""

from __future__ import annotations

import subprocess
from pathlib import Path


def capture(root: Path, command: list[str]) -> str:
    """Run a Git query and return its trimmed standard output."""
    return subprocess.check_output(command, cwd=root, text=True).strip()


def verify_release_state(
    root: Path, version: str, expected_head: str, *, dry_run: bool
) -> None:
    """Fail unless the clean, signed release state still matches `expected_head`."""
    if dry_run:
        print(f"[dry-run] would revalidate signed release state {expected_head}")
        return

    current_head = capture(root, ["git", "rev-parse", "HEAD"])
    if current_head != expected_head:
        raise RuntimeError("HEAD changed after release approval")
    if capture(root, ["git", "status", "--porcelain"]):
        raise RuntimeError("working tree changed after release approval")

    tag = f"v{version}"
    if capture(root, ["git", "rev-list", "-n", "1", tag]) != expected_head:
        raise RuntimeError(f"{tag} no longer identifies the approved commit")
    if capture(root, ["git", "cat-file", "-t", tag]) != "tag":
        raise RuntimeError(f"{tag} is no longer an annotated tag")
    subprocess.run(["git", "verify-tag", tag], cwd=root, check=True)
