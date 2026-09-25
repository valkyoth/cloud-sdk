#!/usr/bin/env python3
"""Offline token contract and committed-artifact gate regressions."""
import copy
import json
import tempfile
from pathlib import Path
from unittest.mock import patch
import generate_cratesio_tokens as generator


def main():
    document = {"paths": {}, "components": {"schemas": {}}}
    for name, (method, path, status) in generator.PATHS.items():
        response = {} if status == "204" else {"content": {"application/json": {"schema": {"type": "object"}}}}
        document["paths"].setdefault(path, {})[method] = {
            "operationId": name, "responses": {status: response}}
    original = copy.deepcopy(document)
    table = generator.render(document)
    assert document == original
    for name, (method, path, status) in generator.PATHS.items():
        for kind in ("identity", "body", "status", "shape"):
            changed = copy.deepcopy(document)
            op = changed["paths"][path][method]
            if kind == "identity":
                op["operationId"] = "other"
            elif kind == "body":
                op["requestBody"] = {}
            elif kind == "status":
                op["responses"]["201"] = {}
            elif status == "204":
                op["responses"][status]["content"] = {}
            else:
                op["responses"][status]["content"]["application/json"]["schema"] = {"oneOf": []}
            try:
                generator.render(changed)
            except (ValueError, KeyError):
                pass
            else:
                raise AssertionError("unreviewed token contract accepted")
    with tempfile.TemporaryDirectory() as directory:
        output = Path(directory) / "schema.rs"
        with patch.object(generator, "OUTPUT", output), patch.object(
            generator, "fetch_source", return_value=json.dumps(document).encode()
        ), patch("sys.argv", ["generate_cratesio_tokens.py"]):
            output.write_text(table)
            generator.main()
            output.write_text("stale")
            try:
                generator.main()
            except ValueError:
                pass
            else:
                raise AssertionError("stale token table accepted")
            assert output.read_text() == "stale"
    for gate in ("scripts/checks.sh", "scripts/release_1_1_gate.sh"):
        assert "python3 scripts/generate_cratesio_tokens.py" in (generator.ROOT / gate).read_text().splitlines()
    print("Token operation, status, body, schema and gate regressions passed.")


if __name__ == "__main__":
    main()
