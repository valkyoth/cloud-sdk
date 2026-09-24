#!/usr/bin/env python3
"""Require exact reviewed versions for direct third-party dependencies."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
import sys
import tomllib
import urllib.error
import urllib.parse
import urllib.request
from collections.abc import Callable


ROOT = Path(__file__).resolve().parent.parent
AUXILIARY_MANIFESTS = (
    "fuzz/Cargo.toml",
    "tests/reqwest-feature-unification/Cargo.toml",
    "tools/prepared-coverage-check/Cargo.toml",
)
EXACT_VERSION = re.compile(r"^=[0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?$")
CRATE_NAME = re.compile(r"^[0-9A-Za-z_-]+$")
MAX_REGISTRY_RESPONSE_BYTES = 1_048_576
USER_AGENT = "cloud-sdk-dependency-freshness/1.1.0 (https://github.com/valkyoth/cloud-sdk)"


class RejectRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        del req, fp, code, msg, headers, newurl
        return None


def pin_problems(dependencies: dict) -> list[str]:
    problems = []
    for name, specification in sorted(dependencies.items()):
        if name.startswith("cloud-sdk"):
            continue
        if isinstance(specification, str):
            requirement = specification
        elif isinstance(specification, dict):
            requirement = specification.get("version")
            if requirement is None and "path" in specification:
                continue
        else:
            requirement = None
        if not isinstance(requirement, str) or not EXACT_VERSION.fullmatch(
            requirement
        ):
            problems.append(
                f"{name}: direct third-party requirement must be an exact =X.Y.Z pin"
            )
    return problems


def direct_pins(dependencies: dict) -> dict[str, str]:
    pins = {}
    for name, specification in sorted(dependencies.items()):
        if name.startswith("cloud-sdk"):
            continue
        requirement = (
            specification
            if isinstance(specification, str)
            else specification.get("version")
            if isinstance(specification, dict)
            else None
        )
        if isinstance(requirement, str) and EXACT_VERSION.fullmatch(requirement):
            pins[name] = requirement[1:]
    return pins


def parse_registry_payload(payload: bytes, name: str) -> str:
    try:
        document = json.loads(payload)
        crate = document["crate"]
        returned_name = crate["name"]
        version = crate["max_stable_version"]
    except (KeyError, TypeError, json.JSONDecodeError) as error:
        raise ValueError(f"crates.io returned malformed metadata for {name}") from error
    if returned_name != name:
        raise ValueError(f"crates.io returned mismatched metadata for {name}")
    if not isinstance(version, str) or not EXACT_VERSION.fullmatch(f"={version}"):
        raise ValueError(f"crates.io returned an invalid stable version for {name}")
    return version


def registry_version(name: str) -> str:
    if not CRATE_NAME.fullmatch(name):
        raise ValueError(f"invalid crates.io package name {name!r}")
    url = f"https://crates.io/api/v1/crates/{urllib.parse.quote(name, safe='')}"
    request = urllib.request.Request(
        url,
        headers={"Accept": "application/json", "User-Agent": USER_AGENT},
    )
    response = urllib.request.build_opener(RejectRedirect()).open(request, timeout=20)
    with response:
        if response.geturl() != url or response.status != 200:
            raise ValueError(f"crates.io returned an unexpected response for {name}")
        media_type = response.headers.get_content_type()
        if media_type != "application/json":
            raise ValueError(f"crates.io returned an unexpected media type for {name}")
        length = response.headers.get("Content-Length")
        if length is not None and int(length) > MAX_REGISTRY_RESPONSE_BYTES:
            raise ValueError(f"crates.io metadata exceeds the bound for {name}")
        payload = response.read(MAX_REGISTRY_RESPONSE_BYTES + 1)
    if len(payload) > MAX_REGISTRY_RESPONSE_BYTES:
        raise ValueError(f"crates.io metadata exceeds the bound for {name}")
    return parse_registry_payload(payload, name)


def freshness_problems(
    dependencies: dict, latest: Callable[[str], str]
) -> list[str]:
    problems = []
    for name, expected in direct_pins(dependencies).items():
        actual = latest(name)
        if actual != expected:
            problems.append(
                f"{name}: crates.io reports {actual}; exact reviewed pin is {expected}"
            )
    return problems


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fetch", action="store_true")
    args = parser.parse_args()
    try:
        tables = dependency_tables(ROOT)
    except (OSError, UnicodeError, KeyError, tomllib.TOMLDecodeError) as error:
        print(f"dependency pins: {error}", file=sys.stderr)
        return 1
    problems = [f"{label}: {problem}" for label, dependencies in tables
                for problem in pin_problems(dependencies)]
    if problems:
        for problem in problems:
            print(f"dependency pins: {problem}", file=sys.stderr)
        return 1
    if args.fetch:
        try:
            cache = {}
            def latest(name):
                if name not in cache:
                    cache[name] = registry_version(name)
                return cache[name]
            problems = [f"{label}: {problem}" for label, dependencies in tables
                        for problem in freshness_problems(dependencies, latest)]
        except (OSError, urllib.error.URLError, ValueError) as error:
            print(f"dependency freshness: {error}", file=sys.stderr)
            return 1
        if problems:
            for problem in problems:
                print(f"dependency freshness: {problem}", file=sys.stderr)
            return 1
        print("Every direct third-party workspace and auxiliary pin is current on crates.io.")
    print("Every direct third-party workspace and auxiliary dependency has an exact reviewed pin.")
    return 0


def dependency_tables(root: Path) -> list[tuple[str, dict]]:
    manifest = tomllib.loads((root / "Cargo.toml").read_text("ascii"))
    result = [("workspace", manifest["workspace"]["dependencies"])]
    for name in AUXILIARY_MANIFESTS:
        document = tomllib.loads((root / name).read_text("ascii"))
        tables = [(name, document)]
        tables.extend((f"{name}:{target}", table)
                      for target, table in document.get("target", {}).items())
        for label, table in tables:
            for section in ("dependencies", "dev-dependencies", "build-dependencies"):
                if section in table:
                    result.append((f"{label}:{section}", table[section]))
    return result


if __name__ == "__main__":
    raise SystemExit(main())
