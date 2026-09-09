#!/usr/bin/env python3
"""Check crates.io request bounds and source-pinned implementation contracts."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

from cratesio_source_fetch import fetch_source
from cratesio_source_error import SourceLockError

ROOT = Path(__file__).resolve().parents[1]
COMMIT = "9ae7f769cea32f38ebc2ea9ec2ce455b47641511"
BASE = f"https://raw.githubusercontent.com/rust-lang/crates.io/{COMMIT}/"
SOURCES = {
    "crates/crates_io_validation/src/lib.rs": (12829, "02872dd2803f5ebefbedb7d6d3f8cd3b8c2137e041e0c42a6b01d85b04d1d8a4"),
    "src/controllers/helpers/pagination.rs": (23556, "e65573b6888acdbde657946297203980804afbf3525476d6be20a011a7442a53"),
    "src/controllers/krate/publish.rs": (42593, "4dd23414ff79b5eb1ff63b9ebf320de3c07de245b46d54a9b511a0dbf110d004"),
    "crates/crates_io_database/src/models/keyword.rs": (3662, "6817786f81454e55ebf0de77f44042c2d46f96fff7ef0aa24c94c86fd5cb1299"),
}
PATHS = {
    "Categories": "/api/v1/categories", "Keywords": "/api/v1/keywords",
    "Crates": "/api/v1/crates", "Crate": "/api/v1/crates/{name}",
    "Downloads": "/api/v1/crates/{name}/downloads",
    "Versions": "/api/v1/crates/{name}/versions",
    "ReverseDependencies": "/api/v1/crates/{name}/reverse_dependencies",
    "VersionDownloads": "/api/v1/crates/{name}/{version}/downloads",
    "User": "/api/v1/users/{user}",
    "GithubConfigs": "/api/v1/trusted_publishing/github_configs",
    "GitlabConfigs": "/api/v1/trusted_publishing/gitlab_configs",
}
CRATE = Path("crates/cloud-sdk-cratesio/src")


class PolicyError(ValueError):
    """The reviewed request-policy contract changed."""


def contracts(root: Path) -> dict[str, set[str]]:
    text = (root / CRATE / "query/contract_tests.rs").read_text(encoding="ascii")
    table = re.search(r"const CONTRACTS:.*?= &\[(.*?)\];", text, re.S)
    if table is None:
        raise PolicyError("query contract fixtures are missing")
    pattern = r'\(\s*QueryOperation::([A-Za-z]+),\s*"([a-z_ \[\]]+)",?\s*\),'
    rows = re.findall(pattern, table[1])
    if re.sub(pattern, "", table[1]).strip() or len(rows) != len(PATHS):
        raise PolicyError("query contract fixtures must remain a complete literal table")
    result = {name: set(keys.split()) for name, keys in rows}
    if set(result) != set(PATHS) or any(len(keys.split()) != len(set(keys.split())) for _, keys in rows):
        raise PolicyError("query contract fixtures contain duplicate or missing entries")
    return result


def validate(root: Path, online: bool = False, fetcher=fetch_source) -> None:
    lock = json.loads((root / "provider-drift/providers/cratesio-source.lock.json").read_text())
    if lock["source_commit"] != COMMIT:
        raise PolicyError("request-policy implementation sources require review at the new commit")
    limits = {
        "identifiers/mod.rs": {"MAX_CRATE_NAME_BYTES": 64, "MAX_VERSION_BYTES": 150,
            "MAX_KEYWORD_BYTES": 20, "MAX_IDENTIFIER_BYTES": 256, "MAX_LOGIN_BYTES": 100},
        "query/mod.rs": {"MAX_QUERY_VALUE_BYTES": 1024, "MAX_QUERY_PARAMETERS": 16,
            "MAX_TARGET_BYTES": 4096, "MAX_PER_PAGE": 100, "MAX_PAGE": 10},
    }
    for path, constants in limits.items():
        text = (root / CRATE / path).read_text(encoding="ascii")
        for name, expected in constants.items():
            values = re.findall(rf"^pub const {name}: (?:usize|u32) = ([0-9_]+);$", text, re.M)
            if len(values) != 1 or int(values[0].replace("_", "")) != expected:
                raise PolicyError(f"reviewed request limit changed: {name}")
    expected = contracts(root)
    if not online:
        return
    source = next(s for s in lock["sources"] if s["id"] == "openapi")
    spec = json.loads(fetcher(source))
    actual = {}
    for path, operations in spec["paths"].items():
        for method, operation in operations.items():
            if not isinstance(operation, dict):
                continue
            names = {p["name"] for p in operation.get("parameters", []) if p.get("in") == "query"}
            if names:
                actual[(method, path)] = names
    desired = {("get", PATHS[name]): keys for name, keys in expected.items()}
    if actual != desired:
        raise PolicyError("public query parameter inventory changed; update typed coverage and tests")
    for path, (size, digest) in SOURCES.items():
        fetcher({"id": path, "url": BASE + path, "final_url": BASE + path,
            "redirects": [], "accept": "text/plain", "media_type": "text/plain",
            "size_bytes": size, "max_bytes": 131072, "sha256": digest})


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fetch", action="store_true")
    args = parser.parse_args()
    try:
        validate(ROOT, args.fetch)
    except (OSError, ValueError, KeyError, StopIteration, SourceLockError) as error:
        print(f"crates.io request policy: {error}")
        return 1
    print("crates.io request bounds and query contract fixtures passed" + (" with live source verification." if args.fetch else "."))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
