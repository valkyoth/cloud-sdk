#!/usr/bin/env python3
"""Verify all three source-locked public token-management response contracts."""
import argparse
import json
from pathlib import Path
from cratesio_source_fetch import fetch_source
from generate_cratesio_catalog import projection

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/cloud-sdk-cratesio/src/accounts/tokens/schema_table.rs"
PATHS = {
    "find_api_token": ("get", "/api/v1/me/tokens/{id}", "200"),
    "revoke_api_token": ("delete", "/api/v1/me/tokens/{id}", "200"),
    "revoke_current_api_token": ("delete", "/api/v1/tokens/current", "204"),
}


def render(document):
    roots = {}
    for name, (method, path, status) in PATHS.items():
        op = document["paths"][path][method]
        if op["operationId"] != name or "requestBody" in op:
            raise ValueError("token operation contract changed")
        statuses = {s for s in op["responses"] if s.startswith("2")}
        if statuses != {status}:
            raise ValueError("token success status changed")
        success = op["responses"][status]
        if status == "204":
            if "content" in success:
                raise ValueError("self-revocation acquired a response body")
        else:
            schema = success["content"]["application/json"]["schema"]
            if name == "revoke_api_token":
                if schema != {"type": "object"}:
                    raise ValueError("revoke response shape changed")
            else:
                roots[name.upper()] = schema
    return projection(document, roots).replace(
        "use super::schema::Node;", "use crate::catalog::schema::Node;"
    ).replace("generate_cratesio_catalog.py", "generate_cratesio_tokens.py")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    lock = json.loads((ROOT / "provider-drift/providers/cratesio-source.lock.json").read_bytes())
    result = render(json.loads(fetch_source(next(s for s in lock["sources"] if s["id"] == "openapi"))))
    if args.write:
        OUTPUT.write_text(result, encoding="ascii")
    elif OUTPUT.read_text(encoding="ascii") != result:
        raise ValueError("token projection is stale")
    print("Three public token operation response contracts passed.")


if __name__ == "__main__":
    main()
