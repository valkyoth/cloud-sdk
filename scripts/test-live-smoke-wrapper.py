#!/usr/bin/env python3
"""Regression tests for credential-free live-smoke staging."""

from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parent.parent
WRAPPER = ROOT / "scripts" / "smoke_hetzner_live.sh"
SELECTOR = ROOT / "scripts" / "find-cargo-test-executable.py"
TOKEN_ENV = "CLOUD_SDK_HETZNER_TOKEN_FILE"


def run(
    command: list[str], cwd: Path, extra_env: dict[str, str] | None = None
) -> subprocess.CompletedProcess[str]:
    environment = os.environ.copy()
    environment.pop(TOKEN_ENV, None)
    environment.pop("CLOUD_SDK_HETZNER_ALLOW_DESTRUCTIVE", None)
    if extra_env:
        environment.update(extra_env)
    return subprocess.run(
        command,
        cwd=cwd,
        env=environment,
        check=False,
        capture_output=True,
        text=True,
    )


def assert_credential_rejected(mode: str) -> None:
    sentinel = "/private/credential-path-must-not-appear"
    result = run([str(WRAPPER), mode], ROOT, {TOKEN_ENV: sentinel})
    assert result.returncode == 2, result
    assert "must not be available during Cargo execution" in result.stderr
    assert sentinel not in result.stdout + result.stderr


def test_repository_anchor() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        directory = Path(temporary)
        fake_bin = directory / "bin"
        fake_bin.mkdir()
        cargo = fake_bin / "cargo"
        cargo.write_text("#!/bin/sh\n/usr/bin/pwd -P\n", encoding="ascii")
        cargo.chmod(0o755)
        result = run(
            [str(WRAPPER), "--check"],
            directory,
            {"PATH": f"{fake_bin}:/usr/bin:/bin"},
        )
        assert result.returncode == 0, result
        assert result.stdout.strip() == str(ROOT)


def selector_result(messages: list[dict[str, object] | str]) -> subprocess.CompletedProcess[str]:
    with tempfile.NamedTemporaryFile("w", encoding="utf-8") as stream:
        for message in messages:
            if isinstance(message, str):
                stream.write(message + "\n")
            else:
                stream.write(json.dumps(message) + "\n")
        stream.flush()
        return run([str(SELECTOR), stream.name, "live_smoke"], ROOT)


def test_selector() -> None:
    match = {
        "reason": "compiler-artifact",
        "target": {"name": "live_smoke", "kind": ["test"]},
        "executable": "/tmp/live-smoke-test",
    }
    selected = selector_result([match])
    assert selected.returncode == 0, selected
    assert selected.stdout.strip() == "/tmp/live-smoke-test"

    duplicate = dict(match)
    duplicate["executable"] = "/tmp/other-live-smoke-test"
    rejected = selector_result([match, duplicate])
    assert rejected.returncode != 0
    assert "expected one" in rejected.stderr

    malformed = selector_result(["not-json"])
    assert malformed.returncode != 0
    assert "invalid JSON" in malformed.stderr


def main() -> None:
    assert_credential_rejected("--check")
    assert_credential_rejected("--prepare")
    assert_credential_rejected("--read-only")
    test_repository_anchor()
    test_selector()
    print("7 live smoke staging tests passed.")


if __name__ == "__main__":
    main()
