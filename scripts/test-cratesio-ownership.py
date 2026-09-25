#!/usr/bin/env python3
"""Offline ownership schema and source-verification gate regressions."""
import copy
import json
from unittest.mock import patch
import check_cratesio_ownership as checker


def fixture():
    operations = {}
    for method, name in (("put", "add_owners"), ("delete", "remove_owners")):
        operations[method] = {"operationId": name, "security": [{"api_token": []}, {"cookie": []}],
            "requestBody": {"required": True, "content": {"application/json": {"schema": {
                "type": "object", "required": ["owners"], "properties": {
                    "owners": {"type": "array", "items": {"type": "string"}}}}}}},
            "responses": {"200": {"content": {"application/json": {"schema": {
                "type": "object", "required": ["msg", "ok"], "properties": {
                    "msg": {"type": "string"}, "ok": {"type": "boolean"}}}}}}}}
    return {"paths": {"/api/v1/crates/{name}/owners": operations}}


def main():
    document = fixture()
    checker.validate(document)
    for method in ("put", "delete"):
        for field in ("operationId", "security", "requestBody", "responses"):
            changed = copy.deepcopy(document)
            changed["paths"]["/api/v1/crates/{name}/owners"][method][field] = {}
            try:
                checker.validate(changed)
            except (KeyError, ValueError):
                pass
            else:
                raise AssertionError("changed contract accepted")
    seen = []
    def fetch(source):
        seen.append(source["id"])
        if source["id"] == "openapi":
            return json.dumps(document).encode()
        if source["id"] == "cargo":
            return b"pinned Cargo document"
        assert source == checker.SOURCE
        return b'#[serde(alias = "users")]'
    with patch.object(checker, "fetch_source", side_effect=fetch):
        checker.main()
    assert seen == ["openapi", "cargo", "ownership-namespace-controller"]
    for gate in ("scripts/checks.sh", "scripts/release_1_1_gate.sh"):
        assert "python3 scripts/check_cratesio_ownership.py" in (checker.ROOT / gate).read_text().splitlines()
    print("Ownership schema and verification gate regressions passed.")


if __name__ == "__main__":
    main()
