#!/usr/bin/env python3
"""Source-lock the active native cryptographic build boundary."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
EXPECTED = {
    "aws-lc-rs": ("1.18.0", "aws-lc-sys"),
    "aws-lc-sys": ("0.44.0", None),
    "ring": ("0.17.14", None),
}
EXPECTED_BUILD_SCRIPTS = {
    ("aws-lc-rs", "1.18.0"),
    ("aws-lc-sys", "0.44.0"),
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


def failures(metadata: dict) -> list[str]:
    problems: list[str] = []
    active_ids = {node["id"] for node in metadata["resolve"]["nodes"]}
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
            names = {
                dependency["name"] for dependency in matches[0]["dependencies"]
            }
            if required_dependency not in names:
                problems.append(f"{name} lost {required_dependency}")
    if any(package["name"] == "aws-lc-fips-sys" for package in packages):
        problems.append("FIPS package entered the graph")
    return problems


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
    if problems:
        for problem in problems:
            print(f"native build boundary: {problem}", file=sys.stderr)
        return 1
    print("Native crypto boundary is exact: AWS-LC is active; ring is a target-specific edge.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
