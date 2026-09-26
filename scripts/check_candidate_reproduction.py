#!/usr/bin/env python3
"""Compare all six package archives built from two clean clones of HEAD."""

import hashlib
import os
from pathlib import Path
import subprocess
import tempfile
import tomllib

from check_cratesio_archives import inspect

ROOT = Path(__file__).resolve().parents[1]
PACKAGES = ("cloud-sdk", "cloud-sdk-sanitization", "cloud-sdk-testkit",
            "cloud-sdk-reqwest", "cloud-sdk-hetzner", "cloud-sdk-cratesio")


def git(root, *arguments):
    return subprocess.check_output(["git", *arguments], cwd=root, text=True).strip()


def clean_head(root):
    if git(root, "status", "--porcelain", "--untracked-files=all"):
        raise ValueError("candidate reproduction requires a clean repository")
    return git(root, "rev-parse", "HEAD")


def package_command(name):
    command = ["cargo", "package", "--locked", "--offline", "--no-verify",
               "--all-features", "-p", name]
    for dependency in PACKAGES:
        if dependency != name:
            command += ["--config", f'patch.crates-io.{dependency}.path="crates/{dependency}"']
    return command


def packages(root, target):
    manifest = tomllib.loads((root / "Cargo.toml").read_text("utf-8"))
    results = {}
    environment = dict(os.environ, CARGO_TARGET_DIR=str(target), AWS_LC_SYS_USE_SYSTEM="0")
    for name in PACKAGES:
        package = tomllib.loads((root / "crates" / name / "Cargo.toml").read_text("utf-8"))
        version = package["package"]["version"]
        if isinstance(version, dict):
            version = manifest["workspace"]["package"]["version"]
        prefix = f"{name}-{version}"
        subprocess.run(package_command(name), cwd=root, env=environment, check=True)
        archive = target / "package" / f"{prefix}.crate"
        members = inspect(archive, prefix)
        results[name] = (hashlib.sha256(archive.read_bytes()).hexdigest(), members)
    return results


def reproduce(root=ROOT, builder=packages):
    reviewed = clean_head(root)
    results = []
    with tempfile.TemporaryDirectory(prefix="cloud-sdk-candidate-clones-") as temporary:
        for index in range(2):
            clone = Path(temporary) / f"clone-{index}"
            subprocess.run(["git", "clone", "--local", "--no-hardlinks", "--no-checkout",
                            str(root), str(clone)], check=True)
            subprocess.run(["git", "checkout", "--detach", reviewed], cwd=clone, check=True)
            if clean_head(clone) != reviewed:
                raise ValueError("candidate clone differs from reviewed HEAD")
            results.append(builder(clone, Path(temporary) / f"target-{index}"))
            if clean_head(clone) != reviewed:
                raise ValueError("package creation modified a candidate clone")
    if clean_head(root) != reviewed:
        raise ValueError("candidate changed during reproduction")
    if set(results[0]) != set(PACKAGES) or results[0] != results[1]:
        raise ValueError("candidate archives are incomplete or not reproducible")
    return reviewed, results[0]


def main():
    try:
        reviewed, results = reproduce()
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"candidate reproduction failed: {error}") from error
    print(f"Two clean clones reproduced all six packages at {reviewed}:")
    for name, (digest, members) in results.items():
        print(f"  {name}: {digest} ({len(members)} files)")
    print("Archive reproduction only; packaged compile verification is a separate gate.")


if __name__ == "__main__":
    main()
