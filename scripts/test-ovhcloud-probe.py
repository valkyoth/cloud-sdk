#!/usr/bin/env python3
"""Regression tests for the OVHcloud v2 probe source lock."""

from __future__ import annotations

import hashlib
import json
import string

from check_ovhcloud_probe import (
    ROOT,
    check_evidence,
    ensure_no_probe_packages,
    read_inventory,
)
from ovhcloud_probe_adapter import (
    CANDIDATE_PATHS,
    OvhcloudProbeError,
    build_observation,
    source_digest,
)
from provider_drift_model import read_bounded_json, validate_observation
import provider_drift_fetch as fetch


def operation(path: str) -> dict:
    collection = not path.endswith("}")
    headers = (
        [
            {"name": "X-Pagination-Cursor", "paramType": "header"},
            {"name": "X-Pagination-Size", "paramType": "header"},
        ]
        if collection
        else []
    )
    response = "fixture.Collection[]" if collection else "fixture.Resource"
    return {
        "apiStatus": {"value": "PRODUCTION"},
        "httpMethod": "GET",
        "iamActions": [{"name": "fixture:get"}],
        "noAuthentication": False,
        "parameters": headers,
        "responseType": response,
    }


def payloads() -> dict[str, bytes]:
    index = {
        "apis": [
            {
                "description": "",
                "format": ["json", "yaml"],
                "path": "/iam",
                "schema": "{path}.{format}",
            }
        ],
        "basePath": "https://api.eu.ovhcloud.com/v2",
    }
    schema = {
        "apiVersion": "1.0",
        "apis": [
            {"operations": [operation(path)], "path": path}
            for path in CANDIDATE_PATHS
        ],
        "basePath": "https://api.eu.ovhcloud.com/v2",
        "models": {"fixture.Resource": {"id": "Resource"}},
    }
    principles = "\n".join(
        (
            "X-Schemas-Version",
            "https://eu.api.ovh.com/v2/iam/policy",
            "X-Pagination-Size",
            "X-Pagination-Cursor-Next",
            "X-Pagination-Cursor",
            "a route `/task`",
            "path `/event`",
        )
    )
    oauth = "\n".join(
        (
            "OAuth2 *client credentials* flow",
            "https://www.ovh.com/auth/oauth2/token",
            "https://ca.ovh.com/auth/oauth2/token",
            '"access_token"',
            '"token_type":"Bearer"',
            '"expires_in":3599',
            "https://{eu|ca}.api.ovh.com/v1/",
        )
    )
    return {
        "api-index": json.dumps(index).encode("utf-8"),
        "api-v2-principles": principles.encode("utf-8"),
        "iam-schema": json.dumps(schema).encode("utf-8"),
        "oauth2-service-account": oauth.encode("utf-8"),
    }


def lock() -> dict:
    value = read_bounded_json(
        ROOT / "provider-drift/providers/ovhcloud-v2-probe.lock.json", "lock"
    )
    return value


def assert_rejected(source_payloads: dict[str, bytes]) -> None:
    try:
        build_observation(lock(), source_payloads)
    except OvhcloudProbeError:
        return
    raise AssertionError("malformed OVHcloud probe source was accepted")


def test_current_evidence_and_inventory_are_consistent() -> None:
    check_evidence()
    rows = read_inventory(ROOT / "provider-probes/ovhcloud-v2/CANDIDATES.tsv")
    assert len(rows) == 8
    assert all(row["method"] == "GET" for row in rows)


def test_synthetic_sources_construct_every_category() -> None:
    observation = validate_observation(build_observation(lock(), payloads()))
    assert set(observation["contracts"]) == {
        "authentication",
        "cost",
        "endpoints",
        "headers",
        "idempotency",
        "operations",
        "pagination",
        "retry",
        "schemas",
    }
    assert len(observation["contracts"]["operations"]) == 8


def test_missing_oauth_and_task_contracts_fail_closed() -> None:
    for source, marker in (
        ("oauth2-service-account", b"expires_in"),
        ("api-v2-principles", b"/task"),
    ):
        changed = payloads()
        changed[source] = changed[source].replace(marker, b"removed", 1)
        assert_rejected(changed)


def test_candidate_stability_authentication_and_method_fail_closed() -> None:
    for old, new in (
        (b'"PRODUCTION"', b'"BETA"'),
        (b'"noAuthentication": false', b'"noAuthentication": true'),
        (b'"httpMethod": "GET"', b'"httpMethod": "POST"'),
    ):
        changed = payloads()
        changed["iam-schema"] = changed["iam-schema"].replace(old, new, 1)
        assert_rejected(changed)


def test_duplicate_json_and_authority_substitution_fail_closed() -> None:
    changed = payloads()
    changed["api-index"] = changed["api-index"].replace(
        b'{"apis":', b'{"apis": [], "apis":', 1
    )
    assert_rejected(changed)
    changed = payloads()
    changed["iam-schema"] = changed["iam-schema"].replace(
        b"api.eu.ovhcloud.com", b"attacker.example", 1
    )
    assert_rejected(changed)


def test_every_source_changes_the_observation() -> None:
    baseline = build_observation(lock(), payloads())
    changed_payloads = payloads()
    schema = json.loads(changed_payloads["iam-schema"])
    schema["models"]["fixture.Other"] = {"id": "Other"}
    changed_payloads["iam-schema"] = json.dumps(schema).encode("utf-8")
    changed = build_observation(lock(), changed_payloads)
    assert baseline["contracts"]["schemas"] != changed["contracts"]["schemas"]
    assert baseline["sources"] != changed["sources"]


def test_iam_source_integrity_normalizes_only_unique_path_order() -> None:
    first = payloads()["iam-schema"]
    value = json.loads(first)
    value["apis"].reverse()
    reordered = json.dumps(value).encode("utf-8")
    assert source_digest("iam-schema", first) == source_digest(
        "iam-schema", reordered
    )
    value["apis"][0]["description"] = "semantic drift"
    changed = json.dumps(value).encode("utf-8")
    assert source_digest("iam-schema", first) != source_digest(
        "iam-schema", changed
    )
    value = json.loads(first)
    value["apis"].append(value["apis"][0])
    duplicate = json.dumps(value).encode("utf-8")
    try:
        source_digest("iam-schema", duplicate)
    except OvhcloudProbeError:
        pass
    else:
        raise AssertionError("duplicate OVHcloud IAM paths were accepted")


def test_public_source_integrity_uses_exact_sha256() -> None:
    public_schema = b'{"password":"documented request field","id":"schema field"}'
    assert source_digest("api-index", public_schema) == hashlib.sha256(
        public_schema
    ).hexdigest()


def test_json_sources_require_utf8_finite_standard_values() -> None:
    invalid = (
        '{"apis":[],"value":NaN}'.encode("utf-8"),
        '{"apis":[],"value":Infinity}'.encode("utf-8"),
        '{"apis":[],"value":-Infinity}'.encode("utf-8"),
        '{"apis":[],"value":1e999}'.encode("utf-8"),
        '{"apis":[]}'.encode("utf-16"),
    )
    for payload in invalid:
        try:
            source_digest("iam-schema", payload)
        except OvhcloudProbeError:
            pass
        else:
            raise AssertionError("non-strict OVHcloud JSON was accepted")


def test_probe_package_is_rejected_from_every_publication_boundary() -> None:
    ensure_no_probe_packages(
        {"cloud-sdk"}, {"cloud-sdk", "cloud-sdk-hetzner"}, ("cloud-sdk",)
    )
    for position in range(3):
        values = [set(), set(), tuple()]
        values[position] = (
            ("cloud-sdk-ovhcloud",)
            if position == 2
            else {"cloud-sdk-ovhcloud"}
        )
        try:
            ensure_no_probe_packages(values[0], values[1], values[2])
        except ValueError:
            continue
        raise AssertionError("OVHcloud probe package entered a publication boundary")


def test_every_remote_source_has_one_exact_reviewed_endpoint() -> None:
    def resolver(_host: str, _port: int, **_kwargs):
        return [
            (
                fetch.socket.AF_INET,
                fetch.socket.SOCK_STREAM,
                6,
                "",
                ("93.184.216.34", 443),
            )
        ]

    for source in lock()["sources"]:
        target = fetch.validate_network_target(
            "ovhcloud-v2-probe", source, resolver=resolver
        )
        assert target.addresses == (
            (
                fetch.socket.AF_INET,
                fetch.socket.SOCK_STREAM,
                6,
                ("93.184.216.34", 443),
            ),
        )
        if target.host == "raw.githubusercontent.com":
            revision = source["url"].split("/")[5]
            assert len(revision) == 40
            assert all(character in string.hexdigits.lower() for character in revision)
        changed = dict(source)
        changed["url"] = "https://attacker.example/source"
        try:
            fetch.validate_network_target(
                "ovhcloud-v2-probe", changed, resolver=resolver
            )
        except fetch.FetchError:
            continue
        raise AssertionError("OVHcloud source endpoint substitution was accepted")


def main() -> None:
    tests = (
        test_current_evidence_and_inventory_are_consistent,
        test_synthetic_sources_construct_every_category,
        test_missing_oauth_and_task_contracts_fail_closed,
        test_candidate_stability_authentication_and_method_fail_closed,
        test_duplicate_json_and_authority_substitution_fail_closed,
        test_every_source_changes_the_observation,
        test_iam_source_integrity_normalizes_only_unique_path_order,
        test_json_sources_require_utf8_finite_standard_values,
        test_probe_package_is_rejected_from_every_publication_boundary,
        test_every_remote_source_has_one_exact_reviewed_endpoint,
    )
    for test in tests:
        test()
    print(f"{len(tests)} OVHcloud probe regression groups passed.")


if __name__ == "__main__":
    main()
