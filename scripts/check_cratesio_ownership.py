#!/usr/bin/env python3
"""Check Cargo alias, public ownership mutation schema and pinned namespace semantics."""
import json
from pathlib import Path
from cratesio_source_fetch import fetch_source
from generate_cratesio_settings import structural

ROOT = Path(__file__).resolve().parents[1]
COMMIT = "8ee3e10d68792af6a8caf362f1ab19cd1a5e7f26"
URL = f"https://raw.githubusercontent.com/rust-lang/crates.io/{COMMIT}/src/controllers/krate/owners.rs"
SOURCE = {"id": "ownership-namespace-controller", "url": URL, "final_url": URL,
          "accept": "text/plain", "media_type": "text/plain", "redirects": [],
          "max_bytes": 65536, "size_bytes": 29752,
          "sha256": "dfca760b7321a85b129d70f67d35eb656dfd84226b9d454fce436a9b2e682963"}


def validate(document):
    operations = document["paths"]["/api/v1/crates/{name}/owners"]
    request = {"type": "object", "required": ["owners"], "properties": {
        "owners": {"type": "array", "items": {"type": "string"}}}}
    response = {"type": "object", "required": ["msg", "ok"], "properties": {
        "msg": {"type": "string"}, "ok": {"type": "boolean"}}}
    for method, name in (("put", "add_owners"), ("delete", "remove_owners")):
        op = operations[method]
        if op["operationId"] != name or op["security"] != [{"api_token": []}, {"cookie": []}]:
            raise ValueError("ownership identity or authority changed")
        if structural(op["requestBody"]) != {"required": True, "content": {
                "application/json": {"schema": request}}}:
            raise ValueError("ownership request contract changed")
        if {s for s in op["responses"] if s.startswith("2")} != {"200"}:
            raise ValueError("ownership success status changed")
        if structural(op["responses"]["200"]["content"]) != {
                "application/json": {"schema": response}}:
            raise ValueError("ownership response contract changed")


def main():
    lock = json.loads((ROOT / "provider-drift/providers/cratesio-source.lock.json").read_bytes())
    validate(json.loads(fetch_source(next(s for s in lock["sources"] if s["id"] == "openapi"))))
    fetch_source(next(s for s in lock["sources"] if s["id"] == "cargo"))
    source = fetch_source(SOURCE).decode("utf-8")
    if '#[serde(alias = "users")]' not in source:
        raise ValueError("Cargo request alias is missing")
    print("Ownership OpenAPI, Cargo contract and supplemental controller digest passed.")


if __name__ == "__main__":
    main()
