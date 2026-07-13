#!/usr/bin/env python3
"""Regression tests for Hetzner spec integrity and download bounds."""

from __future__ import annotations

import argparse
import importlib.util
import io
import os
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "check_hetzner_api_drift.py"


def load_checker():
    spec = importlib.util.spec_from_file_location("check_hetzner_api_drift", SCRIPT)
    if spec is None or spec.loader is None:
        raise RuntimeError("could not load check_hetzner_api_drift.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


checker = load_checker()


class FetchResponse:
    def __init__(self, url: str) -> None:
        self.url = url

    def geturl(self) -> str:
        return self.url


def assert_exits(expected: str, function, *args, **kwargs) -> None:
    try:
        function(*args, **kwargs)
    except SystemExit as error:
        if expected not in str(error):
            raise AssertionError(f"expected {expected!r} in {error!r}") from error
        return
    raise AssertionError("expected SystemExit")


def test_bounded_reader_accepts_exact_limit() -> None:
    result = checker.read_bounded_response(
        io.BytesIO(b"12345678"), "cloud", max_bytes=8
    )
    if result != b"12345678":
        raise AssertionError(f"unexpected bounded read: {result!r}")


def test_bounded_reader_rejects_oversize_response() -> None:
    assert_exits(
        "exceeds 8 bytes",
        checker.read_bounded_response,
        io.BytesIO(b"123456789"),
        "cloud",
        max_bytes=8,
    )


def test_bounded_reader_rejects_total_timeout() -> None:
    ticks = iter((0, 61))
    assert_exits(
        "exceeded 60 seconds",
        checker.read_bounded_response,
        io.BytesIO(b"{}"),
        "cloud",
        monotonic=lambda: next(ticks),
    )


def test_fetch_response_requires_exact_https_url() -> None:
    expected = checker.SPECS["cloud"]
    checker.validate_fetch_response(FetchResponse(expected), expected, "cloud")
    assert_exits(
        "non-HTTPS URL",
        checker.validate_fetch_response,
        FetchResponse("http://docs.hetzner.cloud/cloud.spec.json"),
        expected,
        "cloud",
    )
    assert_exits(
        "redirected away",
        checker.validate_fetch_response,
        FetchResponse("https://example.invalid/cloud.spec.json"),
        expected,
        "cloud",
    )


def test_redirect_handler_refuses_followup_requests() -> None:
    request = checker.urllib.request.Request(checker.SPECS["cloud"])
    redirected = checker.RejectRedirects().redirect_request(
        request,
        None,
        302,
        "Found",
        {},
        "https://example.invalid/cloud.spec.json",
    )
    if redirected is not None:
        raise AssertionError("redirect handler created a follow-up request")


def test_load_specs_authenticates_before_parsing() -> None:
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        cloud = root / "cloud.json"
        hetzner = root / "hetzner.json"
        cloud.write_bytes(b"not-json")
        hetzner.write_bytes(b"not-json")
        args = argparse.Namespace(
            fetch=False,
            current_cloud=str(cloud),
            current_hetzner=str(hetzner),
            write_lock=False,
        )
        assert_exits("cloud spec SHA-256 mismatch", checker.load_specs, args)


def test_local_reader_rejects_oversize_before_reading() -> None:
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "oversize.json"
        path.write_bytes(b"123456789")
        assert_exits(
            "exceeds 8 bytes",
            checker.read_bounded_file,
            "cloud",
            path,
            max_bytes=8,
        )


def test_local_reader_rejects_symlink() -> None:
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        target = root / "target.json"
        link = root / "link.json"
        target.write_bytes(b"{}")
        link.symlink_to(target)
        assert_exits("regular file", checker.read_bounded_file, "cloud", link)


def test_local_reader_rejects_fifo_without_blocking() -> None:
    with tempfile.TemporaryDirectory() as directory:
        fifo = Path(directory) / "spec.fifo"
        os.mkfifo(fifo)
        assert_exits("regular file", checker.read_bounded_file, "cloud", fifo)


def main() -> None:
    tests = (
        test_bounded_reader_accepts_exact_limit,
        test_bounded_reader_rejects_oversize_response,
        test_bounded_reader_rejects_total_timeout,
        test_fetch_response_requires_exact_https_url,
        test_redirect_handler_refuses_followup_requests,
        test_load_specs_authenticates_before_parsing,
        test_local_reader_rejects_oversize_before_reading,
        test_local_reader_rejects_symlink,
        test_local_reader_rejects_fifo_without_blocking,
    )
    for test in tests:
        test()
    print(f"{len(tests)} Hetzner API drift tests passed.")


if __name__ == "__main__":
    main()
