#!/usr/bin/env python3
"""Regression tests for the Hetzner zero-fallback wire migration gate."""

from __future__ import annotations

import shutil
import subprocess
import tempfile
from pathlib import Path

import check_hetzner_wire_migration as checker

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "check_hetzner_wire_migration.py"


def stage(destination: Path) -> None:
    """Copy only source-locked evidence consumed by the checker."""
    matrix = destination / "docs" / "API_MATRIX.md"
    matrix.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(ROOT / "docs" / "API_MATRIX.md", matrix)
    for relative in checker.FILES.values():
        target = destination / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(ROOT / relative, target)


def run(root: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(SCRIPT), "--root", str(root)],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )


def replace(root: Path, name: str, old: str, new: str) -> None:
    path = root / checker.FILES[name]
    source = path.read_text(encoding="utf-8")
    if old not in source:
        raise AssertionError(f"test mutation is stale: {name}: {old}")
    path.write_text(source.replace(old, new, 1), encoding="utf-8")


def replace_all(root: Path, name: str, old: str, new: str) -> None:
    path = root / checker.FILES[name]
    source = path.read_text(encoding="utf-8")
    if old not in source:
        raise AssertionError(f"test mutation is stale: {name}: {old}")
    path.write_text(source.replace(old, new), encoding="utf-8")


def main() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary)
        stage(root)
        complete = run(root)
        assert complete.returncode == 0, complete
        assert "208 active operations" in complete.stdout

        mutations = [
            (
                "core_prepared",
                "T: BlockingAuthenticatedTransport + BoundTransport",
                "T: BlockingTransport + BoundTransport",
                "compatibility fallback",
            ),
            (
                "core_prepared_construction",
                'raw_response_policy.admits_header("x-request-id")',
                'raw_response_policy.admits_header("content-type")',
                "missing required wire control",
            ),
            (
                "provider_operation",
                "provider_service(endpoint.endpoint_group())",
                "provider_service_from_base(endpoint.api_base_url())",
                "missing required wire control",
            ),
            (
                "provider_policy",
                "ApiSurface::Dns => ProviderService::from_marker::<DnsService>",
                "ApiSurface::Dns => ProviderService::from_marker::<CloudService>",
                "missing required wire control",
            ),
            (
                "provider_policy",
                '"ratelimit-reset"',
                '"x-ratelimit-reset"',
                "missing required wire control",
            ),
            (
                "blocking_client",
                "client: RawBlockingClient",
                "client: reqwest::blocking::Client",
                "compatibility fallback",
            ),
            (
                "async_client",
                "authenticated.response_policy()",
                "legacy_response_limit()",
                "missing required wire control",
            ),
            (
                "raw_hyper",
                "headers.insert(AUTHORIZATION, authorization)",
                "drop(authorization)",
                "missing required wire control",
            ),
            (
                "raw_hyper",
                "if let Some(error) = state.informational_rejection()",
                "if let Some(error) = None",
                "missing required wire control",
            ),
            (
                "live_smoke",
                ".execute_blocking(",
                ".execute_unchecked(",
                "missing required wire control",
            ),
        ]
        for name, old, new, diagnostic in mutations:
            stage(root)
            replace(root, name, old, new)
            result = run(root)
            assert result.returncode == 1, (name, result)
            assert diagnostic in result.stderr, (name, result.stderr)

        stage(root)
        replace_all(
            root,
            "core_prepared",
            "T: BlockingAuthenticatedTransport + BoundTransport",
            "T: ReplacementTransport + BoundTransport",
        )
        missing_authenticated = run(root)
        assert missing_authenticated.returncode == 1, missing_authenticated
        assert "missing required wire control" in missing_authenticated.stderr

        stage(root)
        operation = root / checker.FILES["provider_operation"]
        source = operation.read_text(encoding="utf-8")
        operation.write_text(source + "\nfn decoy() { PreparedRequest::new(); }\n")
        duplicate = run(root)
        assert duplicate.returncode == 1, duplicate
        assert "exactly one wire assembly point" in duplicate.stderr

    print("12 Hetzner wire-migration regression groups passed.")


if __name__ == "__main__":
    main()
