#!/usr/bin/env python3
"""Regression tests for compile-time operation association generation."""

from __future__ import annotations

import importlib.util
import sys
import tempfile
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
    associations = generator.read_associations()
    assert len(operations) == 208
    assert len(associations) == 208
    assert len({operation.operation_id for operation in operations}) == 208
    assert generator.pascal("change_zone_rrset_ttl") == "ChangeZoneRrsetTtl"
    by_id = {operation.operation_id: operation for operation in operations}
    assert by_id["get_actions"].query_policy == "required"
    assert by_id["list_servers"].query_policy == "optional"
    assert by_id["list_storage_box_folders"].query_policy == "forbidden"
    assert by_id["list_zones"].service == "dns"
    assert by_id["list_certificates"].service == "security"
    assert generator.source_authentication() == {
        "cloud": "bearer",
        "hetzner": "bearer",
    }
    assert by_id["list_storage_boxes"].authentication == "bearer"
    assert by_id["create_server"].permit_class == "cost"
    assert by_id["delete_server"].permit_class == "destructive"
    assert by_id["update_server"].retry_policy == "explicit"

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
        assert all(marker in generated_row for marker in expected_markers)

    with tempfile.TemporaryDirectory() as directory:
        source = Path(directory) / "associations.tsv"
        source.write_text("operation_id\tservice\nget_action\tcloud\n", encoding="ascii")
        try:
            generator.read_associations(source)
        except ValueError as error:
            assert "invalid schema" in str(error)
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
            assert "unknown permit_class" in str(error)
        else:
            raise AssertionError("unknown classification was accepted")
    generated = generator.formatted_render()
    assert generated.count("Association for Hetzner operation") == 1
    assert generated.count("        (") == 208
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
    assert all(binding in generated for binding in fixed_associations)
    print("208 exhaustive association rows and strict manifest failures checked.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
