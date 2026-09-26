#!/usr/bin/env python3
"""Offline regression checks for all trusted publishing source contracts."""
import copy
import json
import tempfile
from pathlib import Path
from unittest.mock import patch
import generate_cratesio_trusted_publishing as generator


def fixture():
    document = {"paths": {}, "components": {"schemas": {}}}
    for name, method, path, status, security in generator.contracts():
        response = {} if status == "204" else {"content": {"application/json": {
            "schema": {"type": "object", "required": ["token"], "properties": {
                "token": {"type": "string"}}}}}}
        document["paths"].setdefault(path, {})[method] = {
            "operationId": name, "security": security,
            "responses": {status: response}, "parameters": []}
    return document


def rejected(document):
    try:
        generator.render(document)
    except (KeyError, ValueError):
        return
    raise AssertionError("changed security contract accepted")


def main():
    document = fixture()
    original = copy.deepcopy(document)
    output = generator.render(document)
    assert document == original
    for _, method, path, status, _ in generator.contracts():
        for mutation in ("name", "auth", "status", "inherited", "media"):
            changed = copy.deepcopy(document)
            op = changed["paths"][path][method]
            if mutation == "name":
                op["operationId"] = "other"
            elif mutation == "auth":
                op["security"] = [{"unreviewed": []}]
            elif mutation == "status":
                op["responses"]["201"] = {}
            elif mutation == "inherited":
                changed["paths"][path]["parameters"] = [{"name": "force", "in": "query"}]
            elif status == "204":
                op["responses"][status]["content"] = {}
            else:
                op["responses"][status]["content"]["text/plain"] = {}
            rejected(changed)
        changed = copy.deepcopy(document)
        changed["paths"][path][method]["parameters"] = [{"name": "new", "in": "query", "schema": {"type": "string"}}]
        assert generator.render(changed)["request_contract.json"] != output["request_contract.json"]
    seen = []
    def fetch(source):
        seen.append(source["id"])
        return json.dumps(document).encode() if source["id"] == "openapi" else b"verified"
    with tempfile.TemporaryDirectory() as directory:
        directory = Path(directory)
        for name, text in output.items():
            (directory / name).write_text(text)
        with patch.object(generator, "OUTPUT", directory), patch.object(
            generator, "fetch_source", side_effect=fetch
        ), patch("sys.argv", ["generator"]):
            generator.main()
            assert seen == ["openapi", *generator.SOURCES]
            (directory / "request_contract.json").write_text("stale")
            try:
                generator.main()
            except ValueError:
                pass
            else:
                raise AssertionError("stale request evidence accepted")
            assert (directory / "request_contract.json").read_text() == "stale"
    for gate in ("scripts/checks.sh", "scripts/release_1_1_gate.sh"):
        lines = (generator.ROOT / gate).read_text().splitlines()
        assert "python3 scripts/generate_cratesio_trusted_publishing.py" in lines
        assert "python3 scripts/test-cratesio-trusted-publishing.py" in lines
        commands = (generator.ROOT / gate).read_text().replace("\\\n", "")
        assert ("cargo test --locked --release -p cloud-sdk-cratesio --no-default-features --features std "
                "    trusted_publishing::tests::assertions::preflight_on_bounded_stack -- --exact") in commands
    print("Eight trusted publishing contract, source and gate regression groups passed.")


if __name__ == "__main__":
    main()
