#!/usr/bin/env python3
"""Verify bodyless Cargo yank/unyank contracts and pinned implementation semantics."""
import json
from pathlib import Path
from cratesio_source_fetch import fetch_source
from generate_cratesio_settings import structural
from check_cratesio_request_policy import BASE, SOURCES

ROOT = Path(__file__).resolve().parents[1]


def validate(document):
    response = {"type": "object", "required": ["ok"], "properties": {
        "ok": {"type": "boolean"}}}
    parameters = [{"name": name, "in": "path", "required": True,
                   "schema": {"type": "string"}} for name in ("name", "version")]
    for verb, action in (("delete", "yank"), ("put", "unyank")):
        path = document["paths"][f"/api/v1/crates/{{name}}/{{version}}/{action}"]
        op = path[verb]
        if op["operationId"] != f"{action}_version" or op["security"] != [{"api_token": []}, {"cookie": []}]:
            raise ValueError("yank identity or authority changed")
        if "requestBody" in op or path.get("parameters") or structural(op["parameters"]) != parameters:
            raise ValueError("bodyless yank request or parameter contract changed")
        if {s for s in op["responses"] if s.startswith("2")} != {"200"}:
            raise ValueError("yank success status changed")
        if structural(op["responses"]["200"]["content"]) != {
                "application/json": {"schema": response}}:
            raise ValueError("yank response contract changed")


def main():
    lock = json.loads((ROOT / "provider-drift/providers/cratesio-source.lock.json").read_bytes())
    validate(json.loads(fetch_source(next(s for s in lock["sources"] if s["id"] == "openapi"))))
    fetch_source(next(s for s in lock["sources"] if s["id"] == "cargo"))
    for path in ("src/controllers/version/yank.rs", "src/controllers/version/update.rs"):
        size, digest = SOURCES[path]
        fetch_source({"id": path, "url": BASE + path, "final_url": BASE + path,
            "redirects": [], "accept": "text/plain", "media_type": "text/plain",
            "size_bytes": size, "max_bytes": 131072, "sha256": digest})
    print("Yank Cargo/OpenAPI contracts and implementation digests passed.")


if __name__ == "__main__":
    main()
