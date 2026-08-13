#!/usr/bin/env python3
"""Validate the v0.85 Robot boot source lock and implementation policy."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / "tests/fixtures/robot-boot/v0.85.0.json"
LOCK_SHA256 = "1345e41767116eedb9406df2520454e1d5c54ec9712b3ddd5df6173acef74e64"
SOURCE_SHA256 = "4b396790acc449f47b2b3b893f8eff759c0c25196dc38b1e5e92a12c9704771a"
MAX_LOCK_BYTES = 16 * 1024
BOOT_SOURCE = ROOT / "crates/cloud-sdk-hetzner/src/robot/boot"

OPERATIONS = [
    ("robot_get_boot", "GET", "/boot/{server-number}", "all", "current", []),
    ("robot_get_rescue", "GET", "/boot/{server-number}/rescue", "rescue", "current", []),
    ("robot_activate_rescue", "POST", "/boot/{server-number}/rescue", "rescue", "active", ["os", "authorized_key[]", "keyboard"]),
    ("robot_deactivate_rescue", "DELETE", "/boot/{server-number}/rescue", "rescue", "inactive", []),
    ("robot_get_last_rescue", "GET", "/boot/{server-number}/rescue/last", "rescue", "last", []),
    ("robot_get_linux", "GET", "/boot/{server-number}/linux", "linux", "current", []),
    ("robot_activate_linux", "POST", "/boot/{server-number}/linux", "linux", "active", ["dist", "lang", "authorized_key[]"]),
    ("robot_deactivate_linux", "DELETE", "/boot/{server-number}/linux", "linux", "inactive", []),
    ("robot_get_last_linux", "GET", "/boot/{server-number}/linux/last", "linux", "last", []),
    ("robot_get_vnc", "GET", "/boot/{server-number}/vnc", "vnc", "current", []),
    ("robot_activate_vnc", "POST", "/boot/{server-number}/vnc", "vnc", "active", ["dist", "lang"]),
    ("robot_deactivate_vnc", "DELETE", "/boot/{server-number}/vnc", "vnc", "inactive", []),
    ("robot_get_windows", "GET", "/boot/{server-number}/windows", "windows", "current", []),
    ("robot_activate_windows", "POST", "/boot/{server-number}/windows", "windows", "active", ["lang", "os"]),
    ("robot_deactivate_windows", "DELETE", "/boot/{server-number}/windows", "windows", "inactive", []),
]

FAMILIES = {
    "rescue": ("os", False, True, True, True, ["os", "authorized_key[]", "keyboard"], ["arch"]),
    "linux": ("dist", True, True, True, True, ["dist", "lang", "authorized_key[]"], ["arch"]),
    "vnc": ("dist", True, False, False, False, ["dist", "lang"], ["arch"]),
    "windows": ("os", True, False, False, False, ["lang", "os"], ["arch", "dist"]),
}

POLICY = {
    "canonical_server_number_route": True,
    "deprecated_server_ip_routes": "excluded",
    "deprecated_request_fields": "excluded",
    "deprecated_response_fields": "validated-then-discarded",
    "unknown_fields": "reject",
    "response_identity": "exact-request-number-and-canonical-address-families",
    "response_body_bytes": 1048576,
    "selector_bytes": 256,
    "key_bytes": 16384,
    "maximum_options": 256,
    "maximum_authorized_keys": 64,
    "generated_passwords_and_keys": "protected-owned-storage",
    "mutation_retry": "never",
    "linux_vnc_windows_activation": "destructive",
    "windows_activation_warning": "reboot-starts-installation-and-deletes-all-server-data",
}

ERRORS = {
    "robot_get_boot": [[404, "SERVER_NOT_FOUND"], [404, "BOOT_NOT_AVAILABLE"]],
    "robot_get_rescue": [[404, "SERVER_NOT_FOUND"], [404, "BOOT_NOT_AVAILABLE"]],
    "robot_activate_rescue": [[400, "INVALID_INPUT"], [404, "SERVER_NOT_FOUND"], [404, "BOOT_NOT_AVAILABLE"], [500, "BOOT_ACTIVATION_FAILED"]],
    "robot_deactivate_rescue": [[404, "SERVER_NOT_FOUND"], [404, "BOOT_NOT_AVAILABLE"], [500, "BOOT_DEACTIVATION_FAILED"]],
    "robot_get_last_rescue": [[404, "SERVER_NOT_FOUND"], [404, "BOOT_NOT_AVAILABLE"]],
    "robot_get_linux": [[404, "SERVER_NOT_FOUND"], [404, "BOOT_NOT_AVAILABLE"]],
    "robot_activate_linux": [[400, "INVALID_INPUT"], [404, "SERVER_NOT_FOUND"], [404, "BOOT_NOT_AVAILABLE"], [500, "BOOT_ACTIVATION_FAILED"]],
    "robot_deactivate_linux": [[404, "SERVER_NOT_FOUND"], [404, "BOOT_NOT_AVAILABLE"], [500, "BOOT_DEACTIVATION_FAILED"]],
    "robot_get_last_linux": [[404, "SERVER_NOT_FOUND"], [404, "BOOT_NOT_AVAILABLE"]],
    "robot_get_vnc": [[404, "SERVER_NOT_FOUND"], [404, "BOOT_NOT_AVAILABLE"]],
    "robot_activate_vnc": [[400, "INVALID_INPUT"], [404, "SERVER_NOT_FOUND"], [404, "BOOT_NOT_AVAILABLE"], [500, "BOOT_ACTIVATION_FAILED"]],
    "robot_deactivate_vnc": [[404, "SERVER_NOT_FOUND"], [404, "BOOT_NOT_AVAILABLE"], [500, "BOOT_DEACTIVATION_FAILED"]],
    "robot_get_windows": [[404, "SERVER_NOT_FOUND"], [404, "BOOT_NOT_AVAILABLE"], [404, "WINDOWS_OUTDATED_VERSION"]],
    "robot_activate_windows": [[400, "INVALID_INPUT"], [404, "SERVER_NOT_FOUND"], [404, "BOOT_NOT_AVAILABLE"], [404, "WINDOWS_MISSING_ADDON"], [404, "WINDOWS_OUTDATED_VERSION"], [500, "BOOT_ACTIVATION_FAILED"]],
    "robot_deactivate_windows": [[404, "SERVER_NOT_FOUND"], [404, "BOOT_NOT_AVAILABLE"], [404, "WINDOWS_OUTDATED_VERSION"], [500, "BOOT_DEACTIVATION_FAILED"]],
}


def fail(message: str) -> None:
    raise SystemExit(f"Robot boot contract: {message}")


def require(condition: bool, message: str) -> None:
    if not condition:
        fail(message)


def read_lock() -> tuple[dict[str, Any], bytes]:
    try:
        payload = LOCK.read_bytes()
    except OSError as error:
        fail(f"could not read fixture: {error}")
    require(len(payload) <= MAX_LOCK_BYTES, "fixture exceeds 16 KiB")
    try:
        value = json.loads(payload)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        fail(f"fixture is invalid UTF-8 JSON: {error}")
    require(isinstance(value, dict), "fixture root must be an object")
    return value, payload


def validate_contract(value: dict[str, Any]) -> None:
    require(set(value) == {"schema_version", "source", "identity_fields", "families", "operations", "quota", "policy"}, "top-level fields changed")
    require(value["schema_version"] == 1, "schema version changed")
    require(value["source"] == {"retrieved": "2026-08-13", "url": "https://robot.hetzner.com/doc/webservice/en.html", "sha256": SOURCE_SHA256}, "source identity changed")
    require(value["identity_fields"] == ["server_ip", "server_ipv6_net", "server_number"], "identity fields changed")
    require(value["quota"] == {"requests": 500, "seconds": 3600}, "quota changed")
    require(value["policy"] == POLICY, "security policy changed")
    families = value.get("families")
    require(isinstance(families, dict) and set(families) == set(FAMILIES), "families changed")
    fields = ("primary", "languages", "authorized_key", "host_key", "last", "activation_fields", "deprecated_fields")
    for name, expected in FAMILIES.items():
        require(set(families[name]) == set(fields), f"{name} fields changed")
        require(tuple(families[name][field] for field in fields) == expected, f"{name} contract changed")
    operations = value.get("operations")
    require(isinstance(operations, list) and len(operations) == 15, "expected 15 operations")
    for item, expected in zip(operations, OPERATIONS, strict=True):
        require(set(item) == {"id", "method", "path", "family", "state", "input", "errors"}, "operation fields changed")
        observed = tuple(item[field] for field in ("id", "method", "path", "family", "state", "input"))
        require(observed == expected, f"operation changed: {expected[0]}")
        require(item["errors"] == ERRORS[expected[0]], f"operation errors changed: {expected[0]}")
    ids = [item["id"] for item in operations]
    require(len(ids) == len(set(ids)), "duplicate operation id")
    require(all("{server-ip}" not in item["path"] for item in operations), "deprecated IP route admitted")


def validate_implementation_policy() -> None:
    sources = {path.name: path.read_text(encoding="utf-8") for path in BOOT_SOURCE.glob("*.rs")}
    required = {
        "prepare.rs": ["/boot/", "OperationImpact::Destructive", "RetryEligibility::Never", "authorized_key[]", "RequestBodySensitivity::Sensitive"],
        "decode.rs": ["MAX_ROBOT_BOOT_KEY_BYTES", "ResponseIdentityMismatch", "reject_duplicate_secrets", "@deprecated arch"],
        "exchange.rs": ["require_active_choice", "require_inactive", "RobotWindowsActivateRequest"],
        "failure.rs": ["BOOT_ACTIVATION_FAILED", "BOOT_DEACTIVATION_FAILED", "WINDOWS_MISSING_ADDON", "WINDOWS_OUTDATED_VERSION"],
        "model.rs": ["RobotBootSecret", "try_with_secret", "RobotBootChoice"],
        "request.rs": ["TooManyAuthorizedKeys", "DuplicateAuthorizedKey", "ROBOT_BOOT_QUOTA", "max_requests: 500", "RobotWindowsActivateRequest"],
    }
    for name, tokens in required.items():
        for token in tokens:
            require(token in sources[name], f"implementation lost {name}: {token}")
    combined = "\n".join(sources.values())
    for forbidden in ["{server-ip}", '"arch"', "RetryEligibility::Always"]:
        require(forbidden not in combined, f"deprecated or unsafe request token admitted: {forbidden}")


def main() -> None:
    value, payload = read_lock()
    require(hashlib.sha256(payload).hexdigest() == LOCK_SHA256, "fixture digest changed")
    validate_contract(value)
    validate_implementation_policy()
    print("15 Robot boot operations and 7 source/security policy groups passed.")


if __name__ == "__main__":
    main()
