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
COMMIT = "e37aa705b096b0f64b8ad5c0312d8e8fc8637315"
BASE = f"https://raw.githubusercontent.com/rust-lang/crates.io/{COMMIT}/"
SOURCES = {
    "src/controllers/version/yank.rs": (2973, "be6d39543023d81c98eb7dc2392bc96cdbb03cdef34075e79437b74d051782af"),
    "src/controllers/krate/update.rs": (7281, "d8608ef4b7e0c207423f257bf8c1f11bdb32975869ee4cda6282936d7c92cea0"),
    "src/controllers/version/update.rs": (6194, "e12743c911d524b931b41fc80153db9424e4629506d7da1bdb5574b0281124e9"),
    "src/controllers/token.rs": (11490, "931f184de5b06f3713e1f3842bcdc13283c3a515dc98ddc3db1171280fb7b875"),
    "src/controllers/user/update.rs": (4881, "b2e5bc645b025670b923dc332bbf404895947f0b99795db6e2e27d807ef5e68e"),
    "src/controllers/user/email_verification.rs": (3764, "d8ce3e5d1bd9ecd61855e6c76abe8fc0fe9f7ba5d81fcdd1f262f81ef1144750"),
    "src/controllers/user/email_notifications.rs": (3230, "8dcd046e1d4203427540bc24a75afbbe2eb1052c16acbaa8bbca4f424105b74a"),
    "src/controllers/krate/follow.rs": (4197, "c9eea899901aa1d7affc3b2f968e2062838c5f84f696accae99056a74faab30f"),
    "src/controllers/crate_owner_invitation.rs": (20736, "41b16c0f37353b18c2d60e369c1b4432385d0b5bb2eafe45ef331e2c2412135e"),
    "crates/crates_io_database/tests/canon_username.rs": (1097, "b8b793aa0eb28dd2ab6bd0b767a0805a641019bae1a4cc668c0c36566a8ddd39"),
    "src/controllers/user/other.rs": (5446, "3ac97197f2e023fcbbbe710d921bdf1fede042b13589129f7d38714ceac06f81"),
    "src/controllers/team.rs": (1357, "9faaa3c9ed6f1e94ed76cb34319aa70764145d0da591fb321d7bb4a7940bd180"),
    "src/controllers/krate/owners.rs": (29817, "5978cb34728d50aa1ebd11d1bde0c26ec2824d55f3efb477bbb9f19ad8812f6e"),
    "src/controllers/krate/downloads.rs": (7293, "2a1e7cb2a931f5b0852fd145f62351dcc6e039af31bbfc1417b3bdc5941b507a"),
    "src/controllers/version/downloads.rs": (4389, "9bf674031f86cf160d297fd20171867da04c8ce261eeede3410ac1e6af189eb3"),
    "src/controllers/krate/rev_deps.rs": (3387, "c9ed6499ccf2f6538e5dd1e3d5f11277a8a10bd3aaeb1e3e70c2c64762ad42e1"),
    "src/controllers/krate/versions.rs": (11710, "4a03de16908afafacccedeb206cd7fd7a86ad8bbed9125ab5b2dfa7c9231f62b"),
    "src/controllers/version/readme.rs": (1742, "52c964ddcf2465d51299976236221d1c4de09445c0e30ede5265f2924de3e8f6"),
    "src/controllers/version/authors.rs": (876, "e9b6133bd5e2f1a6526ffe80161d91cb89142fa1f79f4dc668eb4a7995ee4658"),
    "src/controllers/version/dependencies.rs": (1827, "4472318fa0ff54aceec778780a6ecf9b463bbfbecff2deb364aa9839d7e72377"),
    "src/controllers/krate/search.rs": (34131, "7b3edaa2b3d11625d172d43d07cb6ed667666c37e1c7e3bcc2d66a8665a13bc9"),
    "src/controllers/krate/metadata.rs": (12252, "5047343f0a804cef130f7db533812b49dbe1521a0c58df057b73034f7c5927a9"),
    "src/controllers/category.rs": (5949, "f2c5aac350751d18486ca5379e15e61880db61e84de71bd74b4770290945bb07"),
    "src/controllers/keyword.rs": (4101, "01a5909ec7ed6976bdb614335af3fa03ed3fab212345c339d6573bf53cd13549"),
    "crates/crates_io_validation/src/lib.rs": (12813, "d2747b30b5f0241110cbd6e84c197975d72529e2f21a1e3c9242b8937615a48b"),
    "src/controllers/helpers/pagination.rs": (23575, "507334f94048c1c2f77b4494e6fd48d03c5b3b1ac252f84b57d1cc70b9f0fd3a"),
    "src/controllers/krate/publish.rs": (44717, "35df1c9b9eb693f98a0c4a642858052333397301633af3f266ec8cf25c24c750"),
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
    # The controller disables numbered pages; the authenticated request type
    # admits one filter and seek pagination instead of the generic query API.
    desired[("get", "/api/v1/crate_owner_invitations")] = {
        "crate_name", "invitee_id", "page", "per_page", "seek"
    }
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
