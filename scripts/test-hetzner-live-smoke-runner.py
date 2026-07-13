#!/usr/bin/env python3
"""Regression tests for the root-owned live-smoke runtime."""

from __future__ import annotations

import importlib.util
import os
from pathlib import Path
import shutil
import stat
import tempfile


ROOT = Path(__file__).resolve().parent.parent
RUNNER = ROOT / "scripts" / "hetzner-live-smoke-runner.py"


def load_runner():
    specification = importlib.util.spec_from_file_location("live_smoke_runner", RUNNER)
    assert specification is not None and specification.loader is not None
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def descriptor_for(data: bytes) -> int:
    descriptor, name = tempfile.mkstemp()
    os.unlink(name)
    os.write(descriptor, data)
    os.lseek(descriptor, 0, os.SEEK_SET)
    return descriptor


def valid_manifest() -> bytes:
    return (
        b"format=2\n"
        + b"commit="
        + b"a" * 40
        + b"\nartifact_sha256="
        + b"b" * 64
        + b"\nrunner_sha256="
        + b"c" * 64
        + b"\nlauncher_sha256="
        + b"d" * 64
        + b"\n"
    )


def test_manifest(runner) -> None:
    descriptor = descriptor_for(valid_manifest())
    try:
        fields = runner.read_manifest(descriptor)
    finally:
        os.close(descriptor)
    assert fields["format"] == "2"
    assert fields["commit"] == "a" * 40

    for invalid in (
        valid_manifest() + b"extra=value\n",
        valid_manifest().replace(b"format=2", b"format=1"),
        valid_manifest().replace(b"commit=" + b"a" * 40, b"commit=UPPER"),
        b"x" * 513,
    ):
        descriptor = descriptor_for(invalid)
        try:
            try:
                runner.read_manifest(descriptor)
            except runner.RunnerError:
                pass
            else:
                raise AssertionError("invalid manifest was accepted")
        finally:
            os.close(descriptor)


def test_ownership(runner) -> None:
    with tempfile.TemporaryDirectory() as temporary:
        artifact = Path(temporary) / "artifact"
        artifact.write_bytes(b"not trusted")
        artifact.chmod(0o555)
        try:
            runner.open_root_owned_file(artifact, 0o555)
        except runner.RunnerError:
            pass
        else:
            raise AssertionError("user-owned artifact was accepted")

    trusted = Path("/usr/bin/true")
    metadata = trusted.stat()
    try:
        descriptor = runner.open_root_owned_file(trusted, stat.S_IMODE(metadata.st_mode))
    except runner.RunnerError:
        assert metadata.st_uid != 0
    else:
        assert metadata.st_uid == 0
        os.close(descriptor)


def test_descriptor_hash(runner) -> None:
    descriptor = descriptor_for(b"cloud-sdk descriptor test")
    try:
        first = runner.sha256_descriptor(descriptor)
        second = runner.sha256_descriptor(descriptor)
    finally:
        os.close(descriptor)
    assert first == second
    assert len(first) == 64
    assert os.execve in os.supports_fd


def test_descriptor_execution_ignores_path_replacement(runner) -> None:
    with tempfile.TemporaryDirectory() as temporary:
        directory = Path(temporary)
        executable = directory / "selected"
        replaced = directory / "selected-before-replacement"
        shutil.copyfile("/usr/bin/true", executable)
        executable.chmod(0o755)
        descriptor = os.open(executable, os.O_RDONLY)
        runner.sha256_descriptor(descriptor)
        executable.rename(replaced)
        shutil.copyfile("/usr/bin/false", executable)
        executable.chmod(0o755)

        child = os.fork()
        if child == 0:
            runner.execute_descriptor(descriptor, "/unused-test-token")
        os.close(descriptor)
        _, status = os.waitpid(child, 0)
        assert os.waitstatus_to_exitcode(status) == 0


def main() -> None:
    runner = load_runner()
    test_manifest(runner)
    test_ownership(runner)
    test_descriptor_hash(runner)
    test_descriptor_execution_ignores_path_replacement(runner)
    print("9 live smoke runner tests passed.")


if __name__ == "__main__":
    main()
