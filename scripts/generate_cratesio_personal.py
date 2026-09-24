#!/usr/bin/env python3
"""Verify source-locked personal-workflow response and request projections."""
import argparse
import json
from pathlib import Path
from cratesio_source_fetch import fetch_source
from generate_cratesio_catalog import projection

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/cloud-sdk-cratesio/src/accounts/personal/schema_table.rs"
PATHS = {
    "confirm_user_email": ("put", "/api/v1/confirm/{email_token}"),
    "follow_crate": ("put", "/api/v1/crates/{name}/follow"),
    "unfollow_crate": ("delete", "/api/v1/crates/{name}/follow"),
    "accept_crate_owner_invitation_with_token": ("put", "/api/v1/me/crate_owner_invitations/accept/{token}"),
    "handle_crate_owner_invitation": ("put", "/api/v1/me/crate_owner_invitations/{crate_id}"),
    "update_email_notifications": ("put", "/api/v1/me/email_notifications"),
    "resend_email_verification": ("put", "/api/v1/users/{id}/resend"),
    "update_user": ("put", "/api/v1/users/{user}"),
}


def render(document):
    roots = {}
    for name, (method, path) in PATHS.items():
        op = document["paths"][path][method]
        if op["operationId"] != name:
            raise ValueError("personal operation identity changed")
        roots[name.upper()] = op["responses"]["200"]["content"]["application/json"]["schema"]
        if name in ("update_user", "handle_crate_owner_invitation"):
            roots[name.upper() + "_BODY"] = op["requestBody"]["content"]["application/json"]["schema"]
        elif "requestBody" in op:
            raise ValueError("personal operation acquired an unreviewed body")
    table = projection(document, roots)
    table = table.replace("use super::schema::Node;", "use crate::catalog::schema::Node;").replace(
        "generate_cratesio_catalog.py", "generate_cratesio_personal.py")
    for name in ("HANDLE_CRATE_OWNER_INVITATION_BODY", "UPDATE_USER_BODY"):
        table = table.replace(f"pub(super) const {name}:", f"#[cfg(test)]\npub(super) const {name}:")
    return table


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    lock = json.loads((ROOT / "provider-drift/providers/cratesio-source.lock.json").read_bytes())
    source = next(s for s in lock["sources"] if s["id"] == "openapi")
    result = render(json.loads(fetch_source(source)))
    if args.write:
        OUTPUT.write_text(result, encoding="ascii")
    elif OUTPUT.read_text(encoding="ascii") != result:
        raise ValueError("personal projections are stale")
    print("8 personal response and 2 documented body projections passed.")


if __name__ == "__main__":
    main()
