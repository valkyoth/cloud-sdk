#!/usr/bin/env python3
"""Validate the public platform, feature, docs.rs, and native-build contract."""

from __future__ import annotations

import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SUPPORTED_OS_EXPRESSION = (
    'cfg(any(target_os = "freebsd", target_os = "linux", '
    'target_os = "macos", target_os = "windows"))'
)
TRANSPORT_DEPENDENCIES = {
    "aws-lc-rs",
    "aws-lc-sys",
    "base64-ng",
    "bytes",
    "cloud-sdk-sanitization",
    "http",
    "http-body-util",
    "hyper",
    "hyper-rustls",
    "hyper-util",
    "reqwest",
    "rustls",
    "rustls-platform-verifier",
    "tokio",
    "webpki-roots",
}
FEATURES = {
    "cloud-sdk": {"default", "alloc", "std"},
    "cloud-sdk-sanitization": {"default", "alloc", "std"},
    "cloud-sdk-testkit": {"default", "alloc", "std"},
    "cloud-sdk-hetzner": {"default", "alloc", "serde", "std"},
    "cloud-sdk-cratesio": {
        "default",
        "alloc",
        "serde",
        "std",
        "blocking",
        "async",
    },
    "cloud-sdk-reqwest": {
        "default",
        "std",
        "blocking-rustls",
        "blocking-rustls-webpki-roots",
        "async-rustls",
        "fuzzing",
    },
}


def load(path: Path) -> dict:
    with path.open("rb") as source:
        return tomllib.load(source)


def failures(root: Path) -> list[str]:
    problems: list[str] = []
    workspace = load(root / "Cargo.toml")
    if workspace["workspace"]["package"].get("rust-version") != "1.92":
        problems.append("workspace MSRV is not exactly 1.92")
    toolchain = load(root / "rust-toolchain.toml")
    if toolchain.get("toolchain", {}).get("channel") != "1.98.0":
        problems.append("development toolchain is not exactly 1.98.0")

    for crate, expected_features in FEATURES.items():
        manifest = load(root / "crates" / crate / "Cargo.toml")
        if manifest.get("package", {}).get("metadata", {}).get("docs", {}).get("rs", {}).get("all-features") is not True:
            problems.append(f"{crate} does not build all features on docs.rs")
        actual_features = set(manifest.get("features", {}))
        if actual_features != expected_features:
            problems.append(f"{crate} public feature inventory drifted")
        if manifest.get("features", {}).get("default") != []:
            problems.append(f"{crate} default features are not empty")

    reqwest = load(root / "crates" / "cloud-sdk-reqwest" / "Cargo.toml")
    if reqwest["features"].get("std") != ["cloud-sdk-sanitization?/std"]:
        problems.append("reqwest std feature widened the portable core graph")
    dependencies = set(reqwest.get("dependencies", {}))
    leaked = dependencies & TRANSPORT_DEPENDENCIES
    if leaked:
        problems.append(f"transport dependencies are not target-qualified: {sorted(leaked)}")
    target_dependencies = reqwest.get("target", {}).get(SUPPORTED_OS_EXPRESSION, {}).get("dependencies", {})
    if set(target_dependencies) != TRANSPORT_DEPENDENCIES:
        problems.append("target-qualified transport dependency inventory drifted")

    return problems


def main() -> int:
    problems = failures(ROOT)
    if problems:
        for problem in problems:
            print(f"platform contract: {problem}", file=sys.stderr)
        return 1
    print("Platform, feature, docs.rs, and target dependency contracts are exact.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
