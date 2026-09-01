#!/usr/bin/env python3
"""Semantic and transactional tests for crates.io drift detection."""

from __future__ import annotations

import copy
import hashlib
import json
import re
import tempfile
from pathlib import Path

import stage_cratesio_lock_refresh as refresh
from cratesio_drift_adapter import (
    CratesioAdapterError,
    build_observation,
    validate_stable_cargo_matches,
)
from cratesio_drift_documents import provider_lock
from cratesio_source_lock import CARGO_CONTRACTS, observe as observe_sources
from provider_drift_model import CATEGORIES, validate_lock, validate_observation
from provider_drift_report import build_report


def openapi() -> dict:
    response = {
        "description": "fixture",
        "content": {
            "application/json": {"schema": {"$ref": "#/components/schemas/Crate"}}
        },
    }
    return {
        "openapi": "3.1.0",
        "components": {
            "schemas": {"Crate": {"type": "object", "required": ["name"]}},
            "securitySchemes": {
                "api_token": {
                    "type": "apiKey",
                    "in": "header",
                    "name": "authorization",
                },
                "cookie": {
                    "type": "apiKey",
                    "in": "cookie",
                    "name": "cargo_session",
                },
                "trustpub_token": {"type": "http", "scheme": "bearer"},
            },
        },
        "paths": {
            "/api/v1/categories/{category}": {
                "get": {
                    "operationId": "get_category",
                    "parameters": [
                        {
                            "name": "include",
                            "in": "query",
                            "required": False,
                            "schema": {"type": "string"},
                        },
                        {"name": "category", "in": "path", "required": True},
                    ],
                    "responses": {"200": copy.deepcopy(response)},
                }
            },
            "/api/v1/crates": {
                "get": {
                    "operationId": "list_crates",
                    "responses": {"200": copy.deepcopy(response)},
                }
            },
        },
    }


def compatible_openapi() -> dict:
    document = openapi()
    response = {
        "description": "fixture",
        "content": {"application/json": {"schema": {"type": "object"}}},
    }
    for _contract, method, path, operation_id, _section in CARGO_CONTRACTS:
        openapi_path = path.replace("{crate_name}", "{name}")
        item = document["paths"].setdefault(openapi_path, {})
        item[method] = {
            "operationId": operation_id,
            "parameters": [
                {"name": name, "in": "path", "required": True, "schema": {"type": "string"}}
                for name in re.findall(r"\{([^{}]+)\}", openapi_path)
            ],
            "responses": {"200": copy.deepcopy(response)},
        }
    return document


def cargo_html() -> bytes:
    sections = []
    for contract, method, path, _operation_id, section in CARGO_CONTRACTS:
        sections.append(
            f'<h2 id="{section}">{contract}</h2><p>{path} Method: {method.upper()}</p>'
        )
    sections.append('<h2 id="login">login</h2><p>/me</p>')
    return ("<html><body>" + "".join(sections) + "</body></html>").encode()


def policy_source() -> bytes:
    return b"""<PageHeader title="Data Access Policy" />
<h2 id="crate-index">index</h2><h2 id="crate-content">content</h2>
<h2 id="rss-feeds">rss</h2><h2 id="database-dumps">dumps</h2>
<h2 id="api">api</h2>
Please try them in the order below. Should you be unable to use one of the
previous options, use the API. A maximum of 1 request per second. A user-agent
header that identifies your application. We strongly suggest providing a way
for us to contact you.
"""


def payloads(document: dict | None = None) -> dict[str, bytes]:
    policy = policy_source()
    return {
        "cargo": cargo_html(),
        "openapi": json.dumps(document or openapi()).encode(),
        "openapi-source": b"crates.io data access policy OpenApi",
        "policy": b"<html><body>crates.io: Rust Package Registry</body></html>",
        "policy-current": policy,
        "policy-source": policy,
    }


def fixture_lock(data: dict[str, bytes]) -> dict:
    sources = []
    for identity, payload in sorted(data.items()):
        sources.append(
            {
                "id": identity,
                "max_bytes": max(1, len(payload)),
                "sha256": hashlib.sha256(payload).hexdigest(),
                "url": f"https://example.com/{identity}",
            }
        )
    source_lock = {"sources": sources}
    empty = {category: [] for category in CATEGORIES}
    base = provider_lock(source_lock, empty)
    observed = build_observation(base, data)
    return validate_lock(provider_lock(source_lock, observed["contracts"]))


def observe(lock: dict, data: dict[str, bytes]) -> dict:
    current = copy.deepcopy(data)
    return validate_observation(build_observation(lock, current))


def changes(lock: dict, data: dict[str, bytes]) -> list[dict]:
    return build_report(lock, observe(lock, data))["changes"]


def changed(changes: list[dict], category: str, identity: str) -> dict:
    return next(
        item
        for item in changes
        if item["category"] == category and item["id"] == identity
    )


def test_clean_fixture_covers_every_commit_two_fingerprint_family() -> None:
    data = payloads()
    lock = fixture_lock(data)
    assert build_report(lock, observe(lock, data))["result"] == "clean"
    contracts = lock["contracts"]
    assert len(contracts["authentication"]) == 3
    assert len(contracts["operations"]) == 10
    assert len(contracts["schemas"]) == 5
    assert contracts["cost"][0]["values"]["preferred_sources"] == [
        "sparse-index",
        "static-downloads",
        "rss",
        "database-dumps",
    ]


def test_addition_removal_and_rename_are_separately_classified() -> None:
    accepted = payloads()
    lock = fixture_lock(accepted)
    document = openapi()
    document["paths"]["/api/v1/new"] = {
        "get": {"operationId": "new_operation", "responses": {"204": {}}}
    }
    del document["paths"]["/api/v1/categories/{category}"]
    report = changes(lock, payloads(document))
    assert changed(report, "operations", "openapi/new_operation")["kind"] == "added"
    assert changed(report, "operations", "openapi/get_category")["kind"] == "removed"


def test_parameter_schema_auth_media_status_and_stability_drift_is_visible() -> None:
    accepted = payloads()
    lock = fixture_lock(accepted)
    document = openapi()
    category = document["paths"]["/api/v1/categories/{category}"]["get"]
    category["parameters"][0]["required"] = True
    category["security"] = [{"api_token": []}]
    category["responses"]["201"] = category["responses"].pop("200")
    category["responses"]["201"]["content"] = {"text/plain": {"schema": {"type": "string"}}}
    document["components"]["schemas"]["Crate"]["required"].append("description")
    crates = document["paths"]["/api/v1/crates"]["get"]
    crates["operationId"] = "list_crates_v2"
    report = changes(lock, payloads(document))
    assert changed(report, "authentication", "operation/get_category")
    assert changed(report, "schemas", "request/get_category")
    assert changed(report, "schemas", "response/get_category")
    component = next(item for item in report if item["id"].startswith("component/"))
    assert component["category"] == "schemas"
    cargo = changed(report, "operations", "cargo/search")
    assert "/openapi_match" in cargo["fields"]
    assert changed(report, "operations", "openapi/list_crates")["kind"] == "removed"
    assert changed(report, "operations", "openapi/list_crates_v2")["kind"] == "added"


def test_policy_weakening_is_reported_in_each_owned_category() -> None:
    accepted = payloads()
    lock = fixture_lock(accepted)
    weakened = policy_source().replace(b"1 request", b"2 requests")
    weakened = weakened.replace(b"identifies your application", b"is optional")
    weakened = weakened.replace(b"suggest providing", b"do not recommend providing")
    weakened = weakened.replace(b"unable to use one of the\nprevious options", b"prefer not to use prior options")
    weakened = weakened.replace(b'<h2 id="rss-feeds">rss</h2>', b"")
    data = payloads()
    data["policy-current"] = weakened
    try:
        observe(lock, data)
    except CratesioAdapterError:
        pass
    else:
        raise AssertionError("unreviewed policy bytes were semantically interpreted")


def test_unavailable_or_incomplete_policy_cannot_be_clean() -> None:
    data = payloads()
    lock = fixture_lock(data)
    data["policy-current"] = b"truncated"
    try:
        observe(lock, data)
    except CratesioAdapterError:
        pass
    else:
        raise AssertionError("incomplete policy evidence was accepted")


def test_failed_refresh_does_not_publish_and_candidates_never_overwrite() -> None:
    with tempfile.TemporaryDirectory() as directory:
        output = Path(directory) / "candidate.json"
        original = refresh.observe_source
        refresh.observe_source = lambda _source: (_ for _ in ()).throw(
            OSError("fixture unavailable")
        )
        try:
            try:
                refresh.build_candidate("a" * 40, "2026-09-01")
            except OSError:
                pass
            else:
                raise AssertionError("failed refresh unexpectedly completed")
        finally:
            refresh.observe_source = original
        assert not output.exists()
        refresh.write_once(output, b"first")
        try:
            refresh.write_once(output, b"second")
        except FileExistsError:
            pass
        else:
            raise AssertionError("candidate publication overwrote prior evidence")
        assert output.read_bytes() == b"first"


def candidate_fixture() -> dict:
    data = payloads(compatible_openapi())
    sources = [
        {
            "id": identity,
            "max_bytes": max(1, len(payload)),
            "sha256": hashlib.sha256(payload).hexdigest(),
            "size_bytes": len(payload),
            "url": f"https://example.com/{identity}",
        }
        for identity, payload in sorted(data.items())
    ]
    source_lock = {"sources": sources}
    operations, cargo, summary = observe_sources(source_lock, data)
    source_lock.update(summary)
    source_lock["artifacts"] = {
        "operations_sha256": hashlib.sha256(operations).hexdigest(),
        "cargo_compatibility_sha256": hashlib.sha256(cargo).hexdigest(),
    }
    empty = {category: [] for category in CATEGORIES}
    base = provider_lock(source_lock, empty)
    observation = validate_observation(build_observation(base, data))
    lock = provider_lock(source_lock, observation["contracts"])
    return {
        "cargo_tsv": cargo.decode("ascii"),
        "format": refresh.BUNDLE_FORMAT,
        "observation": observation,
        "operations_tsv": operations.decode("ascii"),
        "payloads": refresh.encode_payloads(data),
        "provider_lock": lock,
        "source_lock": source_lock,
    }


def test_every_stable_cargo_contract_must_match_openapi() -> None:
    candidate = candidate_fixture()
    observation = candidate["observation"]
    validate_stable_cargo_matches(observation)
    rows = [
        row
        for row in observation["contracts"]["operations"]
        if row["id"].startswith("cargo/")
        and row["values"]["classification"] == "superseded"
    ]
    assert len(rows) == 7
    for index in range(len(rows)):
        changed_observation = copy.deepcopy(observation)
        changed_rows = [
            row
            for row in changed_observation["contracts"]["operations"]
            if row["id"].startswith("cargo/")
            and row["values"]["classification"] == "superseded"
        ]
        changed_rows[index]["values"]["openapi_match"] = False
        try:
            validate_stable_cargo_matches(changed_observation)
        except CratesioAdapterError:
            pass
        else:
            raise AssertionError(f"Cargo mismatch {index} was accepted")


def test_cargo_and_openapi_parameter_labels_share_one_route_shape() -> None:
    candidate = candidate_fixture()
    yank = next(
        row
        for row in candidate["observation"]["contracts"]["operations"]
        if row["id"] == "cargo/yank"
    )
    assert yank["values"]["path"].endswith("/{crate_name}/{version}/yank")
    assert yank["values"]["openapi_match"] is True


def test_cargo_path_parameter_identity_and_position_fail_closed() -> None:
    original = "/api/v1/crates/{name}/{version}/yank"
    invalid_paths = {
        "swapped": ("/api/v1/crates/{version}/{name}/yank", ("name", "version")),
        "missing": ("/api/v1/crates/{name}/yank", ("name",)),
        "unreviewed": (
            "/api/v1/crates/{package}/{version}/yank",
            ("package", "version"),
        ),
    }
    for label, (invalid_path, declarations) in invalid_paths.items():
        document = compatible_openapi()
        item = document["paths"].pop(original)
        item["delete"]["parameters"] = [
            {"name": name, "in": "path", "required": True, "schema": {"type": "string"}}
            for name in declarations
        ]
        document["paths"][invalid_path] = item
        data = payloads(document)
        lock = fixture_lock(data)
        observation = observe(lock, data)
        yank = next(
            row
            for row in observation["contracts"]["operations"]
            if row["id"] == "cargo/yank"
        )
        assert yank["values"]["openapi_match"] is False, label
        try:
            validate_stable_cargo_matches(observation)
        except CratesioAdapterError:
            pass
        else:
            raise AssertionError(f"{label} Cargo path parameters were accepted")


def test_candidate_derivatives_are_rebuilt_from_embedded_payloads() -> None:
    candidate = candidate_fixture()
    refresh.verify_payload_binding(candidate)

    missing_auth = copy.deepcopy(candidate)
    missing_auth["observation"]["contracts"]["authentication"] = []
    try:
        refresh.verify_payload_binding(missing_auth)
    except refresh.ModelError:
        pass
    else:
        raise AssertionError("candidate accepted an observation missing authentication")

    changed_inventory = copy.deepcopy(candidate)
    changed_inventory["operations_tsv"] += "unexpected\n"
    try:
        refresh.verify_payload_binding(changed_inventory)
    except refresh.ModelError:
        pass
    else:
        raise AssertionError("candidate accepted an inventory not derived from payloads")

    changed_payload = copy.deepcopy(candidate)
    encoded = changed_payload["payloads"]["policy-current"]
    changed_payload["payloads"]["policy-current"] = ("A" if encoded[0] != "A" else "B") + encoded[1:]
    try:
        refresh.verify_payload_binding(changed_payload)
    except refresh.ModelError:
        pass
    else:
        raise AssertionError("candidate accepted a payload outside its manifest")

    changed_pinned_policy = copy.deepcopy(candidate)
    changed = policy_source() + b"\nreviewed fork\n"
    changed_pinned_policy["payloads"]["policy-source"] = (
        refresh.encode_payloads({"policy-source": changed})["policy-source"]
    )
    source = next(
        row
        for row in changed_pinned_policy["source_lock"]["sources"]
        if row["id"] == "policy-source"
    )
    source["size_bytes"] = len(changed)
    source["sha256"] = hashlib.sha256(changed).hexdigest()
    try:
        refresh.verify_payload_binding(changed_pinned_policy)
    except refresh.ModelError:
        pass
    else:
        raise AssertionError("current policy was not bound to the reviewed commit")


def test_candidate_validator_rejects_circularly_rewritten_contracts() -> None:
    candidate = candidate_fixture()
    original = refresh.validate_source_lock
    refresh.validate_source_lock = lambda _source_lock: None
    try:
        refresh.validate_candidate(candidate)
        changed = copy.deepcopy(candidate)
        changed["observation"]["contracts"]["authentication"] = []
        changed["provider_lock"] = provider_lock(
            changed["source_lock"], changed["observation"]["contracts"]
        )
        try:
            refresh.validate_candidate(changed)
        except refresh.ModelError:
            pass
        else:
            raise AssertionError("circularly rewritten candidate was accepted")
    finally:
        refresh.validate_source_lock = original


def test_refresh_dates_and_payload_encoding_are_strict() -> None:
    try:
        refresh.source_template("a" * 40, "2026-99-99")
    except refresh.ModelError:
        pass
    else:
        raise AssertionError("nonexistent review date was accepted")

    candidate = candidate_fixture()
    malformed = copy.deepcopy(candidate)
    malformed["payloads"]["policy-current"] = "***"
    try:
        refresh.decode_candidate_payloads(
            malformed["payloads"], malformed["source_lock"]
        )
    except refresh.ModelError:
        pass
    else:
        raise AssertionError("malformed candidate base64 was accepted")

    try:
        refresh.unique_object([("format", "first"), ("format", "second")])
    except refresh.ModelError:
        pass
    else:
        raise AssertionError("duplicate candidate keys were accepted")


def main() -> None:
    tests = tuple(
        value
        for name, value in globals().items()
        if name.startswith("test_") and callable(value)
    )
    for test in tests:
        test()
    print(f"{len(tests)} crates.io semantic drift tests passed.")


if __name__ == "__main__":
    main()
