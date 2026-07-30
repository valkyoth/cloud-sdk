#!/usr/bin/env python3
"""Validate the credential-free v0.42 Robot protocol source lock."""

from __future__ import annotations

import argparse
import hashlib
import json
import ssl
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any
from urllib.parse import parse_qsl

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "tests" / "fixtures" / "robot-wire" / "v0.42.0.json"
SOURCE_URL = "https://robot.hetzner.com/doc/webservice/en.html"
SOURCE_SHA256 = "4b396790acc449f47b2b3b893f8eff759c0c25196dc38b1e5e92a12c9704771a"
MAX_SOURCE_BYTES = 8 * 1024 * 1024
TIMEOUT_SECONDS = 60


class RejectRedirects(urllib.request.HTTPRedirectHandler):
    def redirect_request(
        self,
        _request: Any,
        _file: Any,
        _code: int,
        _message: str,
        _headers: Any,
        _new_url: str,
    ) -> None:
        return None


def fail(message: str) -> None:
    raise SystemExit(f"Robot wire fixture: {message}")


def require(condition: bool, message: str) -> None:
    if not condition:
        fail(message)


def read_fixture() -> dict[str, Any]:
    try:
        payload = FIXTURE.read_bytes()
    except OSError as error:
        fail(f"could not read fixture: {error}")
    require(len(payload) <= 64 * 1024, "fixture exceeds 64 KiB")
    try:
        value = json.loads(payload)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        fail(f"fixture is not valid UTF-8 JSON: {error}")
    require(isinstance(value, dict), "fixture root must be an object")
    return value


def named(values: Any, label: str) -> dict[str, dict[str, Any]]:
    require(isinstance(values, list), f"{label} must be an array")
    result: dict[str, dict[str, Any]] = {}
    for value in values:
        require(isinstance(value, dict), f"{label} entry must be an object")
        name = value.get("name")
        require(isinstance(name, str) and name, f"{label} name must be text")
        require(name not in result, f"duplicate {label} name {name}")
        result[name] = value
    return result


def validate_protocol(fixture: dict[str, Any]) -> None:
    require(fixture.get("schema_version") == 1, "unexpected schema version")
    source = fixture.get("source")
    require(
        source
        == {
            "retrieved": "2026-07-30",
            "url": SOURCE_URL,
            "sha256": SOURCE_SHA256,
        },
        "source identity changed without a reviewed lock update",
    )
    protocol = fixture.get("protocol")
    require(isinstance(protocol, dict), "protocol must be an object")
    expected = {
        "base_url": "https://robot-ws.your-server.de",
        "authentication": "basic",
        "https_only": True,
        "post_content_type": "application/x-www-form-urlencoded",
        "response_content_type": "application/json",
        "success_statuses": [200, 201],
        "authentication_rejection_status": 401,
        "failed_login_limit": 3,
        "source_ip_lockout_seconds": 600,
        "quota_status": 403,
        "quota_code": "RATE_LIMIT_EXCEEDED",
        "maintenance_status": 503,
    }
    require(protocol == expected, "source-locked protocol facts changed")


def validate_requests(fixture: dict[str, Any]) -> None:
    requests = named(fixture.get("requests"), "request")
    require(
        set(requests)
        == {"single-server-read", "repeated-vswitch-server-mutation"},
        "request fixture set changed",
    )
    read = requests["single-server-read"]
    require(read.get("execute") is False, "read fixture must remain non-executing")
    require(read.get("method") == "GET", "read method changed")
    require(read.get("target") == "/server/321", "read target changed")
    require(read.get("body") == "", "read body must be empty")

    mutation = requests["repeated-vswitch-server-mutation"]
    require(
        mutation.get("execute") is False,
        "mutation fixture must remain non-executing",
    )
    require(mutation.get("method") == "POST", "mutation method changed")
    require(
        mutation.get("target") == "/vswitch/4321/server",
        "mutation target changed",
    )
    headers = mutation.get("headers")
    require(isinstance(headers, dict), "mutation headers must be an object")
    require(
        headers.get("content-type") == "application/x-www-form-urlencoded",
        "mutation content type changed",
    )
    pairs = parse_qsl(
        str(mutation.get("body", "")),
        keep_blank_values=True,
        strict_parsing=True,
    )
    require(
        pairs
        == [
            ("server[]", "123.123.123.123"),
            ("server[]", "123.123.123.124"),
        ],
        "repeated form fields or ordering changed",
    )
    for request in requests.values():
        headers = request.get("headers")
        require(isinstance(headers, dict), "request headers must be an object")
        forbidden = {"authorization", "cookie", "proxy-authorization"} & {
            str(name).lower() for name in headers
        }
        require(not forbidden, "fixture contains credential-bearing headers")


def validate_responses(fixture: dict[str, Any]) -> None:
    responses = named(fixture.get("responses"), "response")
    require(
        set(responses)
        == {
            "single-server-success",
            "general-error",
            "invalid-input-error",
            "authentication-rejection",
            "quota-error",
            "maintenance",
            "empty-success",
        },
        "response fixture set changed",
    )
    require(
        responses["single-server-success"].get("status") == 200,
        "read success status changed",
    )
    general = responses["general-error"]
    require(
        general.get("body", {}).get("error", {}).get("status")
        == general.get("status")
        == 404,
        "general error status is inconsistent",
    )
    invalid = responses["invalid-input-error"].get("body", {}).get("error", {})
    require(
        invalid.get("code") == "INVALID_INPUT"
        and isinstance(invalid.get("missing"), list)
        and invalid.get("invalid") is None,
        "invalid-input shape changed",
    )
    quota_response = responses["quota-error"]
    quota = quota_response.get("body", {}).get("error", {})
    require(
        quota_response.get("status") == quota.get("status") == 403
        and quota.get("code") == "RATE_LIMIT_EXCEEDED"
        and isinstance(quota.get("max_request"), int)
        and isinstance(quota.get("interval"), int),
        "quota error shape changed",
    )
    require(
        responses["authentication-rejection"]
        == {"name": "authentication-rejection", "status": 401, "body": None},
        "authentication rejection fixture changed",
    )
    require(
        responses["maintenance"]
        == {"name": "maintenance", "status": 503, "body": None},
        "maintenance fixture changed",
    )
    require(
        responses["empty-success"]
        == {"name": "empty-success", "status": 200, "body": ""},
        "empty-success fixture changed",
    )


def fetch_source() -> bytes:
    opener = urllib.request.build_opener(
        RejectRedirects(), urllib.request.HTTPSHandler(context=ssl.create_default_context())
    )
    request = urllib.request.Request(
        SOURCE_URL,
        headers={"User-Agent": "cloud-sdk-robot-source-lock/0.42"},
    )
    try:
        with opener.open(request, timeout=TIMEOUT_SECONDS) as response:
            require(response.geturl() == SOURCE_URL, "source redirected")
            payload = response.read(MAX_SOURCE_BYTES + 1)
    except (OSError, urllib.error.URLError) as error:
        fail(f"could not fetch official source: {error}")
    require(len(payload) <= MAX_SOURCE_BYTES, "official source exceeds 8 MiB")
    return payload


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--fetch",
        action="store_true",
        help="verify the current official document against the reviewed digest",
    )
    args = parser.parse_args()
    fixture = read_fixture()
    validate_protocol(fixture)
    validate_requests(fixture)
    validate_responses(fixture)
    require(
        FIXTURE.is_relative_to(ROOT / "tests" / "fixtures"),
        "fixture entered a publishable crate",
    )
    if args.fetch:
        actual = hashlib.sha256(fetch_source()).hexdigest()
        require(actual == SOURCE_SHA256, f"source digest changed to {actual}")
    print("Robot wire fixture: 2 requests and 7 responses passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
