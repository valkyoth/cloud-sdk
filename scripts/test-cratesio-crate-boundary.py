#!/usr/bin/env python3
"""Regression tests for the initial crates.io provider boundary."""

from __future__ import annotations

import importlib.util
import shutil
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/check_cratesio_crate_boundary.py"


def load_checker():
    spec = importlib.util.spec_from_file_location("cratesio_boundary", SCRIPT)
    if spec is None or spec.loader is None:
        raise RuntimeError("could not load crates.io boundary checker")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


checker = load_checker()


def fixture() -> Path:
    root = Path(tempfile.mkdtemp())
    source = ROOT / checker.CRATE
    destination = root / checker.CRATE
    destination.parent.mkdir(parents=True)
    shutil.copytree(source, destination)
    other = root / "crates/cloud-sdk-hetzner"
    other.mkdir()
    (other / "Cargo.toml").write_text(
        '[package]\nname = "cloud-sdk-hetzner"\nversion = "1.1.0"\n',
        encoding="ascii",
    )
    return root


def assert_rejected(root: Path, expected: str) -> None:
    try:
        checker.validate(root)
    except checker.BoundaryError as error:
        assert expected in str(error), error
        return
    raise AssertionError("expected boundary rejection")


def test_repository_boundary() -> None:
    checker.validate(ROOT)


def test_feature_or_dependency_widening_is_rejected() -> None:
    root = fixture()
    manifest = root / checker.CRATE / "Cargo.toml"
    manifest.write_text(
        manifest.read_text(encoding="ascii").replace(
            "default = []", 'default = ["std"]'
        ),
        encoding="ascii",
    )
    assert_rejected(root, "feature inventory")
    shutil.rmtree(root)

    root = fixture()
    manifest = root / checker.CRATE / "Cargo.toml"
    manifest.write_text(
        manifest.read_text(encoding="ascii").replace(
            "serde = { workspace = true, optional = true }",
            "serde = { workspace = true, optional = true }\nreqwest.workspace = true",
        ),
        encoding="ascii",
    )
    assert_rejected(root, "dependency inventory")
    shutil.rmtree(root)


def test_endpoint_code_and_extra_modules_are_rejected() -> None:
    root = fixture()
    catalog = root / checker.CRATE / "src/catalog.rs"
    catalog.write_text("pub const ENDPOINT: &str = \"/api/v1/crates\";\n", encoding="ascii")
    assert_rejected(root, "endpoint implementation")
    shutil.rmtree(root)

    root = fixture()
    (root / checker.CRATE / "src/transport.rs").write_text("//! no\n", encoding="ascii")
    assert_rejected(root, "source-module inventory")
    shutil.rmtree(root)


def test_unrelated_crate_dependency_is_rejected() -> None:
    root = fixture()
    manifest = root / "crates/cloud-sdk-hetzner/Cargo.toml"
    manifest.write_text(
        manifest.read_text(encoding="ascii")
        + '\n[dependencies]\ncloud-sdk-cratesio = "1.1.0"\n',
        encoding="ascii",
    )
    assert_rejected(root, "unrelated crate depends on provider")
    shutil.rmtree(root)


def main() -> None:
    tests = (
        test_repository_boundary,
        test_feature_or_dependency_widening_is_rejected,
        test_endpoint_code_and_extra_modules_are_rejected,
        test_unrelated_crate_dependency_is_rejected,
    )
    for test in tests:
        test()
    print(f"{len(tests)} crates.io crate boundary regression groups passed.")


if __name__ == "__main__":
    main()
