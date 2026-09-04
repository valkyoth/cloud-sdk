#!/usr/bin/env python3
"""Regression tests for the reviewed crates.io provider boundary."""

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
    (root / "Cargo.toml").write_text(
        "[workspace]\n"
        "[workspace.dependencies]\n"
        "cloud-sdk = { path = \"crates/cloud-sdk\", version = \"1.1.0\", "
        "default-features = false }\n"
        "serde = { version = \"=1.0.229\", default-features = false, "
        "features = [\"alloc\", \"derive\"] }\n",
        encoding="ascii",
    )
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
    assert_rejected(root, "dependency specifications")
    shutil.rmtree(root)


def test_automatic_targets_and_build_scripts_are_rejected() -> None:
    for field in checker.EXPECTED_AUTO_TARGETS:
        root = fixture()
        manifest = root / checker.CRATE / "Cargo.toml"
        manifest.write_text(
            manifest.read_text(encoding="ascii").replace(
                f"{field} = false", f"{field} = true"
            ),
            encoding="ascii",
        )
        assert_rejected(root, f"automatic target policy changed: {field}")
        shutil.rmtree(root)

    for relative in ("build.rs", "build/main.rs"):
        root = fixture()
        build_script = root / checker.CRATE / relative
        build_script.parent.mkdir(parents=True, exist_ok=True)
        build_script.write_text("fn main() {}\n", encoding="ascii")
        assert_rejected(root, f"forbidden build-script source: {relative}")
        shutil.rmtree(root)


def test_explicit_target_substitution_is_rejected() -> None:
    root = fixture()
    manifest = root / checker.CRATE / "Cargo.toml"
    manifest.write_text(
        manifest.read_text(encoding="ascii").replace(
            'path = "src/lib.rs"', 'path = "src/identity.rs"'
        ),
        encoding="ascii",
    )
    assert_rejected(root, "library target changed")
    shutil.rmtree(root)

    root = fixture()
    manifest = root / checker.CRATE / "Cargo.toml"
    manifest.write_text(
        manifest.read_text(encoding="ascii").replace(
            'path = "tests/identity.rs"', 'path = "src/identity.rs"'
        ),
        encoding="ascii",
    )
    assert_rejected(root, "test target inventory changed")
    shutil.rmtree(root)

    for target in ("bin", "example", "bench"):
        root = fixture()
        manifest = root / checker.CRATE / "Cargo.toml"
        manifest.write_text(
            manifest.read_text(encoding="ascii")
            + f'\n[[{target}]]\nname = "unexpected"\npath = "src/identity.rs"\n',
            encoding="ascii",
        )
        assert_rejected(root, f"explicit {target} targets changed")
        shutil.rmtree(root)


def test_dependency_substitution_and_extra_sections_are_rejected() -> None:
    root = fixture()
    manifest = root / checker.CRATE / "Cargo.toml"
    manifest.write_text(
        manifest.read_text(encoding="ascii").replace(
            "cloud-sdk.workspace = true",
            'cloud-sdk = { package = "serde_core", version = "=1.0.229" }',
        ),
        encoding="ascii",
    )
    assert_rejected(root, "dependency specifications")
    shutil.rmtree(root)

    for section in ("dev-dependencies", "build-dependencies"):
        root = fixture()
        manifest = root / checker.CRATE / "Cargo.toml"
        manifest.write_text(
            manifest.read_text(encoding="ascii")
            + f"\n[{section}]\nsubtle = \"2.6.1\"\n",
            encoding="ascii",
        )
        assert_rejected(root, section)
        shutil.rmtree(root)

    root = fixture()
    manifest = root / checker.CRATE / "Cargo.toml"
    manifest.write_text(
        manifest.read_text(encoding="ascii")
        + '\n[target.\'cfg(unix)\'.dependencies]\nsubtle = "2.6.1"\n',
        encoding="ascii",
    )
    assert_rejected(root, "provider target changed")
    shutil.rmtree(root)


def test_workspace_dependency_substitution_is_rejected() -> None:
    for name in checker.EXPECTED_WORKSPACE_DEPENDENCIES:
        root = fixture()
        manifest = root / "Cargo.toml"
        manifest.write_text(
            manifest.read_text(encoding="ascii").replace(
                f"{name} = {{", f'{name} = {{ package = "subtle",'
            ),
            encoding="ascii",
        )
        assert_rejected(root, f"workspace {name} dependency identity")
        shutil.rmtree(root)


def test_endpoint_code_and_extra_modules_are_rejected() -> None:
    root = fixture()
    catalog = root / checker.CRATE / "src/catalog.rs"
    catalog.write_text("pub const ENDPOINT: &str = \"/api/v1/crates\";\n", encoding="ascii")
    assert_rejected(root, "endpoint implementation")
    shutil.rmtree(root)

    root = fixture()
    (root / checker.CRATE / "src/endpoint/redirect.rs").unlink()
    assert_rejected(root, "source-module inventory")
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

    root = fixture()
    manifest = root / "crates/cloud-sdk-hetzner/Cargo.toml"
    manifest.write_text(
        manifest.read_text(encoding="ascii")
        + '\n[dependencies]\nregistry = { package = "cloud-sdk-cratesio", '
        'version = "1.1.0" }\n',
        encoding="ascii",
    )
    assert_rejected(root, "unrelated crate depends on provider")
    shutil.rmtree(root)


def main() -> None:
    tests = (
        test_repository_boundary,
        test_feature_or_dependency_widening_is_rejected,
        test_automatic_targets_and_build_scripts_are_rejected,
        test_explicit_target_substitution_is_rejected,
        test_dependency_substitution_and_extra_sections_are_rejected,
        test_workspace_dependency_substitution_is_rejected,
        test_endpoint_code_and_extra_modules_are_rejected,
        test_unrelated_crate_dependency_is_rejected,
    )
    for test in tests:
        test()
    print(f"{len(tests)} crates.io crate boundary regression groups passed.")


if __name__ == "__main__":
    main()
