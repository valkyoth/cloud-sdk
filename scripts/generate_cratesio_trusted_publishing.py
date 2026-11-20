#!/usr/bin/env python3
"""Source-lock the eight trusted-publishing operations and generate response fixtures."""
import argparse
import json
from pathlib import Path
from cratesio_source_fetch import fetch_source
from generate_cratesio_catalog import projection, expanded_example
from generate_cratesio_settings import structural
from check_cratesio_request_policy import BASE

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/cloud-sdk-cratesio/src/trusted_publishing"
SOURCES = {
    "src/controllers/trustpub/github_configs/create.rs": (6565, "85460e50db0c115cc23451e57fd6197cc669ccb664d8322ecaf70772716582e3"),
    "src/controllers/trustpub/gitlab_configs/create.rs": (5354, "739086dfa16706c2e398e9f9ce6d1aab00e825f69951d4286d7c33bcebcf0afa"),
    "src/controllers/trustpub/github_configs/list.rs": (9074, "d7a16ed1c0799c69a56a415c48e5489d3768568c49d2b64449de36c8b725e77f"),
    "src/controllers/trustpub/gitlab_configs/list.rs": (9028, "eccf2abfa2626fd3444cf729363419eca2cf67d3a92558629d5bdea386cd9f55"),
    "src/controllers/trustpub/github_configs/delete.rs": (4298, "caf8d1269ec7c5609691da8d64f4b0a6c7a3d7e701fb1f99085adac39a96c30d"),
    "src/controllers/trustpub/gitlab_configs/delete.rs": (4296, "03f411797d93aec45115bf19a687fcea2804a09bab90929fb8bd2ef9920b9495"),
    "src/controllers/trustpub/tokens/exchange/mod.rs": (16401, "af7c3c6ac826a720d15f7c514426a342628e76bb6e429d92935ab8f5396229ef"),
    "src/controllers/trustpub/tokens/revoke/mod.rs": (1584, "e2d55fd3b8fcb80361e444c437c0fe8c0cc9ee2bebbdf4fa6b799d59540d045f"),
    "crates/crates_io_trustpub/src/github/validation.rs": (8185, "9f3d5d323ec97014f08328f8cdf9bce0561f6e0e23037197eb9f383e3082bed4"),
    "crates/crates_io_trustpub/src/gitlab/validation.rs": (9334, "7c762f8fe8d4d6c3a842f58d14db8db78b1f472de1c69febc63233cd19ea98b5"),
    "crates/crates_io_trustpub/src/github/claims.rs": (16251, "3dadafe8ccd055af7caf31da44082cd91fe5be2511fc4bb90faca199720d606b"),
    "crates/crates_io_trustpub/src/gitlab/claims.rs": (17446, "cfdc8da11b5f7b2b2e6bd27280950263759a56250ce4b46327c4bdfb35b05214"),
    "crates/crates_io_trustpub/src/github/workflows.rs": (4046, "06314044204c763697db620e20ae3f551eb48903b94ffd5a850c17ace270aa3d"),
    "crates/crates_io_trustpub/src/gitlab/workflows.rs": (4236, "36daf99bcf24e37c7f9e3ef85246e114c37155bb925e1202d7152f35c85ad999"),
    "crates/crates_io_trustpub/src/access_token.rs": (5649, "f499a1dbddd111869978530eb175ccf6b02e3a58948fa3d4ac99f8f0dddc0f04"),
}


def contracts():
    for provider in ("github", "gitlab"):
        path = f"/api/v1/trusted_publishing/{provider}_configs"
        for action, method, suffix, status in (("list", "get", "", "200"), ("create", "post", "", "200"), ("delete", "delete", "/{id}", "204")):
            name = f"{action}_trustpub_{provider}_config" + ("s" if action == "list" else "")
            yield name, method, path + suffix, status, [{"cookie": []}, {"api_token": []}]
    yield "exchange_trustpub_token", "post", "/api/v1/trusted_publishing/tokens", "200", []
    yield "revoke_trustpub_token", "delete", "/api/v1/trusted_publishing/tokens", "204", [{"trustpub_token": []}]


def render(document):
    roots, fixtures, requests = {}, {}, {}
    for name, method, path, status, security in contracts():
        item = document["paths"][path]
        op = item[method]
        if item.get("parameters") or op["operationId"] != name or op.get("security", []) != security:
            raise ValueError("trusted publishing authority or inherited parameters changed")
        if {s for s in op["responses"] if s.startswith("2")} != {status}:
            raise ValueError("trusted publishing status changed")
        requests[name] = structural({k: op[k] for k in ("parameters", "requestBody") if k in op})
        response = op["responses"][status]
        if status == "204":
            if "content" in response:
                raise ValueError("empty response gained a body")
        else:
            if set(response["content"]) != {"application/json"}:
                raise ValueError("trusted publishing response media changed")
            schema = response["content"]["application/json"]["schema"]
            roots[name.upper()] = schema
            # Exchange fixtures deliberately contain no reusable credential.
            if name != "exchange_trustpub_token":
                fixtures[name] = expanded_example(schema, document)
    table = projection(document, roots).replace("use super::schema::Node;", "use crate::catalog::schema::Node;").replace("generate_cratesio_catalog.py", "generate_cratesio_trusted_publishing.py")
    return {"schema_table.rs": table,
            "fixtures.json": json.dumps(fixtures, indent=2, sort_keys=True) + "\n",
            "request_contract.json": json.dumps(requests, indent=2, sort_keys=True) + "\n"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    lock = json.loads((ROOT / "provider-drift/providers/cratesio-source.lock.json").read_bytes())
    result = render(json.loads(fetch_source(next(s for s in lock["sources"] if s["id"] == "openapi"))))
    for path, (size, digest) in SOURCES.items():
        fetch_source(dict(id=path, url=BASE+path, final_url=BASE+path, redirects=[], accept="text/plain", media_type="text/plain", size_bytes=size, max_bytes=131072, sha256=digest))
    for name, text in result.items():
        output = OUTPUT / name
        if args.write:
            output.write_text(text, encoding="ascii")
        elif output.read_text(encoding="ascii") != text:
            raise ValueError("trusted publishing generated contract is stale")
    print("Eight trusted publishing operations and 15 controller/validation sources verified.")


if __name__ == "__main__":
    main()
