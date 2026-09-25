#!/usr/bin/env python3
"""Offline regressions for yank wire contracts and verification gate coverage."""
import copy
import json
import re
from unittest.mock import patch
import check_cratesio_yank as checker


def validate_checkpoint_notes(text):
    section = text.split("### Commit 17 - Cargo Yank And Unyank\n", 1)[1].split("\n### ", 1)[0]
    if set(re.findall(r"stop before Commit (\d+)", section, re.IGNORECASE)) != {"18"}:
        raise ValueError("contradictory yank pentest stop")
    if set(re.findall(r"`([0-9a-f]{8})`", section)) != {"9fabe832"}:
        raise ValueError("contradictory yank pentest baseline")


def fixture():
    paths = {}
    for verb, action in (("delete", "yank"), ("put", "unyank")):
        paths[f"/api/v1/crates/{{name}}/{{version}}/{action}"] = {verb: {
            "operationId": f"{action}_version", "security": [{"api_token": []}, {"cookie": []}],
            "parameters": [{"name": n, "in": "path", "required": True,
                            "schema": {"type": "string"}} for n in ("name", "version")],
            "responses": {"200": {"content": {"application/json": {"schema": {
                "type": "object", "required": ["ok"], "properties": {"ok": {"type": "boolean"}}}}}}}}}
    return {"paths": paths}


def main():
    notes = (checker.ROOT / "release-notes/RELEASE_NOTES_1.1.0.md").read_text()
    validate_checkpoint_notes(notes)
    marker = "### Commit 17 - Cargo Yank And Unyank\n"
    stale = notes.replace(marker, marker + "Pentest from `42e534bd`.\nStop before Commit 17.\n", 1)
    try:
        validate_checkpoint_notes(stale)
    except ValueError:
        pass
    else:
        raise AssertionError("contradictory pentest boundaries accepted")
    document = fixture()
    checker.validate(document)
    for verb, action in (("delete", "yank"), ("put", "unyank")):
        path = f"/api/v1/crates/{{name}}/{{version}}/{action}"
        for field, replacement in (("operationId", "wrong"), ("security", []),
                ("parameters", []), ("requestBody", {}), ("responses", {}),
                ("responses", {"201": document["paths"][path][verb]["responses"]["200"]})):
            changed = copy.deepcopy(document)
            changed["paths"][path][verb][field] = replacement
            try:
                checker.validate(changed)
            except (KeyError, ValueError):
                pass
            else:
                raise AssertionError(f"changed {field} contract accepted")
        changed = copy.deepcopy(document)
        changed["paths"][path]["parameters"] = [{"in": "query", "name": "force"}]
        try:
            checker.validate(changed)
        except ValueError:
            pass
        else:
            raise AssertionError("inherited parameters accepted")
    seen = []
    def fetch(source):
        seen.append(source["id"])
        if source["id"] == "openapi":
            return json.dumps(document).encode()
        return b"digest verified by fetch_source"
    with patch.object(checker, "fetch_source", side_effect=fetch):
        checker.main()
    assert seen == ["openapi", "cargo", "src/controllers/version/yank.rs", "src/controllers/version/update.rs"]
    for gate in ("scripts/checks.sh", "scripts/release_1_1_gate.sh"):
        assert "python3 scripts/check_cratesio_yank.py" in (checker.ROOT / gate).read_text().splitlines()
    print("Yank schema, source and gate regressions passed.")


if __name__ == "__main__":
    main()
