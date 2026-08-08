#!/usr/bin/env python3
"""Regression tests for source-derived Hetzner Cloud response models."""

from __future__ import annotations

import csv
import importlib.util
import io
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "generate_cloud_model_schema.py"


def load_generator():
    spec = importlib.util.spec_from_file_location(
        "generate_cloud_model_schema", SCRIPT
    )
    if spec is None or spec.loader is None:
        raise RuntimeError("could not load Cloud model generator")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


generator = load_generator()


def response(root: str, schema: dict) -> dict:
    return {
        "responses": {
            "200": {
                "content": {
                    "application/json": {
                        "schema": {
                            "type": "object",
                            "properties": {root: schema},
                        }
                    }
                }
            }
        }
    }


def synthetic_document() -> dict:
    paths = {}
    for model in sorted(generator.EXPECTED_MODELS):
        schema = {
            "type": "object",
            "required": ["id"],
            "properties": {
                "id": {"type": "integer", "minimum": 1},
                "state": {
                    "type": ["string", "null"],
                    "enum": ["known"],
                },
            },
        }
        paths[f"/{model}"] = {"get": response(model, schema)}
    return {"paths": paths}


def assert_raises(expected: str, function, *args) -> None:
    try:
        function(*args)
    except ValueError as error:
        assert expected in str(error), error
        return
    raise AssertionError("expected ValueError")


def test_render_covers_every_model_and_open_enum() -> None:
    rendered = generator.render(synthetic_document())
    rows = list(csv.DictReader(io.StringIO(rendered), delimiter="\t"))
    assert {row["model"] for row in rows} == generator.EXPECTED_MODELS
    assert len(rows) == len(generator.EXPECTED_MODELS) * 2
    assert all(row["minimum"] == "1" for row in rows if row["path"] == "id")
    assert all(
        row["types"] == "null|string" and row["known_values"] == "known"
        for row in rows
        if row["path"] == "state"
    )


def test_inconsistent_resource_occurrences_are_rejected() -> None:
    document = synthetic_document()
    document["paths"]["/servers"] = {
        "get": response(
            "servers",
            {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["id", "changed"],
                    "properties": {
                        "id": {"type": "integer", "minimum": 1},
                        "changed": {"type": "boolean"},
                    },
                },
            },
        )
    }
    assert_raises(
        "server response schemas are not structurally identical",
        generator.render,
        document,
    )


def test_discriminated_union_has_shared_and_selected_paths() -> None:
    rows = []
    generator.walk_union(
        "load_balancer",
        "services",
        {
            "discriminator": {"propertyName": "protocol"},
            "oneOf": [
                {
                    "type": "object",
                    "required": ["protocol", "port"],
                    "properties": {
                        "protocol": {"type": "string", "enum": ["http"]},
                        "port": {"type": "integer", "minimum": 1},
                    },
                },
                {
                    "type": "object",
                    "required": ["protocol", "certificate"],
                    "properties": {
                        "protocol": {"type": "string", "enum": ["https"]},
                        "certificate": {"type": "integer", "minimum": 1},
                    },
                },
            ],
        },
        rows,
    )
    paths = {row["path"] for row in rows}
    assert "services[]/protocol" in paths
    assert "services[protocol=http]/port" in paths
    assert "services[protocol=https]/certificate" in paths


def test_constraints_are_recorded_and_unsupported_constraints_fail_closed() -> None:
    row = generator.descriptor(
        "location",
        "created",
        True,
        {"type": "string", "format": "date-time", "pattern": r"^\S(.*\S)?$"},
    )
    assert row["format"] == "date-time"
    assert row["pattern"] == r"^\S(.*\S)?$"

    for constraint in sorted(generator.UNSUPPORTED_SECURITY_CONSTRAINTS):
        assert_raises(
            "has unenforced constraints",
            generator.descriptor,
            "server",
            "field",
            True,
            {"type": "string", constraint: 1},
        )
    assert_raises(
        "unsupported format",
        generator.descriptor,
        "server",
        "field",
        True,
        {"type": "string", "format": "future-format"},
    )
    assert_raises(
        "unsupported pattern",
        generator.descriptor,
        "server",
        "field",
        True,
        {"type": "string", "pattern": "^future$"},
    )
    assert_raises(
        "unflattened allOf constraints",
        generator.walk_object,
        "server",
        "field",
        {
            "allOf": [
                {
                    "type": "object",
                    "maxProperties": 4,
                    "properties": {},
                }
            ]
        },
        [],
    )


def test_unknown_schema_composition_fails_closed() -> None:
    for keyword in ("anyOf", "if", "then", "else"):
        assert_raises(
            "has unsupported schema keys",
            generator.descriptor,
            "server",
            "field",
            True,
            {"type": "string", keyword: {}},
        )

    assert_raises(
        "has unsupported schema keys: discriminator, oneOf",
        generator.walk_object,
        "server",
        "",
        {
            "type": "object",
            "properties": {
                "choice": {
                    "oneOf": [
                        {
                            "type": "object",
                            "required": ["secret"],
                            "properties": {
                                "secret": {"type": "string", "minLength": 8}
                            },
                        }
                    ],
                    "discriminator": {"propertyName": "kind"},
                }
            },
        },
        [],
    )


def test_all_of_composition_is_narrow_and_explicit() -> None:
    assert_raises(
        "unsupported sibling schema keys",
        generator.merge_all_of,
        {
            "allOf": [{"type": "object", "properties": {}}],
            "properties": {"lost": {"type": "string"}},
            "required": ["lost"],
        },
    )
    assert_raises(
        "unsupported overlap",
        generator.merge_all_of,
        {
            "allOf": [
                {
                    "type": "object",
                    "properties": {"value": {"type": "integer", "minimum": 1}},
                },
                {
                    "type": "object",
                    "properties": {"value": {"type": "integer", "maximum": 10}},
                },
            ]
        },
    )

    merged = generator.merge_all_of(
        {
            "title": "SelectedProtocol",
            "allOf": [
                {
                    "type": "object",
                    "required": ["protocol"],
                    "properties": {
                        "protocol": {
                            "type": "string",
                            "enum": ["tcp", "http", "https"],
                        }
                    },
                },
                {
                    "type": "object",
                    "properties": {
                        "protocol": {"type": "string", "enum": ["https"]}
                    },
                },
            ],
        }
    )
    assert merged["title"] == "SelectedProtocol"
    assert merged["required"] == ["protocol"]
    assert merged["properties"]["protocol"]["enum"] == ["https"]


def test_committed_evidence_is_structurally_complete() -> None:
    rows = list(
        csv.DictReader(
            generator.DEFAULT_OUTPUT.read_text(encoding="ascii").splitlines(),
            delimiter="\t",
        )
    )
    fixtures = json.loads(generator.DEFAULT_FIXTURES.read_text(encoding="ascii"))
    assert len(rows) == 535
    assert {row["model"] for row in rows} == generator.EXPECTED_MODELS
    assert set(fixtures) == generator.EXPECTED_MODELS
    identities = [(row["model"], row["path"]) for row in rows]
    assert len(identities) == len(set(identities))
    assert all(isinstance(fixture, dict) for fixture in fixtures.values())
    assert {row["format"] for row in rows} == {
        "-",
        "date-time",
        "decimal",
        "double",
        "int64",
    }
    assert {row["pattern"] for row in rows} == {
        "-",
        r"^[a-z0-9]+(-?[a-z0-9]*)*$",
    }


def main() -> None:
    tests = (
        test_render_covers_every_model_and_open_enum,
        test_inconsistent_resource_occurrences_are_rejected,
        test_discriminated_union_has_shared_and_selected_paths,
        test_constraints_are_recorded_and_unsupported_constraints_fail_closed,
        test_unknown_schema_composition_fails_closed,
        test_all_of_composition_is_narrow_and_explicit,
        test_committed_evidence_is_structurally_complete,
    )
    for test in tests:
        test()
    print(f"{len(tests)} Cloud model generator tests passed.")


if __name__ == "__main__":
    main()
