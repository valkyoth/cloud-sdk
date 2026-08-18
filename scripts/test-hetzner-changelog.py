#!/usr/bin/env python3
"""Regression tests for the Hetzner changelog source lock."""

from __future__ import annotations

import importlib.util
import io
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "check_hetzner_changelog.py"


def load_checker():
    spec = importlib.util.spec_from_file_location("check_hetzner_changelog", SCRIPT)
    if spec is None or spec.loader is None:
        raise RuntimeError("could not load Hetzner changelog checker")
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


def feed(
    *,
    guid: str | None = None,
    duplicate: bool = False,
    build_date: str = "Tue, 18 Aug 2026 16:51:15 GMT",
) -> bytes:
    item_guid = guid or checker.PINNED_LATEST_GUID
    second = (
        f"<item><title>Older</title><guid>{item_guid}</guid></item>"
        if duplicate
        else ""
    )
    return f"""<?xml version="1.0" encoding="utf-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title>Hetzner Cloud Changelog</title>
    <link>https://docs.hetzner.cloud/changelog</link>
    <lastBuildDate>{build_date}</lastBuildDate>
    <atom:link href="{checker.SELF_URL}" rel="self" type="application/rss+xml"/>
    <item><title>Latest</title><guid>{item_guid}</guid></item>
    {second}
  </channel>
</rss>""".encode("utf-8")


def test_feed_identity_and_latest_entry_are_structured() -> None:
    latest, title, count, digest = checker.parse_feed(feed())
    assert latest == checker.PINNED_LATEST_GUID
    assert title == "Latest"
    assert count == 1
    assert len(digest) == 64


def test_volatile_build_date_does_not_change_semantic_digest() -> None:
    first = checker.parse_feed(feed(build_date="first"))[3]
    second = checker.parse_feed(feed(build_date="second"))[3]
    assert first == second


def test_only_the_single_channel_build_date_is_excluded() -> None:
    missing = feed().replace(
        b"<lastBuildDate>Tue, 18 Aug 2026 16:51:15 GMT</lastBuildDate>", b""
    )
    assert_exits("missing or ambiguous", checker.parse_feed, missing)

    duplicate = feed().replace(
        b"</lastBuildDate>", b"</lastBuildDate><lastBuildDate>second</lastBuildDate>"
    )
    assert_exits("missing or ambiguous", checker.parse_feed, duplicate)

    first = checker.parse_feed(
        feed().replace(b"<title>Latest</title>", b"<title>Latest</title><lastBuildDate>a</lastBuildDate>")
    )[3]
    second = checker.parse_feed(
        feed().replace(b"<title>Latest</title>", b"<title>Latest</title><lastBuildDate>b</lastBuildDate>")
    )[3]
    assert first != second


def test_duplicate_entry_identity_fails_closed() -> None:
    assert_exits("missing or duplicated", checker.parse_feed, feed(duplicate=True))


def test_malformed_xml_fails_closed() -> None:
    assert_exits("invalid XML", checker.parse_feed, b"<rss>")


def test_doctype_and_entity_declarations_fail_before_parsing() -> None:
    assert_exits(
        "declarations are not permitted",
        checker.parse_feed,
        b'<!DOCTYPE rss [<!ENTITY x "value">]><rss version="2.0"/>',
    )


def test_bounded_reader_accepts_exact_limit_and_rejects_oversize() -> None:
    assert checker.read_bounded_response(io.BytesIO(b"1234"), max_bytes=4) == b"1234"
    assert_exits(
        "exceeds 4 bytes",
        checker.read_bounded_response,
        io.BytesIO(b"12345"),
        max_bytes=4,
    )


def test_bounded_reader_rejects_total_timeout() -> None:
    ticks = iter((0, 61))
    assert_exits(
        "exceeded 60 seconds",
        checker.read_bounded_response,
        io.BytesIO(b"x"),
        monotonic=lambda: next(ticks),
    )


def test_fetch_response_requires_exact_https_url() -> None:
    class Response:
        def __init__(self, url: str) -> None:
            self.url = url

        def geturl(self) -> str:
            return self.url

    checker.validate_response(Response(checker.SOURCE_URL))
    assert_exits(
        "non-HTTPS",
        checker.validate_response,
        Response("http://docs.hetzner.cloud/changelog/feed.rss"),
    )
    assert_exits(
        "redirected",
        checker.validate_response,
        Response("https://example.invalid/feed.rss"),
    )


def test_redirect_handler_never_creates_a_followup_request() -> None:
    result = checker.RejectRedirects().redirect_request(
        None, None, 302, "Found", {}, "https://example.invalid"
    )
    assert result is None


def test_local_lock_binds_source_hash_and_latest_entry() -> None:
    assert checker.validate_local_lock() == 0


def main() -> int:
    tests = [
        value
        for name, value in sorted(globals().items())
        if name.startswith("test_") and callable(value)
    ]
    for test in tests:
        test()
    print(f"{len(tests)} Hetzner changelog regression tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
