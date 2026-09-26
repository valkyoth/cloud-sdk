"""Schema validation and deterministic control-plane inventory projection."""

from __future__ import annotations

import json
from decimal import Decimal
from pathlib import Path
import re
import subprocess

from scaleway_source_fetch import InventoryError, MAX_SOURCE
from scaleway_path import validate_operation_path

ROOT = Path(__file__).resolve().parents[1]
YAML_TOOL = ROOT / "tools/prepared-coverage-check/target/debug/source-yaml-json"
METHODS = {"get", "post", "put", "patch", "delete", "head", "options", "trace"}
OWNERS = {
    "account": (14,), "annotations": (14,), "iam": (15, 16),
    "billing": (17,), "marketplace": (17,), "product_catalog": (17,),
    "partner": (17,), "reseller": (17,), "instance": (18, 19, 20),
    "autoscaling": (20,), "block": (21,), "file": (22,), "object_admin": (23,),
    "registry": (28,), "vpc": (30, 31), "ipam": (30, 31), "vpcgw": (32,),
    "interlink": (33,), "s2s_vpn": (33,), "edge_services": (34,), "lb": (35, 36),
    "baremetal": (37, 38), "flexible_ip": (38,), "dedibox": (39, 40),
    "apple_silicon": (41,), "k8s": (42,), "function": (43,), "container": (44,),
    "jobs": (45,), "rdb": (46, 47), "redis": (48,), "mongodb": (49,),
    "sdb": (50,), "searchdb": (51,), "kafka": (52,), "nats": (52,),
    "datawarehouse": (53,), "datalab": (53,), "mnq": (54,), "messageq": (57,),
    "iot": (58,), "domain": (59,), "webhosting": (60, 61), "tem": (62,),
    "mailbox": (63,), "cockpit": (64,), "audit_trail": (67,), "environmental_footprint": (67,),
    "secret": (68, 69), "key_manager": (68, 69), "qaas": (70,),
    "inference": (71,), "generative_apis": (71, 72, 73, 74), "test": (),
    "functions": (43,), "containers": (44,), "serverless_jobs": (45,),
    "serverless_sqldb": (50,), "secret_manager": (68, 69),
    "transactional_email": (62,), "vpc_gw": (32,),
}
SDK_ALIASES = {"applesilicon": "apple_silicon", "container": "containers",
               "flexibleip": "flexible_ip", "function": "functions",
               "jobs": "serverless_jobs", "secret": "secret_manager",
               "tem": "transactional_email", "vpcgw": "vpc_gw"}


def load_json(raw: bytes, *, exact_numbers: bool = False) -> object:
    def pairs(entries: list) -> dict:
        result = {}
        for key, value in entries:
            if key in result:
                raise InventoryError("duplicate JSON field")
            result[key] = value
        return result
    def invalid(value: str) -> None:
        raise InventoryError("non-finite JSON value")
    return json.loads(raw, object_pairs_hook=pairs, parse_constant=invalid,
                      parse_float=Decimal if exact_numbers else float)


def parse_schema(raw: bytes) -> dict:
    if len(raw) > MAX_SOURCE:
        raise InventoryError("schema size exceeded")
    try:
        result = subprocess.run([str(YAML_TOOL)], input=raw, stdout=subprocess.PIPE,
                                stderr=subprocess.PIPE, timeout=30, check=True)
    except (OSError, subprocess.SubprocessError):
        raise InventoryError("bounded YAML parsing failed") from None
    if len(result.stdout) > MAX_SOURCE * 6:
        raise InventoryError("converted schema size exceeded")
    schema = load_json(result.stdout, exact_numbers=True)
    if not isinstance(schema, dict) or schema.get("openapi") not in {"3.0.0", "3.0.1", "3.0.2", "3.0.3", "3.1.0"}:
        raise InventoryError("unreviewed OpenAPI dialect")
    check_references(schema)
    return schema


def check_references(schema: dict) -> None:
    pending = [schema]
    while pending:
        node = pending.pop()
        if isinstance(node, list):
            pending.extend(node)
        elif isinstance(node, dict):
            pending.extend(node.values())
            if "$ref" not in node:
                continue
            ref = node["$ref"]
            if not isinstance(ref, str) or not ref.startswith("#/") or len(ref) > 1024:
                raise InventoryError("unreviewed external or malformed schema reference")
            target = schema
            for part in ref[2:].split("/"):
                if re.search(r"~(?![01])", part):
                    raise InventoryError("invalid reference escape")
                key = part.replace("~1", "/").replace("~0", "~")
                if not isinstance(target, dict) or key not in target:
                    raise InventoryError("dangling schema reference")
                target = target[key]


def operations(entry: dict, schema: dict) -> list[dict]:
    if schema.get("info", {}).get("version") != entry["version"]:
        raise InventoryError("schema version disagrees with catalog")
    paths = schema.get("paths")
    if not isinstance(paths, dict) or not paths:
        raise InventoryError("missing operation paths")
    owners = OWNERS.get(entry["family"])
    if owners is None:
        raise InventoryError(f"unassigned schema family: {entry['family']}")
    identifiers = set()
    result = []
    for path in paths:
        validate_operation_path(path)
    for path, item in sorted(paths.items()):
        if not isinstance(item, dict):
            raise InventoryError("invalid operation path")
        for method, operation in sorted(item.items()):
            if method not in METHODS:
                if method not in {"summary", "description", "parameters", "servers"} and not method.startswith("x-"):
                    raise InventoryError("unreviewed path item field")
                continue
            if not isinstance(operation, dict):
                raise InventoryError("invalid operation body")
            identifier = operation.get("operationId")
            if not isinstance(identifier, str) or not identifier or identifier in identifiers:
                raise InventoryError("missing or duplicate operation ID")
            identifiers.add(identifier)
            deprecated = operation.get("deprecated", False)
            if type(deprecated) is not bool:
                raise InventoryError("invalid deprecation flag")
            result.append({"method": method.upper(), "path": path, "operation_id": identifier,
                           "deprecated": deprecated, "owner_candidates": list(owners)})
    if not result:
        raise InventoryError("schema has no recognized operations")
    return result


def sdk_candidates(tree: dict, entries: list[dict]) -> list[dict]:
    if tree.get("truncated") is not False or not isinstance(tree.get("tree"), list):
        raise InventoryError("SDK tree is truncated or malformed")
    documented = {(entry["family"], entry["version"]) for entry in entries}
    families = set()
    for item in tree["tree"]:
        match = re.fullmatch(r"api/([a-z0-9_]+)/(v[1-9][0-9]*(?:(?:alpha|beta)[1-9][0-9]*)?)/[^/]+_sdk.go", item["path"])
        if match:
            family, version = match.groups()
            families.add((SDK_ALIASES.get(family, family), version))
    if not families:
        raise InventoryError("SDK comparison found no contracts")
    return [{"family": family, "version": version, "disposition": "unresolved",
             "reason": "SDK version absent from documentation catalog; public support needs evidence"}
            for family, version in sorted(families - documented)]
