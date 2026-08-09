#!/usr/bin/env python3
"""Regression tests for the FIPS deferment gate."""

from __future__ import annotations

import importlib.util
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CHECKER_PATH = ROOT / "scripts" / "check_fips_deferred.py"
SPEC = importlib.util.spec_from_file_location("check_fips_deferred", CHECKER_PATH)
assert SPEC is not None and SPEC.loader is not None
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
        assert CHECKER.collect_failures(root) == []


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
        assert any("exposes blocking-rustls-fips" in item for item in failures)
        assert any("depends on aws-lc-fips-sys" in item for item in failures)


def test_lock_source_ci_and_docs_fail() -> None:
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        fixture(root)
        write(root / "Cargo.lock", 'name = "aws-lc-fips-sys"\n')
        write(root / "crates/cloud-sdk-reqwest/src/lib.rs", "pub struct FipsTlsPolicy;\n")
        write(root / ".github/workflows/ci.yml", "jobs:\n  fips-transport:\n")
        write(root / "README.md", "Enable blocking-rustls-fips.\n")
        failures = CHECKER.collect_failures(root)
        assert any("locks aws-lc-fips-sys" in item for item in failures)
        assert any("exposes removed FIPS API" in item for item in failures)
        assert any("retired FIPS transport" in item for item in failures)
        assert any("advertises the retired FIPS API" in item for item in failures)


def main() -> None:
    test_clean_fixture_passes()
    test_manifest_feature_and_dependency_fail()
    test_lock_source_ci_and_docs_fail()
    print("3 FIPS deferment regression groups passed.")


if __name__ == "__main__":
    main()
