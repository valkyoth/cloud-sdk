#!/usr/bin/env python3
"""Regression tests for exhaustive Hetzner Cloud client method generation."""

from __future__ import annotations

import os
import subprocess
import sys

import generate_cloud_client_methods as generator

if sys.flags.optimize:
    raise SystemExit("security regression tests must not run with Python optimization")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> int:
    operations = [
        operation
        for operation in generator.associations.load_operations()
        if operation.service == "cloud"
    ]
    require(len(operations) == 139, "Cloud operation count changed")
    operation_ids = [operation.operation_id for operation in operations]
    require(operation_ids == sorted(operation_ids), "Cloud operations are not sorted")
    require(len(operation_ids) == len(set(operation_ids)), "Cloud operations are not unique")

    permit_counts = {
        permit: sum(operation.permit_class == permit for operation in operations)
        for permit in ("none", "mutation", "destructive", "cost")
    }
    require(
        permit_counts
        == {"none": 55, "mutation": 37, "destructive": 37, "cost": 10},
        "Cloud permit classes changed",
    )
    require(
        sum(operation.pagination == "yes" for operation in operations) == 29,
        "Cloud pagination count changed",
    )

    rendered = generator.render()
    generated_rows = rendered.rsplit("cloud_client_methods!(", 1)[1]
    require(generated_rows.count("    (") == 139, "generated row count changed")
    for operation in operations:
        row = generator.render_row(operation)
        name = operation.operation_id
        expected = (
            f"prepare_{name}",
            f"{name}_blocking",
            f"{name}_async",
            f"{name}_local_async",
            operation.permit_class,
        )
        require(all(value in row for value in expected), f"method row changed for {name}")

    require(
        "CustomEndpointTrust" not in rendered,
        "generated execution escaped official endpoint trust",
    )
    require(
        "PreparationStorageGuard" in rendered,
        "state-changing preparation lost cleanup-owning storage",
    )
    require(
        "AssociatedPermitAttempt" in rendered,
        "state-changing execution lost plan-confirm permits",
    )
    permitted = rendered.split("macro_rules! permitted_method", 1)[1].split(
        "macro_rules! cloud_client_method", 1
    )[0]
    require(
        "pub async fn $local" not in permitted,
        "local permit execution can defer cleanup until its first poll",
    )
    require(
        "async move {\n                    attempt" not in permitted,
        "Send permit execution can defer cleanup until its first poll",
    )
    require(
        permitted.count("attempt.execute_async(clock, self.transport(), body, headers)") == 1,
        "Send permit execution no longer constructs the cleanup future eagerly",
    )
    require(
        permitted.count(
            "attempt.execute_local_async(clock, self.transport(), body, headers)"
        )
        == 1,
        "local permit execution no longer constructs the cleanup future eagerly",
    )

    optimized = subprocess.run(
        [sys.executable, "-O", str(__file__)],
        cwd=generator.ROOT,
        env={**os.environ, "PYTHONOPTIMIZE": ""},
        capture_output=True,
        text=True,
        check=False,
    )
    require(optimized.returncode != 0, "optimized test execution was accepted")
    require(
        "must not run with Python optimization" in optimized.stderr,
        "optimized rejection was not explicit",
    )
    print("139 Cloud client methods, policy classes, and eager cleanup paths tested.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
