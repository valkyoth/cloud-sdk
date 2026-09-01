#!/usr/bin/env python3
"""Regression tests for crates.io source-lock acquisition and classification."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from cratesio_source_fetch import SameOriginRedirects, read_response
from cratesio_source_error import SourceLockError
from cratesio_source_manifest import validate_artifact_digests, validate_lock
from cratesio_source_lock import (
    CARGO_COLUMNS,
    TSV_COLUMNS,
    cargo_rows,
    html_text,
    operation_rows,
    parse_json,
    policy_observation,
    render_tsv,
    validate_source_evidence,
    validate_tsv,
)


ROOT = Path(__file__).resolve().parents[1]


def committed_lock() -> dict:
    return json.loads(
        (ROOT / "provider-drift/providers/cratesio-source.lock.json").read_text()
    )


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


def path_specification() -> dict:
    document = specification()
    operation_value = operation("path_fixture")
    operation_value["parameters"] = [
        {"name": "crate_name", "in": "path", "required": True}
    ]
    document["paths"] = {
        "/api/v1/crates/{crate_name}/{version}": {
            "parameters": [
                {"name": "version", "in": "path", "required": True}
            ],
            "get": operation_value,
        }
    }
    return document


def stable_path_specification() -> dict:
    document = specification()
    operation_value = operation("yank_version")
    operation_value["parameters"] = [
        {
            "name": "crate_name",
            "in": "path",
            "required": True,
            "schema": {"type": "string"},
            "style": "simple",
            "explode": False,
            "allowReserved": False,
        },
        {
            "name": "version",
            "in": "path",
            "required": True,
            "schema": {"type": "string"},
        },
    ]
    document["paths"] = {
        "/api/v1/crates/{crate_name}/{version}/yank": {"delete": operation_value}
    }
    return document


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
<p>Should you be unable to use one of the previous options, use the API.</p>
<li>A maximum of 1 request per second, and</li>
<li>A <code>user-agent</code> header that identifies your application.</li>
<p>We strongly suggest providing a way for us to contact you.</p>
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
        lock = committed_lock()
        validate_lock(lock)
        operation_data = (ROOT / "docs/CRATESIO_API_SCOPE.tsv").read_bytes()
        cargo_data = (ROOT / "docs/CRATESIO_CARGO_COMPATIBILITY.tsv").read_bytes()
        validate_artifact_digests(lock, operation_data, cargo_data)
        operations = validate_tsv(operation_data, TSV_COLUMNS, 51)
        cargo = validate_tsv(cargo_data, CARGO_COLUMNS, 8)
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

    def test_path_parameters_are_exact_direct_required_declarations(self) -> None:
        path = "/api/v1/crates/{crate_name}/{version}"
        operation_rows(path_specification())
        invalid = []

        missing = path_specification()
        missing["paths"][path]["get"]["parameters"] = []
        invalid.append(("missing", missing))

        extra = path_specification()
        extra["paths"][path]["get"]["parameters"].append(
            {"name": "other", "in": "path", "required": True}
        )
        invalid.append(("extra", extra))

        duplicate = path_specification()
        duplicate["paths"][path]["get"]["parameters"].append(
            {"name": "crate_name", "in": "path", "required": True}
        )
        invalid.append(("duplicate", duplicate))

        misplaced = path_specification()
        misplaced["paths"][path]["get"]["parameters"][0]["in"] = "query"
        invalid.append(("misplaced", misplaced))

        optional = path_specification()
        optional["paths"][path]["parameters"][0]["required"] = False
        invalid.append(("optional", optional))

        referenced = path_specification()
        referenced["components"]["parameters"] = {
            "crate": {"name": "crate_name", "in": "path", "required": True}
        }
        referenced["paths"][path]["get"]["parameters"][0] = {
            "$ref": "#/components/parameters/crate"
        }
        invalid.append(("referenced", referenced))

        invalid_object = path_specification()
        invalid_object["paths"][path]["get"]["parameters"][0] = None
        invalid.append(("invalid-object", invalid_object))

        invalid_array = path_specification()
        invalid_array["paths"][path]["parameters"] = {}
        invalid.append(("invalid-array", invalid_array))

        malformed = path_specification()
        malformed["paths"]["/api/v1/crates/{crate_name}/{version"] = (
            malformed["paths"].pop(path)
        )
        invalid.append(("malformed", malformed))

        repeated = path_specification()
        repeated["paths"]["/api/v1/crates/{crate_name}/{crate_name}"] = (
            repeated["paths"].pop(path)
        )
        invalid.append(("repeated-template", repeated))

        for label, document in invalid:
            with self.subTest(label=label), self.assertRaises(SourceLockError):
                operation_rows(document)

    def test_stable_path_parameters_require_cargo_wire_semantics(self) -> None:
        path = "/api/v1/crates/{crate_name}/{version}/yank"
        operation_rows(stable_path_specification())
        changes = (
            ("missing-schema", None),
            ("array", {"schema": {"type": "array", "items": {"type": "string"}}}),
            ("object", {"schema": {"type": "object"}}),
            ("matrix", {"style": "matrix"}),
            ("label", {"style": "label"}),
            ("exploded", {"explode": True}),
            ("reserved", {"allowReserved": True}),
            ("content", {"content": {"text/plain": {"schema": {"type": "string"}}}}),
        )
        for label, change in changes:
            document = stable_path_specification()
            parameter = document["paths"][path]["delete"]["parameters"][0]
            if change is None:
                parameter.pop("schema")
            else:
                parameter.update(change)
            with self.subTest(label=label), self.assertRaises(SourceLockError):
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

    def test_synthetic_auth_requires_anonymous_upstream_evidence(self) -> None:
        document = specification()
        token = operation("accept_crate_owner_invitation_with_token")
        token["security"] = [{"cookie": []}]
        token["parameters"] = [
            {
                "name": "token",
                "in": "path",
                "required": True,
                "schema": {"type": "string"},
            }
        ]
        document["paths"] = {
            "/api/v1/me/crate_owner_invitations/accept/{token}": {"put": token}
        }
        with self.assertRaises(SourceLockError):
            operation_rows(document)

    def test_synthetic_auth_requires_its_exact_path_or_body_token(self) -> None:
        document = specification()
        confirmation = operation("confirm_user_email")
        document["paths"] = {
            "/api/v1/confirm/{email_token}": {"put": confirmation}
        }
        with self.assertRaises(SourceLockError):
            operation_rows(document)

        exchange = operation("exchange_trustpub_token")
        document["paths"] = {"/api/v1/trusted_publishing/tokens": {"post": exchange}}
        with self.assertRaises(SourceLockError):
            operation_rows(document)

    def test_trace_operations_are_rejected_instead_of_omitted(self) -> None:
        document = specification()
        document["paths"] = {"/api/v1/fixture": {"trace": operation()}}
        with self.assertRaises(SourceLockError):
            operation_rows(document)

    def test_malformed_html_and_changed_cargo_method_are_rejected(self) -> None:
        with self.assertRaises(SourceLockError):
            html_text(b"<html><body>truncated", "fixture")
        with self.assertRaises(SourceLockError):
            cargo_rows(cargo_html().replace(b"Method: PUT", b"Method: POST", 1))

    def test_policy_requires_deployed_route_and_complete_source_text(self) -> None:
        deployed = b"<html><body>crates.io: Rust Package Registry</body></html>"
        source = policy_source()
        expected_sha256 = hashlib.sha256(source).hexdigest()
        observed = policy_observation(deployed, source, expected_sha256)
        self.assertEqual(observed["api_max_requests_per_second"], 1)
        with self.assertRaises(SourceLockError):
            policy_observation(
                deployed,
                b'<h2 id="api">missing</h2>',
                expected_sha256,
            )

    def test_negated_policy_prose_cannot_reuse_the_reviewed_policy(self) -> None:
        deployed = b"<html><body>crates.io: Rust Package Registry</body></html>"
        source = policy_source()
        negated = source.replace(
            b"A user-agent", b"You need not send a user-agent"
        ).replace(b"strongly suggest", b"do not suggest")
        with self.assertRaises(SourceLockError):
            policy_observation(
                deployed,
                negated,
                hashlib.sha256(source).hexdigest(),
            )

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
        lock = committed_lock()
        changed = copy.deepcopy(lock)
        changed["sources"][3]["url"] = "https://raw.githubusercontent.com/rust-lang/crates.io/main/src/openapi.rs"
        with self.assertRaises(SourceLockError):
            validate_lock(changed)

    def test_source_lock_rejects_nonexistent_calendar_date(self) -> None:
        changed = copy.deepcopy(committed_lock())
        changed["reviewed_at"] = "2026-99-99"
        with self.assertRaises(SourceLockError):
            validate_lock(changed)

    def test_source_lock_requires_exact_official_requested_and_final_urls(self) -> None:
        for field in ("url", "final_url"):
            with self.subTest(field=field):
                changed = copy.deepcopy(committed_lock())
                changed["sources"][0][field] = "https://attacker.example/api/openapi.json"
                with self.assertRaises(SourceLockError):
                    validate_lock(changed)

    def test_artifact_digests_reject_shape_preserving_inventory_changes(self) -> None:
        lock = committed_lock()
        operations = (ROOT / "docs/CRATESIO_API_SCOPE.tsv").read_bytes()
        cargo = (ROOT / "docs/CRATESIO_CARGO_COMPATIBILITY.tsv").read_bytes()
        changed = operations.replace(b"/api/v1/categories", b"/api/v1/attacker", 1)
        with self.assertRaises(SourceLockError):
            validate_artifact_digests(lock, changed, cargo)

    def test_release_gate_reconstructs_from_official_sources(self) -> None:
        gate = (ROOT / "scripts/release_1_1_gate.sh").read_text(encoding="ascii")
        self.assertIn("scripts/check_cratesio_source_lock.py --fetch", gate)
        self.assertIn("scripts/check_cratesio_drift.py --fetch", gate)
        self.assertLess(
            gate.index("scripts/check_cratesio_drift.py --fetch"),
            gate.index("scripts/check_cratesio_source_lock.py --fetch"),
        )


if __name__ == "__main__":
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(SourceLockTests)
    result = unittest.TextTestRunner(verbosity=0).run(suite)
    if not result.wasSuccessful():
        raise SystemExit(1)
    print(f"{result.testsRun} crates.io source-lock regression tests passed.")
