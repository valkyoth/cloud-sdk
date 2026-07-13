#!/usr/bin/python3
"""Root-owned launcher for the authenticated Hetzner live-smoke executable."""

from __future__ import annotations

import hashlib
import os
from pathlib import Path
import stat
import sys
from typing import NoReturn


INSTALL_DIR = Path("/usr/local/libexec/cloud-sdk-live-smoke")
ARTIFACT = INSTALL_DIR / "live_smoke"
MANIFEST = INSTALL_DIR / "manifest"
RUNNER = INSTALL_DIR / "runner.py"
LAUNCHER = Path("/usr/local/bin/cloud-sdk-hetzner-smoke")
TOKEN_ENV = "CLOUD_SDK_HETZNER_TOKEN_FILE"
DESTRUCTIVE_ENV = "CLOUD_SDK_HETZNER_ALLOW_DESTRUCTIVE"
MAX_MANIFEST_BYTES = 512


class RunnerError(Exception):
    """Static, payload-free runtime validation failure."""


def fail(message: str) -> NoReturn:
    print(f"live smoke runner: {message}", file=sys.stderr)
    raise SystemExit(1)


def validate_root_owned_directories(path: Path) -> None:
    current = path.parent
    while True:
        metadata = os.lstat(current)
        mode = stat.S_IMODE(metadata.st_mode)
        if (
            not stat.S_ISDIR(metadata.st_mode)
            or metadata.st_uid != 0
            or metadata.st_gid != 0
            or mode & 0o022 != 0
        ):
            raise RunnerError("installation directory trust check failed")
        if current == current.parent:
            return
        current = current.parent


def open_root_owned_file(path: Path, expected_mode: int) -> int:
    validate_root_owned_directories(path)
    flags = os.O_RDONLY | os.O_CLOEXEC
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    descriptor = os.open(path, flags)
    metadata = os.fstat(descriptor)
    mode = stat.S_IMODE(metadata.st_mode)
    if (
        not stat.S_ISREG(metadata.st_mode)
        or metadata.st_uid != 0
        or metadata.st_gid != 0
        or metadata.st_nlink != 1
        or mode != expected_mode
    ):
        os.close(descriptor)
        raise RunnerError("installed file trust check failed")
    return descriptor


def read_manifest(descriptor: int) -> dict[str, str]:
    data = os.read(descriptor, MAX_MANIFEST_BYTES + 1)
    if len(data) > MAX_MANIFEST_BYTES:
        raise RunnerError("manifest validation failed")
    try:
        text = data.decode("ascii")
    except UnicodeDecodeError as error:
        raise RunnerError("manifest validation failed") from error
    fields: dict[str, str] = {}
    for line in text.splitlines():
        key, separator, value = line.partition("=")
        if not separator or not key or key in fields:
            raise RunnerError("manifest validation failed")
        fields[key] = value
    expected = {
        "format",
        "commit",
        "artifact_sha256",
        "runner_sha256",
        "launcher_sha256",
    }
    if set(fields) != expected or fields["format"] != "2":
        raise RunnerError("manifest validation failed")
    if len(fields["commit"]) not in {40, 64} or not is_lower_hex(fields["commit"]):
        raise RunnerError("manifest validation failed")
    for key in ("artifact_sha256", "runner_sha256", "launcher_sha256"):
        if len(fields[key]) != 64 or not is_lower_hex(fields[key]):
            raise RunnerError("manifest validation failed")
    return fields


def is_lower_hex(value: str) -> bool:
    return bool(value) and all(character in "0123456789abcdef" for character in value)


def sha256_descriptor(descriptor: int) -> str:
    os.lseek(descriptor, 0, os.SEEK_SET)
    digest = hashlib.sha256()
    while chunk := os.read(descriptor, 64 * 1024):
        digest.update(chunk)
    os.lseek(descriptor, 0, os.SEEK_SET)
    return digest.hexdigest()


def execute() -> NoReturn:
    token_file = os.environ.get(TOKEN_ENV, "")
    destructive = os.environ.get(DESTRUCTIVE_ENV, "")
    os.environ.clear()
    if not token_file:
        fail("token file is required")
    if destructive:
        fail("destructive opt-in is forbidden")
    if len(sys.argv) != 1:
        fail("arguments are forbidden")
    if os.execve not in os.supports_fd:
        fail("descriptor execution is unsupported")

    try:
        manifest_descriptor = open_root_owned_file(MANIFEST, 0o444)
        try:
            fields = read_manifest(manifest_descriptor)
        finally:
            os.close(manifest_descriptor)
        for path, mode, field in (
            (RUNNER, 0o444, "runner_sha256"),
            (LAUNCHER, 0o555, "launcher_sha256"),
        ):
            descriptor = open_root_owned_file(path, mode)
            try:
                if sha256_descriptor(descriptor) != fields[field]:
                    raise RunnerError("runtime verification failed")
            finally:
                os.close(descriptor)
        artifact_descriptor = open_root_owned_file(ARTIFACT, 0o555)
        if sha256_descriptor(artifact_descriptor) != fields["artifact_sha256"]:
            os.close(artifact_descriptor)
            raise RunnerError("artifact verification failed")
    except (OSError, RunnerError):
        fail("installed bundle verification failed")

    os.set_inheritable(artifact_descriptor, True)
    environment = {
        "PATH": "/usr/bin:/bin",
        "CLOUD_SDK_HETZNER_LIVE_MODE": "read-only",
        TOKEN_ENV: token_file,
    }
    arguments = [
        str(ARTIFACT),
        "read_only_catalog_smoke",
        "--exact",
        "--ignored",
        "--nocapture",
        "--test-threads=1",
    ]
    try:
        os.execve(artifact_descriptor, arguments, environment)
    except OSError:
        os.close(artifact_descriptor)
        fail("descriptor execution failed")


if __name__ == "__main__":
    execute()
