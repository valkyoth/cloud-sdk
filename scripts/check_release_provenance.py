#!/usr/bin/env python3
"""Reproduce release packages and complete SBOMs from two clean clones."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - release host guard.
    print("Python 3.11+ is required because this script uses tomllib.", file=sys.stderr)
    raise

ROOT = Path(__file__).resolve().parents[1]
CONFIG = ROOT / "release-governance.toml"
SBOMS = (
    "cloud-sdk.spdx.json",
    "reqwest-feature-unification.spdx.json",
    "fuzz.spdx.json",
    "prepared-coverage-check.spdx.json",
)
PACKAGE_PATCHES = {
    "cloud-sdk-sanitization": (),
    "cloud-sdk": (
        'patch.crates-io.cloud-sdk-sanitization.path="crates/cloud-sdk-sanitization"',
    ),
    "cloud-sdk-reqwest": (
        'patch.crates-io.cloud-sdk.path="crates/cloud-sdk"',
        'patch.crates-io.cloud-sdk-sanitization.path="crates/cloud-sdk-sanitization"',
    ),
    "cloud-sdk-testkit": (
        'patch.crates-io.cloud-sdk.path="crates/cloud-sdk"',
        'patch.crates-io.cloud-sdk-sanitization.path="crates/cloud-sdk-sanitization"',
    ),
    "cloud-sdk-hetzner": (
        'patch.crates-io.cloud-sdk.path="crates/cloud-sdk"',
        'patch.crates-io.cloud-sdk-reqwest.path="crates/cloud-sdk-reqwest"',
        'patch.crates-io.cloud-sdk-sanitization.path="crates/cloud-sdk-sanitization"',
        'patch.crates-io.cloud-sdk-testkit.path="crates/cloud-sdk-testkit"',
    ),
    "cloud-sdk-cratesio": (
        'patch.crates-io.cloud-sdk.path="crates/cloud-sdk"',
    ),
}


class ProvenanceError(RuntimeError):
    """Release provenance could not be reproduced exactly."""


def capture(command: list[str], *, root: Path) -> str:
    return subprocess.check_output(command, cwd=root, text=True).strip()


def capture_bytes(command: list[str], *, root: Path) -> bytes:
    return subprocess.check_output(command, cwd=root)


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def clean_status(root: Path) -> str:
    return capture(
        ["git", "status", "--porcelain=v1", "--untracked-files=all"], root=root
    )


def source_tree_at(root: Path, head: str) -> str:
    return capture(["git", "rev-parse", f"{head}^{{tree}}"], root=root)


def committed_file(root: Path, head: str, relative: str) -> bytes:
    return capture_bytes(["git", "show", f"{head}:{relative}"], root=root)


def assert_source_unchanged(root: Path, expected_head: str) -> None:
    if capture(["git", "rev-parse", "HEAD"], root=root) != expected_head:
        raise ProvenanceError("source HEAD changed during reproduction")
    if clean_status(root):
        raise ProvenanceError("source worktree changed during reproduction")


def clone_exact(source: Path, destination: Path, head: str) -> None:
    subprocess.run(
        ["git", "clone", "--quiet", "--no-local", str(source), str(destination)],
        check=True,
    )
    subprocess.run(
        ["git", "checkout", "--quiet", "--detach", head],
        cwd=destination,
        check=True,
    )
    if capture(["git", "rev-parse", "HEAD"], root=destination) != head:
        raise ProvenanceError("clean clone does not identify the requested commit")
    if clean_status(destination):
        raise ProvenanceError("new release clone is not clean")


def package_archives(root: Path, packages: tuple[str, ...]) -> dict[str, str]:
    target = root / ".release-provenance-target"
    environment = os.environ.copy()
    environment["CARGO_TARGET_DIR"] = str(target)
    for package in packages:
        command = [
            "cargo",
            "package",
            "--locked",
            "--no-verify",
            "-p",
            package,
        ]
        for patch in PACKAGE_PATCHES.get(package, ()):
            command.extend(("--config", patch))
        subprocess.run(
            command,
            cwd=root,
            env=environment,
            check=True,
        )
    archives = sorted((target / "package").glob("*.crate"))
    if len(archives) != len(packages):
        raise ProvenanceError(
            f"expected {len(packages)} package archives, found {len(archives)}"
        )
    hashes = {archive.name: sha256(archive) for archive in archives}
    if len(hashes) != len(packages):
        raise ProvenanceError("package archive names are not unique")
    return hashes


def canonical_sbom(
    document: dict[str, Any], canonical_name: str | None = None
) -> bytes:
    normalized = json.loads(json.dumps(document))
    creation = normalized.get("creationInfo")
    if isinstance(creation, dict):
        creation.pop("created", None)
    normalized.pop("documentNamespace", None)
    if canonical_name is not None:
        normalized["name"] = canonical_name
    for field in ("files", "packages"):
        values = normalized.get(field)
        if isinstance(values, list):
            values.sort(key=lambda value: value.get("SPDXID", ""))
    relationships = normalized.get("relationships")
    if isinstance(relationships, list):
        relationships.sort(
            key=lambda value: (
                value.get("spdxElementId", ""),
                value.get("relationshipType", ""),
                value.get("relatedSpdxElement", ""),
            )
        )
    return json.dumps(normalized, sort_keys=True, separators=(",", ":")).encode()


def sbom_hashes(root: Path) -> dict[str, str]:
    subprocess.run(["scripts/generate-sbom.sh"], cwd=root, check=True)
    hashes: dict[str, str] = {}
    for name in SBOMS:
        path = root / "sbom" / name
        with path.open("r", encoding="utf-8") as handle:
            document = json.load(handle)
        if not isinstance(document, dict):
            raise ProvenanceError(f"{path}: SPDX document must be an object")
        logical_name = name.removesuffix(".spdx.json")
        allowed_names = {logical_name}
        if logical_name == "cloud-sdk":
            allowed_names.add(root.name)
        if document.get("name") not in allowed_names:
            raise ProvenanceError(f"{path}: unexpected SPDX document name")
        hashes[name] = hashlib.sha256(
            canonical_sbom(document, logical_name)
        ).hexdigest()
    return hashes


def committed_sbom_hashes(root: Path, head: str) -> dict[str, str]:
    hashes: dict[str, str] = {}
    for name in SBOMS:
        document = json.loads(committed_file(root, head, f"sbom/{name}"))
        if not isinstance(document, dict):
            raise ProvenanceError(f"committed {name} is not an SPDX object")
        logical_name = name.removesuffix(".spdx.json")
        if document.get("name") != logical_name:
            raise ProvenanceError(f"committed {name} has the wrong document name")
        hashes[name] = hashlib.sha256(
            canonical_sbom(document, logical_name)
        ).hexdigest()
    return hashes


def compare(label: str, first: dict[str, str], second: dict[str, str]) -> None:
    if first != second:
        missing = sorted(set(first) ^ set(second))
        changed = sorted(name for name in set(first) & set(second) if first[name] != second[name])
        raise ProvenanceError(
            f"{label} differs between clean clones: missing={missing}, changed={changed}"
        )


def tool_provenance(root: Path, head: str) -> dict[str, str]:
    return {
        "source_commit": head,
        "source_tree": source_tree_at(root, head),
        "cargo_lock_sha256": sha256_bytes(committed_file(root, head, "Cargo.lock")),
        "git": capture(["git", "--version"], root=root),
        "rustc": capture(["rustc", "--version", "--verbose"], root=root).replace("\n", "; "),
        "cargo": capture(["cargo", "--version", "--verbose"], root=root).replace("\n", "; "),
        "cargo_sbom": capture(["cargo", "sbom", "--version"], root=root),
    }


def packages_from_policy(root: Path = ROOT, head: str | None = None) -> tuple[str, ...]:
    if head is None:
        with (root / "release-governance.toml").open("rb") as handle:
            config = tomllib.load(handle)
    else:
        config = tomllib.loads(
            committed_file(root, head, "release-governance.toml").decode("utf-8")
        )
    packages = config["packages"]["publishable"]
    if not isinstance(packages, list) or not all(isinstance(item, str) for item in packages):
        raise ProvenanceError("release governance publishable list is malformed")
    if set(packages) != set(PACKAGE_PATCHES):
        raise ProvenanceError("package patch inventory differs from governance policy")
    return tuple(packages)


def run_reproducibility() -> None:
    if clean_status(ROOT):
        raise ProvenanceError("source worktree must be clean")
    head = capture(["git", "rev-parse", "HEAD"], root=ROOT)
    packages = packages_from_policy(ROOT, head)
    with tempfile.TemporaryDirectory(prefix="cloud-sdk-reproduce-") as temporary:
        base = Path(temporary)
        roots = (base / "first", base / "second")
        for root in roots:
            clone_exact(ROOT, root, head)
        first_packages = package_archives(roots[0], packages)
        second_packages = package_archives(roots[1], packages)
        compare("package archives", first_packages, second_packages)
        first_sboms = sbom_hashes(roots[0])
        second_sboms = sbom_hashes(roots[1])
        compare("canonical SBOMs", first_sboms, second_sboms)
        compare("committed SBOMs", committed_sbom_hashes(ROOT, head), first_sboms)

    evidence = tool_provenance(ROOT, head)
    assert_source_unchanged(ROOT, head)
    print(json.dumps(evidence, indent=2, sort_keys=True))
    for name, digest in sorted(first_packages.items()):
        print(f"package sha256 {digest}  {name}")
    for name, digest in sorted(first_sboms.items()):
        print(f"canonical SBOM sha256 {digest}  {name}")
    assert_source_unchanged(ROOT, head)
    print("Two clean clones reproduced every package archive and complete SBOM.")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.parse_args()
    try:
        run_reproducibility()
    except (
        ProvenanceError,
        OSError,
        KeyError,
        ValueError,
        json.JSONDecodeError,
        subprocess.SubprocessError,
    ) as error:
        print(f"release provenance: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
