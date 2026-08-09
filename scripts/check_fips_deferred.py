#!/usr/bin/env python3
"""Keep FIPS out of active cloud-sdk surfaces until Brynja is ready."""

from __future__ import annotations

import argparse
import sys
import tomllib
from pathlib import Path


FORBIDDEN_FEATURE = "blocking-rustls-fips"
FORBIDDEN_DEPENDENCY = "aws-lc-fips-sys"
FORBIDDEN_SYMBOL = "FipsTlsPolicy"
FORBIDDEN_DENY_TERMS = (FORBIDDEN_FEATURE, FORBIDDEN_DEPENDENCY, FORBIDDEN_SYMBOL)


def load_toml(path: Path) -> dict:
    with path.open("rb") as source:
        return tomllib.load(source)


def collect_failures(root: Path) -> list[str]:
    failures: list[str] = []
    workspace = load_toml(root / "Cargo.toml")
    workspace_dependencies = workspace["workspace"]["dependencies"]
    if FORBIDDEN_DEPENDENCY in workspace_dependencies:
        failures.append("workspace dependency admits aws-lc-fips-sys")

    reqwest_manifest = root / "crates" / "cloud-sdk-reqwest" / "Cargo.toml"
    reqwest = load_toml(reqwest_manifest)
    if FORBIDDEN_FEATURE in reqwest.get("features", {}):
        failures.append("cloud-sdk-reqwest exposes blocking-rustls-fips")
    if FORBIDDEN_DEPENDENCY in reqwest.get("dependencies", {}):
        failures.append("cloud-sdk-reqwest depends on aws-lc-fips-sys")

    fixture_manifest = root / "tests" / "reqwest-feature-unification" / "Cargo.toml"
    fixture_text = fixture_manifest.read_text(encoding="utf-8")
    if FORBIDDEN_FEATURE in fixture_text:
        failures.append("feature-unification fixture enables blocking-rustls-fips")

    for lockfile in root.rglob("Cargo.lock"):
        if any(part in {".git", "target"} for part in lockfile.parts):
            continue
        if f'name = "{FORBIDDEN_DEPENDENCY}"' in lockfile.read_text(encoding="utf-8"):
            failures.append(f"{lockfile.relative_to(root)} locks aws-lc-fips-sys")

    reqwest_source = root / "crates" / "cloud-sdk-reqwest" / "src"
    for source in reqwest_source.rglob("*.rs"):
        text = source.read_text(encoding="utf-8")
        if FORBIDDEN_FEATURE in text or FORBIDDEN_SYMBOL in text:
            failures.append(f"{source.relative_to(root)} exposes removed FIPS API")

    ci = (root / ".github" / "workflows" / "ci.yml").read_text(encoding="utf-8")
    if "fips-transport:" in ci or "check_reqwest_fips_boundary.sh" in ci:
        failures.append("CI still activates the retired FIPS transport")

    deny = load_toml(root / "deny.toml")
    for exception in deny.get("bans", {}).get("skip", []):
        exception_text = f'{exception.get("crate", "")} {exception.get("reason", "")}'
        if any(term.lower() in exception_text.lower() for term in FORBIDDEN_DENY_TERMS):
            failures.append("deny.toml retains an obsolete FIPS-specific ban exception")

    for relative in ("README.md", "crates/cloud-sdk/README.md", "crates/cloud-sdk-reqwest/README.md"):
        text = (root / relative).read_text(encoding="utf-8")
        if FORBIDDEN_FEATURE in text or FORBIDDEN_SYMBOL in text:
            failures.append(f"{relative} still advertises the retired FIPS API")

    policy = (root / "docs" / "FIPS_DEFERMENT.md").read_text(encoding="utf-8")
    for required in (
        "not part of the cloud-sdk 1.0 scope",
        "deferred until Brynja",
        "exact cryptographic module",
        "no FIPS compliance claim",
    ):
        if required not in policy:
            failures.append(f"FIPS deferment policy lacks required boundary: {required}")

    return failures


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    failures = collect_failures(args.root.resolve())
    if failures:
        for failure in failures:
            print(f"FIPS deferment: {failure}", file=sys.stderr)
        return 1
    print("FIPS remains absent and deferred to Brynja.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
