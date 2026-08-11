#!/usr/bin/env python3
"""Regression tests for the provider identity release gate."""

from __future__ import annotations

import os
from pathlib import Path
import shutil
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parent.parent
CHECKER = ROOT / "scripts" / "check_provider_identities.sh"
IDENTITIES = (
    "HETZNER_PROVIDER_ID",
    "CLOUD_SERVICE_ID",
    "DNS_SERVICE_ID",
    "ROBOT_SERVICE_ID",
    "SECURITY_SERVICE_ID",
    "STORAGE_SERVICE_ID",
)


def fixture(directory: Path) -> tuple[dict[str, str], Path, Path]:
    core = directory / "crates" / "cloud-sdk" / "src"
    hetzner = directory / "crates" / "cloud-sdk-hetzner" / "src"
    core.mkdir(parents=True)
    hetzner.mkdir(parents=True)
    (core / "lib.rs").write_text(
        "pub struct ProviderId;\n"
        "pub enum ProviderLinkExecutionError<E> { Transport(E) }\n",
        encoding="ascii",
    )
    identity = hetzner / "identity.rs"
    identity.write_text(
        "".join(f"pub const {name}: &str = \"test\";\n" for name in IDENTITIES),
        encoding="ascii",
    )

    fake_bin = directory / "bin"
    fake_bin.mkdir()
    for command in ("grep", "sh"):
        source = shutil.which(command)
        assert source is not None, command
        (fake_bin / command).symlink_to(source)

    cargo_log = directory / "cargo.log"
    cargo = fake_bin / "cargo"
    cargo.write_text(
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$IDENTITY_TEST_LOG\"\n",
        encoding="ascii",
    )
    cargo.chmod(0o755)

    environment = os.environ.copy()
    environment.update(
        {
            "IDENTITY_TEST_LOG": str(cargo_log),
            "PATH": str(fake_bin),
        }
    )
    return environment, cargo_log, identity


def run(directory: Path, environment: dict[str, str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(CHECKER)],
        cwd=directory,
        env=environment,
        check=False,
        capture_output=True,
        text=True,
    )


def test_minimal_path() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        directory = Path(temporary)
        environment, cargo_log, _ = fixture(directory)
        result = run(directory, environment)
        assert result.returncode == 0, result
        assert cargo_log.read_text(encoding="ascii").splitlines() == [
            "test -p cloud-sdk --test provider_extensibility --all-features",
            "test -p cloud-sdk --doc --all-features",
        ]


def test_closed_taxonomy_and_missing_identity() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        directory = Path(temporary)
        environment, cargo_log, identity = fixture(directory)
        core = directory / "crates" / "cloud-sdk" / "src" / "lib.rs"
        core.write_text("pub enum Provider {}\n", encoding="ascii")
        closed = run(directory, environment)
        assert closed.returncode == 1, closed
        assert "closed core provider taxonomy" in closed.stderr
        assert not cargo_log.exists()

        core.write_text("pub struct ProviderId;\n", encoding="ascii")
        identity.write_text(
            identity.read_text(encoding="ascii").replace(
                "pub const DNS_SERVICE_ID:",
                "const DNS_SERVICE_ID:",
            ),
            encoding="ascii",
        )
        missing = run(directory, environment)
        assert missing.returncode == 1, missing
        assert "missing Hetzner identity DNS_SERVICE_ID" in missing.stderr


def main() -> None:
    test_minimal_path()
    test_closed_taxonomy_and_missing_identity()
    print("2 provider identity regression groups passed.")


if __name__ == "__main__":
    main()
