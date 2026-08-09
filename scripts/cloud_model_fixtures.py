"""Deterministic fixtures for source-derived Hetzner response models."""

from __future__ import annotations

import json
from typing import Any

from cloud_schema_policy import merge_all_of


def schema_types(schema: dict[str, Any]) -> list[str]:
    value = schema.get("type")
    if isinstance(value, str):
        return [value]
    if isinstance(value, list) and all(isinstance(item, str) for item in value):
        return sorted(value)
    if "properties" in schema or "allOf" in schema or "oneOf" in schema:
        return ["object"]
    raise ValueError("schema node has no supported fixture type")


def example_value(schema: dict[str, Any]) -> Any:
    schema = merge_all_of(schema)
    if "oneOf" in schema:
        branches = schema["oneOf"]
        if not isinstance(branches, list) or not branches:
            raise ValueError("union has no example branch")
        return example_value(branches[0])
    types = schema_types(schema)
    nonnull = next((value for value in types if value != "null"), "null")
    if nonnull == "null":
        return None
    if nonnull == "boolean":
        return False
    if nonnull == "integer":
        return max(int(schema.get("minimum", 1)), 1)
    if nonnull == "number":
        return max(float(schema.get("minimum", 1.0)), 1.0)
    if nonnull == "string":
        format_value = schema.get("format")
        if format_value == "date-time":
            return "2026-01-01T00:00:00Z"
        if format_value == "decimal":
            return "1.0"
        known = schema.get("enum", [])
        if isinstance(known, list) and known and isinstance(known[0], str):
            return known[0]
        length = max(int(schema.get("minLength", 1)), 1)
        if schema.get("maxLength") is not None:
            length = min(length, int(schema["maxLength"]))
        return "x" * length
    if nonnull == "array":
        items = schema.get("items", {})
        if not isinstance(items, dict):
            raise ValueError("array fixture has invalid items")
        branches = items.get("oneOf")
        if isinstance(branches, list):
            return [example_value(branch) for branch in branches]
        return [example_value(items)]
    if nonnull == "object":
        properties = schema.get("properties", {})
        if not isinstance(properties, dict):
            raise ValueError("object fixture has invalid properties")
        return {
            name: example_value(child)
            for name, child in sorted(properties.items())
            if isinstance(child, dict)
        }
    raise ValueError(f"unsupported fixture type: {nonnull}")


def normalize_fixture(model: str, value: Any) -> Any:
    """Apply deterministic semantic values omitted from machine constraints."""
    if model == "certificate" and isinstance(value, dict):
        value.update(
            certificate=(
                "-----BEGIN CERTIFICATE-----\n"
                "Y2xvdWQtc2RrLXRlc3QtY2VydGlmaWNhdGU=\n"
                "-----END CERTIFICATE-----"
            ),
            created="2026-01-01T00:00:00Z",
            domain_names=["example.com"],
            fingerprint="00:11:22:33",
            not_valid_after="2027-01-01T00:00:00Z",
            not_valid_before="2026-01-01T00:00:00Z",
            type="uploaded",
            status=None,
            used_by=[],
        )
    if model == "zone" and isinstance(value, dict):
        value["mode"] = "secondary"
        value["name"] = "example.com"
        value["authoritative_nameservers"]["assigned"] = ["ns1.example.com."]
        value["authoritative_nameservers"]["delegated"] = ["ns1.example.com."]
        nameserver = value["primary_nameservers"][0]
        nameserver.update(
            address="192.0.2.1",
            port=53,
            tsig_algorithm="hmac-sha256",
            tsig_key="AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        )
    if model == "rrset" and isinstance(value, dict):
        value["name"] = "www"
        value["type"] = "A"
        value["records"][0]["value"] = "192.0.2.1"
    if model == "ssh_key" and isinstance(value, dict):
        value["created"] = "2026-01-01T00:00:00Z"
        value["fingerprint"] = "ae:6f:ba:1b:70:2c:ae:c7:5c:ab:6e:4d:5e:d4:c7:23"
        value["public_key"] = (
            "ssh-ed25519 "
            "AAAAC3NzaC1lZDI1NTE5AAAAILM+rvN+ot98qgEN796jTiQfZfG1KaT0PtFDJ/XFSqti "
            "user@example.com"
        )
    if model.startswith("storage_box") and isinstance(value, dict):
        normalize_storage_fixture(model, value)
    return value


def normalize_storage_fixture(model: str, value: dict[str, Any]) -> None:
    if "created" in value:
        value["created"] = "2026-01-01T00:00:00Z"
    if model == "storage_box":
        value.update(
            name="backup",
            username="u12345",
            server="u12345.your-storagebox.de",
            system="FSN1-BX1",
            status="active",
        )
    if model == "storage_box_snapshot":
        value.update(name="manual-2026-01-01", description="daily backup")
    if model == "storage_box_subaccount":
        value.update(
            name="backup-user",
            home_directory="backups/server01",
            description="backup account",
            username="u12345-sub1",
            server="u12345-sub1.your-storagebox.de",
        )


def render(models: dict[str, dict[str, Any]]) -> str:
    fixtures = {
        model: normalize_fixture(model, example_value(schema))
        for model, schema in sorted(models.items())
    }
    return json.dumps(fixtures, sort_keys=True, separators=(",", ":")) + "\n"
