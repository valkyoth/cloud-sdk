#!/usr/bin/env python3
"""Offline public account schema, fixture and feature-boundary regressions."""
import copy
import importlib.util
import json
from pathlib import Path
import shutil
from generate_cratesio_accounts import PATHS, OUTPUT, render


def document():
    owners = []
    for kind in ("user", "team"):
        owners.append({"type": "object", "required": ["kind"], "properties": {
            "kind": {"type": "string", "enum": [kind]}}})
    paths = {}
    for name, path in PATHS.items():
        field = "user" if name == "find_user" else "team"
        value = {"type": "string", "example": "public"}
        if name == "get_user_stats":
            field, value = "total_downloads", {"type": "integer", "format": "int64", "minimum": 0}
        if name in ("list_owners", "get_user_owners", "get_team_owners"):
            field = "teams" if name == "get_team_owners" else "users"
            value = {"type": "array", "items": {"$ref": "#/components/schemas/Owner"}}
        schema = {"type": "object", "required": [field], "properties": {field: value}}
        paths[path] = {"get": {"operationId": name, "responses": {
            "200": {"content": {"application/json": {"schema": schema}}}}}}
    return {"components": {"schemas": {"Owner": {"oneOf": owners}}}, "paths": paths}


def rejected(call):
    try:
        call()
    except (ValueError, KeyError):
        return
    raise AssertionError("unreviewed source shape accepted")


def main():
    source = document()
    original = copy.deepcopy(source)
    table, fixtures = render(source)
    assert source == original
    assert render(source) == (table, fixtures)
    assert set(fixtures) == {name + ".json" for name in PATHS}
    assert json.loads(fixtures["get_team_owners.json"])["teams"][0]["kind"] == "team"
    assert json.loads(fixtures["get_user_owners.json"])["users"][0]["kind"] == "user"
    for name, path in PATHS.items():
        assert name.upper() in table
        changed = copy.deepcopy(source)
        changed["paths"][path]["get"]["operationId"] = "other"
        rejected(lambda: render(changed))
        for constraint in ({"minimum": 0}, {"oneOf": []}, {"$ref": "https://invalid.test/schema"}):
            changed = copy.deepcopy(source)
            schema = changed["paths"][path]["get"]["responses"]["200"]["content"]["application/json"]["schema"]
            schema.update(constraint)
            rejected(lambda: render(changed))
    for minimum in (-1, 1, False, None):
        changed = copy.deepcopy(source)
        schema = changed["paths"][PATHS["get_user_stats"]]["get"]["responses"]["200"]["content"]["application/json"]["schema"]
        schema["properties"]["total_downloads"]["minimum"] = minimum
        rejected(lambda: render(changed))
    assert {p.name for p in (OUTPUT / "fixtures").glob("*.json")} == set(fixtures)
    feature_guards()
    print("Account projections, owner discriminants, statistics bounds and feature guards passed.")


def feature_guards():
    spec = importlib.util.spec_from_file_location("boundary_tests", Path(__file__).with_name("test-cratesio-crate-boundary.py"))
    tests = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(tests)
    root = tests.fixture()
    try:
        module = root / tests.checker.CRATE / "src/accounts/mod.rs"
        original = module.read_text(encoding="ascii")
        guards = [('#[cfg(any(feature = "blocking", feature = "async"))]\nmod client;', "mod client;", "accounts client guard")]
        guards.extend((f'#[cfg(feature = "alloc")]\nmod {name};', f'mod {name};', "accounts allocation guard")
                      for name in ("models", "decode", "schema_table"))
        for guarded, unguarded, message in guards:
            assert guarded in original
            module.write_text(original.replace(guarded, unguarded), encoding="ascii")
            tests.assert_rejected(root, message)
        module.write_text(original, encoding="ascii")
        tests.checker.validate(root)
    finally:
        shutil.rmtree(root)


if __name__ == "__main__":
    main()
