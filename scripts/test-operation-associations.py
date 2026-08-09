#!/usr/bin/env python3
"""Regression tests for compile-time operation association generation."""

from __future__ import annotations

import importlib.util
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "generate_operation_associations.py"

if sys.flags.optimize:
    raise SystemExit("security regression tests must not run with Python optimization")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


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
    associations = generator.read_associations()
    require(len(operations) == 208, "operation count changed")
    require(len(associations) == 208, "association count changed")
    require(
        len({operation.operation_id for operation in operations}) == 208,
        "operation IDs are not unique",
    )
    require(
        generator.pascal("change_zone_rrset_ttl") == "ChangeZoneRrsetTtl",
        "marker naming changed",
    )
    by_id = {operation.operation_id: operation for operation in operations}
    expectations = (
        (by_id["get_actions"].query_policy == "required", "required query changed"),
        (by_id["list_servers"].query_policy == "optional", "optional query changed"),
        (
            by_id["list_storage_box_folders"].query_policy == "forbidden",
            "forbidden query changed",
        ),
        (by_id["list_zones"].service == "dns", "DNS service changed"),
        (by_id["list_certificates"].service == "security", "security service changed"),
        (
            generator.source_authentication()
            == {"cloud": "bearer", "hetzner": "bearer"},
            "source authentication changed",
        ),
        (
            by_id["list_storage_boxes"].authentication == "bearer",
            "storage authentication changed",
        ),
        (by_id["create_server"].permit_class == "cost", "cost permit changed"),
        (
            by_id["delete_server"].permit_class == "destructive",
            "destructive permit changed",
        ),
        (
            by_id["update_server"].retry_policy == "explicit",
            "retry policy changed",
        ),
        (
            by_id["get_storage_box"].response_identity == "exact-resource",
            "exact response identity changed",
        ),
        (
            by_id["list_storage_box_snapshots"].response_identity == "parent-resource",
            "parent response identity changed",
        ),
        (
            by_id["list_servers"].response_identity == "none",
            "default response identity changed",
        ),
        (len(generator.read_response_identities()) == 11, "identity count changed"),
    )
    for condition, message in expectations:
        require(condition, message)

    for operation in operations:
        generated_row = generator.row(operation)
        expected_markers = (
            generator.SERVICE_TYPES[operation.service][0],
            generator.SERVICE_TYPES[operation.service][1],
            generator.AUTHENTICATION_TYPES[operation.authentication],
            generator.METHOD_TYPES[operation.method],
            generator.QUERY_TYPES[operation.query_policy],
            generator.BODY_TYPES[operation.body_policy],
            generator.STATUS_TYPES[operation.status],
            generator.RESPONSE_TYPES[operation.response],
            (
                "NumberedPagination"
                if operation.pagination == "yes"
                else "NoPagination"
            ),
            generator.RETRY_TYPES[operation.retry_policy],
            generator.PERMIT_TYPES[operation.permit_class],
        )
        require(
            all(marker in generated_row for marker in expected_markers),
            f"generated marker binding changed for {operation.operation_id}",
        )

    with tempfile.TemporaryDirectory() as directory:
        source = Path(directory) / "associations.tsv"
        source.write_text("operation_id\tservice\nget_action\tcloud\n", encoding="ascii")
        try:
            generator.read_associations(source)
        except ValueError as error:
            require("invalid schema" in str(error), "schema error changed")
        else:
            raise AssertionError("invalid schema was accepted")

        invalid = associations[0].copy()
        invalid["permit_class"] = "implicit"
        source.write_text(
            "\t".join(generator.ASSOCIATION_COLUMNS)
            + "\n"
            + "\t".join(invalid[column] for column in generator.ASSOCIATION_COLUMNS)
            + "\n",
            encoding="ascii",
        )
        try:
            generator.read_associations(source)
        except ValueError as error:
            require("unknown permit_class" in str(error), "permit error changed")
        else:
            raise AssertionError("unknown classification was accepted")

        identity_source = Path(directory) / "identities.tsv"
        identity_source.write_text(
            "operation_id\tresponse_identity\nget_server\tunknown\n",
            encoding="ascii",
        )
        try:
            generator.read_response_identities(identity_source)
        except ValueError as error:
            require("invalid response identity" in str(error), "identity error changed")
        else:
            raise AssertionError("unknown response identity was accepted")
    generated = generator.formatted_render()
    require(
        generated.count("Association for Hetzner operation") == 1,
        "generated operation documentation changed",
    )
    require(generated.count("        (") == 208, "generated row count changed")
    fixed_associations = (
        "type AuthenticationScope = RequiredServiceScope;",
        "type RequestHeaders = body_headers!($body);",
        "type RequestMedia = body_media!($body);",
        "type SuccessBody = success_body!($response);",
        "type SuccessMedia = success_media!($response);",
        "type ErrorBody = JsonErrorBody;",
        "type ErrorMedia = JsonErrorMedia;",
        "type ResponseCaps = JsonResponseCaps;",
        "type Quota = HetznerQuota;",
        "type Streaming = BufferedStreaming;",
        "type Error = HetznerErrorResponse;",
    )
    require(
        all(binding in generated for binding in fixed_associations),
        "fixed associated types changed",
    )
    optimized = subprocess.run(
        [sys.executable, "-O", str(Path(__file__).resolve())],
        cwd=ROOT,
        env={**os.environ, "PYTHONOPTIMIZE": ""},
        check=False,
        capture_output=True,
        text=True,
    )
    require(optimized.returncode != 0, "optimized test execution was accepted")
    require(
        "must not run with Python optimization" in optimized.stderr,
        "optimized execution did not fail for the expected reason",
    )
    print("208 exhaustive association rows and strict manifest failures checked.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
