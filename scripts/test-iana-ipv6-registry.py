#!/usr/bin/env python3
"""Regression tests for the IANA IPv6 source-lock checker."""

from __future__ import annotations

import importlib.util
import hashlib
import io
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "check_iana_ipv6_registry.py"


def load_checker():
    spec = importlib.util.spec_from_file_location("check_iana_ipv6_registry", SCRIPT)
    if spec is None or spec.loader is None:
        raise RuntimeError("could not load check_iana_ipv6_registry.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


checker = load_checker()


def assert_exits(expected: str, function, *args, **kwargs) -> None:
    try:
        function(*args, **kwargs)
    except SystemExit as error:
        if expected not in str(error):
            raise AssertionError(f"expected {expected!r} in {error!r}") from error
        return
    raise AssertionError("expected SystemExit")


def test_allocated_rows_apply_policy_exclusions() -> None:
    payload = (
        b"Prefix,Designation,Date,WHOIS,RDAP,Status,Note\n"
        b"2001::/23,IANA,1999-07-01,,,ALLOCATED,partial\n"
        b"2001:200::/23,APNIC,1999-07-01,,,ALLOCATED,\n"
        b"2002::/16,6to4,2001-02-01,,,ALLOCATED,special\n"
        b"3000::/5,IANA,1999-07-01,,,RESERVED,\n"
    )
    expected = {"2001:200::/23": ("APNIC", "1999-07-01")}
    if checker.allocated_rows(payload) != expected:
        raise AssertionError("allocation policy exclusions changed")


def test_special_prefix_footnotes_are_normalized() -> None:
    payload = b"Address Block,Name\n2002::/16 [3],6to4\n"
    if checker.special_prefixes(payload) != {"2002::/16"}:
        raise AssertionError("special-purpose footnote was not normalized")


def test_bounded_reader_accepts_exact_limit() -> None:
    result = checker.read_bounded_response(
        io.BytesIO(b"12345678"), "global-unicast", max_bytes=8
    )
    if result != b"12345678":
        raise AssertionError(f"unexpected bounded read: {result!r}")


def test_bounded_reader_rejects_oversize() -> None:
    assert_exits(
        "exceeds 8 bytes",
        checker.read_bounded_response,
        io.BytesIO(b"123456789"),
        "global-unicast",
        max_bytes=8,
    )


def test_bounded_reader_rejects_total_timeout() -> None:
    ticks = iter((0, 31))
    assert_exits(
        "exceeded 30 seconds",
        checker.read_bounded_response,
        io.BytesIO(b"data"),
        "global-unicast",
        monotonic=lambda: next(ticks),
    )


def test_local_lock_matches_rust_policy() -> None:
    if checker.validate_local() != 0:
        raise AssertionError("local IANA lock does not match Rust policy")


def test_registry_is_authenticated_before_parsing() -> None:
    payload = b"not authenticated CSV"
    original = checker.REGISTRIES["global-unicast"]
    checker.REGISTRIES["global-unicast"] = (original[0], hashlib.sha256(b"x").hexdigest())
    try:
        assert_exits(
            "SHA-256 mismatch", checker.verified_registry, "global-unicast", payload
        )
    finally:
        checker.REGISTRIES["global-unicast"] = original


def test_rust_policy_rejects_unrepresentable_prefixes() -> None:
    with tempfile.TemporaryDirectory() as directory:
        policy = Path(directory) / "public_ip.rs"
        policy.write_text(
            "Ipv6Prefix::new(0x2001, 0x0200, 33),\n", encoding="utf-8"
        )
        assert_exits("longer than", checker.rust_prefixes, policy)


def main() -> None:
    tests = (
        test_allocated_rows_apply_policy_exclusions,
        test_special_prefix_footnotes_are_normalized,
        test_bounded_reader_accepts_exact_limit,
        test_bounded_reader_rejects_oversize,
        test_bounded_reader_rejects_total_timeout,
        test_local_lock_matches_rust_policy,
        test_registry_is_authenticated_before_parsing,
        test_rust_policy_rejects_unrepresentable_prefixes,
    )
    for test in tests:
        test()
    print(f"{len(tests)} IANA IPv6 registry tests passed.")


if __name__ == "__main__":
    main()
