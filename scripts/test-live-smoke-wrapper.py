#!/usr/bin/env python3
"""Regression tests for the two-phase live-smoke wrapper."""

from __future__ import annotations

import hashlib
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


def initialize_repository(path: Path) -> str:
    result = run(["git", "init", "-q"], path)
    assert result.returncode == 0, result
    (path / "reviewed").write_text("reviewed\n", encoding="utf-8")
    (path / ".gitignore").write_text("/target/\n/read-only-token\n", encoding="ascii")
    assert run(["git", "add", "reviewed", ".gitignore"], path).returncode == 0
    commit = run(
        [
            "git",
            "-c",
            "user.name=cloud-sdk test",
            "-c",
            "user.email=test.invalid@example.invalid",
            "commit",
            "-q",
            "-m",
            "test",
        ],
        path,
    )
    assert commit.returncode == 0, commit
    head = run(["git", "rev-parse", "HEAD"], path)
    assert head.returncode == 0, head
    return head.stdout.strip()


def write_sealed_fixture(path: Path, commit: str) -> None:
    artifact_dir = path / "target" / "cloud-sdk-live-smoke"
    artifact_dir.mkdir(parents=True, exist_ok=True)
    artifact = artifact_dir / "live_smoke"
    if artifact.exists():
        artifact.chmod(0o755)
    artifact.write_text(
        "#!/bin/sh\n"
        "test \"${CLOUD_SDK_HETZNER_LIVE_MODE:-}\" = read-only || exit 20\n"
        "test -n \"${CLOUD_SDK_HETZNER_TOKEN_FILE:-}\" || exit 21\n"
        "test -z \"${HOME:-}\" || exit 22\n"
        "test \"${PATH:-}\" = /usr/bin:/bin || exit 23\n"
        "test \"$#\" -eq 5 || exit 24\n"
        "printf 'direct execution passed\\n'\n",
        encoding="utf-8",
    )
    artifact.chmod(0o555)
    digest = hashlib.sha256(artifact.read_bytes()).hexdigest()
    manifest = artifact_dir / "live_smoke.manifest"
    if manifest.exists():
        manifest.chmod(0o644)
    manifest.write_text(
        f"format=1\ncommit={commit}\nsha256={digest}\n", encoding="ascii"
    )
    manifest.chmod(0o444)


def test_direct_execution() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        directory = Path(temporary)
        commit = initialize_repository(directory)
        token = directory / "read-only-token"
        token.write_text("not-used-by-fixture\n", encoding="ascii")

        missing = run([str(WRAPPER), "--read-only"], directory, {TOKEN_ENV: str(token)})
        assert missing.returncode == 1, missing
        assert "sealed executable unavailable" in missing.stderr
        assert str(token) not in missing.stdout + missing.stderr

        write_sealed_fixture(directory, commit)
        result = run([str(WRAPPER), "--read-only"], directory, {TOKEN_ENV: str(token)})
        assert result.returncode == 0, result
        assert result.stdout == "direct execution passed\n"
        assert str(token) not in result.stdout + result.stderr

        artifact = directory / "target" / "cloud-sdk-live-smoke" / "live_smoke"
        artifact.chmod(0o755)
        artifact.write_text(artifact.read_text(encoding="utf-8") + "# tampered\n")
        artifact.chmod(0o555)
        tampered = run([str(WRAPPER), "--read-only"], directory, {TOKEN_ENV: str(token)})
        assert tampered.returncode == 1, tampered
        assert "sealed executable verification failed" in tampered.stderr

        write_sealed_fixture(directory, commit)
        (directory / "reviewed").write_text("new commit\n", encoding="utf-8")
        assert run(["git", "add", "reviewed"], directory).returncode == 0
        next_commit = run(
            [
                "git",
                "-c",
                "user.name=cloud-sdk test",
                "-c",
                "user.email=test.invalid@example.invalid",
                "commit",
                "-q",
                "-m",
                "next",
            ],
            directory,
        )
        assert next_commit.returncode == 0, next_commit
        stale = run([str(WRAPPER), "--read-only"], directory, {TOKEN_ENV: str(token)})
        assert stale.returncode == 1, stale
        assert "sealed executable verification failed" in stale.stderr


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
    test_direct_execution()
    test_selector()
    print("9 live smoke wrapper tests passed.")


if __name__ == "__main__":
    main()
