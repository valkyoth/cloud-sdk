#!/usr/bin/env python3
"""Offline regressions for the complete personal projection inventory."""
import copy
from generate_cratesio_personal import PATHS, render


def main():
    document = {"paths": {}, "components": {"schemas": {}}}
    schema = {"type": "object", "required": ["ok"], "properties": {"ok": {"type": "boolean"}}}
    for name, (method, path) in PATHS.items():
        op = {"operationId": name, "responses": {"200": {"content": {"application/json": {"schema": schema}}}}}
        if name in ("update_user", "handle_crate_owner_invitation"):
            op["requestBody"] = {"content": {"application/json": {"schema": schema}}}
        document["paths"].setdefault(path, {})[method] = op
    original = copy.deepcopy(document)
    table = render(document)
    assert original == document
    assert render(document) == table
    for name, (method, path) in PATHS.items():
        assert name.upper() in table
        for mutation in ("identity", "schema", "body"):
            changed = copy.deepcopy(document)
            op = changed["paths"][path][method]
            if mutation == "identity":
                op["operationId"] = "wrong"
            elif mutation == "schema":
                op["responses"]["200"]["content"]["application/json"]["schema"]["oneOf"] = []
            elif name in ("update_user", "handle_crate_owner_invitation"):
                del op["requestBody"]
            else:
                op["requestBody"] = {}
            try:
                render(changed)
            except (ValueError, KeyError):
                pass
            else:
                raise AssertionError("personal projection accepted unreviewed contract")
    print("Personal operation inventory and projection rejection regressions passed.")


if __name__ == "__main__":
    main()
