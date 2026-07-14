#!/usr/bin/env python3
"""Regression tests for the bundled AWS-LC build policy."""

from __future__ import annotations

import os
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
POLICY = ROOT / "scripts" / "enforce_bundled_aws_lc.sh"
PROTECTED_SCRIPTS = (
    "checks.sh",
    "release_0_24_gate.sh",
    "check_packaged_reqwest_tests.sh",
    "check_platform_matrix.sh",
    "check_reqwest_boundary.sh",
    "check_reqwest_fips_boundary.sh",
    "check_reqwest_webpki_roots_boundary.sh",
    "check_rust_version_matrix.sh",
    "check_serde_boundary.sh",
    "smoke_hetzner_live.sh",
)


def clean_environment() -> dict[str, str]:
    return {
        name: value
        for name, value in os.environ.items()
        if not name.startswith("AWS_LC_SYS_USE_SYSTEM")
        and not name.startswith("AWS_LC_FIPS_SYS_USE_SYSTEM")
    }


def run_policy(environment: dict[str, str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            "sh",
            "-c",
            '. "$1"; printf "%s\\n%s\\n" '
            '"$AWS_LC_SYS_USE_SYSTEM" "$AWS_LC_FIPS_SYS_USE_SYSTEM"',
            "sh",
            str(POLICY),
        ],
        cwd=ROOT,
        env=environment,
        text=True,
        capture_output=True,
        check=False,
    )


def test_generic_controls_are_forced_off() -> None:
    environment = clean_environment()
    environment["AWS_LC_SYS_USE_SYSTEM"] = "1"
    environment["AWS_LC_FIPS_SYS_USE_SYSTEM"] = "yes"
    result = run_policy(environment)
    assert result.returncode == 0, result
    assert result.stdout == "0\n0\n", result


def test_target_specific_controls_are_rejected() -> None:
    for name in (
        "AWS_LC_SYS_USE_SYSTEM_x86_64_unknown_linux_gnu",
        "AWS_LC_FIPS_SYS_USE_SYSTEM_AARCH64_APPLE_DARWIN",
    ):
        environment = clean_environment()
        environment[name] = "0"
        result = run_policy(environment)
        assert result.returncode == 1, (name, result)
        assert f"forbidden target-specific override: {name}" in result.stderr
        assert result.stdout == ""


def test_every_native_build_entry_point_sources_the_policy() -> None:
    marker = ". scripts/enforce_bundled_aws_lc.sh"
    for relative in PROTECTED_SCRIPTS:
        source = (ROOT / "scripts" / relative).read_text(encoding="ascii")
        assert marker in source, relative
        cargo_position = source.find("cargo ")
        assert cargo_position == -1 or source.index(marker) < cargo_position, relative


def main() -> None:
    test_generic_controls_are_forced_off()
    test_target_specific_controls_are_rejected()
    test_every_native_build_entry_point_sources_the_policy()
    print("3 bundled AWS-LC build-policy regression groups passed.")


if __name__ == "__main__":
    main()
