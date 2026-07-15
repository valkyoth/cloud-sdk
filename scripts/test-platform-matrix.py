#!/usr/bin/env python3
"""Regression tests for the explicit platform-matrix command boundary."""

from __future__ import annotations

import os
from pathlib import Path
import shutil
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parent.parent
CHECKER = ROOT / "scripts" / "check_platform_matrix.sh"


def fake_environment(
    directory: Path,
    *,
    installed: str = "x86_64-unknown-linux-gnu",
    tree: str = "cloud-sdk v0.29.0\ncloud-sdk-hetzner v0.22.1\n",
) -> tuple[dict[str, str], Path]:
    fake_bin = directory / "bin"
    fake_bin.mkdir()
    log = directory / "cargo.log"
    cargo = fake_bin / "cargo"
    cargo.write_text(
        "#!/bin/sh\n"
        "printf '%s\\n' \"$*\" >> \"$PLATFORM_TEST_LOG\"\n"
        "if [ \"${1:-}\" = tree ]; then printf '%s\\n' \"$PLATFORM_TEST_TREE\"; fi\n",
        encoding="ascii",
    )
    cargo.chmod(0o755)
    rustup = fake_bin / "rustup"
    rustup.write_text(
        "#!/bin/sh\n"
        "if [ \"$*\" = 'target list --installed' ]; then\n"
        "    printf '%s\\n' \"$PLATFORM_TEST_INSTALLED\"\n"
        "    exit 0\n"
        "fi\n"
        "exit 2\n",
        encoding="ascii",
    )
    rustup.chmod(0o755)
    environment = os.environ.copy()
    environment.update(
        {
            "PATH": f"{fake_bin}:/usr/bin:/bin",
            "PLATFORM_TEST_INSTALLED": installed,
            "PLATFORM_TEST_LOG": str(log),
            "PLATFORM_TEST_TREE": tree,
        }
    )
    return environment, log


def run(
    arguments: list[str],
    environment: dict[str, str],
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(CHECKER), *arguments],
        cwd=ROOT,
        env=environment,
        check=False,
        capture_output=True,
        text=True,
    )


def test_portable_target() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        environment, log = fake_environment(Path(temporary))
        result = run(["--portable", "x86_64-unknown-linux-gnu"], environment)
        assert result.returncode == 0, result
        commands = log.read_text(encoding="ascii").splitlines()
        assert len(commands) == 2, commands
        assert all("--locked --target x86_64-unknown-linux-gnu" in item for item in commands)
        assert "--no-default-features" in commands[0]
        assert "cloud-sdk-hetzner/serde" in commands[1]


def test_rejected_and_missing_targets() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        environment, log = fake_environment(Path(temporary))
        rejected = run(["--portable", "attacker-controlled-target"], environment)
        assert rejected.returncode == 2, rejected
        assert "unsupported portable target" in rejected.stderr
        assert not log.exists()

        missing = run(["--portable", "aarch64-apple-ios"], environment)
        assert missing.returncode == 2, missing
        assert "target is not installed" in missing.stderr
        assert not log.exists()


def test_rustup_failures() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        directory = Path(temporary)
        environment, log = fake_environment(directory)
        isolated_bin = directory / "isolated-bin"
        isolated_bin.mkdir()
        for command in ("sh", "dirname", "grep"):
            source = shutil.which(command)
            assert source is not None, command
            (isolated_bin / command).symlink_to(source)
        environment["PATH"] = str(isolated_bin)
        missing = run(["--portable", "x86_64-unknown-linux-gnu"], environment)
        assert missing.returncode == 2, missing
        assert "rustup not found on PATH" in missing.stderr
        assert not log.exists()

    with tempfile.TemporaryDirectory() as temporary:
        directory = Path(temporary)
        environment, log = fake_environment(directory)
        rustup = directory / "bin" / "rustup"
        rustup.write_text(
            "#!/bin/sh\nprintf '%s\\n' x86_64-unknown-linux-gnu\nexit 1\n",
            encoding="ascii",
        )
        broken = run(["--portable", "x86_64-unknown-linux-gnu"], environment)
        assert broken.returncode == 2, broken
        assert "could not list installed targets" in broken.stderr
        assert not log.exists()


def test_native_mode() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        environment, log = fake_environment(Path(temporary))
        result = run(["--native"], environment)
        assert result.returncode == 0, result
        assert log.read_text(encoding="ascii").splitlines() == [
            "check --locked --all-targets --all-features -p cloud-sdk "
            "-p cloud-sdk-hetzner -p cloud-sdk-sanitization -p cloud-sdk-testkit",
            "check --locked --all-targets --no-default-features "
            "-p cloud-sdk-reqwest "
            "--features std,blocking-rustls,blocking-rustls-webpki-roots,async-rustls",
        ]


def test_default_dependency_boundary() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        directory = Path(temporary)
        environment, log = fake_environment(directory)
        accepted = run(["--default-boundary"], environment)
        assert accepted.returncode == 0, accepted
        assert log.read_text(encoding="ascii").splitlines() == [
            "tree --locked --workspace --target all --edges normal --prefix none"
        ]

    with tempfile.TemporaryDirectory() as temporary:
        directory = Path(temporary)
        environment, log = fake_environment(
            directory,
            tree=(
                "cloud-sdk v0.29.0\n"
                "sanitization v1.2.4\n"
                "ureq v3.1.2\n"
                "curl v0.4.49\n"
                "async-std v1.13.2"
            ),
        )
        rejected = run(["--default-boundary"], environment)
        assert rejected.returncode == 1, rejected
        assert "unexpected default dependency" in rejected.stderr
        assert "ureq v3.1.2" in rejected.stderr
        assert "curl v0.4.49" in rejected.stderr
        assert "async-std v1.13.2" in rejected.stderr
        assert log.read_text(encoding="ascii").splitlines() == [
            "tree --locked --workspace --target all --edges normal --prefix none"
        ]


def test_argument_validation() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        environment, log = fake_environment(Path(temporary))
        for arguments in ([], ["--native", "extra"], ["--portable"]):
            result = run(list(arguments), environment)
            assert result.returncode == 2, (arguments, result)
        assert not log.exists()


def main() -> None:
    test_portable_target()
    test_rejected_and_missing_targets()
    test_rustup_failures()
    test_native_mode()
    test_default_dependency_boundary()
    test_argument_validation()
    print("6 platform matrix regression groups passed.")


if __name__ == "__main__":
    main()
