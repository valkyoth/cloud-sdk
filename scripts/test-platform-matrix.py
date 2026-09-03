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
    reject_transport: bool = False,
    tree: str = (
        "cloud-sdk v0.48.0\n"
        "cloud-sdk-hetzner v0.36.2\n"
        "ovhcloud-v2-probe v0.61.0\n"
        "subtle v2.6.1\n"
    ),
) -> tuple[dict[str, str], Path]:
    fake_bin = directory / "bin"
    fake_bin.mkdir()
    log = directory / "cargo.log"
    cargo = fake_bin / "cargo"
    cargo.write_text(
        "#!/bin/sh\n"
        "printf '%s\\n' \"$*\" >> \"$PLATFORM_TEST_LOG\"\n"
        "if [ \"${1:-}\" = tree ]; then printf '%s\\n' \"$PLATFORM_TEST_TREE\"; fi\n"
        "if [ \"${PLATFORM_TEST_REJECT_TRANSPORT:-0}\" = 1 ]; then\n"
        "    case \"$*\" in\n"
        "    *'-p cloud-sdk-reqwest --features '*)\n"
        "        printf '%s\\n' 'cloud-sdk-reqwest transport features are unsupported on this target' >&2\n"
        "        exit 1\n"
        "        ;;\n"
        "    esac\n"
        "fi\n",
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
            "PLATFORM_TEST_REJECT_TRANSPORT": "1" if reject_transport else "0",
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
        assert len(commands) == 8, commands
        assert all("--locked --target x86_64-unknown-linux-gnu" in item for item in commands)
        assert "--no-default-features" in commands[0]
        assert commands[1].endswith("-p cloud-sdk --features alloc")
        assert commands[2].endswith("-p cloud-sdk-cratesio --features alloc")
        assert commands[3].endswith("-p cloud-sdk-cratesio --features serde")
        assert commands[4].endswith("-p cloud-sdk-sanitization --features alloc")
        assert commands[5].endswith("-p cloud-sdk-hetzner --features alloc")
        assert commands[6].endswith("-p cloud-sdk-hetzner --features serde")
        assert commands[7].endswith("-p cloud-sdk-testkit --features alloc")


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
            "-p cloud-sdk-cratesio -p cloud-sdk-hetzner "
            "-p cloud-sdk-sanitization -p cloud-sdk-testkit",
            "check --locked --all-targets --no-default-features -p cloud-sdk-reqwest",
            "check --locked --all-targets --no-default-features "
            "-p cloud-sdk-reqwest --features std",
            "check --locked --all-targets --no-default-features "
            "-p cloud-sdk-reqwest --features blocking-rustls",
            "test --locked --no-default-features "
            "-p cloud-sdk-reqwest --features blocking-rustls",
            "check --locked --all-targets --no-default-features "
            "-p cloud-sdk-reqwest --features blocking-rustls-webpki-roots",
            "test --locked --no-default-features "
            "-p cloud-sdk-reqwest --features blocking-rustls-webpki-roots",
            "check --locked --all-targets --no-default-features "
            "-p cloud-sdk-reqwest --features async-rustls",
            "test --locked --no-default-features "
            "-p cloud-sdk-reqwest --features async-rustls",
            "check --locked --all-targets --no-default-features "
            "-p cloud-sdk-reqwest "
            "--features blocking-rustls,blocking-rustls-webpki-roots,async-rustls",
            "test --locked --all-features -p cloud-sdk-reqwest",
            "test --locked -p cloud-sdk-hetzner --test live_smoke --all-features",
        ]


def test_transport_has_an_explicit_unsupported_target_diagnostic() -> None:
    source = (ROOT / "crates/cloud-sdk-reqwest/src/lib.rs").read_text(
        encoding="ascii"
    )
    assert "cloud-sdk-reqwest transport features are unsupported" in source
    for supported in ('target_os = "linux"', 'target_os = "windows"',
                      'target_os = "macos"', 'target_os = "freebsd"'):
        assert supported in source
    manifest = (ROOT / "crates/cloud-sdk-reqwest/Cargo.toml").read_text(
        encoding="ascii"
    )
    target_dependencies = (
        '[target.\'cfg(any(target_os = "freebsd", target_os = "linux", '
        'target_os = "macos", target_os = "windows"))\'.dependencies]'
    )
    assert target_dependencies in manifest
    checker = CHECKER.read_text(encoding="ascii")
    for unsupported in (
        "aarch64-linux-android",
        "aarch64-apple-ios",
        "wasm32-unknown-unknown",
        "thumbv7em-none-eabihf",
    ):
        assert unsupported in checker
    assert "unsupported $feature compiled" in checker
    assert "missing $feature diagnostic" in checker


def test_every_transport_feature_fails_on_unsupported_targets() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        environment, log = fake_environment(
            Path(temporary),
            installed="aarch64-linux-android",
            reject_transport=True,
        )
        result = run(["--portable", "aarch64-linux-android"], environment)
        assert result.returncode == 0, result
        commands = log.read_text(encoding="ascii").splitlines()
        assert len(commands) == 11, commands
        assert commands[-3:] == [
            "check --locked --target aarch64-linux-android "
            "--no-default-features -p cloud-sdk-reqwest --features blocking-rustls",
            "check --locked --target aarch64-linux-android "
            "--no-default-features -p cloud-sdk-reqwest "
            "--features blocking-rustls-webpki-roots",
            "check --locked --target aarch64-linux-android "
            "--no-default-features -p cloud-sdk-reqwest --features async-rustls",
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
                "cloud-sdk v0.43.0\n"
                "sanitization v2.0.3\n"
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
    test_transport_has_an_explicit_unsupported_target_diagnostic()
    test_every_transport_feature_fails_on_unsupported_targets()
    test_default_dependency_boundary()
    test_argument_validation()
    print("8 platform matrix regression groups passed.")


if __name__ == "__main__":
    main()
