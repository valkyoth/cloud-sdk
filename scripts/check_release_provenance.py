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


class ProvenanceError(RuntimeError):
    """Release provenance could not be reproduced exactly."""


def capture(command: list[str], *, root: Path) -> str:
    return subprocess.check_output(command, cwd=root, text=True).strip()


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
        subprocess.run(
            [
                "cargo",
                "package",
                "--locked",
                "--no-verify",
                "-p",
                package,
            ],
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


def canonical_sbom(document: dict[str, Any]) -> bytes:
    normalized = json.loads(json.dumps(document))
    creation = normalized.get("creationInfo")
    if isinstance(creation, dict):
        creation.pop("created", None)
    normalized.pop("documentNamespace", None)
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
        hashes[name] = hashlib.sha256(canonical_sbom(document)).hexdigest()
    return hashes


def committed_sbom_hashes(root: Path) -> dict[str, str]:
    hashes: dict[str, str] = {}
    for name in SBOMS:
        with (root / "sbom" / name).open("r", encoding="utf-8") as handle:
            document = json.load(handle)
        if not isinstance(document, dict):
            raise ProvenanceError(f"committed {name} is not an SPDX object")
        hashes[name] = hashlib.sha256(canonical_sbom(document)).hexdigest()
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
        "source_tree": capture(["git", "rev-parse", "HEAD^{tree}"], root=root),
        "cargo_lock_sha256": sha256(root / "Cargo.lock"),
        "git": capture(["git", "--version"], root=root),
        "rustc": capture(["rustc", "--version", "--verbose"], root=root).replace("\n", "; "),
        "cargo": capture(["cargo", "--version", "--verbose"], root=root).replace("\n", "; "),
        "cargo_sbom": capture(["cargo", "sbom", "--version"], root=root),
    }


def packages_from_policy() -> tuple[str, ...]:
    with CONFIG.open("rb") as handle:
        config = tomllib.load(handle)
    packages = config["packages"]["publishable"]
    if not isinstance(packages, list) or not all(isinstance(item, str) for item in packages):
        raise ProvenanceError("release governance publishable list is malformed")
    return tuple(packages)


def run_reproducibility() -> None:
    if clean_status(ROOT):
        raise ProvenanceError("source worktree must be clean")
    head = capture(["git", "rev-parse", "HEAD"], root=ROOT)
    packages = packages_from_policy()
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
        compare("committed SBOMs", committed_sbom_hashes(ROOT), first_sboms)

    print(json.dumps(tool_provenance(ROOT, head), indent=2, sort_keys=True))
    for name, digest in sorted(first_packages.items()):
        print(f"package sha256 {digest}  {name}")
    for name, digest in sorted(first_sboms.items()):
        print(f"canonical SBOM sha256 {digest}  {name}")
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

