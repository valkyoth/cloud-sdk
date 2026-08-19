#!/usr/bin/env python3
"""Regression tests for prose-only Hetzner Server Metadata extraction."""

from __future__ import annotations

import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

from hetzner_metadata_contracts import EXPECTED, render  # noqa: E402


def document(section: str) -> dict[str, object]:
    return {"info": {"description": f"intro\n\n{section}\n## Sorting\nrest"}}


def fixture() -> str:
    table = "\n".join(
        [
            "## Server Metadata",
            "",
            "| Data | Format | Contents |",
            "| - | - | - |",
            *[
                f"| {name} | {wire} | reviewed |"
                for name, wire, _path in EXPECTED[1:]
            ],
            "",
        ]
    )
    examples = "\n".join(
        f"$ curl http://169.254.169.254{path}"
        for _name, _wire, path in EXPECTED
    )
    return f"{table}{examples}\n"


def rejects(label: str, value: dict[str, object]) -> None:
    try:
        render(value)
    except ValueError:
        return
    raise AssertionError(f"metadata mutation was accepted: {label}")


def main() -> None:
    source = fixture()
    rendered = render(document(source))
    assert rendered.count("\n") == 8
    for operation, wire, path in EXPECTED:
        assert f"{operation}\t{wire}\t{path}\t" in rendered

    rejects("route", document(source.replace("/metadata/region", "/metadata/zone")))
    rejects("format", document(source.replace("| region | text |", "| region | yaml |")))
    rejects("duplicate", document(source + "$ curl http://169.254.169.254/hetzner/v1/metadata\n"))
    rejects("removed alias", document(source + "$ curl http://169.254.169.254/2009-04-04/meta-data\n"))
    rejects("missing section", {"info": {"description": "none"}})
    rejects("duplicate section", document(source + source))
    rejects("non-string", json.loads('{"info":{"description":42}}'))
    print("7 Server Metadata source-lock regression groups passed.")


if __name__ == "__main__":
    main()
