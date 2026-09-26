#!/usr/bin/env python3
"""Build the provider archive twice and check content and byte reproducibility."""

import hashlib
import os
from pathlib import Path, PurePosixPath
import subprocess
import tarfile
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def inspect(path, prefix):
    names = set()
    with tarfile.open(path, "r:gz") as archive:
        for member in archive:
            parts = PurePosixPath(member.name).parts
            if (not parts or parts[0] != prefix or ".." in parts
                    or not member.isfile() or member.name in names):
                raise ValueError("invalid package member")
            if any(part in {".git", ".env", "PENTEST.md", "target"}
                   or part.endswith((".token", ".secret", ".key")) for part in parts):
                raise ValueError("private or generated package member")
            names.add(member.name)
    for required in ("Cargo.toml", "Cargo.lock", "README.md", "src/lib.rs"):
        if f"{prefix}/{required}" not in names:
            raise ValueError(f"missing package member: {required}")
    return names


def main():
    with (ROOT / "crates/cloud-sdk-cratesio/Cargo.toml").open("rb") as source:
        manifest = tomllib.load(source)
    version = manifest["package"]["version"]
    if not isinstance(version, str):
        with (ROOT / "Cargo.toml").open("rb") as source:
            version = tomllib.load(source)["workspace"]["package"]["version"]
    prefix = f"cloud-sdk-cratesio-{version}"
    results = []
    with tempfile.TemporaryDirectory(prefix="cloud-sdk-archive-") as temporary:
        for index in range(2):
            target = Path(temporary) / str(index)
            environment = dict(os.environ, CARGO_TARGET_DIR=str(target),
                               AWS_LC_SYS_USE_SYSTEM="0")
            command = ["cargo", "package", "--locked", "--allow-dirty",
                       "--no-verify", "--all-features", "-p", "cloud-sdk-cratesio"]
            for name in ("cloud-sdk", "cloud-sdk-reqwest", "cloud-sdk-sanitization"):
                command += ["--config", f'patch.crates-io.{name}.path="crates/{name}"']
            subprocess.run(command, cwd=ROOT, env=environment, check=True)
            archive = target / "package" / f"{prefix}.crate"
            names = inspect(archive, prefix)
            results.append((hashlib.sha256(archive.read_bytes()).hexdigest(), names))
    if results[0] != results[1]:
        raise SystemExit("crates.io archive is not reproducible")
    print(f"crates.io archive: {len(results[0][1])} files; SHA-256 {results[0][0]}")
    print("Two independent target directories matched; no publication performed.")


if __name__ == "__main__":
    main()
