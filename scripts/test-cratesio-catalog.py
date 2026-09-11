#!/usr/bin/env python3
"""Offline fail-closed tests for catalog schema and fixture generation."""
import copy
import json

from generate_cratesio_catalog import OUTPUT, PATHS, expanded_example, render


def rejected(call):
    try:
        call()
    except (ValueError, KeyError):
        return
    raise AssertionError("invalid catalog schema accepted")


def document():
    schemas = {
        "Version": {"type": "object", "required": ["id", "num", "crate"],
                    "properties": {"id": {"type": "integer", "example": 1},
                                   "num": {"type": "string", "example": "1.0.0"},
                                   "crate": {"type": "string", "example": "fixture"}}},
        "Keyword": {"example": {"keyword": "example"}},
        "Category": {"example": {"slug": "example"}},
    }
    paths = {}
    for name, path in PATHS.items():
        value = {"crates": [], "meta": {}} if name == "list_crates" else {
            "crate": {"id": "fixture", "name": "fixture"}}
        paths[path] = {"get": {"operationId": name, "responses": {
            "200": {"content": {"application/json": {"schema": {"example": value}}}}}}}
    return {"components": {"schemas": schemas}, "paths": paths}


def main():
    source = document()
    table, fixtures = render(source)
    assert "Node::Object" in table and "Node::Integer" in table
    assert len(fixtures) == 3
    new = json.loads(fixtures["find_new_crate.json"])
    assert new["crate"]["name"] == new["versions"][0]["crate"] == "new"
    assert new["crate"]["default_version"] == new["versions"][0]["num"]
    assert json.loads(fixtures["list_crates.json"])["meta"] == {
        "total": 1, "next_page": None, "prev_page": None}
    assert render(source) == (table, fixtures)
    for constraint in (
        {"minimum": 0}, {"pattern": "a"}, {"allOf": []},
        {"propertyNames": {"pattern": "a"}}, {"additionalProperties": False},
        {"format": "unreviewed"}, {"type": ["integer", "string"]},
        {"required": ["missing"]}, {"$ref": "https://untrusted.invalid/schema"},
        {"$ref": "#/components/schemas/Version"},
    ):
        mutated = copy.deepcopy(source)
        mutated["components"]["schemas"]["Version"].update(constraint)
        rejected(lambda: render(mutated))
    cycle = {"components": {"schemas": {"Loop": {"$ref": "#/components/schemas/Loop"}}}}
    rejected(lambda: expanded_example(cycle["components"]["schemas"]["Loop"], cycle))
    rejected(lambda: expanded_example({"$ref": "https://untrusted.invalid/schema"}, {}))
    mutated = copy.deepcopy(source)
    mutated["paths"][PATHS["list_crates"]]["get"]["operationId"] = "../other"
    rejected(lambda: render(mutated))
    del mutated["paths"][PATHS["list_crates"]]
    rejected(lambda: render(mutated))
    assert {p.name for p in (OUTPUT / "fixtures").glob("*.json")} == set(fixtures)
    print("5 catalog schema and fixture regression groups passed.")


if __name__ == "__main__":
    main()
