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
    "src/controllers/trustpub/github_configs/create.rs": (6554, "2edf1a142f933f0b0fbc202777ca615cd1ad573423302e4a9b0f0340b73380be"),
    "src/controllers/trustpub/gitlab_configs/create.rs": (5370, "1cd641c18dd5848956605a968732a6c0459a4c85bcb32df491937d4545e1aba3"),
    "src/controllers/trustpub/github_configs/list.rs": (9113, "77c61f7abdef8825e03f3b153aef0a69374ffd2c2041d3a3e3ac27e1fa116b2a"),
    "src/controllers/trustpub/gitlab_configs/list.rs": (9067, "1bf62c8beab17eae28d14417c650febd6c64eeeea03303fca9926d28fd4c4261"),
    "src/controllers/trustpub/github_configs/delete.rs": (4078, "5b5a4df728ffc1c5f3bc9af159844d0794fbc7dace785ec6c3e7f09676cff689"),
    "src/controllers/trustpub/gitlab_configs/delete.rs": (4076, "141d893136bf0c94147c15b60d30ae12005307721a84539316be9079f6475281"),
    "src/controllers/trustpub/tokens/exchange/mod.rs": (16254, "fcab24787373533d138a9fa26de09d3457ae8b54be80c7a89cd96ecfc95210c9"),
    "src/controllers/trustpub/tokens/revoke/mod.rs": (1571, "447dd7ec4da30664cd09e02b796235f2f2828e6c6e669ed4fc5eaccc3e8eb2a6"),
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
