#!/usr/bin/env python3
"""Regression tests for Hetzner spec integrity and download bounds."""

from __future__ import annotations

import argparse
import contextlib
import importlib.util
import io
import json
import os
import tempfile
import urllib.error
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "check_hetzner_api_drift.py"
FIXTURES = ROOT / "tests" / "fixtures" / "hetzner-api-drift"
WORKFLOW = ROOT / ".github" / "workflows" / "hetzner-api-drift.yml"


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


class RedirectingOpener:
    def open(self, url: str, *, timeout: int):
        raise urllib.error.HTTPError(url, 302, "Found", {}, None)


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


def test_fetch_spec_reports_rejected_redirect_without_traceback() -> None:
    original = checker.urllib.request.build_opener
    checker.urllib.request.build_opener = lambda *_handlers: RedirectingOpener()
    try:
        with tempfile.TemporaryDirectory() as directory:
            assert_exits(
                "could not fetch cloud spec: HTTP Error 302: Found",
                checker.fetch_spec,
                "cloud",
                Path(directory),
            )
    finally:
        checker.urllib.request.build_opener = original


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


def read_fixture(name: str) -> dict:
    with (FIXTURES / name).open("r", encoding="utf-8") as handle:
        return json.load(handle)


def fixture_rows(prefix: str) -> tuple[list[dict[str, str]], list[dict[str, str]]]:
    operations = []
    schemas = []
    for api in ("cloud", "hetzner"):
        document = read_fixture(f"{prefix}-{api}.json")
        operations.extend(checker.operation_rows(api, document))
        schemas.extend(checker.schema_rows(api, document))
    return operations, schemas


def test_fixture_report_groups_every_drift_category() -> None:
    locked_operations, locked_schemas = fixture_rows("locked")
    current_operations, current_schemas = fixture_rows("current")
    report = checker.build_drift_report(
        locked_operations,
        current_operations,
        locked_schemas,
        current_schemas,
        checker.PINNED_SPEC_SHA256.copy(),
        checker.PINNED_SPEC_SHA256,
    )

    assert report["added"] == [("cloud", "PUT", "/new")]
    assert report["removed"] == [("cloud", "DELETE", "/removed")]
    assert [change["key"] for change in report["deprecated"]] == [
        ("cloud", "GET", "/deprecated")
    ]
    assert [change["key"] for change in report["changed"]] == [
        ("cloud", "POST", "/changed")
    ]
    schemas = report["schema_only"]
    assert schemas["added"] == [("cloud", "Added")]
    assert schemas["removed"] == [("cloud", "Removed")]
    assert [change["key"] for change in schemas["changed"]] == [
        ("cloud", "Changed")
    ]

    errors = io.StringIO()
    with contextlib.redirect_stderr(errors):
        status = checker.print_drift_report(report)
    output = errors.getvalue()
    assert status == 1
    for heading in (
        "added operations:",
        "removed operations:",
        "deprecated operations:",
        "changed operations:",
        "schema-only changes:",
    ):
        assert heading in output


def test_fixture_report_ignores_prose_only_changes() -> None:
    locked_operations, locked_schemas = fixture_rows("locked")
    stable_operation = next(
        row for row in locked_operations if row["path"] == "/stable"
    )
    stable_schema = next(row for row in locked_schemas if row["schema"] == "Stable")
    current_operations, current_schemas = fixture_rows("current")
    current_operation = next(
        row for row in current_operations if row["path"] == "/stable"
    )
    current_schema = next(
        row for row in current_schemas if row["schema"] == "Stable"
    )
    assert stable_operation == current_operation
    assert stable_schema == current_schema


def test_changed_fetched_digest_is_reported_after_safe_parsing() -> None:
    payload = b'{"openapi":"3.1.0","paths":{}}'
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "cloud.json"
        path.write_bytes(payload)
        document, actual = checker.read_spec("cloud", path, expected_sha256=None)
    assert document["openapi"] == "3.1.0"
    report = checker.build_drift_report(
        [],
        [],
        [],
        [],
        {"cloud": actual, "hetzner": checker.PINNED_SPEC_SHA256["hetzner"]},
        checker.PINNED_SPEC_SHA256,
    )
    assert [change["key"] for change in report["changed_sources"]] == [
        ("cloud",)
    ]


def test_duplicate_operation_identity_is_rejected() -> None:
    row = {
        "api": "cloud",
        "method": "GET",
        "path": "/duplicate",
    }
    assert_exits(
        "duplicate current operation identity",
        checker.compare_row_sets,
        "operation",
        [],
        [row, row.copy()],
        ("api", "method", "path"),
    )


def test_deprecation_with_contract_change_appears_in_both_groups() -> None:
    locked = {
        "api": "cloud",
        "method": "GET",
        "path": "/changing",
        "deprecated": "no",
        "fingerprint": "old-contract",
    }
    current = locked | {"deprecated": "yes", "fingerprint": "new-contract"}
    report = checker.build_drift_report(
        [locked],
        [current],
        [],
        [],
        checker.PINNED_SPEC_SHA256.copy(),
        checker.PINNED_SPEC_SHA256,
    )
    expected = [("cloud", "GET", "/changing")]
    assert [change["key"] for change in report["deprecated"]] == expected
    assert [change["key"] for change in report["changed"]] == expected


def test_lock_refresh_rejects_unreviewed_source_digest() -> None:
    assert_exits(
        "lock refresh requires reviewed source pins for: cloud",
        checker.ensure_refresh_sources_pinned,
        {"cloud": "0" * 64, "hetzner": checker.PINNED_SPEC_SHA256["hetzner"]},
    )


def test_tsv_writer_uses_portable_lf_endings() -> None:
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "lock.tsv"
        checker.write_tsv(path, [{"api": "cloud"}], ["api"])
        assert path.read_bytes() == b"api\ncloud\n"


def test_report_escapes_control_and_terminal_bytes() -> None:
    report = {
        "added": [("cloud", "GET", "/line\n\x1b[31m")],
        "removed": [],
        "deprecated": [],
        "changed": [
            {
                "key": ("cloud", "GET", "/changed"),
                "fields": {"operation_id": ("old\rvalue", "new\tvalue")},
            }
        ],
        "changed_sources": [],
        "schema_only": {"added": [], "removed": [], "changed": []},
    }
    errors = io.StringIO()
    with contextlib.redirect_stderr(errors):
        assert checker.print_drift_report(report) == 1
    output = errors.getvalue()
    assert "\x1b" not in output
    assert "/line\\n\\u001b[31m" in output
    assert "old\\rvalue -> new\\tvalue" in output


def test_malformed_openapi_shapes_and_lock_rows_fail_cleanly() -> None:
    assert_exits(
        "cloud spec paths must be an object",
        checker.operation_rows,
        "cloud",
        {"paths": []},
    )
    assert_exits(
        "cloud spec contains invalid operation parameters",
        checker.operation_rows,
        "cloud",
        {"paths": {"/bad": {"get": {"parameters": [None]}}}},
    )
    assert_exits(
        "cloud spec schemas must be a text-keyed object",
        checker.schema_rows,
        "cloud",
        {"components": {"schemas": []}},
    )
    assert_exits(
        "missing identity field path",
        checker.compare_row_sets,
        "operation",
        [],
        [{"api": "cloud", "method": "GET"}],
        ("api", "method", "path"),
    )


def test_maintenance_workflow_is_read_only_and_runs_live_detector() -> None:
    workflow = WORKFLOW.read_text(encoding="utf-8")
    assert "schedule:" in workflow
    assert "workflow_dispatch:" in workflow
    assert "permissions:\n  contents: read" in workflow
    assert "scripts/check_hetzner_api_drift.py --fetch" in workflow
    assert "--write-lock" not in workflow
    assert "pull-requests: write" not in workflow


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
        test_fetch_spec_reports_rejected_redirect_without_traceback,
        test_load_specs_authenticates_before_parsing,
        test_fixture_report_groups_every_drift_category,
        test_fixture_report_ignores_prose_only_changes,
        test_changed_fetched_digest_is_reported_after_safe_parsing,
        test_duplicate_operation_identity_is_rejected,
        test_deprecation_with_contract_change_appears_in_both_groups,
        test_lock_refresh_rejects_unreviewed_source_digest,
        test_tsv_writer_uses_portable_lf_endings,
        test_report_escapes_control_and_terminal_bytes,
        test_malformed_openapi_shapes_and_lock_rows_fail_cleanly,
        test_maintenance_workflow_is_read_only_and_runs_live_detector,
        test_local_reader_rejects_oversize_before_reading,
        test_local_reader_rejects_symlink,
        test_local_reader_rejects_fifo_without_blocking,
    )
    for test in tests:
        test()
    print(f"{len(tests)} Hetzner API drift tests passed.")


if __name__ == "__main__":
    main()
