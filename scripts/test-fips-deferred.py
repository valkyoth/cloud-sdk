#!/usr/bin/env python3
"""Regression tests for the FIPS deferment gate."""

from __future__ import annotations

import importlib.util
import os
import subprocess
import sys
import tempfile
from pathlib import Path


if sys.flags.optimize:
    raise SystemExit("security regression tests must not run with Python optimization")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


ROOT = Path(__file__).resolve().parents[1]
CHECKER_PATH = ROOT / "scripts" / "check_fips_deferred.py"
SPEC = importlib.util.spec_from_file_location("check_fips_deferred", CHECKER_PATH)
require(SPEC is not None and SPEC.loader is not None, "cannot load FIPS checker")
CHECKER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECKER)


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def fixture(root: Path) -> None:
    write(root / "Cargo.toml", '[workspace]\nmembers = []\n[workspace.dependencies]\n')
    write(
        root / "crates/cloud-sdk-reqwest/Cargo.toml",
        '[package]\nname = "cloud-sdk-reqwest"\nversion = "0.1.0"\n'
        '[features]\ndefault = []\n[dependencies]\n',
    )
    write(
        root / "tests/reqwest-feature-unification/Cargo.toml",
        '[package]\nname = "fixture"\nversion = "0.1.0"\n'
        '[dependencies.cloud-sdk-reqwest]\npath = "../../crates/cloud-sdk-reqwest"\n'
        'features = ["blocking-rustls"]\n',
    )
    write(root / "Cargo.lock", 'version = 4\n')
    write(
        root / "deny.toml",
        '[bans]\nskip = [{ crate = "syn@2.0.119", reason = "Platform macros '
        'require syn 2 while current Serde requires syn 3" }]\n',
    )
    write(root / "crates/cloud-sdk-reqwest/src/lib.rs", "#![no_std]\n")
    write(root / ".github/workflows/ci.yml", "jobs:\n  fips-deferment:\n")
    write(root / "README.md", "Transport features are opt in.\n")
    write(root / "crates/cloud-sdk/README.md", "Transport features are opt in.\n")
    write(root / "crates/cloud-sdk-reqwest/README.md", "Rustls transports are opt in.\n")
    write(
        root / "docs/FIPS_DEFERMENT.md",
        "FIPS is not part of the cloud-sdk 1.0 scope and is deferred until Brynja "
        "can bind an exact cryptographic module. This makes no FIPS compliance claim.\n",
    )


def test_clean_fixture_passes() -> None:
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        fixture(root)
        require(CHECKER.collect_failures(root) == [], "clean fixture was rejected")


def test_manifest_feature_and_dependency_fail() -> None:
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        fixture(root)
        write(
            root / "crates/cloud-sdk-reqwest/Cargo.toml",
            '[package]\nname = "cloud-sdk-reqwest"\nversion = "0.1.0"\n'
            '[features]\nblocking-rustls-fips = []\n'
            '[dependencies]\naws-lc-fips-sys = "0.1"\n',
        )
        failures = CHECKER.collect_failures(root)
        require(
            any("exposes blocking-rustls-fips" in item for item in failures),
            "retired FIPS feature was accepted",
        )
        require(
            any("depends on aws-lc-fips-sys" in item for item in failures),
            "retired FIPS dependency was accepted",
        )


def test_lock_source_ci_and_docs_fail() -> None:
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        fixture(root)
        write(root / "Cargo.lock", 'name = "aws-lc-fips-sys"\n')
        write(root / "crates/cloud-sdk-reqwest/src/lib.rs", "pub struct FipsTlsPolicy;\n")
        write(root / ".github/workflows/ci.yml", "jobs:\n  fips-transport:\n")
        write(root / "README.md", "Enable blocking-rustls-fips.\n")
        write(
            root / "deny.toml",
            '[bans]\nskip = [{ crate = "shlex@1.3.0", reason = '
            '"aws-lc-fips-sys requires this duplicate" }]\n',
        )
        failures = CHECKER.collect_failures(root)
        require(
            any("locks aws-lc-fips-sys" in item for item in failures),
            "retired FIPS lock entry was accepted",
        )
        require(
            any("exposes removed FIPS API" in item for item in failures),
            "retired FIPS source API was accepted",
        )
        require(
            any("retired FIPS transport" in item for item in failures),
            "retired FIPS CI job was accepted",
        )
        require(
            any("advertises the retired FIPS API" in item for item in failures),
            "retired FIPS documentation was accepted",
        )
        require(
            any("obsolete FIPS-specific ban exception" in item for item in failures),
            "obsolete FIPS deny exception was accepted",
        )


def test_optimized_execution_fails_closed() -> None:
    optimized = subprocess.run(
        [sys.executable, "-O", str(__file__)],
        cwd=ROOT,
        env={**os.environ, "PYTHONOPTIMIZE": ""},
        capture_output=True,
        text=True,
        check=False,
    )
    require(optimized.returncode != 0, "optimized test execution was accepted")
    require(
        "must not run with Python optimization" in optimized.stderr,
        "optimized rejection was not explicit",
    )


def main() -> None:
    test_clean_fixture_passes()
    test_manifest_feature_and_dependency_fail()
    test_lock_source_ci_and_docs_fail()
    test_optimized_execution_fails_closed()
    print("4 FIPS deferment regression groups passed.")


if __name__ == "__main__":
    main()
