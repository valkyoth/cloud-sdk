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


def main() -> None:
    tests = (
        test_render_covers_every_model_and_open_enum,
        test_inconsistent_resource_occurrences_are_rejected,
        test_discriminated_union_has_shared_and_selected_paths,
        test_committed_evidence_is_structurally_complete,
    )
    for test in tests:
        test()
    print(f"{len(tests)} Cloud model generator tests passed.")


if __name__ == "__main__":
    main()
