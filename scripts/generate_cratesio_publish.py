#!/usr/bin/env python3
"""Verify stable Cargo publication framing sources and crates.io response schema."""
import argparse
import json
from pathlib import Path
from cratesio_source_fetch import fetch_source
from generate_cratesio_catalog import projection, expanded_example
from generate_cratesio_settings import structural
from check_cratesio_request_policy import SOURCES, BASE

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/cloud-sdk-cratesio/src/publishing/publish"


def render(document):
    path = document["paths"]["/api/v1/crates/new"]
    op = path["put"]
    if op["operationId"] != "publish" or op["security"] != [{"api_token": []}, {"trustpub_token": []}, {"cookie": []}]:
        raise ValueError("publication authority changed")
    if "requestBody" in op or op.get("parameters") or path.get("parameters"):
        raise ValueError("publish framing needs review against new OpenAPI")
    if {s for s in op["responses"] if s.startswith("2")} != {"200"}:
        raise ValueError("publish status changed")
    content = op["responses"]["200"]["content"]
    if set(content) != {"application/json"}:
        raise ValueError("publish response media changed")
    schema = content["application/json"]["schema"]
    if structural(schema) != {"type": "object", "required": ["crate", "warnings"], "properties": {
            "crate": {"$ref": "#/components/schemas/Crate"}, "warnings": {"$ref": "#/components/schemas/PublishWarnings"}}}:
        raise ValueError("publish acknowledgement changed")
    table = projection(document, {"PUBLISH": schema}).replace("use super::schema::Node;", "use crate::catalog::schema::Node;").replace("generate_cratesio_catalog.py", "generate_cratesio_publish.py")
    return table, json.dumps(expanded_example(schema, document), indent=2, sort_keys=True) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    lock = json.loads((ROOT / "provider-drift/providers/cratesio-source.lock.json").read_bytes())
    results = render(json.loads(fetch_source(next(s for s in lock["sources"] if s["id"] == "openapi"))))
    fetch_source(next(s for s in lock["sources"] if s["id"] == "cargo"))
    for path in ("src/controllers/krate/publish.rs", "crates/crates_io_validation/src/lib.rs"):
        size, digest = SOURCES[path]
        fetch_source({"id": path, "url": BASE + path, "final_url": BASE + path, "redirects": [],
            "accept": "text/plain", "media_type": "text/plain", "size_bytes": size, "max_bytes": 131072, "sha256": digest})
    for name, text in zip(("schema_table.rs", "success.json"), results):
        target = OUTPUT / name
        if args.write:
            target.write_text(text, encoding="ascii")
        elif target.read_text(encoding="ascii") != text:
            raise ValueError("publish schema/fixture is stale")
    print("Publish Cargo/controller source digests and response projection passed.")


if __name__ == "__main__":
    main()
