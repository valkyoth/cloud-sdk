#!/usr/bin/env python3
"""Regression tests for source-derived Hetzner Cloud response models."""

from __future__ import annotations

import csv
import importlib.util
import io
import json
import subprocess
import sys
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


def synthetic_console_document() -> dict:
    paths = {}
    schemas = {}
    for model in generator.CONSOLE_EXPECTED_MODELS:
        schemas[model] = {
            "type": "object",
            "required": ["id", "state"],
            "properties": {
                "id": {"type": "integer", "minimum": 1},
                "state": {"type": ["string", "null"], "enum": ["known"]},
            },
    }
    for operation_id, (root, model) in generator.CONSOLE_MODEL_OPERATIONS.items():
        schema = json.loads(json.dumps(schemas[model]))
        if operation_id.startswith("list_"):
            schema = {"type": "array", "items": schema}
        operation = response(root, schema)
        operation["operationId"] = operation_id
        paths[f"/{operation_id}"] = {"get": operation}
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


def test_render_covers_every_console_model_and_required_operation() -> None:
    rendered = generator.render(synthetic_document(), synthetic_console_document())
    rows = list(csv.DictReader(io.StringIO(rendered), delimiter="\t"))
    assert {row["model"] for row in rows} == generator.ALL_EXPECTED_MODELS
    assert len(rows) == len(generator.ALL_EXPECTED_MODELS) * 2

    console = synthetic_console_document()
    del console["paths"]["/get_storage_box"]
    assert_raises(
        "missing Console model operations: get_storage_box",
        generator.render,
        synthetic_document(),
        console,
    )

    console = synthetic_console_document()
    changed = console["paths"]["/get_storage_box"]["get"]
    changed["responses"]["200"]["content"]["application/json"]["schema"][
        "properties"
    ]["storage_box"]["properties"]["changed"] = {"type": "boolean"}
    assert_raises(
        "storage_box response schemas are not structurally identical",
        generator.render,
        synthetic_document(),
        console,
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


def test_root_union_is_flattened_only_when_branches_share_one_shape() -> None:
    common = {
        "type": "object",
        "required": ["id"],
        "properties": {"id": {"type": "integer", "minimum": 1}},
    }
    schema = {
        "oneOf": [
            {
                "allOf": [
                    common,
                    {
                        "type": "object",
                        "required": ["mode"],
                        "properties": {"mode": {"type": "string", "enum": ["primary"]}},
                    },
                ]
            },
            {
                "allOf": [
                    common,
                    {
                        "type": "object",
                        "required": ["mode"],
                        "properties": {"mode": {"type": "string", "enum": ["secondary"]}},
                    },
                ]
            },
        ],
        "discriminator": {"propertyName": "mode"},
    }
    flattened = generator.flatten_root_union("zone", schema)
    assert flattened["required"] == ["id", "mode"]
    assert flattened["properties"]["mode"]["enum"] == ["primary", "secondary"]

    schema["oneOf"][1]["allOf"][0] = {
        "type": "object",
        "required": ["id", "changed"],
        "properties": {
            "id": {"type": "integer", "minimum": 1},
            "changed": {"type": "boolean"},
        },
    }
    assert_raises("do not share one field shape", generator.flatten_root_union, "zone", schema)


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
    assert len(rows) == 718
    assert {row["model"] for row in rows} == generator.ALL_EXPECTED_MODELS
    assert set(fixtures) == generator.ALL_EXPECTED_MODELS
    identities = [(row["model"], row["path"]) for row in rows]
    assert len(identities) == len(set(identities))
    assert all(isinstance(fixture, dict) for fixture in fixtures.values())
    assert fixtures["certificate"]["certificate"].startswith(
        "-----BEGIN CERTIFICATE-----\n"
    )
    assert fixtures["ssh_key"]["public_key"].startswith("ssh-ed25519 ")
    assert fixtures["ssh_key"]["fingerprint"] == (
        "ae:6f:ba:1b:70:2c:ae:c7:5c:ab:6e:4d:5e:d4:c7:23"
    )
    assert fixtures["ssh_key"]["public_key"] == (
        "ssh-ed25519 "
        "AAAAC3NzaC1lZDI1NTE5AAAAILM+rvN+ot98qgEN796jTiQfZfG1KaT0PtFDJ/XFSqti "
        "user@example.com"
    )
    assert {row["format"] for row in rows} == {
        "-",
        "date-time",
        "decimal",
        "double",
        "int64",
    }
    assert {row["pattern"] for row in rows} == {
        "-",
        r"^[a-zA-Z0-9 ./_-]+$",
        r"^[a-z0-9]+(-?[a-z0-9]*)*$",
        r"[a-zA-Z0-9-_,:<>+#!\(\)\[\]\{\} ]*",
    }


def test_cli_requires_the_console_specification() -> None:
    result = subprocess.run(
        [sys.executable, str(SCRIPT), "missing-cloud.json"],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    assert result.returncode == 2, result
    assert "--console-spec" in result.stderr and "required" in result.stderr


def main() -> None:
    tests = (
        test_render_covers_every_model_and_open_enum,
        test_render_covers_every_console_model_and_required_operation,
        test_inconsistent_resource_occurrences_are_rejected,
        test_discriminated_union_has_shared_and_selected_paths,
        test_constraints_are_recorded_and_unsupported_constraints_fail_closed,
        test_unknown_schema_composition_fails_closed,
        test_all_of_composition_is_narrow_and_explicit,
        test_committed_evidence_is_structurally_complete,
        test_cli_requires_the_console_specification,
    )
    for test in tests:
        test()
    print(f"{len(tests)} Cloud model generator tests passed.")


if __name__ == "__main__":
    main()
