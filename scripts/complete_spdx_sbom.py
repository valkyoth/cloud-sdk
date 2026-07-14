#!/usr/bin/env python3
"""Complete cargo-sbom SPDX output with Cargo's full resolved graph."""

from __future__ import annotations

import hashlib
import json
import os
import re
import sys
import tempfile
from pathlib import Path
from typing import Any
from urllib.parse import quote


def package_key(package: dict[str, Any], version_field: str) -> tuple[str, str]:
    name = package.get("name")
    version = package.get(version_field)
    if not isinstance(name, str) or not name:
        raise ValueError("SBOM completion: package name must be nonempty text")
    if not isinstance(version, str) or not version:
        raise ValueError(f"SBOM completion: {name} version must be nonempty text")
    return name, version


def safe_component(value: str) -> str:
    return re.sub(
        r"[^A-Za-z0-9.-]",
        lambda match: f"-{ord(match.group(0)):02x}-",
        value,
    )


def new_spdx_id(package: dict[str, Any], used: set[str]) -> str:
    name, version = package_key(package, "version")
    base = f"SPDXRef-Package-{safe_component(name)}-{safe_component(version)}"
    candidate = base
    if candidate in used:
        package_id = package.get("id")
        if not isinstance(package_id, str):
            raise ValueError(f"SBOM completion: {name}@{version} has no Cargo ID")
        suffix = hashlib.sha256(package_id.encode("utf-8")).hexdigest()[:12]
        candidate = f"{base}-{suffix}"
    if candidate in used:
        raise ValueError(f"SBOM completion: duplicate SPDX ID {candidate}")
    return candidate


def new_package(package: dict[str, Any], spdx_id: str) -> dict[str, Any]:
    name, version = package_key(package, "version")
    source = package.get("source")
    license_expression = package.get("license")
    record: dict[str, Any] = {
        "SPDXID": spdx_id,
        "downloadLocation": source if isinstance(source, str) else "NONE",
        "licenseConcluded": (
            license_expression
            if isinstance(license_expression, str) and license_expression
            else "NOASSERTION"
        ),
        "name": name,
        "versionInfo": version,
    }
    if isinstance(license_expression, str) and license_expression:
        record["licenseDeclared"] = license_expression
    for field in ("description", "homepage"):
        value = package.get(field)
        if isinstance(value, str) and value:
            record[field] = value
    if isinstance(source, str) and source.startswith("registry+"):
        record["externalRefs"] = [
            {
                "referenceCategory": "PACKAGE-MANAGER",
                "referenceLocator": (
                    f"pkg:cargo/{quote(name, safe='-._~')}@"
                    f"{quote(version, safe='-._~')}"
                ),
                "referenceType": "purl",
            }
        ]
    return record


def unique_metadata_packages(
    packages: list[dict[str, Any]],
) -> tuple[dict[str, dict[str, Any]], dict[tuple[str, str], dict[str, Any]]]:
    by_id: dict[str, dict[str, Any]] = {}
    by_key: dict[tuple[str, str], dict[str, Any]] = {}
    for package in packages:
        key = package_key(package, "version")
        package_id = package.get("id")
        if not isinstance(package_id, str) or not package_id:
            raise ValueError(f"SBOM completion: {key[0]}@{key[1]} has no Cargo ID")
        if package_id in by_id:
            raise ValueError(f"SBOM completion: duplicate Cargo ID {package_id}")
        if key in by_key:
            raise ValueError(
                f"SBOM completion: ambiguous package identity {key[0]}@{key[1]}"
            )
        by_id[package_id] = package
        by_key[key] = package
    return by_id, by_key


def complete_document(
    metadata: dict[str, Any], document: dict[str, Any]
) -> dict[str, Any]:
    packages = metadata.get("packages")
    resolve = metadata.get("resolve")
    sbom_packages = document.get("packages")
    relationships = document.get("relationships")
    if not isinstance(packages, list) or not isinstance(resolve, dict):
        raise ValueError("SBOM completion: malformed Cargo metadata")
    if not isinstance(sbom_packages, list) or not isinstance(relationships, list):
        raise ValueError("SBOM completion: malformed SPDX document")

    metadata_by_id, metadata_by_key = unique_metadata_packages(packages)
    spdx_by_key: dict[tuple[str, str], str] = {}
    used_ids: set[str] = set()
    for package in sbom_packages:
        key = package_key(package, "versionInfo")
        spdx_id = package.get("SPDXID")
        if key not in metadata_by_key:
            raise ValueError(
                f"SBOM completion: unexpected package {key[0]}@{key[1]}"
            )
        if not isinstance(spdx_id, str) or not spdx_id:
            raise ValueError(f"SBOM completion: {key[0]}@{key[1]} has no SPDX ID")
        if key in spdx_by_key or spdx_id in used_ids:
            raise ValueError(f"SBOM completion: duplicate package {key[0]}@{key[1]}")
        spdx_by_key[key] = spdx_id
        used_ids.add(spdx_id)

    for key in sorted(metadata_by_key):
        if key in spdx_by_key:
            continue
        package = metadata_by_key[key]
        spdx_id = new_spdx_id(package, used_ids)
        sbom_packages.append(new_package(package, spdx_id))
        spdx_by_key[key] = spdx_id
        used_ids.add(spdx_id)

    spdx_by_cargo_id = {
        package_id: spdx_by_key[package_key(package, "version")]
        for package_id, package in metadata_by_id.items()
    }
    relationship_keys = {
        (
            relationship.get("spdxElementId"),
            relationship.get("relationshipType"),
            relationship.get("relatedSpdxElement"),
        )
        for relationship in relationships
    }
    nodes = resolve.get("nodes")
    if not isinstance(nodes, list):
        raise ValueError("SBOM completion: Cargo metadata has no resolve nodes")
    for node in nodes:
        parent_id = node.get("id")
        dependencies = node.get("deps")
        if parent_id not in spdx_by_cargo_id or not isinstance(dependencies, list):
            raise ValueError("SBOM completion: malformed Cargo resolve node")
        parent_spdx = spdx_by_cargo_id[parent_id]
        for dependency in dependencies:
            dependency_id = dependency.get("pkg")
            dep_kinds = dependency.get("dep_kinds")
            if dependency_id not in spdx_by_cargo_id or not isinstance(dep_kinds, list):
                raise ValueError("SBOM completion: malformed Cargo dependency edge")
            dependency_spdx = spdx_by_cargo_id[dependency_id]
            relationship_keys.add((parent_spdx, "DEPENDS_ON", dependency_spdx))
            if any(kind.get("kind") == "build" for kind in dep_kinds):
                relationship_keys.add(
                    (dependency_spdx, "BUILD_DEPENDENCY_OF", parent_spdx)
                )

    document["packages"] = sorted(sbom_packages, key=lambda item: item["SPDXID"])
    document["relationships"] = [
        {
            "spdxElementId": source,
            "relationshipType": relationship,
            "relatedSpdxElement": target,
        }
        for source, relationship, target in sorted(relationship_keys)
    ]
    creation = document.get("creationInfo")
    if not isinstance(creation, dict) or not isinstance(creation.get("creators"), list):
        raise ValueError("SBOM completion: malformed SPDX creation information")
    creator = "Tool: cloud-sdk-complete-spdx-v1"
    if creator not in creation["creators"]:
        creation["creators"].append(creator)
    return document


def load_object(path: Path) -> dict[str, Any]:
    with path.open("r", encoding="utf-8") as handle:
        value = json.load(handle)
    if not isinstance(value, dict):
        raise ValueError(f"SBOM completion: {path} does not contain an object")
    return value


def write_atomic(path: Path, document: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary_name = ""
    try:
        with tempfile.NamedTemporaryFile(
            mode="w", encoding="utf-8", dir=path.parent, delete=False
        ) as handle:
            temporary_name = handle.name
            json.dump(document, handle, indent=2)
            handle.write("\n")
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary_name, path)
    finally:
        if temporary_name:
            try:
                os.unlink(temporary_name)
            except FileNotFoundError:
                pass


def main() -> int:
    if len(sys.argv) != 4:
        print(
            "usage: scripts/complete_spdx_sbom.py METADATA BASE_SBOM OUTPUT",
            file=sys.stderr,
        )
        return 2
    metadata_path, sbom_path, output_path = map(Path, sys.argv[1:])
    try:
        completed = complete_document(
            load_object(metadata_path), load_object(sbom_path)
        )
        write_atomic(output_path, completed)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(str(error), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
