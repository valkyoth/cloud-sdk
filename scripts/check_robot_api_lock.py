#!/usr/bin/env python3
"""Validate the complete v0.74 Robot operation and protocol source lock."""

from __future__ import annotations

import argparse
from contextlib import contextmanager
import hashlib
from html import unescape
from html.parser import HTMLParser
import json
from pathlib import Path
import re
import signal
import ssl
import urllib.error
import urllib.request
from typing import Any, Iterator

ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / "tests" / "fixtures" / "robot-api" / "v0.74.0.json"
SOURCE_URL = "https://robot.hetzner.com/doc/webservice/en.html"
SOURCE_SHA256 = "4b396790acc449f47b2b3b893f8eff759c0c25196dc38b1e5e92a12c9704771a"
MAX_LOCK_BYTES = 256 * 1024
MAX_SOURCE_BYTES = 8 * 1024 * 1024
TIMEOUT_SECONDS = 60
TOTAL_FETCH_SECONDS = 90
OPERATIONS_POLICY_SHA256 = (
    "896e23812d536999ad0deb1509fec9a23"
    "f92eae28ca0a404e11063b3644a5d76"
)
ID = re.compile(r"^[a-z][a-z0-9]*(?:_[a-z0-9]+)*$")
HEADING = re.compile(r"^(GET|POST|PUT|DELETE) (/\S+)$")
DEPRECATION_MARKER = re.compile(
    r"@deprecated\s+(GET|POST|PUT|DELETE)\s+(/[^\s<]+)"
)

GROUPS = {
    "server": (3, "v0.78.0"),
    "cancellation": (9, "v0.79.0"),
    "ip": (6, "v0.80.0"),
    "subnet": (6, "v0.81.0"),
    "reset": (3, "v0.82.0"),
    "failover": (4, "v0.83.0"),
    "wol": (2, "v0.84.0"),
    "boot": (15, "v0.85.0"),
    "rdns": (5, "v0.86.0"),
    "traffic": (1, "v0.87.0"),
    "ssh_keys": (5, "v0.88.0"),
    "firewall": (8, "v0.89.0"),
    "vswitch": (7, "v0.90.0"),
    "ordering_catalog": (6, "v0.91.0"),
    "ordering_transaction": (6, "v0.92.0"),
    "ordering_mutation": (3, "v0.93.0"),
    "legacy_storage_box": (16, "excluded"),
}


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


class HeadingParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.in_h2 = False
        self.parts: list[str] = []
        self.headings: list[str] = []

    def handle_starttag(self, tag: str, _attrs: list[tuple[str, str | None]]) -> None:
        if tag == "h2":
            self.in_h2 = True
            self.parts = []

    def handle_data(self, data: str) -> None:
        if self.in_h2:
            self.parts.append(data)

    def handle_endtag(self, tag: str) -> None:
        if tag != "h2" or not self.in_h2:
            return
        heading = " ".join("".join(self.parts).split())
        if HEADING.fullmatch(heading):
            self.headings.append(heading)
        self.in_h2 = False
        self.parts = []


def fail(message: str) -> None:
    raise SystemExit(f"Robot API lock: {message}")


def require(condition: bool, message: str) -> None:
    if not condition:
        fail(message)


def read_lock() -> dict[str, Any]:
    try:
        with LOCK.open("rb") as stream:
            payload = stream.read(MAX_LOCK_BYTES + 1)
    except OSError as error:
        fail(f"could not read lock: {error}")
    require(len(payload) <= MAX_LOCK_BYTES, "lock exceeds 256 KiB")
    try:
        value = json.loads(payload)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        fail(f"lock is not valid UTF-8 JSON: {error}")
    require(isinstance(value, dict), "lock root must be an object")
    return value


def expected_protocol() -> dict[str, Any]:
    return {
        "origin": "https://robot-ws.your-server.de",
        "https_only": True,
        "authentication": "http-basic",
        "authentication_rejection": {
            "status": 401,
            "failed_attempt_limit": 3,
            "source_ip_lockout_seconds": 600,
            "automatic_retry": False,
        },
        "post_content_type": "application/x-www-form-urlencoded",
        "response_content_type": "application/json",
        "success_statuses": [200, 201],
        "invalid_input": {
            "status": 400,
            "code": "INVALID_INPUT",
            "nullable_fields": ["missing", "invalid"],
        },
        "quota": {
            "status": 403,
            "code": "RATE_LIMIT_EXCEEDED",
            "integer_fields": ["max_request", "interval"],
        },
        "maintenance_status": 503,
        "empty_success_body": True,
        "alternate_yaml_format": "excluded",
    }


def validate_metadata(lock: dict[str, Any]) -> None:
    require(
        set(lock)
        == {
            "schema_version",
            "source",
            "wire_source_lock",
            "inventory",
            "protocol",
            "deprecation_policy",
            "operations",
        },
        "top-level fields changed",
    )
    require(lock.get("schema_version") == 1, "unexpected schema version")
    require(
        lock.get("source")
        == {
            "retrieved": "2026-08-10",
            "url": SOURCE_URL,
            "sha256": SOURCE_SHA256,
        },
        "source identity changed without review",
    )
    require(
        lock.get("wire_source_lock")
        == "tests/fixtures/robot-wire/v0.42.0.json",
        "wire source-lock relationship changed",
    )
    require(
        lock.get("inventory") == {"total": 105, "active": 89, "deprecated": 16},
        "inventory totals changed",
    )
    require(lock.get("protocol") == expected_protocol(), "protocol policy changed")
    require(
        lock.get("deprecation_policy")
        == {
            "excluded_prefix": "/storagebox",
            "replacement": "Hetzner Console Storage Box API",
            "deprecated_route_aliases": "excluded",
        },
        "deprecation policy changed",
    )


def operations(lock: dict[str, Any]) -> list[dict[str, str]]:
    values = lock.get("operations")
    require(isinstance(values, list), "operations must be an array")
    result: list[dict[str, str]] = []
    for value in values:
        require(isinstance(value, dict), "operation must be an object")
        require(
            set(value) == {"id", "group", "method", "path", "status", "milestone"},
            "operation fields changed",
        )
        require(
            all(isinstance(item, str) for item in value.values()),
            "operation values must be text",
        )
        result.append(value)
    return result


def validate_operations(lock: dict[str, Any]) -> None:
    values = operations(lock)
    require(len(values) == 105, "expected 105 operation headings")
    ids: set[str] = set()
    routes: set[tuple[str, str]] = set()
    group_counts = {name: 0 for name in GROUPS}
    active = 0
    deprecated = 0
    for operation in values:
        operation_id = operation["id"]
        route = (operation["method"], operation["path"])
        require(ID.fullmatch(operation_id) is not None, f"invalid operation id {operation_id}")
        require(operation_id not in ids, f"duplicate operation id {operation_id}")
        require(route not in routes, f"duplicate operation route {route}")
        require(HEADING.fullmatch(f"{route[0]} {route[1]}") is not None, f"invalid route {route}")
        require(operation["group"] in GROUPS, f"unknown group {operation['group']}")
        expected_milestone = GROUPS[operation["group"]][1]
        require(operation["milestone"] == expected_milestone, f"wrong milestone for {operation_id}")
        is_storage = operation["path"].startswith("/storagebox")
        require(is_storage == (operation["group"] == "legacy_storage_box"), "storage group mismatch")
        expected_status = "deprecated" if is_storage else "active"
        require(operation["status"] == expected_status, f"wrong status for {operation_id}")
        active += operation["status"] == "active"
        deprecated += operation["status"] == "deprecated"
        group_counts[operation["group"]] += 1
        ids.add(operation_id)
        routes.add(route)
    require((active, deprecated) == (89, 16), "active/deprecated counts changed")
    expected_counts = {name: value[0] for name, value in GROUPS.items()}
    require(group_counts == expected_counts, "operation group counts changed")
    canonical = json.dumps(
        values,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
    ).encode("utf-8")
    actual = hashlib.sha256(canonical).hexdigest()
    require(
        actual == OPERATIONS_POLICY_SHA256,
        f"reviewed operation policy changed to {actual}",
    )


def extract_headings(payload: bytes) -> list[str]:
    try:
        text = payload.decode("utf-8")
    except UnicodeDecodeError as error:
        fail(f"official source is not UTF-8: {error}")
    parser = HeadingParser()
    parser.feed(text)
    parser.close()
    return parser.headings


def validate_source(lock: dict[str, Any], payload: bytes) -> None:
    actual = hashlib.sha256(payload).hexdigest()
    require(actual == SOURCE_SHA256, f"source digest changed to {actual}")
    values = operations(lock)
    locked = [f"{value['method']} {value['path']}" for value in values]
    require(extract_headings(payload) == locked, "official operation headings changed")
    source_text = unescape(payload.decode("utf-8"))
    markers = set(DEPRECATION_MARKER.findall(source_text))
    for operation in values:
        if operation["status"] == "deprecated":
            marker = (operation["method"], operation["path"])
            display = f"@deprecated {marker[0]} {marker[1]}"
            require(marker in markers, f"missing upstream deprecation marker {display}")


@contextmanager
def fetch_deadline() -> Iterator[None]:
    require(
        hasattr(signal, "SIGALRM")
        and hasattr(signal, "ITIMER_REAL")
        and hasattr(signal, "setitimer"),
        "hard fetch deadline is unavailable on this platform",
    )

    def expired(_signum: int, _frame: Any) -> None:
        raise TimeoutError("Robot source fetch exceeded total deadline")

    try:
        previous = signal.signal(signal.SIGALRM, expired)
    except (OSError, ValueError) as error:
        fail(f"could not arm hard fetch deadline: {error}")
    try:
        signal.setitimer(signal.ITIMER_REAL, TOTAL_FETCH_SECONDS)
    except (OSError, ValueError) as error:
        signal.signal(signal.SIGALRM, previous)
        fail(f"could not arm hard fetch deadline: {error}")
    try:
        yield
    finally:
        try:
            signal.setitimer(signal.ITIMER_REAL, 0)
        finally:
            signal.signal(signal.SIGALRM, previous)


def fetch_source() -> bytes:
    opener = urllib.request.build_opener(
        RejectRedirects(), urllib.request.HTTPSHandler(context=ssl.create_default_context())
    )
    request = urllib.request.Request(
        SOURCE_URL,
        headers={"User-Agent": "cloud-sdk-robot-api-lock/0.74"},
    )
    try:
        with fetch_deadline():
            with opener.open(request, timeout=TIMEOUT_SECONDS) as response:
                require(response.geturl() == SOURCE_URL, "source redirected")
                payload = response.read(MAX_SOURCE_BYTES + 1)
    except (OSError, TimeoutError, urllib.error.URLError) as error:
        fail(f"could not fetch official source: {error}")
    require(len(payload) <= MAX_SOURCE_BYTES, "official source exceeds 8 MiB")
    return payload


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fetch", action="store_true")
    args = parser.parse_args()
    lock = read_lock()
    validate_metadata(lock)
    validate_operations(lock)
    require(LOCK.is_relative_to(ROOT / "tests" / "fixtures"), "lock entered a publishable crate")
    if args.fetch:
        validate_source(lock, fetch_source())
    print("Robot API lock: 89 active and 16 deprecated operations passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
