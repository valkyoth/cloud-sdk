#!/usr/bin/env python3
"""Check the public IPv6 policy against source-locked IANA registries."""

from __future__ import annotations

import argparse
import csv
import hashlib
import io
import ipaddress
import re
import sys
import time
import urllib.request
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / "docs" / "IANA_IPV6_GLOBAL_UNICAST.tsv"
DOCUMENTATION = ROOT / "docs" / "IANA_IPV6_SOURCE_LOCK.md"
RUST_POLICY = ROOT / "crates/cloud-sdk-hetzner/src/cloud/load_balancers/public_ip.rs"
REGISTRIES = {
    "global-unicast": (
        "https://www.iana.org/assignments/ipv6-unicast-address-assignments/"
        "ipv6-unicast-address-assignments.csv",
        "ebff425bb1acbbea29c4f28146930873faddd5ee57260e95b57ed9e04ea21dd8",
    ),
    "special-purpose": (
        "https://www.iana.org/assignments/iana-ipv6-special-registry/"
        "iana-ipv6-special-registry-1.csv",
        "775feea0621dec8735a44fbf30f762e721e8f0a1b3ab7eb341961a88cfce2139",
    ),
}
POLICY_EXCLUSIONS = {"2001::/23", "2002::/16"}
REQUIRED_SPECIAL_RANGES = {
    "2001::/23",
    "2001:db8::/32",
    "2002::/16",
    "2620:4f:8000::/48",
}
MAX_REGISTRY_BYTES = 1024 * 1024
READ_CHUNK_BYTES = 64 * 1024
CONNECT_TIMEOUT_SECONDS = 10
TOTAL_TIMEOUT_SECONDS = 30
PREFIX_PATTERN = re.compile(
    r"Ipv6Prefix::new\(0x([0-9a-f]{4}), 0x([0-9a-f]{4}), ([0-9]{1,3})\)"
)


def canonical_prefix(value: str) -> str:
    without_footnote = re.sub(r"\s+\[[0-9]+\]$", "", value.strip())
    try:
        return str(ipaddress.IPv6Network(without_footnote, strict=True))
    except ValueError as error:
        raise SystemExit(f"invalid IANA IPv6 prefix {value!r}") from error


def parse_csv(payload: bytes, registry: str) -> list[dict[str, str]]:
    try:
        text = payload.decode("utf-8-sig")
    except UnicodeDecodeError as error:
        raise SystemExit(f"{registry} registry is not UTF-8") from error
    return list(csv.DictReader(io.StringIO(text, newline="")))


def allocated_rows(payload: bytes) -> dict[str, tuple[str, str]]:
    rows: dict[str, tuple[str, str]] = {}
    for row in parse_csv(payload, "global-unicast"):
        if row.get("Status") != "ALLOCATED":
            continue
        prefix = canonical_prefix(row.get("Prefix", ""))
        if prefix in POLICY_EXCLUSIONS:
            continue
        if prefix in rows:
            raise SystemExit(f"duplicate IANA global-unicast prefix {prefix}")
        rows[prefix] = (row.get("Designation", ""), row.get("Date", ""))
    return rows


def special_prefixes(payload: bytes) -> set[str]:
    return {
        canonical_prefix(row.get("Address Block", ""))
        for row in parse_csv(payload, "special-purpose")
    }


def locked_rows(path: Path = LOCK) -> dict[str, tuple[str, str]]:
    try:
        with path.open("r", encoding="utf-8", newline="") as handle:
            reader = csv.DictReader(handle, delimiter="\t")
            if reader.fieldnames != ["prefix", "designation", "date"]:
                raise SystemExit(f"invalid IANA lock header: {path}")
            rows: dict[str, tuple[str, str]] = {}
            for row in reader:
                prefix = canonical_prefix(row["prefix"])
                if prefix in rows:
                    raise SystemExit(f"duplicate locked IPv6 prefix {prefix}")
                rows[prefix] = (row["designation"], row["date"])
            return rows
    except OSError as error:
        raise SystemExit(f"cannot read IANA lock: {path}") from error


def rust_prefixes(path: Path = RUST_POLICY) -> set[str]:
    try:
        source = path.read_text(encoding="utf-8")
    except OSError as error:
        raise SystemExit(f"cannot read Rust IPv6 policy: {path}") from error
    prefixes = set()
    for first, second, length in PREFIX_PATTERN.findall(source):
        if int(length) > 32:
            raise SystemExit(
                "Rust IPv6 policy uses a prefix longer than its 32-bit storage"
            )
        network = (int(first, 16) << 112) | (int(second, 16) << 96)
        try:
            prefix = ipaddress.IPv6Network((network, int(length)), strict=True)
        except ValueError as error:
            raise SystemExit("Rust IPv6 prefix is not canonical") from error
        prefixes.add(str(prefix))
    if not prefixes:
        raise SystemExit("Rust IPv6 policy contains no source-locked prefixes")
    return prefixes


def report_drift(
    name: str,
    expected: dict[str, tuple[str, str]] | set[str],
    actual: dict[str, tuple[str, str]] | set[str],
) -> int:
    if expected == actual:
        print(f"{name}: no drift")
        return 0
    expected_keys = set(expected)
    actual_keys = set(actual)
    print(f"{name}: drift detected", file=sys.stderr)
    for prefix in sorted(actual_keys - expected_keys):
        print(f"  added: {prefix}", file=sys.stderr)
    for prefix in sorted(expected_keys - actual_keys):
        print(f"  removed: {prefix}", file=sys.stderr)
    if isinstance(expected, dict) and isinstance(actual, dict):
        for prefix in sorted(expected_keys & actual_keys):
            if expected[prefix] != actual[prefix]:
                print(
                    f"  changed: {prefix}: {expected[prefix]} -> {actual[prefix]}",
                    file=sys.stderr,
                )
    return 1


def read_bounded_response(
    response: Any,
    registry: str,
    *,
    max_bytes: int = MAX_REGISTRY_BYTES,
    total_seconds: int = TOTAL_TIMEOUT_SECONDS,
    monotonic: Any = time.monotonic,
) -> bytes:
    started = monotonic()
    data = bytearray()
    while True:
        if monotonic() - started > total_seconds:
            raise SystemExit(
                f"{registry} registry download exceeded {total_seconds} seconds"
            )
        remaining = max_bytes + 1 - len(data)
        chunk = response.read(min(READ_CHUNK_BYTES, remaining))
        if monotonic() - started > total_seconds:
            raise SystemExit(
                f"{registry} registry download exceeded {total_seconds} seconds"
            )
        if not chunk:
            return bytes(data)
        data.extend(chunk)
        if len(data) > max_bytes:
            raise SystemExit(f"{registry} registry exceeds {max_bytes} bytes")


def fetch_registry(name: str) -> bytes:
    url, _ = REGISTRIES[name]
    try:
        with urllib.request.urlopen(url, timeout=CONNECT_TIMEOUT_SECONDS) as response:
            return read_bounded_response(response, name)
    except OSError as error:
        raise SystemExit(f"could not fetch {name} registry: {error}") from error


def verified_registry(name: str, payload: bytes) -> bytes:
    actual = hashlib.sha256(payload).hexdigest()
    expected = REGISTRIES[name][1]
    if actual != expected:
        raise SystemExit(
            f"{name} registry SHA-256 mismatch: expected {expected}, got {actual}"
        )
    print(f"{name} registry SHA-256 verified")
    return payload


def validate_local() -> int:
    locked = locked_rows()
    status = report_drift("Rust IPv6 policy", set(locked), rust_prefixes())
    try:
        documentation = DOCUMENTATION.read_text(encoding="utf-8")
    except OSError as error:
        raise SystemExit(f"cannot read IANA source-lock documentation: {error}") from error
    for url, digest in REGISTRIES.values():
        if url not in documentation or digest not in documentation:
            print("IANA source-lock documentation is incomplete", file=sys.stderr)
            status = 1
    return status


def validate_fetched() -> int:
    payloads = {
        name: verified_registry(name, fetch_registry(name)) for name in REGISTRIES
    }
    status = report_drift(
        "IANA global-unicast allocations",
        locked_rows(),
        allocated_rows(payloads["global-unicast"]),
    )
    missing_special = REQUIRED_SPECIAL_RANGES - special_prefixes(
        payloads["special-purpose"]
    )
    if missing_special:
        print(
            "IANA special-purpose policy ranges disappeared: "
            + ", ".join(sorted(missing_special)),
            file=sys.stderr,
        )
        status = 1
    return status


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--local-only", action="store_true")
    parser.add_argument("--fetch", action="store_true")
    args = parser.parse_args()
    if args.local_only == args.fetch:
        parser.error("choose exactly one of --local-only or --fetch")
    status = validate_local()
    if args.fetch:
        status |= validate_fetched()
    return status


if __name__ == "__main__":
    raise SystemExit(main())
