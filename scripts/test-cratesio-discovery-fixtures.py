#!/usr/bin/env python3
"""Offline regression tests for source-derived discovery evidence."""
import copy
import json
import tempfile
from pathlib import Path

from generate_cratesio_discovery_fixtures import (
    OPERATIONS, OUTPUT, PATHS, example, render, verify,
)


def rejected(call):
    try:
        call()
    except (ValueError, KeyError):
        return
    raise AssertionError("invalid discovery evidence accepted")


def main():
    schema = {"type": "object", "required": ["name", "nullable"],
              "properties": {"name": {"type": "string", "example": "fixture"},
                             "nullable": {"type": ["string", "null"]},
                             "optional": {"type": "boolean"}}}
    assert example(schema, {}) == {"name": "fixture", "nullable": None}
    spec = {"paths": {path: {"get": {"operationId": name,
        "responses": {"200": {"content": {"application/json": {"schema": schema}}}}}}
        for path, name in zip(PATHS, OPERATIONS, strict=True)}}
    fixtures = render(spec)
    assert len(fixtures) == 7
    assert all(json.loads(text) == example(schema, {}) for text in fixtures.values())
    mutated = copy.deepcopy(spec)
    mutated["paths"][PATHS[0]]["get"]["operationId"] = "../escape"
    rejected(lambda: render(mutated))
    mutated = copy.deepcopy(spec)
    del mutated["paths"][PATHS[0]]
    rejected(lambda: render(mutated))
    rejected(lambda: example({"$ref": "https://untrusted.invalid/schema"}, {}))
    rejected(lambda: example({"type": "unreviewed"}, {}))
    cycle = {"components": {"schemas": {"Loop": {"$ref": "#/components/schemas/Loop"}}}}
    rejected(lambda: example(cycle["components"]["schemas"]["Loop"], cycle))
    with tempfile.TemporaryDirectory() as temp:
        directory = Path(temp)
        for name, content in fixtures.items():
            (directory / name).write_text(content, encoding="ascii")
        verify(fixtures, directory)
        path = directory / "get_summary.json"
        path.write_text("{}", encoding="ascii")
        rejected(lambda: verify(fixtures, directory))
        path.write_text(fixtures[path.name], encoding="ascii")
        path.unlink()
        rejected(lambda: verify(fixtures, directory))
        path.write_text(fixtures[path.name], encoding="ascii")
        (directory / "extra.json").write_text("{}", encoding="ascii")
        rejected(lambda: verify(fixtures, directory))
    assert {p.name for p in OUTPUT.glob("*.json")} == {name + ".json" for name in OPERATIONS}
    print("10 crates.io discovery fixture regression groups passed.")


if __name__ == "__main__":
    main()
