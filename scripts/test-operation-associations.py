#!/usr/bin/env python3
"""Regression tests for compile-time operation association generation."""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "generate_operation_associations.py"


def load_generator():
    specification = importlib.util.spec_from_file_location("operation_associations", SCRIPT)
    if specification is None or specification.loader is None:
        raise AssertionError("generator module cannot be loaded")
    module = importlib.util.module_from_spec(specification)
    sys.modules[specification.name] = module
    specification.loader.exec_module(module)
    return module


def main() -> int:
    generator = load_generator()
    operations = generator.load_operations()
    assert len(operations) == 208
    assert len({operation.operation_id for operation in operations}) == 208
    assert generator.pascal("change_zone_rrset_ttl") == "ChangeZoneRrsetTtl"
    by_id = {operation.operation_id: operation for operation in operations}
    assert generator.query(by_id["get_actions"]) == "RequiredQuery"
    assert generator.query(by_id["list_servers"]) == "OptionalQuery"
    assert generator.query(by_id["list_storage_box_folders"]) == "QueryForbidden"
    assert generator.query(by_id["get_server"]) == "QueryForbidden"
    assert generator.service(by_id["list_zones"])[0] == "DnsService"
    assert generator.service(by_id["list_certificates"])[0] == "SecurityService"
    assert generator.service(by_id["list_storage_boxes"])[0] == "StorageService"
    assert generator.permit(by_id["create_server"]) == "CostPermit"
    assert generator.permit(by_id["delete_server"]) == "DestructivePermit"
    assert generator.permit(by_id["update_server"]) == "MutationPermit"
    assert generator.permit(by_id["get_server"]) == "NoPermit"
    generated = generator.formatted_render()
    assert generated.count("Association for Hetzner operation") == 1
    assert generated.count("        (") == 208
    print("13 operation association generator checks passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
