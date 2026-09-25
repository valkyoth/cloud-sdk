#!/usr/bin/env python3
"""Offline publication schema, pinned-source and gate regression checks."""
import copy
import json
import tempfile
from pathlib import Path
from unittest.mock import patch
import generate_cratesio_publish as generator


def fixture():
    schema = {"type": "object", "required": ["crate", "warnings"], "properties": {
        "crate": {"$ref": "#/components/schemas/Crate"},
        "warnings": {"$ref": "#/components/schemas/PublishWarnings"}}}
    return {"paths": {"/api/v1/crates/new": {"put": {
        "operationId": "publish", "security": [{"api_token": []}, {"trustpub_token": []}, {"cookie": []}],
        "responses": {"200": {"content": {"application/json": {"schema": schema}}}}}}},
        "components": {"schemas": {"Crate": {"type": "object"},
            "PublishWarnings": {"type": "object", "required": ["other"], "properties": {
                "other": {"type": "array", "items": {"type": "string"}}}}}}}


def rejected(document):
    try:
        generator.render(document)
    except (KeyError, ValueError):
        return
    raise AssertionError("unreviewed publish contract accepted")


def main():
    document = fixture()
    original = copy.deepcopy(document)
    table, success = generator.render(document)
    assert document == original
    for field, value in (("operationId", "other"), ("security", []),
                         ("parameters", [{"in": "query", "name": "force"}]),
                         ("requestBody", {}), ("responses", {})):
        changed = copy.deepcopy(document)
        changed["paths"]["/api/v1/crates/new"]["put"][field] = value
        rejected(changed)
    for mutation in ("inherited", "status", "media", "schema"):
        changed = copy.deepcopy(document)
        path = changed["paths"]["/api/v1/crates/new"]
        response = path["put"]["responses"]
        if mutation == "inherited":
            path["parameters"] = [{"in": "query", "name": "force"}]
        elif mutation == "status":
            response["201"] = response["200"]
        elif mutation == "media":
            response["200"]["content"]["text/plain"] = {}
        else:
            response["200"]["content"]["application/json"]["schema"]["required"] = []
        rejected(changed)
    seen = []
    def fetch(source):
        seen.append(source["id"])
        return json.dumps(document).encode() if source["id"] == "openapi" else b"source verified"
    with tempfile.TemporaryDirectory() as directory:
        output = Path(directory)
        (output / "schema_table.rs").write_text(table)
        (output / "success.json").write_text(success)
        with patch.object(generator, "OUTPUT", output), patch.object(
            generator, "fetch_source", side_effect=fetch
        ), patch("sys.argv", ["generate_cratesio_publish.py"]):
            generator.main()
            assert seen == ["openapi", "cargo", "src/controllers/krate/publish.rs",
                            "crates/crates_io_validation/src/lib.rs"]
            (output / "success.json").write_text("stale")
            try:
                generator.main()
            except ValueError:
                pass
            else:
                raise AssertionError("stale publish fixture accepted")
            assert (output / "success.json").read_text() == "stale"
    for gate in ("scripts/checks.sh", "scripts/release_1_1_gate.sh"):
        lines = (generator.ROOT / gate).read_text().splitlines()
        assert "python3 scripts/generate_cratesio_publish.py" in lines
        assert "python3 scripts/test-cratesio-publish.py" in lines
    print("Publish schema, pinned source and gate regression checks passed.")


if __name__ == "__main__":
    main()
