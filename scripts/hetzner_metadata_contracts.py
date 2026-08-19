#!/usr/bin/env python3
"""Extract the finite canonical Server Metadata contract from Cloud OpenAPI prose."""

from __future__ import annotations

import hashlib
import re
from typing import Any


BASE_URL = "http://169.254.169.254"
SECTION_START = "## Server Metadata\n"
SECTION_END = "\n## Sorting\n"
EXPECTED = (
    ("summary", "yaml", "/hetzner/v1/metadata"),
    ("hostname", "text", "/hetzner/v1/metadata/hostname"),
    ("instance-id", "number", "/hetzner/v1/metadata/instance-id"),
    ("public-ipv4", "text", "/hetzner/v1/metadata/public-ipv4"),
    ("private-networks", "yaml", "/hetzner/v1/metadata/private-networks"),
    ("availability-zone", "text", "/hetzner/v1/metadata/availability-zone"),
    ("region", "text", "/hetzner/v1/metadata/region"),
)
REMOVED_PREFIX = "/2009-04-04/meta-data"


def section(document: dict[str, Any]) -> str:
    """Return the exact metadata section or reject an ambiguous source."""
    info = document.get("info")
    if not isinstance(info, dict):
        raise ValueError("Cloud OpenAPI info object is missing")
    description = info.get("description")
    if not isinstance(description, str):
        raise ValueError("Cloud OpenAPI description is missing")
    if description.count(SECTION_START) != 1 or description.count(SECTION_END) != 1:
        raise ValueError("Server Metadata section is missing or ambiguous")
    start = description.index(SECTION_START)
    end = description.index(SECTION_END, start)
    value = description[start:end].replace("\r\n", "\n")
    if "\r" in value:
        raise ValueError("Server Metadata section has unsupported line endings")
    return value


def rows(document: dict[str, Any]) -> tuple[list[tuple[str, str, str]], str]:
    """Validate table and examples, returning finite routes and section digest."""
    value = section(document)
    table = re.findall(
        r"^\| ([a-z0-9-]+)\s+\| (text|number|yaml)\s+\|",
        value,
        re.MULTILINE,
    )
    expected_children = [(name, wire) for name, wire, _path in EXPECTED[1:]]
    if table != expected_children:
        raise ValueError("Server Metadata field table changed")
    urls = re.findall(r"\$ curl (http://[^\s]+)", value)
    expected_urls = [f"{BASE_URL}{path}" for _name, _wire, path in EXPECTED]
    if urls != expected_urls or len(set(urls)) != len(urls):
        raise ValueError("Server Metadata canonical route examples changed")
    if REMOVED_PREFIX in value or "2009-04-04" in value:
        raise ValueError("removed EC2-compatible metadata aliases reappeared")
    digest = hashlib.sha256(value.encode("utf-8")).hexdigest()
    return list(EXPECTED), digest


def render(document: dict[str, Any]) -> str:
    """Render deterministic metadata operation evidence."""
    operations, digest = rows(document)
    lines = ["operation\tformat\tpath\tsection_sha256"]
    lines.extend(
        f"{operation}\t{wire}\t{path}\t{digest}"
        for operation, wire, path in operations
    )
    return "\n".join(lines) + "\n"
