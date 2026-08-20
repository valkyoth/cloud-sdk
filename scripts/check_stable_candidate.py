#!/usr/bin/env python3
"""Prove v1.0 changes only stable versions, documentation, and release evidence."""

from __future__ import annotations

from pathlib import Path
import re
import subprocess
import sys
import tomllib


ROOT = Path(__file__).resolve().parent.parent
BASELINE = "v0.100.0"
STABLE_VERSION = "1.0.0"
PUBLIC_PACKAGES = {
    "cloud-sdk",
    "cloud-sdk-hetzner",
    "cloud-sdk-reqwest",
    "cloud-sdk-sanitization",
    "cloud-sdk-testkit",
}
WORKSPACE_IDENTITIES = PUBLIC_PACKAGES | {"ovhcloud-v2-probe"}
PACKAGE_FILES = {"Cargo.toml", "README.md"}
LOCKFILES = (
    "Cargo.lock",
    "fuzz/Cargo.lock",
    "tests/reqwest-feature-unification/Cargo.lock",
    "tools/prepared-coverage-check/Cargo.lock",
)


class StableCandidateError(Exception):
    """Stable-candidate equivalence failure."""


def git(*arguments: str) -> str:
    return subprocess.check_output(
        ["git", *arguments], cwd=ROOT, text=True
    )


def baseline_bytes(path: str) -> bytes:
    return subprocess.check_output(["git", "show", f"{BASELINE}:{path}"], cwd=ROOT)


def normalize_manifest(path: str, data: bytes) -> dict:
    manifest = tomllib.loads(data.decode("utf-8"))
    package = manifest.get("package")
    if isinstance(package, dict) and isinstance(package.get("version"), str):
        package["version"] = "<stable-version>"
    workspace = manifest.get("workspace")
    if isinstance(workspace, dict):
        workspace_package = workspace.get("package")
        if isinstance(workspace_package, dict):
            workspace_package["version"] = "<stable-version>"
        dependencies = workspace.get("dependencies")
        if isinstance(dependencies, dict):
            normalize_dependency_versions(dependencies)
    for table_name in ("dependencies", "dev-dependencies", "build-dependencies"):
        dependencies = manifest.get(table_name)
        if isinstance(dependencies, dict):
            normalize_dependency_versions(dependencies)
    target = manifest.get("target")
    if isinstance(target, dict):
        for target_table in target.values():
            if not isinstance(target_table, dict):
                continue
            for table_name in (
                "dependencies",
                "dev-dependencies",
                "build-dependencies",
            ):
                dependencies = target_table.get(table_name)
                if isinstance(dependencies, dict):
                    normalize_dependency_versions(dependencies)
    return manifest


def normalize_dependency_versions(dependencies: dict) -> None:
    for name, specification in dependencies.items():
        if name not in PUBLIC_PACKAGES or not isinstance(specification, dict):
            continue
        if "version" in specification:
            specification["version"] = "<stable-version>"


def normalize_lock(data: bytes) -> dict:
    lock = tomllib.loads(data.decode("utf-8"))
    for package in lock.get("package", []):
        if package.get("name") in WORKSPACE_IDENTITIES:
            package["version"] = "<stable-version>"
        dependencies = package.get("dependencies")
        if not isinstance(dependencies, list):
            continue
        normalized = []
        for dependency in dependencies:
            fields = dependency.split(" ", maxsplit=2)
            if fields[0] in WORKSPACE_IDENTITIES and len(fields) >= 2:
                fields[1] = "<stable-version>"
            normalized.append(" ".join(fields))
        package["dependencies"] = normalized
    return lock


def changed_package_files() -> None:
    changed = git("diff", "--name-only", BASELINE, "--", "crates").splitlines()
    changed.extend(
        git(
            "ls-files",
            "--others",
            "--exclude-standard",
            "--",
            "crates",
        ).splitlines()
    )
    invalid = []
    for raw_path in changed:
        path = Path(raw_path)
        if len(path.parts) != 3 or path.parts[0] != "crates":
            invalid.append(raw_path)
            continue
        if path.parts[1] not in PUBLIC_PACKAGES or path.name not in PACKAGE_FILES:
            invalid.append(raw_path)
    if invalid:
        raise StableCandidateError(
            f"runtime package files changed after {BASELINE}: {tuple(invalid)}"
        )


def manifests_are_equivalent() -> None:
    paths = ["Cargo.toml", "fuzz/Cargo.toml"]
    paths.extend(f"crates/{name}/Cargo.toml" for name in sorted(PUBLIC_PACKAGES))
    for path in paths:
        baseline = normalize_manifest(path, baseline_bytes(path))
        current = normalize_manifest(path, (ROOT / path).read_bytes())
        if baseline != current:
            raise StableCandidateError(
                f"{path} changed beyond stable first-party version metadata"
            )
    fixture = "tests/reqwest-feature-unification/Cargo.toml"
    pattern = re.compile(
        rb'(path = "../../crates/cloud-sdk(?:-reqwest)?",\n'
        rb'    version = )"[^"]+"'
    )
    replacement = rb'\1"<stable-version>"'
    baseline = pattern.sub(replacement, baseline_bytes(fixture))
    current = pattern.sub(replacement, (ROOT / fixture).read_bytes())
    if baseline != current:
        raise StableCandidateError(
            f"{fixture} changed beyond stable first-party version metadata"
        )


def locks_are_equivalent() -> None:
    for path in LOCKFILES:
        baseline = normalize_lock(baseline_bytes(path))
        current = normalize_lock((ROOT / path).read_bytes())
        if baseline != current:
            raise StableCandidateError(
                f"{path} dependency graph differs from {BASELINE}"
            )


def versions_are_stable() -> None:
    root_manifest = tomllib.loads((ROOT / "Cargo.toml").read_text("ascii"))
    if root_manifest["workspace"]["package"]["version"] != STABLE_VERSION:
        raise StableCandidateError("workspace package version is not v1.0.0")
    dependencies = root_manifest["workspace"]["dependencies"]
    for name in PUBLIC_PACKAGES:
        requirement = dependencies[name].get("version")
        if requirement != STABLE_VERSION:
            raise StableCandidateError(
                f"workspace dependency {name} is not pinned to v1.0.0"
            )
    for name in PUBLIC_PACKAGES - {"cloud-sdk"}:
        manifest = tomllib.loads(
            (ROOT / "crates" / name / "Cargo.toml").read_text("ascii")
        )
        if manifest["package"]["version"] != STABLE_VERSION:
            raise StableCandidateError(f"{name} manifest is not v1.0.0")
    fuzz_path = ROOT / "fuzz" / "Cargo.toml"
    fuzz_manifest = tomllib.loads(fuzz_path.read_text("ascii"))
    for name, specification in fuzz_manifest["dependencies"].items():
        if name in PUBLIC_PACKAGES and specification.get("version") != "=1.0.0":
            raise StableCandidateError(f"fuzz dependency {name} is not =1.0.0")
    fixture_path = ROOT / "tests" / "reqwest-feature-unification" / "Cargo.toml"
    fixture_text = fixture_path.read_text("ascii")
    for name in ("cloud-sdk", "cloud-sdk-reqwest"):
        expected = (
            f'path = "../../crates/{name}",\n'
            f'    version = "={STABLE_VERSION}",'
        )
        if expected not in fixture_text:
            raise StableCandidateError(f"feature fixture {name} is not =1.0.0")
    metadata = tomllib.loads((ROOT / "release-crates.toml").read_text("ascii"))
    if metadata["release"]["version"] != STABLE_VERSION:
        raise StableCandidateError("release plan is not v1.0.0")
    for name in PUBLIC_PACKAGES:
        if metadata["crates"][name]["version"] != STABLE_VERSION:
            raise StableCandidateError(f"{name} is not planned at v1.0.0")


def main() -> int:
    try:
        subprocess.run(
            ["git", "merge-base", "--is-ancestor", BASELINE, "HEAD"],
            cwd=ROOT,
            check=True,
        )
        changed_package_files()
        manifests_are_equivalent()
        locks_are_equivalent()
        versions_are_stable()
    except (OSError, UnicodeError, KeyError, tomllib.TOMLDecodeError) as error:
        print(f"stable candidate: {error}", file=sys.stderr)
        return 1
    except (subprocess.CalledProcessError, StableCandidateError) as error:
        print(f"stable candidate: {error}", file=sys.stderr)
        return 1
    print("Stable 1.0 candidate matches v0.100.0 runtime and dependency behavior.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
