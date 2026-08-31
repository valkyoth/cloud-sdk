#!/usr/bin/env python3
"""Regression tests for crates.io source-lock acquisition and classification."""

from __future__ import annotations

import copy
import json
import subprocess
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from cratesio_source_fetch import SameOriginRedirects, read_response
from cratesio_source_lock import (
    CARGO_COLUMNS,
    TSV_COLUMNS,
    SourceLockError,
    cargo_rows,
    html_text,
    operation_rows,
    parse_json,
    policy_observation,
    render_tsv,
    validate_lock,
    validate_source_evidence,
    validate_tsv,
)


ROOT = Path(__file__).resolve().parents[1]


def auth_schemes() -> dict:
    return {
        "api_token": {"type": "apiKey", "in": "header", "name": "authorization"},
        "cookie": {"type": "apiKey", "in": "cookie", "name": "cargo_session"},
        "trustpub_token": {"type": "http", "scheme": "bearer"},
    }


def operation(operation_id: str = "fixture") -> dict:
    return {
        "operationId": operation_id,
        "responses": {
            "200": {
                "description": "ok",
                "content": {
                    "application/json": {"schema": {"$ref": "#/components/schemas/Ok"}}
                },
            }
        },
    }


def specification() -> dict:
    return {
        "openapi": "3.1.0",
        "paths": {"/api/v1/fixture": {"get": operation()}},
        "components": {
            "securitySchemes": auth_schemes(),
            "schemas": {"Ok": {"type": "object"}},
        },
    }


def cargo_html() -> bytes:
    sections = [
        ("publish", "PUT", "/api/v1/crates/new", "h2"),
        ("yank", "DELETE", "/api/v1/crates/{crate_name}/{version}/yank", "h2"),
        ("unyank", "PUT", "/api/v1/crates/{crate_name}/{version}/unyank", "h2"),
        ("owners", "-", "-", "h2"),
        ("owners-list", "GET", "/api/v1/crates/{crate_name}/owners", "h3"),
        ("owners-add", "PUT", "/api/v1/crates/{crate_name}/owners", "h3"),
        ("owners-remove", "DELETE", "/api/v1/crates/{crate_name}/owners", "h3"),
        ("search", "GET", "/api/v1/crates", "h2"),
        ("login", "INSTRUCTION", "/me", "h2"),
    ]
    body = []
    for section, method, path, heading in sections:
        body.append(f'<{heading} id="{section}">{section}</{heading}>')
        body.append(f"<p>Endpoint: <code>{path}</code> Method: {method}</p>")
    return f"<!doctype html><html><body>{''.join(body)}</body></html>".encode()


def policy_source() -> bytes:
    return b"""<h2 id="crate-index">sparse</h2>
<h2 id="crate-content">static</h2><h2 id="rss-feeds">rss</h2>
<h2 id="database-dumps">dumps</h2><h2 id="api">api</h2>
<p>Please try them in the order below</p>
<li>A maximum of 1 request per second, and</li>
<li>A <code>user-agent</code> header that identifies your application.</li>
<p>Data Access Policy</p>"""


class Headers:
    def __init__(self, content_type: str, length: str | None = None) -> None:
        self.content_type = content_type
        self.length = length

    def get_content_type(self) -> str:
        return self.content_type

    def get(self, name: str) -> str | None:
        return self.length if name == "Content-Length" else None


class Response:
    def __init__(self, payload: bytes, source: dict, length: str | None = None) -> None:
        self.payload = payload
        self.offset = 0
        self.source = source
        self.headers = Headers(source["media_type"], length)

    def geturl(self) -> str:
        return self.source["final_url"]

    def read(self, amount: int) -> bytes:
        result = self.payload[self.offset : self.offset + amount]
        self.offset += len(result)
        return result


def fetch_source_fixture(max_bytes: int = 4) -> dict:
    return {
        "id": "fixture",
        "final_url": "https://example.test/source",
        "redirects": [],
        "media_type": "application/json",
        "max_bytes": max_bytes,
    }


class SourceLockTests(unittest.TestCase):
    def test_committed_lock_and_matrix_are_structurally_valid(self) -> None:
        lock = json.loads(
            (ROOT / "provider-drift/providers/cratesio-source.lock.json").read_text()
        )
        validate_lock(lock)
        operations = validate_tsv(
            (ROOT / "docs/CRATESIO_API_SCOPE.tsv").read_bytes(), TSV_COLUMNS, 51
        )
        cargo = validate_tsv(
            (ROOT / "docs/CRATESIO_CARGO_COMPATIBILITY.tsv").read_bytes(),
            CARGO_COLUMNS,
            8,
        )
        self.assertEqual(sum(row["stability"] == "stable-cargo" for row in operations), 7)
        self.assertEqual(sum(row["classification"] == "superseded" for row in cargo), 7)

    def test_offline_cli_accepts_the_committed_evidence(self) -> None:
        result = subprocess.run(
            [str(ROOT / "scripts/check_cratesio_source_lock.py")],
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_json_duplicate_keys_are_rejected(self) -> None:
        with self.assertRaises(SourceLockError):
            parse_json(b'{"openapi":"3.1.0","openapi":"3.0.0"}', "fixture")

    def test_unresolved_references_are_rejected(self) -> None:
        document = specification()
        document["paths"]["/api/v1/fixture"]["get"]["responses"]["200"]["content"]["application/json"]["schema"]["$ref"] = "#/components/schemas/Missing"
        with self.assertRaises(SourceLockError):
            operation_rows(document)

    def test_duplicate_operation_ids_are_rejected(self) -> None:
        document = specification()
        document["paths"]["/api/v1/second"] = {"post": operation()}
        with self.assertRaises(SourceLockError):
            operation_rows(document)

    def test_unknown_auth_is_rejected(self) -> None:
        document = specification()
        document["paths"]["/api/v1/fixture"]["get"]["security"] = [{"unknown": []}]
        with self.assertRaises(SourceLockError):
            operation_rows(document)

    def test_malformed_html_and_changed_cargo_method_are_rejected(self) -> None:
        with self.assertRaises(SourceLockError):
            html_text(b"<html><body>truncated", "fixture")
        with self.assertRaises(SourceLockError):
            cargo_rows(cargo_html().replace(b"Method: PUT", b"Method: POST", 1))

    def test_policy_requires_deployed_route_and_complete_source_text(self) -> None:
        deployed = b"<html><body>crates.io: Rust Package Registry</body></html>"
        observed = policy_observation(deployed, policy_source())
        self.assertEqual(observed["api_max_requests_per_second"], 1)
        with self.assertRaises(SourceLockError):
            policy_observation(deployed, b"<h2 id=\"api\">missing</h2>")

    def test_source_evidence_must_contain_both_contracts(self) -> None:
        validate_source_evidence(
            b"OpenApi crates.io data access policy", policy_source()
        )
        with self.assertRaises(SourceLockError):
            validate_source_evidence(b"OpenApi", policy_source())

    def test_unclassified_rows_are_rejected(self) -> None:
        row = {column: "value" for column in TSV_COLUMNS}
        row["classification"] = "unknown"
        with self.assertRaises(SourceLockError):
            render_tsv(TSV_COLUMNS, [row])

    def test_cross_origin_redirects_are_rejected(self) -> None:
        handler = SameOriginRedirects("https://crates.io/source")
        with self.assertRaises(SourceLockError):
            handler.redirect_request(
                None, None, 302, "moved", None, "https://attacker.example/source"
            )

    def test_response_bounds_and_deadline_fail_closed(self) -> None:
        source = fetch_source_fixture()
        with self.assertRaises(SourceLockError):
            read_response(Response(b"12345", source), source, [], 0.0, lambda: 0.0)
        with self.assertRaises(SourceLockError):
            read_response(Response(b"1", source), source, [], 0.0, lambda: 61.0)
        with self.assertRaises(SourceLockError):
            read_response(Response(b"1", source, "invalid"), source, [], 0.0, lambda: 0.0)

    def test_source_lock_rejects_unpinned_source_evidence(self) -> None:
        lock = json.loads(
            (ROOT / "provider-drift/providers/cratesio-source.lock.json").read_text()
        )
        changed = copy.deepcopy(lock)
        changed["sources"][3]["url"] = "https://raw.githubusercontent.com/rust-lang/crates.io/main/src/openapi.rs"
        with self.assertRaises(SourceLockError):
            validate_lock(changed)


if __name__ == "__main__":
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(SourceLockTests)
    result = unittest.TextTestRunner(verbosity=0).run(suite)
    if not result.wasSuccessful():
        raise SystemExit(1)
    print(f"{result.testsRun} crates.io source-lock regression tests passed.")
