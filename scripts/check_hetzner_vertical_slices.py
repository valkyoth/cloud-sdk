#!/usr/bin/env python3
"""Check the v0.62 source-complete Hetzner freeze assignments."""

from __future__ import annotations

import csv
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
RESPONSE_LOCK = ROOT / "crates/cloud-sdk-hetzner/src/serde/response_operations.tsv"
EXPECTED = {
    "list_locations": ("cloud", "cloud", "200", "resource-page"),
    "poweron_server": ("cloud", "cloud", "201", "action"),
    "get_zone_zonefile": ("cloud", "dns", "200", "zonefile"),
    "get_certificate": ("cloud", "security", "200", "resource"),
    "list_storage_boxes": ("hetzner", "storage", "200", "resource-page"),
    "delete_certificate": ("cloud", "security", "204", "empty"),
}


def require(path: Path, fragments: tuple[str, ...]) -> None:
    text = path.read_text(encoding="utf-8")
    for fragment in fragments:
        if fragment not in text:
            raise ValueError(f"{path.relative_to(ROOT)} lacks {fragment!r}")


def main() -> int:
    with RESPONSE_LOCK.open(encoding="ascii", newline="") as handle:
        rows = {row["operation_id"]: row for row in csv.DictReader(handle, delimiter="\t")}
    for operation, expected in EXPECTED.items():
        row = rows.get(operation)
        actual = None if row is None else (
            row["api"], row["service"], row["status"], row["shape"]
        )
        if actual != expected:
            raise ValueError(f"wrong vertical binding for {operation}: {actual!r}")

    require(
        ROOT / "crates/cloud-sdk-hetzner/src/serde/models.rs",
        ("Locations(LocationPage)", "Certificate(Certificate)", "StorageBoxes(StorageBoxPage)"),
    )
    require(
        ROOT / "crates/cloud-sdk-hetzner/src/serde/checked.rs",
        ("decode_associated_checked_response", "AssociatedCheckedResponse<'_, O>"),
    )
    require(
        ROOT / "crates/cloud-sdk-hetzner/src/association/prepared.rs",
        ("pub struct AssociatedCheckedResponse", "map(AssociatedCheckedResponse::new)"),
    )
    require(
        ROOT / "crates/cloud-sdk-hetzner/src/serde/checked/success.rs",
        ("validate_incremental(bytes.as_slice())", "validate_page_item_count"),
    )
    require(
        ROOT / "crates/cloud-sdk-hetzner/src/serde/vertical_tests.rs",
        tuple(EXPECTED) + ("source_complete_pages_reject_more_items",),
    )
    require(
        ROOT / "crates/cloud-sdk-hetzner/tests/vertical_execution.rs",
        (
            "decode_associated_checked_response",
            "AuthenticationClass::Bearer",
            "HetznerSuccess::Locations",
            "HetznerSuccess::Certificate",
            "HetznerSuccess::ZoneFile",
            "HetznerSuccess::StorageBoxes",
            "INVALID_JSON",
        ),
    )
    require(
        ROOT / "docs/REJECTED_ABSTRACTIONS_0.62.0.md",
        ("Provider Enum In Core", "A Second Response Decoder", "Unprotected PEM"),
    )
    print("6 source-complete Hetzner vertical assignments checked.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
