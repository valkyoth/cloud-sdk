#!/usr/bin/env python3
"""Strict bounded models for provider-generic drift evidence."""

from __future__ import annotations

import hashlib
import json
import os
import re
import stat
from pathlib import Path
from typing import Any
from urllib.parse import urlsplit


LOCK_FORMAT = "cloud-sdk-provider-lock/v1"
OBSERVATION_FORMAT = "cloud-sdk-provider-observation/v1"
PLUGIN_FORMAT = "cloud-sdk-provider-drift-plugin/v1"
CATEGORIES = (
    "authentication",
    "cost",
    "endpoints",
    "headers",
    "idempotency",
    "operations",
    "pagination",
    "retry",
    "schemas",
)
CHANGE_KINDS = ("added", "changed", "removed")
SEVERITIES = ("review", "blocking")
MAX_DOCUMENT_BYTES = 2 * 1024 * 1024
MAX_SOURCES = 32
MAX_TOTAL_SOURCE_BYTES = 128 * 1024 * 1024
MAX_ROWS_PER_CATEGORY = 4096
MAX_VALUE_DEPTH = 8
MAX_COLLECTION_ITEMS = 4096
MAX_TEXT_BYTES = 4096
IDENTIFIER = re.compile(r"^[a-z][a-z0-9._/-]{0,127}$")
SHA256 = re.compile(r"^[0-9a-f]{64}$")


class ModelError(ValueError):
    """A provider drift document violates the closed model."""


def _pairs(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ModelError("JSON object contains a duplicate key")
        result[key] = value
    return result


def read_bounded_bytes(path: Path, label: str, maximum: int) -> bytes:
    """Read one bounded regular file without following its final path link."""
    required = ("O_CLOEXEC", "O_NOFOLLOW", "O_NONBLOCK")
    if any(not hasattr(os, name) for name in required):
        raise ModelError("platform lacks secure no-follow evidence reads")
    flags = os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW | os.O_NONBLOCK
    try:
        descriptor = os.open(path, flags)
    except OSError as error:
        raise ModelError(f"{label} must be a readable regular file") from error
    try:
        metadata = os.fstat(descriptor)
        if not stat.S_ISREG(metadata.st_mode):
            raise ModelError(f"{label} must be a regular file")
        if metadata.st_size > maximum:
            raise ModelError(f"{label} exceeds {maximum} bytes")
        payload = bytearray()
        while True:
            remaining = maximum + 1 - len(payload)
            try:
                chunk = os.read(descriptor, min(64 * 1024, remaining))
            except OSError as error:
                raise ModelError(f"{label} could not be read") from error
            if not chunk:
                break
            payload.extend(chunk)
            if len(payload) > maximum:
                raise ModelError(f"{label} exceeds {maximum} bytes")
    finally:
        os.close(descriptor)
    return bytes(payload)


def read_bounded_json(path: Path, label: str) -> dict[str, Any]:
    payload = read_bounded_bytes(path, label, MAX_DOCUMENT_BYTES)
    try:
        value = json.loads(payload, object_pairs_hook=_pairs)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ModelError(f"{label} is not strict UTF-8 JSON") from error
    if not isinstance(value, dict):
        raise ModelError(f"{label} root must be an object")
    return value


def exact_keys(value: dict[str, Any], expected: set[str], label: str) -> None:
    actual = set(value)
    if actual != expected:
        raise ModelError(f"{label} fields are incomplete or unsupported")


def identifier(value: Any, label: str) -> str:
    if not isinstance(value, str) or IDENTIFIER.fullmatch(value) is None:
        raise ModelError(f"{label} must be a canonical bounded identifier")
    return value


def sha256(value: Any, label: str) -> str:
    if not isinstance(value, str) or SHA256.fullmatch(value) is None:
        raise ModelError(f"{label} must be lowercase SHA-256")
    return value


def source_url(value: Any, label: str) -> str:
    if (
        not isinstance(value, str)
        or not value.isascii()
        or any(ord(character) <= 0x20 or ord(character) == 0x7F for character in value)
        or "\\" in value
    ):
        raise ModelError(f"{label} must be an ASCII HTTPS URL")
    parsed = urlsplit(value)
    try:
        port = parsed.port
    except ValueError as error:
        raise ModelError(f"{label} has an invalid port") from error
    if (parsed.scheme != "https" or not parsed.hostname or parsed.username is not None
            or parsed.password is not None or parsed.fragment or parsed.query):
        raise ModelError(f"{label} must be an exact credential-free HTTPS URL")
    if parsed.netloc != parsed.netloc.lower() or port == 443:
        raise ModelError(f"{label} authority must be lowercase and canonical")
    return value


def _bounded_value(value: Any, label: str, depth: int = 0) -> None:
    if depth > MAX_VALUE_DEPTH:
        raise ModelError(f"{label} exceeds nesting depth {MAX_VALUE_DEPTH}")
    if value is None or isinstance(value, (bool, int)):
        return
    if isinstance(value, float):
        raise ModelError(f"{label} must not contain floating-point values")
    if isinstance(value, str):
        if len(value.encode("utf-8")) > MAX_TEXT_BYTES:
            raise ModelError(f"{label} contains oversized text")
        return
    if isinstance(value, list):
        if len(value) > MAX_COLLECTION_ITEMS:
            raise ModelError(f"{label} contains too many list items")
        for index, item in enumerate(value):
            _bounded_value(item, f"{label}[{index}]", depth + 1)
        return
    if isinstance(value, dict):
        if len(value) > MAX_COLLECTION_ITEMS:
            raise ModelError(f"{label} contains too many object fields")
        for key, item in value.items():
            identifier(key, f"{label} field")
            _bounded_value(item, f"{label}.{key}", depth + 1)
        return
    raise ModelError(f"{label} contains an unsupported JSON value")


def canonical_bytes(value: Any) -> bytes:
    return json.dumps(
        value, ensure_ascii=True, sort_keys=True, separators=(",", ":")
    ).encode("ascii")


def canonical_sha256(value: Any) -> str:
    return hashlib.sha256(canonical_bytes(value)).hexdigest()


def validate_plugin(value: dict[str, Any]) -> dict[str, Any]:
    exact_keys(value, {"categories", "format", "id", "version"}, "plugin")
    if value["format"] != PLUGIN_FORMAT:
        raise ModelError("plugin format is unsupported")
    identifier(value["id"], "plugin id")
    if type(value["version"]) is not int or not 1 <= value["version"] <= 65535:
        raise ModelError("plugin version must be an integer from 1 through 65535")
    if value["categories"] != list(CATEGORIES):
        raise ModelError("plugin must declare every category in canonical order")
    return value


def _validate_plugin_ref(value: Any, label: str) -> None:
    if not isinstance(value, dict):
        raise ModelError(f"{label} must be an object")
    exact_keys(value, {"id", "version"}, label)
    identifier(value["id"], f"{label} id")
    if type(value["version"]) is not int or not 1 <= value["version"] <= 65535:
        raise ModelError(f"{label} version is invalid")


def _validate_sources(value: Any, label: str) -> None:
    if not isinstance(value, list) or not 1 <= len(value) <= MAX_SOURCES:
        raise ModelError(f"{label} must contain 1 through {MAX_SOURCES} sources")
    seen: set[str] = set()
    total_maximum = 0
    for index, source in enumerate(value):
        item_label = f"{label}[{index}]"
        if not isinstance(source, dict):
            raise ModelError(f"{item_label} must be an object")
        exact_keys(source, {"id", "max_bytes", "sha256", "url"}, item_label)
        source_id = identifier(source["id"], f"{item_label} id")
        if source_id in seen:
            raise ModelError(f"{label} contains duplicate source id {source_id}")
        seen.add(source_id)
        source_url(source["url"], f"{item_label} URL")
        sha256(source["sha256"], f"{item_label} digest")
        maximum = source["max_bytes"]
        if type(maximum) is not int or not 1 <= maximum <= 64 * 1024 * 1024:
            raise ModelError(f"{item_label} max_bytes is outside the hard bound")
        total_maximum += maximum
        if total_maximum > MAX_TOTAL_SOURCE_BYTES:
            raise ModelError(f"{label} exceeds its aggregate byte bound")


def _validate_contracts(value: Any, label: str) -> None:
    if not isinstance(value, dict):
        raise ModelError(f"{label} must be an object")
    exact_keys(value, set(CATEGORIES), label)
    for category in CATEGORIES:
        rows = value[category]
        if not isinstance(rows, list) or len(rows) > MAX_ROWS_PER_CATEGORY:
            raise ModelError(f"{label}.{category} exceeds its row bound")
        seen: set[str] = set()
        for index, row in enumerate(rows):
            row_label = f"{label}.{category}[{index}]"
            if not isinstance(row, dict):
                raise ModelError(f"{row_label} must be an object")
            exact_keys(row, {"id", "values"}, row_label)
            row_id = identifier(row["id"], f"{row_label} id")
            if row_id in seen:
                raise ModelError(f"{label}.{category} contains duplicate id {row_id}")
            seen.add(row_id)
            if not isinstance(row["values"], dict):
                raise ModelError(f"{row_label} values must be an object")
            _bounded_value(row["values"], f"{row_label}.values")


def validate_lock(value: dict[str, Any]) -> dict[str, Any]:
    exact_keys(
        value,
        {
            "compatibility",
            "contracts",
            "format",
            "owners",
            "plugin",
            "provider",
            "sources",
        },
        "provider lock",
    )
    if value["format"] != LOCK_FORMAT:
        raise ModelError("provider lock format is unsupported")
    identifier(value["provider"], "provider")
    _validate_plugin_ref(value["plugin"], "provider lock plugin")
    _validate_sources(value["sources"], "provider lock sources")
    _validate_contracts(value["contracts"], "provider lock contracts")
    owners = value["owners"]
    if not isinstance(owners, dict):
        raise ModelError("provider lock owners must be an object")
    exact_keys(owners, {"provider", "release", "security"}, "provider lock owners")
    for role, owner in owners.items():
        identifier(owner, f"provider lock {role} owner")
    compatibility = value["compatibility"]
    if not isinstance(compatibility, dict):
        raise ModelError("provider lock compatibility must be an object")
    exact_keys(compatibility, set(CATEGORIES), "provider lock compatibility")
    for category in CATEGORIES:
        policy = compatibility[category]
        if not isinstance(policy, dict):
            raise ModelError(f"compatibility.{category} must be an object")
        exact_keys(policy, {"added", "changed", "owner", "removed"}, f"compatibility.{category}")
        if policy["owner"] not in owners:
            raise ModelError(f"compatibility.{category} owner role is unknown")
        for change in CHANGE_KINDS:
            if policy[change] not in SEVERITIES:
                raise ModelError(f"compatibility.{category}.{change} is invalid")
    return value


def validate_observation(value: dict[str, Any]) -> dict[str, Any]:
    exact_keys(
        value,
        {"contracts", "format", "plugin", "provider", "sources"},
        "provider observation",
    )
    if value["format"] != OBSERVATION_FORMAT:
        raise ModelError("provider observation format is unsupported")
    identifier(value["provider"], "observation provider")
    _validate_plugin_ref(value["plugin"], "provider observation plugin")
    _validate_sources(value["sources"], "provider observation sources")
    _validate_contracts(value["contracts"], "provider observation contracts")
    return value
