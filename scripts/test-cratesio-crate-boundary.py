#!/usr/bin/env python3
"""Regression tests for the reviewed crates.io provider boundary."""

from __future__ import annotations

import importlib.util
import json
import os
import shutil
import subprocess
import sys
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
    (root / "docs").mkdir()
    shutil.copyfile(ROOT / "docs/CRATESIO_API_SCOPE.tsv", root / "docs/CRATESIO_API_SCOPE.tsv")
    (root / "Cargo.toml").write_text(
        "[workspace]\n"
        "[workspace.dependencies]\n"
        "cloud-sdk = { path = \"crates/cloud-sdk\", version = \"1.1.0\", "
        "default-features = false }\n"
        "cloud-sdk-sanitization = { path = \"crates/cloud-sdk-sanitization\", "
        "version = \"1.1.0\", default-features = false }\n"
        "serde = { version = \"=1.0.229\", default-features = false, "
        "features = [\"alloc\", \"derive\"] }\n"
        "serde_json = { version = \"=1.0.151\", default-features = false, features = [\"alloc\"] }\n"
        "semver = { version = \"=1.0.28\", default-features = false }\n"
        "spdx = { version = \"=0.13.5\", default-features = false }\n"
        "base64-ng = { version = \"=2.0.4\", default-features = false }\n",
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
        text = manifest.read_text(encoding="ascii")
        header = f"[{section}]"
        text = text.replace(header, header + '\nsubtle = "2.6.1"', 1) if header in text else text + f'\n{header}\nsubtle = "2.6.1"\n'
        manifest.write_text(text, encoding="ascii")
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


def test_unguarded_trusted_publishing_and_extra_modules_are_rejected() -> None:
    root = fixture()
    accounts = root / checker.CRATE / "src/trusted_publishing.rs"
    accounts.write_text(accounts.read_text(encoding="ascii").replace(
        '#[cfg(feature = "alloc")]\nmod config;', 'mod config;'), encoding="ascii")
    assert_rejected(root, "trusted publishing allocation guard")
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


def test_credential_inventory_and_feature_regressions_are_rejected() -> None:
    for old, new in (
        ('"/api/v1/crates/new"', '"/api/v1/crates/other"'),
        ('(Method::Get, "/api/v1/crates"),', ''),
        ('(Method::Get, "/api/v1/crates"),', '(Method::Get, "/api/v1/crates"),\n(Method::Get, "/api/v1/crates"),'),
        ('(Method::Get, "/api/v1/crates"),', 'other_routes(),'),
    ):
        root = fixture()
        path = root / checker.CRATE / "src/credentials/policy.rs"
        text = path.read_text(encoding="ascii")
        assert old in text
        path.write_text(text.replace(old, new), encoding="ascii")
        assert_rejected(root, "credential route inventory")
        shutil.rmtree(root)
    root = fixture()
    path = root / "docs/CRATESIO_API_SCOPE.tsv"
    path.write_text(path.read_text(encoding="ascii").replace("trustpub_token", "anonymous"), encoding="ascii")
    assert_rejected(root, "contexts need source review")
    shutil.rmtree(root)
    root = fixture()
    path = root / checker.CRATE / "src/lib.rs"
    path.write_text(path.read_text(encoding="ascii").replace('#[cfg(feature = "alloc")]\npub mod credentials;', 'pub mod credentials;'), encoding="ascii")
    assert_rejected(root, "credential allocation boundary")
    shutil.rmtree(root)


def test_packaged_candidate_uses_both_local_dependency_patches() -> None:
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        scripts = root / "scripts"
        scripts.mkdir()
        shutil.copyfile(ROOT / "scripts/enforce_bundled_aws_lc.sh", scripts / "enforce_bundled_aws_lc.sh")
        reqwest = scripts / "check_packaged_reqwest_tests.sh"
        reqwest.write_text("#!/bin/sh\nexit 0\n", encoding="ascii")
        reqwest.chmod(0o700)
        binary = root / "bin"
        binary.mkdir()
        log = root / "cargo.jsonl"
        cargo = binary / "cargo"
        cargo.write_text(
            f"#!{sys.executable}\n"
            "import json, sys\n"
            f"with open({str(log)!r}, 'a', encoding='ascii') as output:\n"
            "    output.write(json.dumps(sys.argv[1:]) + '\\n')\n",
            encoding="ascii",
        )
        cargo.chmod(0o700)
        environment = dict(os.environ, PATH=str(binary) + os.pathsep + os.environ["PATH"])
        result = subprocess.run(
            ["sh", str(ROOT / "scripts/check_packaged_feature_graphs.sh")],
            cwd=root, env=environment, text=True, capture_output=True, check=False,
        )
        assert result.returncode == 0, result
        calls = [json.loads(line) for line in log.read_text(encoding="ascii").splitlines()]
        provider_calls = [call for call in calls if "cloud-sdk-cratesio" in call]
        assert provider_calls == [[
            "package", "--locked", "-p", "cloud-sdk-cratesio", "--allow-dirty", "--all-features",
            "--config", 'patch.crates-io.cloud-sdk.path="crates/cloud-sdk"',
            "--config", 'patch.crates-io.cloud-sdk-reqwest.path="crates/cloud-sdk-reqwest"',
            "--config", 'patch.crates-io.cloud-sdk-sanitization.path="crates/cloud-sdk-sanitization"',
        ]], provider_calls


def test_wire_scheduling_cannot_lose_its_std_guard() -> None:
    root = fixture()
    try:
        module = root / checker.CRATE / "src/wire/mod.rs"
        module.write_text(module.read_text(encoding="ascii").replace(
            '#[cfg(feature = "std")]\nmod shared_rate;', 'mod shared_rate;'
        ), encoding="ascii")
        assert_rejected(root, "scheduling std guard")
    finally:
        shutil.rmtree(root)


def main() -> None:
    tests = (
        test_repository_boundary,
        test_feature_or_dependency_widening_is_rejected,
        test_automatic_targets_and_build_scripts_are_rejected,
        test_explicit_target_substitution_is_rejected,
        test_dependency_substitution_and_extra_sections_are_rejected,
        test_workspace_dependency_substitution_is_rejected,
        test_unguarded_trusted_publishing_and_extra_modules_are_rejected,
        test_unrelated_crate_dependency_is_rejected,
        test_credential_inventory_and_feature_regressions_are_rejected,
        test_packaged_candidate_uses_both_local_dependency_patches,
        test_wire_scheduling_cannot_lose_its_std_guard,
        test_discovery_feature_guards_cannot_be_removed,
        test_catalog_feature_guards_cannot_be_removed,
        test_settings_feature_guards_cannot_be_removed,
    )
    for test in tests:
        test()
    print(f"{len(tests)} crates.io crate boundary regression groups passed.")


def test_discovery_feature_guards_cannot_be_removed() -> None:
    root = fixture()
    try:
        module = root / checker.CRATE / "src/discovery/mod.rs"
        original = module.read_text(encoding="ascii")
        guards = [('#[cfg(any(feature = "blocking", feature = "async"))]\nmod client;',
                   "mod client;", "client execution guard")]
        guards.extend((f'#[cfg(feature = "alloc")]\n{("" if name == "decode" else "pub(crate) ")}mod {name};',
                       f'{("" if name == "decode" else "pub(crate) ")}mod {name};', "allocation guard")
                      for name in ("crate_model", "decode", "models", "value"))
        for guarded, unguarded, message in guards:
            assert guarded in original
            module.write_text(original.replace(guarded, unguarded), encoding="ascii")
            assert_rejected(root, message)
        module.write_text(original, encoding="ascii")
        checker.validate(root)
    finally:
        shutil.rmtree(root)


def test_catalog_feature_guards_cannot_be_removed() -> None:
    root = fixture()
    try:
        module = root / checker.CRATE / "src/catalog/mod.rs"
        original = module.read_text(encoding="ascii")
        guards = [('#[cfg(any(feature = "blocking", feature = "async"))]\nmod client;',
                   "mod client;", "catalog client guard")]
        guards.extend((f'#[cfg(feature = "alloc")]\n{("pub(crate) " if name == "schema" else "")}mod {name};',
                       f'{("pub(crate) " if name == "schema" else "")}mod {name};', "catalog allocation guard")
                      for name in ("models", "decode", "pagination", "schema", "schema_table"))
        for guarded, unguarded, message in guards:
            assert guarded in original
            module.write_text(original.replace(guarded, unguarded), encoding="ascii")
            assert_rejected(root, message)
        module.write_text(original, encoding="ascii")
        checker.validate(root)
    finally:
        shutil.rmtree(root)


def test_settings_feature_guards_cannot_be_removed() -> None:
    for path, guard, replacement, error in (
        ("lib.rs", '#[cfg(feature = "alloc")]\npub mod settings;',
         "pub mod settings;", "settings allocation guard"),
        ("accounts/mod.rs", '#[cfg(feature = "alloc")]\npub mod cargo;',
         "pub mod cargo;", "Cargo owners allocation guard"),
        ("accounts/cargo.rs", '#[cfg(any(feature = "blocking", feature = "async"))]\nmod execution;',
         "mod execution;", "Cargo owners execution guard"),
        ("settings/mod.rs", '#[cfg(feature = "blocking")]\nmod client;',
         "mod client;", "settings client guard"),
        ("ownership.rs", '#[cfg(feature = "alloc")]\nmod changes;',
         "mod changes;", "ownership allocation guard"),
        ("ownership/changes.rs", '#[cfg(feature = "blocking")]\nmod client;',
         "mod client;", "ownership client guard"),
        ("publishing.rs", '#[cfg(feature = "alloc")]\nmod yank;',
         "mod yank;", "yank allocation guard"),
        ("publishing/yank.rs", '#[cfg(feature = "blocking")]\nmod client;',
         "mod client;", "yank client guard"),
        ("publishing.rs", '#[cfg(feature = "alloc")]\nmod publish;',
         "mod publish;", "publish allocation guard"),
        ("publishing/publish.rs", '#[cfg(any(feature = "blocking", feature = "async"))]\nmod client;',
         "mod client;", "publish client guard"),
        ("publishing/publish.rs", '#[cfg(feature = "blocking-rustls")]\nmod bundled;',
         "mod bundled;", "bundled publish transport guard"),
        ("publishing/publish.rs", '#[cfg(feature = "async-rustls")]\nmod bundled_async;',
         "mod bundled_async;", "bundled publish transport guard"),
    ):
        root = fixture()
        try:
            module = root / checker.CRATE / "src" / path
            original = module.read_text(encoding="ascii")
            assert guard in original
            module.write_text(original.replace(guard, replacement), encoding="ascii")
            assert_rejected(root, error)
        finally:
            shutil.rmtree(root)


if __name__ == "__main__":
    main()
