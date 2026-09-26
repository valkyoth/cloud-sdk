#!/usr/bin/env python3
"""Deterministic semantic mutations of the independent drift fixture."""

import importlib.util
from pathlib import Path
import random

SPEC = importlib.util.spec_from_file_location(
    "drift_fixture", Path(__file__).with_name("test-cratesio-drift.py"))
fixture = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(fixture)


def main():
    rng = random.Random(21)
    baseline = fixture.fixture_lock(fixture.payloads())
    for index in range(128):
        document = fixture.openapi()
        operation = document["paths"]["/api/v1/categories/{category}"]["get"]
        mode = index % 4
        if mode == 0:
            operation["parameters"][0]["schema"]["maxLength"] = rng.randrange(1, 4097)
        elif mode == 1:
            operation["responses"][str(rng.randrange(401, 600))] = {
                "description": "new failure"}
        elif mode == 2:
            operation["security"] = [{"api_token": [f"scope-{index}"]}]
        else:
            document["components"]["schemas"]["Crate"]["required"].append(f"field{index}")
        report = fixture.changes(baseline, fixture.payloads(document))
        if not report:
            raise AssertionError(f"semantic drift mutation {index} went undetected")
    print("128 deterministic OpenAPI drift mutations detected.")


if __name__ == "__main__":
    main()
