#!/usr/bin/env python3
"""Source-lock the active native cryptographic build boundary."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PRODUCTION_FEATURES = (
    "blocking-rustls",
    "blocking-rustls-webpki-roots",
    "async-rustls",
)
PRODUCTION_TARGETS = (
    "x86_64-unknown-linux-gnu",
    "x86_64-pc-windows-msvc",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "x86_64-unknown-freebsd",
)
EXPECTED = {
    "aws-lc-rs": ("1.18.1", "aws-lc-sys"),
    "aws-lc-sys": ("0.45.0", None),
    "ring": ("0.17.14", None),
}
EXPECTED_BUILD_SCRIPTS = {
    ("aws-lc-rs", "1.18.1"),
    ("aws-lc-sys", "0.45.0"),
    ("getrandom", "0.4.3"),
    ("httparse", "1.10.1"),
    ("icu_normalizer_data", "2.3.0"),
    ("icu_properties_data", "2.3.0"),
    ("jni", "0.22.4"),
    ("jni-macros", "0.22.4"),
    ("libc", "0.2.189"),
    ("proc-macro2", "1.0.107"),
    ("quinn", "0.11.11"),
    ("quinn-udp", "0.5.15"),
    ("quote", "1.0.47"),
    ("ring", "0.17.14"),
    ("rustix", "1.1.4"),
    ("rustls", "0.23.43"),
    ("rustversion", "1.0.23"),
    ("serde", "1.0.229"),
    ("serde_core", "1.0.229"),
    ("serde_json", "1.0.151"),
    ("thiserror", "2.0.20"),
    ("wasm-bindgen", "0.2.127"),
    ("wasm-bindgen-shared", "0.2.127"),
    ("windows_aarch64_gnullvm", "0.52.6"),
    ("windows_aarch64_msvc", "0.52.6"),
    ("windows_i686_gnu", "0.52.6"),
    ("windows_i686_gnullvm", "0.52.6"),
    ("windows_i686_msvc", "0.52.6"),
    ("windows_x86_64_gnu", "0.52.6"),
    ("windows_x86_64_gnullvm", "0.52.6"),
    ("windows_x86_64_msvc", "0.52.6"),
    ("zmij", "1.0.23"),
}


def node_map(metadata: dict) -> dict[str, dict]:
    return {node["id"]: node for node in metadata["resolve"]["nodes"]}


def package_map(metadata: dict) -> dict[str, dict]:
    return {package["id"]: package for package in metadata["packages"]}


def optional_dependency_is_active(node: dict, package: dict, alias: str) -> bool:
    active_features = set(node["features"])
    if alias in active_features:
        return True
    prefixes = (f"dep:{alias}", f"{alias}/")
    return any(
        requirement == prefixes[0]
        or requirement.startswith(prefixes[1])
        for feature in active_features
        for requirement in package["features"].get(feature, [])
    )


def normal_dependencies(node: dict, package: dict) -> set[str]:
    declarations = {
        (dependency["rename"] or dependency["name"]).replace("-", "_"): (
            dependency,
            dependency["rename"] or dependency["name"],
        )
        for dependency in package["dependencies"]
        if dependency["kind"] is None
    }
    active: set[str] = set()
    for dependency in node["deps"]:
        if not any(kind["kind"] is None for kind in dependency["dep_kinds"]):
            continue
        matched = declarations.get(dependency["name"].replace("-", "_"))
        if matched is None:
            continue
        declaration, alias = matched
        if not declaration["optional"] or optional_dependency_is_active(
            node, package, alias
        ):
            active.add(dependency["pkg"])
    return active


def reachable_from(
    root: str, nodes: dict[str, dict], packages: dict[str, dict]
) -> set[str]:
    reachable: set[str] = set()
    pending = [root]
    while pending:
        package_id = pending.pop()
        if package_id in reachable:
            continue
        reachable.add(package_id)
        pending.extend(normal_dependencies(nodes[package_id], packages[package_id]))
    return reachable


def production_failures(metadata: dict, feature: str, target: str) -> list[str]:
    problems: list[str] = []
    nodes = node_map(metadata)
    packages = package_map(metadata)
    roots = [
        package_id
        for package_id, package in packages.items()
        if package_id in nodes
        and package["name"] == "cloud-sdk-reqwest"
        and package["source"] is None
    ]
    context = f"{target}/{feature}"
    if len(roots) != 1:
        return [f"{context}: expected exactly one workspace reqwest root"]

    root = roots[0]
    reachable_ids = reachable_from(root, nodes, packages)
    reachable = [packages[package_id] for package_id in reachable_ids]
    active = {(package["name"], package["version"]) for package in reachable}
    required = {("aws-lc-rs", "1.18.1"), ("aws-lc-sys", "0.45.0")}
    if not required <= active:
        problems.append(f"{context}: AWS-LC production graph drifted")
    forbidden = sorted(
        (name, version)
        for name, version in active
        if name in {"ring", "aws-lc-fips-sys"}
    )
    if forbidden:
        problems.append(f"{context}: forbidden crypto backend active: {forbidden}")

    root_features = set(nodes[root]["features"])
    if feature not in root_features or "fuzzing" in root_features:
        problems.append(f"{context}: production feature selection drifted")

    aws_ids = [
        package_id
        for package_id in reachable_ids
        if (packages[package_id]["name"], packages[package_id]["version"])
        == ("aws-lc-rs", "1.18.1")
    ]
    sys_ids = [
        package_id
        for package_id in reachable_ids
        if (packages[package_id]["name"], packages[package_id]["version"])
        == ("aws-lc-sys", "0.45.0")
    ]
    root_dependencies = normal_dependencies(nodes[root], packages[root])
    if len(aws_ids) != 1 or aws_ids[0] not in root_dependencies:
        problems.append(f"{context}: reqwest root lost its direct aws-lc-rs edge")
    if len(aws_ids) == 1 and (
        len(sys_ids) != 1
        or sys_ids[0]
        not in normal_dependencies(nodes[aws_ids[0]], packages[aws_ids[0]])
    ):
        problems.append(f"{context}: aws-lc-rs lost its resolved aws-lc-sys edge")
    return problems


def failures(metadata: dict) -> list[str]:
    problems: list[str] = []
    nodes = node_map(metadata)
    active_ids = set(nodes)
    packages = [
        package for package in metadata["packages"] if package["id"] in active_ids
    ]
    build_scripts = {
        (package["name"], package["version"])
        for package in packages
        if any("custom-build" in target["kind"] for target in package["targets"])
    }
    if build_scripts != EXPECTED_BUILD_SCRIPTS:
        problems.append(
            "build-script inventory drifted: "
            f"expected {sorted(EXPECTED_BUILD_SCRIPTS)}, actual {sorted(build_scripts)}"
        )
    for name, (version, required_dependency) in EXPECTED.items():
        matches = [package for package in packages if package["name"] == name]
        if len(matches) != 1 or matches[0]["version"] != version:
            problems.append(f"expected exactly one {name} {version}")
            continue
        if required_dependency is not None:
            dependency_ids = normal_dependencies(
                nodes[matches[0]["id"]], matches[0]
            )
            names = {package["name"] for package in packages if package["id"] in dependency_ids}
            if required_dependency not in names:
                problems.append(f"{name} lost {required_dependency}")
    if any(package["name"] == "aws-lc-fips-sys" for package in packages):
        problems.append("FIPS package entered the graph")
    return problems


def resolved_graph(feature: str, target: str) -> dict:
    result = subprocess.run(
        [
            "cargo",
            "metadata",
            "--locked",
            "--format-version",
            "1",
            "--no-default-features",
            "--features",
            f"cloud-sdk-reqwest/{feature}",
            "--filter-platform",
            target,
        ],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(result.stdout)


def main() -> int:
    result = subprocess.run(
        ["cargo", "metadata", "--locked", "--format-version", "1", "--all-features"],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        sys.stderr.write(result.stderr)
        return result.returncode
    problems = failures(json.loads(result.stdout))
    try:
        for target in PRODUCTION_TARGETS:
            for feature in PRODUCTION_FEATURES:
                problems.extend(
                    production_failures(resolved_graph(feature, target), feature, target)
                )
    except subprocess.CalledProcessError as error:
        sys.stderr.write(error.stderr)
        return error.returncode
    if problems:
        for problem in problems:
            print(f"native build boundary: {problem}", file=sys.stderr)
        return 1
    print(
        "Native crypto boundary is exact: every production graph uses AWS-LC; "
        "ring remains target-specific outside those graphs."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
