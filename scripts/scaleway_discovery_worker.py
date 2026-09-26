"""Killable worker for non-executing Scaleway documentation discovery."""

import json
import sys

from scaleway_discovery import parse_catalog, parse_index
from scaleway_source_fetch import InventoryError, MAX_SOURCE


def main():
    limit = MAX_SOURCE * 6 + 65536
    payload = sys.stdin.buffer.read(limit + 1)
    if len(payload) > limit:
        raise InventoryError("discovery message exceeds bounds")
    kind, text, navigation = json.loads(payload)
    raw = text.encode("utf-8")
    if len(raw) > MAX_SOURCE or len(navigation) > 256:
        raise InventoryError("discovery input exceeds bounds")
    if kind == "index":
        script, routes, build = parse_index(raw)
        result = [script, sorted(routes), build]
    elif kind == "catalog":
        result = parse_catalog(raw, set(navigation))
    else:
        raise InventoryError("unknown discovery mode")
    output = json.dumps(result).encode()
    if len(output) > MAX_SOURCE * 6:
        raise InventoryError("discovery output exceeds bounds")
    sys.stdout.buffer.write(output)


if __name__ == "__main__":
    try:
        main()
    except (InventoryError, ValueError, TypeError, OSError):
        sys.exit(1)
