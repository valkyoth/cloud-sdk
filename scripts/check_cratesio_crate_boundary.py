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
EXPECTED_FEATURES = {
    "default": [],
    "alloc": ["cloud-sdk/alloc", "dep:cloud-sdk-sanitization", "cloud-sdk-sanitization/alloc", "dep:semver", "dep:spdx", "dep:base64-ng"],
    "serde": ["alloc", "dep:serde"],
    "std": ["alloc", "cloud-sdk/std"],
    "blocking": ["serde", "std"],
    "async": ["serde", "std"],
    "artifact-sha256": ["dep:sha2"],
    "blocking-rustls": ["blocking", "artifact-sha256", "dep:cloud-sdk-reqwest", "cloud-sdk-reqwest/blocking-rustls"],
    "async-rustls": ["async", "artifact-sha256", "dep:cloud-sdk-reqwest", "cloud-sdk-reqwest/async-rustls"],
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
    "semver": {"workspace": True, "optional": True},
    "spdx": {"workspace": True, "optional": True},
    "base64-ng": {"workspace": True, "optional": True},
    "sha2": {"workspace": True, "optional": True},
    "cloud-sdk-reqwest": {"workspace": True, "optional": True, "default-features": False},
}
EXPECTED_LIBRARY = {"path": "src/lib.rs"}
EXPECTED_TESTS = [
    {"name": "identity", "path": "tests/identity.rs"},
    {"name": "version_metadata", "path": "tests/version_metadata.rs", "required-features": ["alloc"]},
    {"name": "live_read", "path": "tests/live_read.rs", "required-features": ["blocking-rustls"]},
]
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
    "base64-ng": {"version": "=2.0.4", "default-features": False},
    "spdx": {"version": "=0.13.5", "default-features": False},
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
    "serde_json": {"version": "=1.0.151", "default-features": False,
        "features": ["alloc"]},
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
    if manifest.get("dev-dependencies") != {"semver": {"workspace": True}, "serde_json": {"workspace": True}}:
        raise BoundaryError("provider dev-dependencies oracle changed")
    if load(root / "Cargo.toml")["workspace"]["dependencies"].get("semver") != {
        "version": "=1.0.28", "default-features": False
    }:
        raise BoundaryError("provider SemVer oracle pin changed")
    for section in ("build-dependencies", "target"):
        if manifest.get(section, {}) != {}:
            raise BoundaryError(f"provider {section} changed")

    workspace = load(root / "Cargo.toml").get("workspace", {})
    workspace_dependencies = workspace.get("dependencies", {})
    for name, expected in EXPECTED_WORKSPACE_DEPENDENCIES.items():
        if workspace_dependencies.get(name) != expected:
            raise BoundaryError(f"workspace {name} dependency identity changed")

    expected_sources = {
        "lib.rs",
        *(f"client/{name}.rs" for name in ("mod", "blocking", "buffers", "tests")),
        *(f"client/{name}.rs" for name in ("prepared", "asynchronous", "asynchronous/runner", "asynchronous/catalog", "asynchronous/tests", "asynchronous/tests/lifecycle", "asynchronous/tests/operations", "asynchronous/tests/catalog", "asynchronous/tests/secret_paths")),
        "client/asynchronous/tests/coverage.rs",
        "client/asynchronous/tests/coverage_routes.rs",
        "client/asynchronous/tests/cargo.rs",
        "client/asynchronous/tests/publish.rs",
        "client/asynchronous/tests/publish/adapter.rs",
        "client/asynchronous/tests/publish/failures.rs",
        *(f"accounts/{name}.rs" for name in ("cargo", "cargo/execution", "cargo/tests")),
        *(f"{area}/unified.rs" for area in ("accounts/personal", "accounts/tokens", "settings", "ownership/changes", "publishing/yank", "trusted_publishing")),
        *(f"{area}/tests/unified.rs" for area in ("discovery", "catalog", "versions", "downloads", "accounts", "accounts/tokens", "settings", "trusted_publishing")),
        *(f"bundled/{name}.rs" for name in ("mod", "artifacts", "tests")),
        "downloads/sha256.rs",
        "downloads/artifact/tests/storage.rs",
        "downloads/artifact/tests/storage/cases.rs",
        "client/publish.rs",
        "client/publish_async.rs",
        "publishing/publish/bundled.rs",
        "publishing/publish/bundled_async.rs",
        "publishing/publish/stream/asynchronous.rs",
        "publishing/publish/tests/asynchronous.rs",
        "publishing/publish/tests/bundled.rs",
        "identity.rs",
        "ownership.rs",
        "publishing.rs",
        "trusted_publishing.rs",
        *(f"trusted_publishing/{name}.rs" for name in ("config", "request", "oidc", "temporary", "decode", "schema_table", "client", "empty", "tests", "tests/configurations", "tests/assertions", "tests/tokens", "tests/execution", "tests/workflow")),
        "publishing/yank.rs",
        "publishing/publish.rs",
        *(f"publishing/publish/{name}.rs" for name in ("metadata", "validation", "target", "request", "stream", "response", "schema_table", "client", "tests", "tests/metadata", "tests/execution")),
        *(f"publishing/yank/{name}.rs" for name in ("request", "response", "client", "tests", "tests/execution")),
        "ownership/changes.rs",
        *(f"ownership/changes/{name}.rs" for name in ("identity", "request", "preflight", "decode", "client", "tests", "tests/execution")),
        *ENDPOINT_SOURCES,
        *CREDENTIAL_SOURCES,
        *(f"identifiers/{name}.rs" for name in ("mod", "date", "version", "tests")),
        *(f"query/{name}.rs" for name in ("mod", "values", "path", "parameters", "encode", "tests", "contract_tests")),
        *(f"pagination/{name}.rs" for name in ("mod", "link", "compare", "tests", "execution_tests")),
        *(f"discovery/{name}.rs" for name in (
            "mod", "request", "value", "models", "crate_model", "decode",
            "client", "tests", "client/asynchronous", "client/tests",
            "tests/model_contracts", "tests/pagination_contracts",
            "checked",
        )),
        *(f"catalog/{name}.rs" for name in ("mod", "request", "models", "decode", "pagination", "schema", "schema_table", "client", "client/token", "client/tests", "tests", "tests/pagination", "tests/metadata")),
        *(f"versions/{name}.rs" for name in ("mod", "request", "models", "decode", "schema_table", "client", "client/tests", "tests", "tests/pagination")),
        *(f"downloads/{name}.rs" for name in ("mod", "request", "models", "decode", "schema_table", "client", "client/tests", "tests", "artifact", "artifact/asynchronous", "artifact/tests")),
        *(f"accounts/{name}.rs" for name in ("mod", "request", "models", "decode", "schema_table", "client", "client/tests", "tests")),
        *(f"accounts/personal/{name}.rs" for name in ("mod", "request", "permit", "decode", "schema_table", "client", "tests", "tests/execution", "tests/admission")),
        *(f"accounts/tokens/{name}.rs" for name in ("mod", "request", "models", "decode", "schema_table", "client", "tests", "tests/execution")),
        *(f"settings/{name}.rs" for name in ("mod", "request", "decode", "schema_table", "client", "tests", "tests/execution")),
        *(f"wire/{name}.rs" for name in (
            "mod", "error", "rate", "shared_rate", "user_agent", "envelope",
            "response", "policy_tests", "response_tests", "boundary_tests",
        )),
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
    ownership = (crate / "src/ownership.rs").read_text(encoding="ascii")
    publishing = (crate / "src/publishing.rs").read_text(encoding="ascii")
    trusted = (crate / "src/trusted_publishing.rs").read_text(encoding="ascii")
    if "pub mod trusted_publishing;" not in library:
        raise BoundaryError("trusted publishing export missing")
    for module in ("config", "request", "oidc", "temporary", "decode", "schema_table"):
        if f'#[cfg(feature = "alloc")]\nmod {module};' not in trusted:
            raise BoundaryError("trusted publishing allocation guard changed")
    if '#[cfg(feature = "blocking")]\nmod client;' not in trusted:
        raise BoundaryError("trusted publishing client guard changed")
    if '#[cfg(feature = "alloc")]\nmod publish;' not in publishing:
        raise BoundaryError("publish allocation guard changed")
    publish = (crate / "src/publishing/publish.rs").read_text(encoding="ascii")
    if '#[cfg(any(feature = "blocking", feature = "async"))]\nmod client;' not in publish:
        raise BoundaryError("publish client guard changed")
    for feature, module in (("blocking", "bundled"), ("async", "bundled_async")):
        if f'#[cfg(feature = "{feature}")]\nmod {module};' not in publish:
            raise BoundaryError("bundled publish transport guard changed")
    if '#[cfg(feature = "alloc")]\nmod yank;' not in publishing:
        raise BoundaryError("yank allocation guard changed")
    yank = (crate / "src/publishing/yank.rs").read_text(encoding="ascii")
    if '#[cfg(feature = "blocking")]\nmod client;' not in yank:
        raise BoundaryError("yank client guard changed")
    if "pub mod publishing;" not in library:
        raise BoundaryError("publishing export missing")
    if '#[cfg(feature = "alloc")]\nmod changes;' not in ownership:
        raise BoundaryError("ownership allocation guard changed")
    changes = (crate / "src/ownership/changes.rs").read_text(encoding="ascii")
    if '#[cfg(feature = "blocking")]\nmod client;' not in changes:
        raise BoundaryError("ownership client guard changed")
    if '#[cfg(feature = "alloc")]\npub mod settings;' not in library:
        raise BoundaryError("settings allocation guard changed")
    settings = (crate / "src/settings/mod.rs").read_text(encoding="ascii")
    if '#[cfg(feature = "blocking")]\nmod client;' not in settings:
        raise BoundaryError("settings client guard changed")
    wire = (crate / "src/wire/mod.rs").read_text(encoding="ascii")
    discovery = (crate / "src/discovery/mod.rs").read_text(encoding="ascii")
    if '#[cfg(any(feature = "blocking", feature = "async"))]\nmod client;' not in discovery:
        raise BoundaryError("discovery client execution guard changed")
    for module in ("crate_model", "decode", "models", "value"):
        visibility = "" if module == "decode" else "pub(crate) "
        if f'#[cfg(feature = "alloc")]\n{visibility}mod {module};' not in discovery:
            raise BoundaryError("discovery allocation guard changed")
    catalog = (crate / "src/catalog/mod.rs").read_text(encoding="ascii")
    for module in ("models", "decode", "pagination", "schema", "schema_table"):
        visibility = "pub(crate) " if module == "schema" else ""
        if f'#[cfg(feature = "alloc")]\n{visibility}mod {module};' not in catalog:
            raise BoundaryError("catalog allocation guard changed")
    versions = (crate / "src/versions/mod.rs").read_text(encoding="ascii")
    accounts = (crate / "src/accounts/mod.rs").read_text(encoding="ascii")
    if '#[cfg(feature = "alloc")]\npub mod cargo;' not in accounts:
        raise BoundaryError("Cargo owners allocation guard changed")
    cargo_owners = (crate / "src/accounts/cargo.rs").read_text(encoding="ascii")
    if '#[cfg(any(feature = "blocking", feature = "async"))]\nmod execution;' not in cargo_owners:
        raise BoundaryError("Cargo owners execution guard changed")
    if '#[cfg(feature = "alloc")]\npub mod personal;' not in accounts:
        raise BoundaryError("personal allocation guard changed")
    if '#[cfg(feature = "alloc")]\npub mod tokens;' not in accounts:
        raise BoundaryError("token allocation guard changed")
    tokens = (crate / "src/accounts/tokens/mod.rs").read_text(encoding="ascii")
    if '#[cfg(feature = "blocking")]\nmod client;' not in tokens:
        raise BoundaryError("token client guard changed")
    personal = (crate / "src/accounts/personal/mod.rs").read_text(encoding="ascii")
    if '#[cfg(feature = "blocking")]\nmod client;' not in personal:
        raise BoundaryError("personal client guard changed")
    if "pub mod accounts;" not in library or "pub mod ownership;" not in library:
        raise BoundaryError("account/ownership exports missing")
    for module in ("models", "decode", "schema_table"):
        if f'#[cfg(feature = "alloc")]\nmod {module};' not in accounts:
            raise BoundaryError("accounts allocation guard changed")
    if '#[cfg(any(feature = "blocking", feature = "async"))]\nmod client;' not in accounts:
        raise BoundaryError("accounts client guard changed")
    downloads = (crate / "src/downloads/mod.rs").read_text(encoding="ascii")
    for module in ("models", "decode", "schema_table"):
        if f'#[cfg(feature = "alloc")]\nmod {module};' not in downloads:
            raise BoundaryError("downloads allocation guard changed")
    if '#[cfg(any(feature = "blocking", feature = "async"))]\nmod client;' not in downloads:
        raise BoundaryError("downloads client guard changed")
    for module in ("models", "decode", "schema_table"):
        if f'#[cfg(feature = "alloc")]\nmod {module};' not in versions:
            raise BoundaryError("versions allocation guard changed")
    if '#[cfg(any(feature = "blocking", feature = "async"))]\nmod client;' not in versions:
        raise BoundaryError("versions client guard changed")
    if '#[cfg(any(feature = "blocking", feature = "async"))]\nmod client;' not in catalog:
        raise BoundaryError("catalog client guard changed")
    if '#[cfg(feature = "std")]\nmod shared_rate;' not in wire:
        raise BoundaryError("wire scheduling std guard changed")
    if "#![no_std]" not in library:
        raise BoundaryError("provider lost its no_std crate boundary")
    if "pub mod endpoint;" not in library:
        raise BoundaryError("provider does not export the endpoint boundary")
    if '#[cfg(feature = "alloc")]\npub mod credentials;' not in library:
        raise BoundaryError("provider credential allocation boundary changed")
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
