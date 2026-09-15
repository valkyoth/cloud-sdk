#!/usr/bin/env python3
"""Offline version projection and feature-boundary regression tests."""
import copy
import importlib.util
from pathlib import Path
import shutil
from generate_cratesio_downloads import PATHS, OUTPUT, render


def document():
    paths = {}
    for name, path in PATHS.items():
        schema = {"type": "object", "required": ["field"], "properties": {
            "field": {"type": "string", "example": "value"}}}
        paths[path] = {"get": {"operationId": name, "responses": {
            "200": {"content": {"application/json": {"schema": schema}}}}}}
    return {"components": {"schemas": {}}, "paths": paths}


def rejected(call):
    try:
        call()
    except (ValueError, KeyError):
        return
    raise AssertionError("unreviewed source shape accepted")


def main():
    source = document()
    table, fixtures = render(source)
    assert set(fixtures) == {name + ".json" for name in PATHS}
    assert render(source) == (table, fixtures)
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
    assert {p.name for p in (OUTPUT / "fixtures").glob("*.json")} == set(fixtures)
    feature_guards()
    print("Download projection identities, fail-closed schema constraints and feature guards passed.")


def feature_guards():
    spec = importlib.util.spec_from_file_location("boundary_tests", Path(__file__).with_name("test-cratesio-crate-boundary.py"))
    tests = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(tests)
    root = tests.fixture()
    try:
        module = root / tests.checker.CRATE / "src/downloads/mod.rs"
        original = module.read_text(encoding="ascii")
        guards = [('#[cfg(any(feature = "blocking", feature = "async"))]\nmod client;', "mod client;", "downloads client guard")]
        guards.extend((f'#[cfg(feature = "alloc")]\nmod {name};', f'mod {name};', "downloads allocation guard")
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
