#!/usr/bin/env python3
"""Capture and verify Scaleway control-plane sources without SDK credentials."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import gzip
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time

from scaleway_discovery import INDEX, discover, index_sources
from scaleway_inventory import ROOT, load_json, operations, parse_schema, sdk_candidates
from scaleway_source_fetch import InventoryError, MAX_SOURCE, MAX_TOTAL, SDK_REVISION, SDK_TREE, approved_url, fetch

DIRECTORY = ROOT / "provider-drift/scaleway"
LOCK = DIRECTORY / "inventory.json"
MAX_SOURCES = 256


def build_parser() -> None:
    subprocess.run(["cargo", "build", "--quiet", "--locked", "--offline", "--manifest-path",
                    str(ROOT / "tools/prepared-coverage-check/Cargo.toml"),
                    "--bin", "source-yaml-json"], cwd=ROOT, check=True)


class Sources:
    def __init__(self, directory: Path, *, capture: bool = False, live: bool = False) -> None:
        self.directory = directory
        self.capture = capture
        self.live = live
        self.records: list[dict] = []
        self.total = 0
        self.started = time.monotonic()

    def get(self, url: str, expected: dict | None = None) -> bytes:
        approved_url(url)
        if len(self.records) >= MAX_SOURCES or any(r["url"] == url for r in self.records):
            raise InventoryError("duplicate or excessive sources")
        remaining = 1800 - (time.monotonic() - self.started)
        if remaining <= 0:
            raise InventoryError("whole inventory deadline exceeded")
        budget = min(MAX_SOURCE, MAX_TOTAL - self.total)
        if budget <= 0:
            raise InventoryError("aggregate source byte budget exhausted")
        if self.capture or self.live:
            raw = fetch(url, budget, timeout=min(60, remaining))
        else:
            if expected is None or not isinstance(expected.get("sha256"), str):
                raise InventoryError("missing source digest")
            digest = expected["sha256"]
            if len(digest) != 64 or any(c not in "0123456789abcdef" for c in digest):
                raise InventoryError("invalid source digest")
            path = self.directory / "sources" / f"{digest}.gz"
            if path.is_symlink() or not path.is_file() or path.stat().st_size > MAX_SOURCE * 2:
                raise InventoryError("invalid source archive")
            with gzip.open(path, "rb") as handle:
                raw = handle.read(budget + 1)
        if len(raw) > budget:
            raise InventoryError("aggregate or per-source byte limit exceeded")
        record = {"url": url, "sha256": hashlib.sha256(raw).hexdigest(), "size_bytes": len(raw)}
        if expected is not None and expected != record:
            raise InventoryError(f"source evidence changed: {url}")
        self.records.append(record)
        self.total += len(raw)
        if self.capture:
            target = self.directory / "sources" / f"{record['sha256']}.gz"
            target.parent.mkdir(parents=True, exist_ok=True)
            content = gzip.compress(raw, mtime=0)
            if target.exists():
                if target.is_symlink() or target.read_bytes() != content:
                    raise InventoryError("existing source archive differs")
            else:
                with target.open("xb") as handle:
                    handle.write(content)
            print(f"captured {len(self.records)}: {url}", flush=True)
        return raw


def observe(sources: Sources, locked_sources: list[dict] | None = None) -> dict:
    pending = iter(locked_sources) if locked_sources is not None else None

    def get(url: str) -> bytes:
        expected = next(pending, None) if pending is not None else None
        if pending is not None and (expected is None or expected.get("url") != url):
            raise InventoryError("source inventory order or membership changed")
        return sources.get(url, expected)

    bundle_url, navigation, build = index_sources(get(INDEX))
    catalog = discover(get(bundle_url), navigation)
    tree = load_json(get(SDK_TREE))
    if not isinstance(tree, dict) or tree.get("sha") != SDK_REVISION:
        raise InventoryError("SDK tree is not the pinned revision")
    entries = []
    for entry in catalog:
        row = dict(entry)
        if "href" in entry:
            row.update(disposition="data-plane-review", owner_candidates=[2, 23],
                       reason="Official catalog links separate protocol documentation; Commit 2 source lock",
                       operations=[])
        else:
            url = INDEX.removesuffix("/api") + entry["route"].rstrip("/") + "/" + entry["version"] + "/schema.yml"
            schema = parse_schema(get(url))
            observed = operations(entry, schema)
            disposition = "supported" if entry.get("visibility", "public") == "public" else "unresolved"
            reason = "Listed in official public API catalog; all documented versions retained"
            if disposition == "unresolved":
                reason = "Official catalog explicitly marks unlisted; reachability alone does not establish public support"
            if entry["route"] == "/api/test":
                disposition, reason = "private", "Official catalog labels Fake; schema describes a fake testing service"
            row.update(disposition=disposition, reason=reason, operations=observed,
                       schema_url=url, schema_title=schema["info"]["title"])
        entries.append(row)
    candidates = sdk_candidates(tree, catalog)
    # Retain the exact SDK source for the known Billing discrepancy.
    billing_url = f"https://raw.githubusercontent.com/scaleway/scaleway-sdk-go/{SDK_REVISION}/api/billing/v2/billing_sdk.go"
    billing = get(billing_url).decode("utf-8")
    if "Package billing" not in billing or "/billing/v2/" not in billing:
        raise InventoryError("Billing SDK comparison source changed")
    if pending is not None and next(pending, None) is not None:
        raise InventoryError("unconsumed source record")
    return {"format": 1, "documentation_build": build, "sdk_revision": SDK_REVISION,
            "sources": sources.records, "entries": entries, "sdk_only_candidates": candidates}


def unresolved(inventory: dict) -> list[str]:
    return [f"{row['route']} {row['version']}" for row in inventory["entries"]
            if row["disposition"] == "unresolved"] + [
                f"SDK {row['family']} {row['version']}" for row in inventory["sdk_only_candidates"]
                if row["disposition"] == "unresolved"]


def qualify(inventory: dict, raw_lock: bytes, followups: dict, *, release: bool = False) -> None:
    problems = unresolved(inventory)
    if not problems:
        return
    if release:
        raise InventoryError("release NOT qualified; unresolved candidates: " + "; ".join(problems))
    if (set(followups) != {"format", "authorized_at", "authorization", "inventory_sha256",
                          "resolve_by_checkpoint", "release_allowed_with_unresolved", "candidates"}
            or followups["format"] != 1 or followups["resolve_by_checkpoint"] != 76
            or followups["release_allowed_with_unresolved"] is not False
            or followups["inventory_sha256"] != hashlib.sha256(raw_lock).hexdigest()
            or followups["candidates"] != problems
            or not isinstance(followups["authorization"], str) or not followups["authorization"]
            or followups["authorized_at"] != "2026-09-26"):
        raise InventoryError("checkpoint NOT qualified: follow-up approval does not match exact inventory")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--capture", action="store_true", help="capture a new candidate into a new directory")
    mode.add_argument("--fetch", action="store_true", help="compare fresh sources with the committed snapshot")
    parser.add_argument("--qualify", action="store_true", help="require exact approved checkpoint follow-ups")
    parser.add_argument("--release", action="store_true", help="reject every unresolved candidate, ignoring checkpoint allowances")
    parser.add_argument("--directory", type=Path, default=DIRECTORY)
    args = parser.parse_args()
    directory = args.directory
    try:
        lock_path = directory / "inventory.json"
        if args.capture and directory.exists():
            raise InventoryError("capture requires a new directory; never overwrite accepted evidence")
        build_parser()
        if args.capture:
            inventory = observe(Sources(directory, capture=True))
            inventory["observed_at"] = datetime.now(timezone.utc).isoformat()
            lock_path.write_text(json.dumps(inventory, indent=2, ensure_ascii=True) + "\n", encoding="ascii")
        else:
            if lock_path.stat().st_size > MAX_SOURCE:
                raise InventoryError("inventory lock is oversized")
            inventory = load_json(lock_path.read_bytes())
            if not isinstance(inventory, dict) or len(inventory.get("sources", [])) > MAX_SOURCES:
                raise InventoryError("invalid inventory lock")
            rebuilt = observe(Sources(directory, live=args.fetch), inventory["sources"])
            rebuilt["observed_at"] = inventory["observed_at"]
            if rebuilt != inventory:
                raise InventoryError("inventory differs from source-derived evidence")
        count = sum(len(entry["operations"]) for entry in inventory["entries"])
        problems = unresolved(inventory)
        print(f"Scaleway inventory: {len(inventory['sources'])} sources, {count} operations, {len(problems)} unresolved candidates")
        if args.qualify or args.release:
            followups = {} if args.release or not problems else load_json((directory / "followups.json").read_bytes())
            qualify(inventory, lock_path.read_bytes(), followups, release=args.release)
            if problems:
                print("Checkpoint allowance: exact maintainer-approved follow-ups; resolution required by Commit 76, never waived for release.")
    except (InventoryError, OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        print(f"Scaleway inventory: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
