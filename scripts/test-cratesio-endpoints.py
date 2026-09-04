#!/usr/bin/env python3
"""Regression tests for the crates.io endpoint source lock."""

from __future__ import annotations

import importlib.util
import json
import shutil
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/check_cratesio_endpoints.py"


def load_checker():
    spec = importlib.util.spec_from_file_location("cratesio_endpoints", SCRIPT)
    if spec is None or spec.loader is None:
        raise RuntimeError("could not load crates.io endpoint checker")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


checker = load_checker()


def fixture() -> Path:
    root = Path(tempfile.mkdtemp())
    lock = root / checker.LOCK
    lock.parent.mkdir(parents=True)
    shutil.copy2(ROOT / checker.LOCK, lock)
    authority = root / checker.AUTHORITY
    authority.parent.mkdir(parents=True)
    shutil.copy2(ROOT / checker.AUTHORITY, authority)
    return root


def assert_rejected(root: Path, expected: str) -> None:
    try:
        checker.validate(root)
    except checker.SourceLockError as error:
        assert expected in str(error), error
        return
    raise AssertionError("expected endpoint lock rejection")


def test_repository_lock() -> None:
    checker.validate(ROOT)
    release_gate = (ROOT / "scripts/release_1_1_gate.sh").read_text(encoding="ascii")
    assert "scripts/check_cratesio_endpoints.py --fetch" in release_gate


def test_manifest_and_constants_are_exact() -> None:
    root = fixture()
    document = json.loads((root / checker.LOCK).read_text(encoding="ascii"))
    document["sources"][0]["url"] = "https://evil.example/config.json"
    (root / checker.LOCK).write_text(json.dumps(document), encoding="ascii")
    assert_rejected(root, "source 'index-config' changed")
    shutil.rmtree(root)

    root = fixture()
    authority = root / checker.AUTHORITY
    authority.write_text(
        authority.read_text(encoding="ascii").replace(
            '"https://static.crates.io"', '"https://static.crates.io.evil.example"'
        ),
        encoding="ascii",
    )
    assert_rejected(root, "STATIC_DOWNLOAD")
    shutil.rmtree(root)


def test_live_payload_semantics_fail_closed() -> None:
    payloads = {
        "index-config": (
            b'{"api":"https://crates.io","dl":"https://static.crates.io/crates"}'
        ),
        "staging-source": (
            b"https://staging.crates.io pnpm dev:staging"
        ),
    }

    def fetch(source):
        return payloads[source["id"]]

    checker.validate(ROOT, live=True, fetcher=fetch)
    payloads["index-config"] = (
        b'{"api":"https://crates.io","dl":"https://evil.example/crates"}'
    )
    try:
        checker.validate(ROOT, live=True, fetcher=fetch)
    except checker.SourceLockError as error:
        assert "configuration changed" in str(error), error
    else:
        raise AssertionError("changed live download authority was accepted")


def main() -> None:
    tests = (
        test_repository_lock,
        test_manifest_and_constants_are_exact,
        test_live_payload_semantics_fail_closed,
    )
    for test in tests:
        test()
    print(f"{len(tests)} crates.io endpoint source-lock groups passed.")


if __name__ == "__main__":
    main()
