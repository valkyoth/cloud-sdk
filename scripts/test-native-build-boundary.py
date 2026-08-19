#!/usr/bin/env python3
"""Regression tests for the source-locked native build boundary."""

from __future__ import annotations

import copy
import importlib.util
import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "scripts" / "check_native_build_boundary.py"
SPEC = importlib.util.spec_from_file_location("native_build_boundary", CHECKER)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def metadata() -> dict:
    result = subprocess.run(
        ["cargo", "metadata", "--locked", "--format-version", "1", "--all-features"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(result.stdout)


def active_package(document: dict, name: str) -> dict:
    active = {node["id"] for node in document["resolve"]["nodes"]}
    return next(
        package
        for package in document["packages"]
        if package["id"] in active and package["name"] == name
    )


def activate_copy(document: dict, package: dict, name: str, version: str) -> dict:
    duplicate = copy.deepcopy(package)
    duplicate["name"] = name
    duplicate["version"] = version
    duplicate["id"] = f"registry+fixture#{name}@{version}"
    document["packages"].append(duplicate)
    document["resolve"]["nodes"].append(
        {"id": duplicate["id"], "dependencies": [], "deps": [], "features": []}
    )
    return duplicate


def main() -> None:
    document = metadata()
    assert MODULE.failures(document) == []

    duplicate = copy.deepcopy(document)
    activate_copy(
        duplicate,
        active_package(duplicate, "aws-lc-rs"),
        "aws-lc-rs",
        "99.0.0",
    )
    problems = MODULE.failures(duplicate)
    assert any("build-script inventory drifted" in item for item in problems)
    assert any("exactly one aws-lc-rs" in item for item in problems)

    fips = copy.deepcopy(document)
    activate_copy(
        fips,
        active_package(fips, "aws-lc-sys"),
        "aws-lc-fips-sys",
        "99.0.0",
    )
    assert any("FIPS package entered" in item for item in MODULE.failures(fips))

    unlinked = copy.deepcopy(document)
    active_package(unlinked, "aws-lc-rs")["dependencies"] = []
    assert any("lost aws-lc-sys" in item for item in MODULE.failures(unlinked))

    print("4 native build boundary regression groups passed.")


if __name__ == "__main__":
    main()
