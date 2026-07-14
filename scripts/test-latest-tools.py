#!/usr/bin/env python3
"""Regression tests for the release-tool freshness gate."""

from __future__ import annotations

import os
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "scripts" / "check_latest_tools.sh"


def fake_environment(directory: Path, *, stale: bool = False) -> tuple[dict[str, str], Path]:
    binary = directory / "bin"
    binary.mkdir()
    log = directory / "cargo.log"
    rustc = binary / "rustc"
    rustc.write_text(
        "#!/bin/sh\nprintf '%s\\n' 'rustc 1.97.0 (test 2026-07-07)'\n",
        encoding="ascii",
    )
    rustc.chmod(0o755)
    cargo = binary / "cargo"
    cargo.write_text(
        """#!/bin/sh
set -eu
printf '%s\n' "$*" >> "$TOOL_TEST_LOG"
case "$*" in
'deny --version') echo 'cargo-deny 0.20.2' ;;
'audit --version') echo 'cargo-audit-audit 0.22.2' ;;
'sbom --version') echo 'cargo-sbom 0.10.0' ;;
'fuzz --version') echo 'cargo-fuzz 0.13.2' ;;
'search --registry crates-io --limit 1 cargo-deny')
    printf 'cargo-deny = "%s"\n' "${STALE_DENY_VERSION:-0.20.2}" ;;
'search --registry crates-io --limit 1 cargo-audit') echo 'cargo-audit = "0.22.2"' ;;
'search --registry crates-io --limit 1 cargo-sbom') echo 'cargo-sbom = "0.10.0"' ;;
'search --registry crates-io --limit 1 cargo-fuzz') echo 'cargo-fuzz = "0.13.2"' ;;
*) echo "unexpected cargo arguments: $*" >&2; exit 64 ;;
esac
""",
        encoding="ascii",
    )
    cargo.chmod(0o755)
    environment = os.environ.copy()
    environment["PATH"] = f"{binary}:{environment['PATH']}"
    environment["TOOL_TEST_LOG"] = str(log)
    if stale:
        environment["STALE_DENY_VERSION"] = "0.20.3"
    return environment, log


def run(mode: str, environment: dict[str, str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(CHECKER), mode],
        cwd=ROOT,
        env=environment,
        text=True,
        capture_output=True,
        check=False,
    )


def test_fetch_checks_exact_registry_commands() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        environment, log = fake_environment(Path(temporary))
        result = run("--fetch", environment)
        assert result.returncode == 0, result
        lines = log.read_text(encoding="ascii").splitlines()
        assert "search --registry crates-io --limit 1 cargo-deny" in lines
        assert "search --registry crates-io --limit 1 cargo-fuzz" in lines


def test_fetch_rejects_a_newer_release() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        environment, _ = fake_environment(Path(temporary), stale=True)
        result = run("--fetch", environment)
        assert result.returncode == 1, result
        assert "reports cargo-deny 0.20.3; expected 0.20.2" in result.stderr


def test_local_only_never_searches_the_registry() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        environment, log = fake_environment(Path(temporary), stale=True)
        result = run("--local-only", environment)
        assert result.returncode == 0, result
        assert "search " not in log.read_text(encoding="ascii")


def main() -> None:
    test_fetch_checks_exact_registry_commands()
    test_fetch_rejects_a_newer_release()
    test_local_only_never_searches_the_registry()
    print("3 release-tool freshness tests passed.")


if __name__ == "__main__":
    main()
