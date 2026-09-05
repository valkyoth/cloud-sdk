#!/usr/bin/env python3
"""Validate the reviewed cloud-sdk-cratesio crate and feature boundary."""

from __future__ import annotations

import csv
import re
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CRATE = Path("crates/cloud-sdk-cratesio")
DOMAIN_MODULES = (
    "accounts",
    "catalog",
    "ownership",
    "publishing",
    "trusted_publishing",
)
EXPECTED_FEATURES = {
    "default": [],
    "alloc": ["cloud-sdk/alloc", "dep:cloud-sdk-sanitization", "cloud-sdk-sanitization/alloc"],
    "serde": ["alloc", "dep:serde"],
    "std": ["alloc", "cloud-sdk/std"],
    "blocking": ["serde", "std"],
    "async": ["serde", "std"],
}
EXPECTED_AUTO_TARGETS = {
    "build": False,
    "autolib": False,
    "autobins": False,
    "autoexamples": False,
    "autotests": False,
    "autobenches": False,
}
EXPECTED_DEPENDENCIES = {
    "cloud-sdk": {"workspace": True},
    "cloud-sdk-sanitization": {"workspace": True, "optional": True},
    "serde": {"workspace": True, "optional": True},
}
EXPECTED_LIBRARY = {"path": "src/lib.rs"}
EXPECTED_TESTS = [{"name": "identity", "path": "tests/identity.rs"}]
ENDPOINT_SOURCES = {
    "endpoint/mod.rs",
    "endpoint/authority.rs",
    "endpoint/redirect.rs",
    "endpoint/redirect_source.rs",
    "endpoint/redirect_source_tests.rs",
    "endpoint/redirect_tests.rs",
    "endpoint/target.rs",
    "endpoint/tests.rs",
}
CREDENTIAL_SOURCES = {
    f"credentials/{name}.rs"
    for name in ("mod", "context", "context_tests", "kind", "material", "policy", "secret", "tests")
}
EXPECTED_WORKSPACE_DEPENDENCIES = {
    "cloud-sdk-sanitization": {
        "path": "crates/cloud-sdk-sanitization",
        "version": "1.1.0",
        "default-features": False,
    },
    "cloud-sdk": {
        "path": "crates/cloud-sdk",
        "version": "1.1.0",
        "default-features": False,
    },
    "serde": {
        "version": "=1.0.229",
        "default-features": False,
        "features": ["alloc", "derive"],
    },
}


class BoundaryError(RuntimeError):
    """The crates.io provider boundary differs from its reviewed topology."""


def load(path: Path) -> dict:
    with path.open("rb") as source:
        return tomllib.load(source)


def dependency_package_names(document: dict) -> set[str]:
    names: set[str] = set()
    for section in ("dependencies", "dev-dependencies", "build-dependencies"):
        table = document.get(section, {})
        if isinstance(table, dict):
            for alias, specification in table.items():
                package = (
                    specification.get("package")
                    if isinstance(specification, dict)
                    else None
                )
                names.add(package if isinstance(package, str) else alias)
    targets = document.get("target", {})
    if isinstance(targets, dict):
        for target in targets.values():
            if not isinstance(target, dict):
                continue
            for section in (
                "dependencies",
                "dev-dependencies",
                "build-dependencies",
            ):
                table = target.get(section, {})
                if isinstance(table, dict):
                    for alias, specification in table.items():
                        package = (
                            specification.get("package")
                            if isinstance(specification, dict)
                            else None
                        )
                        names.add(package if isinstance(package, str) else alias)
    return names


def validate_authentication_inventory(root: Path) -> None:
    """Compare the deliberately literal Rust route inventory, not arbitrary Rust."""
    source = (root / CRATE / "src/credentials/policy.rs").read_text(encoding="ascii")
    table = re.search(r"const API_ROUTES:.*?= &\[(.*?)\];", source, re.DOTALL)
    if table is None:
        raise BoundaryError("credential route inventory is missing")
    entry = r'\(\s*Method::(Get|Put|Post|Patch|Delete),\s*"([^"\n]+)"\s*,?\s*\),'
    entries = re.findall(entry, table[1])
    if re.sub(entry, "", table[1]).strip():
        raise BoundaryError("credential route inventory is not a literal table")
    actual = [(method.upper(), path) for method, path in entries]
    with (root / "docs/CRATESIO_API_SCOPE.tsv").open(encoding="ascii", newline="") as stream:
        rows = list(csv.DictReader(stream, delimiter="\t"))
    expected = [
        (row["method"], row["path"])
        for row in rows
        if "api_token" in row["admitted_auth"].split("|")
    ]
    if sorted(actual) != sorted(expected):
        raise BoundaryError("credential route inventory differs from source lock")
    special = {
        "trustpub_token": {("PUT", "/api/v1/crates/new"), ("DELETE", "/api/v1/trusted_publishing/tokens")},
        "oidc_assertion_body": {("POST", "/api/v1/trusted_publishing/tokens")},
        "email_confirmation_path_token": {("PUT", "/api/v1/confirm/{email_token}")},
        "owner_invitation_path_token": {("PUT", "/api/v1/me/crate_owner_invitations/accept/{token}")},
    }
    for mode, routes in special.items():
        observed = [(row["method"], row["path"]) for row in rows if mode in row["admitted_auth"].split("|")]
        if sorted(observed) != sorted(routes):
            raise BoundaryError(f"credential {mode} contexts need source review")


def validate(root: Path) -> None:
    crate = root / CRATE
    manifest = load(crate / "Cargo.toml")
    package = manifest.get("package", {})
    if package.get("name") != "cloud-sdk-cratesio":
        raise BoundaryError("provider package name changed")
    if package.get("version") != "1.1.0":
        raise BoundaryError("provider candidate version changed")
    for field, expected in EXPECTED_AUTO_TARGETS.items():
        if package.get(field) is not expected:
            raise BoundaryError(f"provider automatic target policy changed: {field}")
    docs = package.get("metadata", {}).get("docs", {}).get("rs", {})
    if docs.get("all-features") is not True:
        raise BoundaryError("docs.rs must expose every provider feature")
    if manifest.get("features") != EXPECTED_FEATURES:
        raise BoundaryError("provider feature inventory changed")
    if manifest.get("dependencies") != EXPECTED_DEPENDENCIES:
        raise BoundaryError("provider dependency specifications changed")
    if manifest.get("lib") != EXPECTED_LIBRARY:
        raise BoundaryError("provider library target changed")
    if manifest.get("test") != EXPECTED_TESTS:
        raise BoundaryError("provider test target inventory changed")
    for target in ("bin", "example", "bench"):
        if manifest.get(target, []) != []:
            raise BoundaryError(f"provider explicit {target} targets changed")
    for section in ("dev-dependencies", "build-dependencies", "target"):
        if manifest.get(section, {}) != {}:
            raise BoundaryError(f"provider {section} changed")

    workspace = load(root / "Cargo.toml").get("workspace", {})
    workspace_dependencies = workspace.get("dependencies", {})
    for name, expected in EXPECTED_WORKSPACE_DEPENDENCIES.items():
        if workspace_dependencies.get(name) != expected:
            raise BoundaryError(f"workspace {name} dependency identity changed")

    expected_sources = {
        "lib.rs",
        "identity.rs",
        *ENDPOINT_SOURCES,
        *CREDENTIAL_SOURCES,
        *(f"{module}.rs" for module in DOMAIN_MODULES),
    }
    actual_sources = {
        str(path.relative_to(crate / "src"))
        for path in (crate / "src").rglob("*.rs")
    }
    if actual_sources != expected_sources:
        raise BoundaryError("provider source-module inventory changed")
    for forbidden in ("build.rs", "build/main.rs"):
        if (crate / forbidden).exists():
            raise BoundaryError(f"forbidden build-script source: {forbidden}")
    library = (crate / "src/lib.rs").read_text(encoding="ascii")
    if "#![no_std]" not in library:
        raise BoundaryError("provider lost its no_std crate boundary")
    if "pub mod endpoint;" not in library:
        raise BoundaryError("provider does not export the endpoint boundary")
    if '#[cfg(feature = "alloc")]\npub mod credentials;' not in library:
        raise BoundaryError("provider credential allocation boundary changed")
    for module in DOMAIN_MODULES:
        if f"pub mod {module};" not in library:
            raise BoundaryError(f"provider does not export {module}")
        text = (crate / f"src/{module}.rs").read_text(encoding="ascii")
        lines = tuple(line for line in text.splitlines() if line)
        if not lines or any(not line.startswith("//!") for line in lines):
            raise BoundaryError(f"{module} contains endpoint implementation")

    identity = (crate / "src/identity.rs").read_text(encoding="ascii")
    for name in ("CRATES_IO_PROVIDER_ID", "REGISTRY_SERVICE_ID"):
        if f"pub const {name}:" not in identity:
            raise BoundaryError(f"provider identity is missing {name}")
    readme = (crate / "README.md").read_text(encoding="utf-8")
    if "main [`cloud-sdk`]" not in readme and "main\n[`cloud-sdk`]" not in readme:
        raise BoundaryError("provider README does not identify the main crate")

    for path in sorted((root / "crates").glob("*/Cargo.toml")):
        if path.parent == crate:
            continue
        if "cloud-sdk-cratesio" in dependency_package_names(load(path)):
            raise BoundaryError(f"unrelated crate depends on provider: {path}")
    validate_authentication_inventory(root)


def main() -> int:
    root = Path(sys.argv[1]) if len(sys.argv) == 2 else ROOT
    if len(sys.argv) > 2:
        print("usage: scripts/check_cratesio_crate_boundary.py [ROOT]", file=sys.stderr)
        return 2
    try:
        validate(root)
    except (OSError, UnicodeError, tomllib.TOMLDecodeError, BoundaryError) as error:
        print(f"crates.io crate boundary: {error}", file=sys.stderr)
        return 1
    print("crates.io crate identity, modules, features, and dependencies are exact.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
