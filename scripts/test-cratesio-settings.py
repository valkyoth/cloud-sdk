#!/usr/bin/env python3
"""Offline settings request/schema drift and gate regression tests."""
import copy
import json
import tempfile
from pathlib import Path
from unittest.mock import patch
import generate_cratesio_settings as generator


def fixture():
    document = {"paths": {}, "components": {"schemas": {
        "Crate": {"type": "object"}, "Version": {"type": "object"},
        "VersionUpdate": {"type": "object", "properties": {
            "yanked": {"type": ["boolean", "null"]},
            "yank_message": {"type": ["string", "null"]}}}}}}
    for name, path in generator.PATHS.items():
        field = "crate" if name == "update_crate" else "version"
        body = {"type": "object", "required": [field], "properties": {field:
            {"oneOf": [{"type": "object", "properties": {
                "trustpub_only": {"type": ["boolean", "null"]}}}]}
            if field == "crate" else {"$ref": "#/components/schemas/VersionUpdate"}}}
        schema = {"type": "object", "required": [field], "properties": {
            field: {"$ref": "#/components/schemas/" + field.capitalize()}}}
        responses = {"200": {"content": {"application/json": {"schema": schema}}}}
        document["paths"][path] = {
            "patch": {"operationId": name, "security": [{"api_token": []}, {"cookie": []}],
                "requestBody": {"required": True, "content": {"application/json": {"schema": body}}},
                "responses": responses},
            "get": {"responses": copy.deepcopy(responses)}}
    return document


def rejected(document):
    try:
        generator.render(document)
    except (KeyError, ValueError):
        return
    raise AssertionError("unreviewed contract accepted")


def main():
    document = fixture()
    original = copy.deepcopy(document)
    table = generator.render(document)
    assert document == original
    for path in generator.PATHS.values():
        for kind in ("identity", "auth", "required", "field", "status", "response"):
            changed = copy.deepcopy(document)
            op = changed["paths"][path]["patch"]
            if kind == "identity":
                op["operationId"] = "other"
            elif kind == "auth":
                op["security"] = []
            elif kind == "required":
                op["requestBody"]["required"] = False
            elif kind == "field":
                op["requestBody"]["content"]["application/json"]["schema"]["properties"]["other"] = {"type": "string"}
            elif kind == "status":
                op["responses"]["204"] = {}
            else:
                op["responses"]["200"]["content"]["application/json"]["schema"]["required"] = []
            rejected(changed)
    changed = copy.deepcopy(document)
    changed["components"]["schemas"]["VersionUpdate"]["properties"]["yank_message"] = {"type": "string"}
    rejected(changed)
    changed = copy.deepcopy(document)
    changed["paths"][generator.PATHS["update_version"]]["get"]["responses"]["200"]["content"]["application/json"]["schema"]["required"] = []
    rejected(changed)
    with tempfile.TemporaryDirectory() as directory:
        output = Path(directory) / "schema.rs"
        with patch.object(generator, "OUTPUT", output), patch.object(
            generator, "fetch_source", return_value=json.dumps(document).encode()
        ), patch("sys.argv", ["generate_cratesio_settings.py"]):
            output.write_text(table)
            generator.main()
            output.write_text("stale")
            try:
                generator.main()
            except ValueError:
                pass
            else:
                raise AssertionError("stale artifact accepted")
            assert output.read_text() == "stale"
    for gate in ("scripts/checks.sh", "scripts/release_1_1_gate.sh"):
        assert "python3 scripts/generate_cratesio_settings.py" in (generator.ROOT / gate).read_text().splitlines()
    print("Settings contract drift and gate regressions passed.")


if __name__ == "__main__":
    main()
