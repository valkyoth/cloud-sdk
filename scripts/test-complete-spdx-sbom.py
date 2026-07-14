#!/usr/bin/env python3
"""Regression tests for SPDX completion from Cargo metadata."""

from __future__ import annotations

import copy
import importlib.util
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MODULE_PATH = ROOT / "scripts" / "complete_spdx_sbom.py"
SPEC = importlib.util.spec_from_file_location("complete_spdx_sbom", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def package(name: str, version: str, *, source: str | None = "registry") -> dict:
    return {
        "name": name,
        "version": version,
        "id": f"{source or 'path'}#{name}@{version}",
        "source": (
            "registry+https://github.com/rust-lang/crates.io-index"
            if source == "registry"
            else None
        ),
        "license": "MIT OR Apache-2.0" if source else None,
        "description": f"{name} description",
        "homepage": f"https://example.invalid/{name}",
    }


def metadata() -> dict:
    root = package("root_pkg", "1.0.0", source=None)
    runtime = package("runtime", "2.0.0")
    build = package("build-helper", "3.0.0")
    return {
        "packages": [root, runtime, build],
        "resolve": {
            "nodes": [
                {
                    "id": root["id"],
                    "deps": [
                        {
                            "pkg": runtime["id"],
                            "dep_kinds": [{"kind": None, "target": None}],
                        },
                        {
                            "pkg": build["id"],
                            "dep_kinds": [{"kind": "build", "target": None}],
                        },
                    ],
                },
                {"id": runtime["id"], "deps": []},
                {"id": build["id"], "deps": []},
            ]
        },
    }


def base_document() -> dict:
    return {
        "creationInfo": {"creators": ["Tool: cargo-sbom-v0.10.0"]},
        "packages": [
            {
                "SPDXID": "SPDXRef-Package-root--pkg-1.0.0",
                "downloadLocation": "NONE",
                "licenseConcluded": "NOASSERTION",
                "name": "root_pkg",
                "versionInfo": "1.0.0",
            },
            {
                "SPDXID": "SPDXRef-Package-runtime-2.0.0",
                "downloadLocation": "registry",
                "licenseConcluded": "MIT OR Apache-2.0",
                "name": "runtime",
                "versionInfo": "2.0.0",
            },
        ],
        "relationships": [],
    }


def assert_raises(fragment: str, callback) -> None:
    try:
        callback()
    except ValueError as error:
        assert fragment in str(error), error
    else:
        raise AssertionError(f"expected ValueError containing {fragment!r}")


def test_adds_build_package_and_complete_edges() -> None:
    completed = MODULE.complete_document(metadata(), base_document())
    packages = {(item["name"], item["versionInfo"]): item for item in completed["packages"]}
    assert set(packages) == {
        ("root_pkg", "1.0.0"),
        ("runtime", "2.0.0"),
        ("build-helper", "3.0.0"),
    }
    build = packages[("build-helper", "3.0.0")]
    assert build["licenseDeclared"] == "MIT OR Apache-2.0"
    assert build["externalRefs"][0]["referenceLocator"] == "pkg:cargo/build-helper@3.0.0"
    root_id = packages[("root_pkg", "1.0.0")]["SPDXID"]
    runtime_id = packages[("runtime", "2.0.0")]["SPDXID"]
    build_id = build["SPDXID"]
    relationships = {
        (
            item["spdxElementId"],
            item["relationshipType"],
            item["relatedSpdxElement"],
        )
        for item in completed["relationships"]
    }
    assert (root_id, "DEPENDS_ON", runtime_id) in relationships
    assert (root_id, "DEPENDS_ON", build_id) in relationships
    assert (build_id, "BUILD_DEPENDENCY_OF", root_id) in relationships
    assert "Tool: cloud-sdk-complete-spdx-v1" in completed["creationInfo"]["creators"]


def test_completion_is_idempotent() -> None:
    first = MODULE.complete_document(metadata(), base_document())
    second = MODULE.complete_document(metadata(), copy.deepcopy(first))
    assert first == second


def test_rejects_ambiguous_and_unresolved_graphs() -> None:
    ambiguous = metadata()
    duplicate = copy.deepcopy(ambiguous["packages"][1])
    duplicate["id"] = "git#runtime@2.0.0"
    ambiguous["packages"].append(duplicate)
    assert_raises(
        "ambiguous package identity",
        lambda: MODULE.complete_document(ambiguous, base_document()),
    )

    unresolved = metadata()
    unresolved["resolve"]["nodes"][0]["deps"][0]["pkg"] = "missing#package"
    assert_raises(
        "malformed Cargo dependency edge",
        lambda: MODULE.complete_document(unresolved, base_document()),
    )


def main() -> None:
    tests = (
        test_adds_build_package_and_complete_edges,
        test_completion_is_idempotent,
        test_rejects_ambiguous_and_unresolved_graphs,
    )
    for test in tests:
        test()
    print(f"{len(tests)} complete SPDX SBOM tests passed.")


if __name__ == "__main__":
    main()
