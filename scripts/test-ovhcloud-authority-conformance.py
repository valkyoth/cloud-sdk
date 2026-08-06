#!/usr/bin/env python3
"""Regression tests for OVHcloud authority/OAuth source binding."""

from __future__ import annotations

import importlib.util
import tempfile
from pathlib import Path


SCRIPT = Path(__file__).with_name("check_ovhcloud_authority_conformance.py")
SPEC = importlib.util.spec_from_file_location("ovh_authority", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def assert_rejected(action) -> None:
    try:
        action()
    except ValueError:
        return
    raise AssertionError("invalid authority evidence was accepted")


def endpoint(region: str, api: str, token: str) -> dict:
    return {
        "values": {
            "region": region,
            "host": api,
            "base_path": "/v2",
            "token_host": token,
        }
    }


def test_reviewed_pairs_are_finite_and_region_sorted() -> None:
    lock = {
        "contracts": {
            "endpoints": [
                endpoint("eu", "eu.api.ovh.com", "www.ovh.com"),
                endpoint("ca", "ca.api.ovh.com", "ca.ovh.com"),
            ]
        }
    }
    rows = MODULE.reviewed_pairs(lock)
    assert [row["region"] for row in rows] == ["ca", "eu"]
    assert rows[0]["token_host"] == "ca.ovh.com"
    assert rows[1]["token_host"] == "www.ovh.com"

    assert_rejected(
        lambda: MODULE.reviewed_pairs(
            {"contracts": {"endpoints": [lock["contracts"]["endpoints"][0]]}}
        )
    )
    assert_rejected(
        lambda: MODULE.reviewed_pairs(
            {
                "contracts": {
                    "endpoints": [
                        endpoint("eu", "eu.api.ovh.com", "www.ovh.com"),
                        endpoint("eu", "alias.ovh.com", "www.ovh.com"),
                    ]
                }
            }
        )
    )


def test_fixture_parser_rejects_shape_count_and_encoding_changes() -> None:
    header = "\t".join(MODULE.FIELDS)
    rows = [
        "ca\tca.api.ovh.com\t443\t/v2\tca.ovh.com\t443\t/auth/oauth2/token",
        "eu\teu.api.ovh.com\t443\t/v2\twww.ovh.com\t443\t/auth/oauth2/token",
    ]
    with tempfile.TemporaryDirectory() as directory:
        fixture = Path(directory) / "pairs.tsv"
        fixture.write_text("\n".join([header, *rows, ""]), encoding="ascii")
        assert len(MODULE.fixture_pairs(fixture)) == 2

        fixture.write_text("\n".join([header, rows[0], ""]), encoding="ascii")
        assert_rejected(lambda: MODULE.fixture_pairs(fixture))

        fixture.write_text("wrong\n", encoding="ascii")
        assert_rejected(lambda: MODULE.fixture_pairs(fixture))

        fixture.write_bytes((header + "\n").encode("ascii") + b"\xff\n")
        assert_rejected(lambda: MODULE.fixture_pairs(fixture))


def main() -> None:
    test_reviewed_pairs_are_finite_and_region_sorted()
    test_fixture_parser_rejects_shape_count_and_encoding_changes()
    print("2 OVHcloud authority conformance regression groups passed.")


if __name__ == "__main__":
    main()
