#!/usr/bin/env python3
"""Bind rustix and policy-checker admissions to exact manifest/lock artifacts."""
from pathlib import Path
import re
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def read_toml(path):
    return tomllib.loads(path.read_text(encoding="utf-8"))


def artifact(lock, name, requirement):
    packages = [p for p in lock["package"] if p["name"] == name]
    if len(packages) != 1 or requirement != "=" + packages[0]["version"]:
        raise ValueError(f"{name}: manifest and locked artifact differ")
    package = packages[0]
    if package.get("source") != "registry+https://github.com/rust-lang/crates.io-index":
        raise ValueError(f"{name}: unreviewed source")
    if re.fullmatch(r"[a-f0-9]{64}", package.get("checksum", "")) is None:
        raise ValueError(f"{name}: missing registry checksum")
    return package["version"], package["checksum"]


def features(dependency, expected):
    if dependency.get("default-features") is not False or dependency.get("features", []) != expected:
        raise ValueError("unreviewed direct feature selection")


def validate(root):
    manifest = read_toml(root / "Cargo.toml")
    dependency = manifest["workspace"]["dependencies"]["rustix"]
    features(dependency, ["fs", "process", "std"])
    version, checksum = artifact(read_toml(root / "Cargo.lock"), "rustix", dependency["version"])
    document = (root / "docs/dependency-admission-rustix.md").read_text(encoding="ascii")
    if (f"Admit exact `rustix {version}`" not in document
            or f"- Version: `{version}`" not in document
            or f"- Registry checksum:\n  `{checksum}`" not in document):
        raise ValueError("rustix admission is stale")
    tool = root / "tools/prepared-coverage-check"
    dependencies = read_toml(tool / "Cargo.toml")["dependencies"]
    if set(dependencies) != {"syn", "saphyr", "saphyr-parser"}:
        raise ValueError("unreviewed checker dependency")
    lock = read_toml(tool / "Cargo.lock")
    document = (root / "docs/dependency-admission-prepared-coverage.md").read_text(encoding="ascii")
    for name, dependency in dependencies.items():
        features(dependency, ["full", "parsing", "visit"] if name == "syn" else [])
        version, checksum = artifact(lock, name, dependency["version"])
        if f"| `{name}` | `{version}` | `{checksum}` |" not in document.splitlines():
            raise ValueError(f"{name}: admission artifact is stale")
        versions = re.findall(rf"^\| `{re.escape(name)}` \| `([^`]+)` \|", document, re.M)
        if len(versions) != 2 or set(versions) != {version}:
            raise ValueError(f"{name}: conflicting admission versions")


def main():
    try:
        validate(ROOT)
    except (OSError, UnicodeError, ValueError, KeyError, TypeError) as error:
        print(f"admission evidence: {error}", file=sys.stderr)
        return 1
    print("Rustix and checker admission artifacts match exact pins, features and checksums.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
