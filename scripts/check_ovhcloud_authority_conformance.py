#!/usr/bin/env python3
"""Bind OVHcloud authority/OAuth conformance fixtures to reviewed evidence."""

from __future__ import annotations

import csv
import sys
from pathlib import Path

from provider_drift_model import read_bounded_json, validate_lock


ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / "provider-drift/providers/ovhcloud-v2-probe.lock.json"
FIXTURE = ROOT / "crates/cloud-sdk/tests/fixtures/ovhcloud-authority-pairs.tsv"
FIELDS = (
    "region",
    "api_host",
    "api_port",
    "api_base_path",
    "token_host",
    "token_port",
    "token_base_path",
)


def reviewed_pairs(lock: dict) -> list[dict[str, str]]:
    endpoints = lock["contracts"]["endpoints"]
    rows = []
    for endpoint in endpoints:
        values = endpoint["values"]
        if "region" not in values:
            continue
        rows.append(
            {
                "region": values["region"],
                "api_host": values["host"],
                "api_port": "443",
                "api_base_path": values["base_path"],
                "token_host": values["token_host"],
                "token_port": "443",
                "token_base_path": "/auth/oauth2/token",
            }
        )
    rows.sort(key=lambda row: row["region"])
    if [row["region"] for row in rows] != ["ca", "eu"]:
        raise ValueError("reviewed OVHcloud regional endpoint set is invalid")
    return rows


def fixture_pairs(path: Path) -> list[dict[str, str]]:
    try:
        with path.open("r", encoding="ascii", newline="") as handle:
            reader = csv.DictReader(handle, delimiter="\t")
            if tuple(reader.fieldnames or ()) != FIELDS:
                raise ValueError("authority fixture fields are invalid")
            rows = list(reader)
    except (OSError, UnicodeError, csv.Error) as error:
        raise ValueError("authority fixture could not be read") from error
    if len(rows) != 2:
        raise ValueError("authority fixture must contain exactly two pairs")
    return rows


def check() -> None:
    lock = validate_lock(read_bounded_json(LOCK, "OVHcloud probe lock"))
    if fixture_pairs(FIXTURE) != reviewed_pairs(lock):
        raise ValueError("authority fixture differs from source-locked endpoint pairs")
    authentication = lock["contracts"]["authentication"]
    if len(authentication) != 1:
        raise ValueError("OVHcloud OAuth contract is ambiguous")
    values = authentication[0]["values"]
    if (
        values.get("flow") != "client_credentials"
        or values.get("scheme") != "bearer"
        or values.get("request_media") != "application/x-www-form-urlencoded"
        or values.get("response_fields")
        != ["access_token", "expires_in", "scope", "token_type"]
    ):
        raise ValueError("OVHcloud OAuth contract differs from reviewed evidence")
    expected_tokens = {
        (row["region"], f"https://{row['token_host']}{row['token_base_path']}")
        for row in fixture_pairs(FIXTURE)
    }
    observed_tokens = {
        (row["region"], row["url"]) for row in values.get("token_endpoints", [])
    }
    if observed_tokens != expected_tokens:
        raise ValueError("OAuth token endpoints are not paired with reviewed regions")


def main() -> int:
    try:
        check()
    except (KeyError, OSError, TypeError, UnicodeError, ValueError) as error:
        print(f"OVHcloud authority conformance: {error}", file=sys.stderr)
        return 1
    print("OVHcloud regional authorities and OAuth evidence are source-bound.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
