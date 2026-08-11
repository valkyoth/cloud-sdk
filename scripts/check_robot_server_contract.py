#!/usr/bin/env python3
"""Validate the v0.78 Robot server source and implementation contract."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "tests/fixtures/robot-server/v0.78.0.json"
API_LOCK = ROOT / "tests/fixtures/robot-api/v0.74.0.json"
MAX_BYTES = 64 * 1024
SOURCE = {
    "url": "https://robot.hetzner.com/doc/webservice/en.html",
    "sha256": "4b396790acc449f47b2b3b893f8eff759c0c25196dc38b1e5e92a12c9704771a",
}
OPERATIONS = [
    {"id": "list_servers", "method": "GET", "path": "/server", "success": 200, "shape": "list"},
    {"id": "get_server", "method": "GET", "path": "/server/{server-number}", "success": 200, "shape": "detail"},
    {"id": "update_server", "method": "POST", "path": "/server/{server-number}", "success": 200, "shape": "detail"},
]
SUMMARY = [
    "server_ip", "server_ipv6_net", "server_number", "server_name", "product", "dc",
    "traffic", "status", "cancelled", "paid_until", "ip", "subnet",
]
DETAIL = ["reset", "rescue", "vnc", "windows", "plesk", "cpanel", "wol", "hot_swap"]


def fail(message: str) -> None:
    raise SystemExit(f"Robot server contract: {message}")


def require(condition: bool, message: str) -> None:
    if not condition:
        fail(message)


def read_json(path: Path, limit: int = MAX_BYTES) -> dict[str, Any]:
    try:
        payload = path.read_bytes()
    except OSError as error:
        fail(f"could not read {path.relative_to(ROOT)}: {error}")
    require(len(payload) <= limit, f"{path.name} exceeds its byte limit")
    try:
        value = json.loads(payload)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        fail(f"{path.name} is invalid JSON: {error}")
    require(isinstance(value, dict), f"{path.name} root must be an object")
    return value


def validate_fixture(value: dict[str, Any]) -> None:
    require(set(value) == {
        "schema_version", "source", "operations", "summary_fields", "detail_fields",
        "optional_detail_fields", "status_values", "nullable_fields", "update_form_fields",
        "identity", "deprecated_aliases",
    }, "fixture fields changed")
    require(value["schema_version"] == 1, "schema version changed")
    require(value["source"] == SOURCE, "source identity changed")
    require(value["operations"] == OPERATIONS, "operation contract changed")
    require(value["summary_fields"] == SUMMARY, "summary fields changed")
    require(value["detail_fields"] == DETAIL, "detail fields changed")
    require(value["optional_detail_fields"] == ["linked_storagebox"], "optional detail fields changed")
    require(value["status_values"] == ["ready", "in process"], "status values changed")
    require(value["nullable_fields"] == ["subnet"], "nullable fields changed")
    require(value["update_form_fields"] == ["server_name"], "update form changed")
    require(value["identity"] == "positive-server-number", "canonical identity changed")
    require(value["deprecated_aliases"] == [
        "GET /server/{server-ip}", "POST /server/{server-ip}"
    ], "deprecated alias policy changed")


def validate_api_relationship(value: dict[str, Any], api: dict[str, Any]) -> None:
    require(api.get("source", {}).get("sha256") == value["source"]["sha256"], "API source digest differs")
    rows = [row for row in api.get("operations", []) if row.get("group") == "server"]
    expected = [(item["id"], item["method"], item["path"]) for item in OPERATIONS]
    actual = [(row.get("id"), row.get("method"), row.get("path")) for row in rows]
    require(actual == expected, "server operations differ from complete API lock")
    require(all(row.get("milestone") == "v0.78.0" for row in rows), "server milestone changed")


def validate_implementation() -> None:
    request = (ROOT / "crates/cloud-sdk-hetzner/src/robot/server/request.rs").read_text(encoding="ascii")
    decoder = (ROOT / "crates/cloud-sdk-hetzner/src/robot/server/decode.rs").read_text(encoding="ascii")
    model = (ROOT / "crates/cloud-sdk-hetzner/src/robot/server/model.rs").read_text(encoding="ascii")
    protected = (ROOT / "crates/cloud-sdk-hetzner/src/robot/server/protected.rs").read_text(encoding="ascii")
    protected_parse = (ROOT / "crates/cloud-sdk-hetzner/src/robot/server/protected_parse.rs").read_text(encoding="ascii")
    identity = (ROOT / "crates/cloud-sdk-hetzner/src/robot/server/identity.rs").read_text(encoding="ascii")
    duplicates = (ROOT / "crates/cloud-sdk-hetzner/src/robot/server/duplicates.rs").read_text(encoding="ascii")
    strict_json = (ROOT / "crates/cloud-sdk-hetzner/src/serde/strict_json.rs").read_text(encoding="utf-8")
    ip_fuzz = (ROOT / "fuzz/fuzz_targets/robot_ip_parser.rs").read_text(encoding="ascii")
    require('write_str(output, &mut len, "/server"' in request, "canonical server path is absent")
    require('RobotFormField::public("server_name"' in request, "rename form is absent")
    require("server-ip" not in request, "deprecated IP alias entered request code")
    for status in ['"ready"', '"in process"']:
        require(status in decoder, f"missing status decoder {status}")
    require("ResponseIdentityMismatch" in decoder, "detail identity binding is absent")
    require("RobotServerSubnet::parse" in decoder, "protected subnet parsing is absent")
    require("reject_duplicates_by(&servers" in decoder, "sorted server duplicate check is absent")
    require("reject_duplicates(&result" in decoder, "sorted topology duplicate check is absent")
    require("pub struct ProtectedIpAddr(SecretBoxBytes)" in protected,
            "stable protected address owner is absent")
    require("pub struct RobotServerNumber(SecretBoxBytes)" in identity,
            "stable protected server identity is absent")
    require("from_decimal_bytes" in identity, "protected decimal identity admission is absent")
    require("with_decimal_bytes" in request and "write_u64" not in request,
            "request identity is reconstructed as an ordinary scalar")
    require("struct Octets" in protected_parse and "struct Segments" in protected_parse,
            "clear-on-drop topology scratch is absent")
    require("left.as_bytes().contains(&b'.')" in protected_parse,
            "compressed IPv6 accepts an embedded IPv4 tail before the compression marker")
    require("canonical_network" in protected_parse, "subnet canonicalization is absent")
    require("enum ProtectedValueError" in protected
            and "Invalid," in protected
            and "Allocation," in protected,
            "protected value failure classes are collapsed")
    require("map_protected_error" in decoder
            and "ProtectedValueError::Allocation => RobotServerDecodeError::Allocation" in decoder,
            "protected allocation failures lose their decoder classification")
    require("SecretBoxBytes::try_zeroed(1, 1)" in protected
            and "SecretBoxBytes::try_zeroed(8, 8)" in protected
            and "with_secret_mut" in protected
            and "let mut value = 0_u8" not in protected,
            "protected Boolean transfer uses an ordinary scalar temporary")
    require("Bool(ProtectedBoolean)" in strict_json and "value: T" not in strict_json,
            "strict JSON retains ordinary scalar payloads")
    require("required_u64" not in decoder and ".as_bool()" not in decoder,
            "Robot decoding extracts classified ordinary scalars")
    require("Vec<usize>" in duplicates, "public index duplicate scratch is absent")
    require("identity_key" not in protected + identity + duplicates,
            "copied classified identity keys re-entered server decoding")
    require("IpAddr::from_str(candidate)" in ip_fuzz
            and "AddressField::Ipv4" in ip_fuzz
            and "AddressField::Ipv6" in ip_fuzz
            and "request.decode_response(checked).ok()?" in ip_fuzz
            and "summary.with_main_ipv4" in ip_fuzz
            and "summary.with_main_ipv6_network" in ip_fuzz,
            "public-decoder differential IP fuzzing is absent")
    require("RobotServerSummary([redacted])" in model, "summary diagnostics are not redacted")


def main() -> None:
    value = read_json(FIXTURE)
    validate_fixture(value)
    validate_api_relationship(value, read_json(API_LOCK, 256 * 1024))
    validate_implementation()
    print("3 Robot server operations and their source-locked fields passed.")


if __name__ == "__main__":
    main()
