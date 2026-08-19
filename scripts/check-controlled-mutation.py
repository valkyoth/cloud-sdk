#!/usr/bin/env python3
"""Validate bounded v0.100 controlled-mutation evidence without using credentials."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[1]
POLICY_PATH = ROOT / "controlled-mutation-policy.json"
HEX_DIGEST = re.compile(r"[0-9a-f]{64}").fullmatch
HEX_COMMIT = re.compile(r"(?:[0-9a-f]{40}|[0-9a-f]{64})").fullmatch
RUN_ID = re.compile(r"cloud-sdk-live-v0-100-[a-z0-9-]{8,40}").fullmatch


class EvidenceError(Exception):
    """Static validation failure that never embeds evidence values."""


def exact_keys(value: dict, expected: set[str], label: str) -> None:
    if set(value) != expected:
        raise EvidenceError(f"{label} fields are invalid")


def read_json(path: Path, maximum: int) -> object:
    try:
        raw = path.read_bytes()
    except OSError as error:
        raise EvidenceError("evidence file could not be read") from error
    if not raw or len(raw) > maximum or any(byte > 0x7F for byte in raw):
        raise EvidenceError("evidence file size or encoding is invalid")
    try:
        return json.loads(raw)
    except (json.JSONDecodeError, RecursionError) as error:
        raise EvidenceError("evidence JSON is invalid") from error


def load_policy(path: Path = POLICY_PATH) -> dict:
    value = read_json(path, 32_768)
    if not isinstance(value, dict):
        raise EvidenceError("policy root is invalid")
    exact_keys(
        value,
        {
            "format",
            "release",
            "maximum_evidence_bytes",
            "resource_prefix",
            "maximum_attempts_per_scenario",
            "scenarios",
        },
        "policy",
    )
    if (
        value["format"] != 1
        or value["release"] != "0.100.0"
        or value["maximum_evidence_bytes"] != 65_536
        or value["resource_prefix"] != "cloud-sdk-live-v0-100-"
        or value["maximum_attempts_per_scenario"] != 1
        or not isinstance(value["scenarios"], list)
    ):
        raise EvidenceError("policy constants are invalid")
    names: set[str] = set()
    services: set[str] = set()
    for scenario in value["scenarios"]:
        if not isinstance(scenario, dict):
            raise EvidenceError("policy scenario is invalid")
        exact_keys(
            scenario,
            {"name", "service", "classification", "live_dispatch"},
            "policy scenario",
        )
        if (
            scenario["name"] in names
            or scenario["service"] not in {"cloud", "dns", "security", "storage", "robot"}
            or scenario["classification"] not in {"mutation", "destructive", "cost"}
            or scenario["live_dispatch"] not in {"required", "forbidden"}
        ):
            raise EvidenceError("policy scenario value is invalid")
        names.add(scenario["name"])
        services.add(scenario["service"])
    if services != {"cloud", "dns", "security", "storage", "robot"}:
        raise EvidenceError("policy service coverage is incomplete")
    return value


def validate_scenario(actual: dict, expected: dict, maximum_attempts: int) -> None:
    exact_keys(
        actual,
        {
            "name",
            "service",
            "classification",
            "plan_fingerprint_sha256",
            "permit_bound",
            "attempts",
            "delivery",
            "outcome",
            "reconciliation",
            "cleanup",
            "resource_reference_sha256",
            "observed_cost_minor",
        },
        "scenario",
    )
    for field in ("name", "service", "classification"):
        if actual[field] != expected[field]:
            raise EvidenceError("scenario identity is invalid")
    if not isinstance(actual["plan_fingerprint_sha256"], str) or not HEX_DIGEST(
        actual["plan_fingerprint_sha256"]
    ):
        raise EvidenceError("scenario plan fingerprint is invalid")
    if actual["permit_bound"] is not True or actual["attempts"] != maximum_attempts:
        raise EvidenceError("scenario authority or attempt count is invalid")
    if not isinstance(actual["observed_cost_minor"], int) or isinstance(
        actual["observed_cost_minor"], bool
    ) or actual["observed_cost_minor"] < 0:
        raise EvidenceError("scenario cost is invalid")

    dispatch = expected["live_dispatch"]
    if dispatch == "required":
        if (
            actual["delivery"] not in {"response-started", "possibly-sent"}
            or actual["outcome"] != "applied"
            or actual["reconciliation"] != "confirmed-applied"
            or actual["cleanup"] != "confirmed-removed"
            or not isinstance(actual["resource_reference_sha256"], str)
            or not HEX_DIGEST(actual["resource_reference_sha256"])
        ):
            raise EvidenceError("live scenario is incomplete or ambiguous")
    elif (
        actual["delivery"] != "not-sent"
        or actual["outcome"] != "withheld-by-policy"
        or actual["reconciliation"] != "confirmed-not-applied"
        or actual["cleanup"] != "not-created"
        or actual["resource_reference_sha256"] is not None
        or actual["observed_cost_minor"] != 0
    ):
        raise EvidenceError("cost scenario crossed the no-purchase boundary")


def validate_inventory(value: object, reviewer: str) -> None:
    if not isinstance(value, list) or len(value) != 5:
        raise EvidenceError("final inventory is incomplete")
    expected = {"cloud", "dns", "security", "storage", "robot"}
    observed: set[str] = set()
    for item in value:
        if not isinstance(item, dict):
            raise EvidenceError("inventory entry is invalid")
        exact_keys(item, {"service", "prefixed_resources", "verified_by"}, "inventory")
        service = item["service"]
        if (
            service not in expected
            or service in observed
            or item["prefixed_resources"] != 0
            or item["verified_by"] != reviewer
        ):
            raise EvidenceError("final inventory is not independently empty")
        observed.add(service)
    if observed != expected:
        raise EvidenceError("final inventory service set is incomplete")


def validate_cleanup_ledger(value: object, scenarios: dict[str, dict]) -> None:
    live = {
        name: scenario
        for name, scenario in scenarios.items()
        if scenario["resource_reference_sha256"] is not None
    }
    if not isinstance(value, list) or len(value) != len(live):
        raise EvidenceError("cleanup ledger is incomplete")
    observed: set[str] = set()
    resources: set[str] = set()
    for entry in value:
        if not isinstance(entry, dict):
            raise EvidenceError("cleanup ledger entry is invalid")
        exact_keys(
            entry,
            {
                "scenario",
                "resource_reference_sha256",
                "cleanup_plan_fingerprint_sha256",
                "status",
            },
            "cleanup ledger",
        )
        name = entry["scenario"]
        resource = entry["resource_reference_sha256"]
        cleanup_plan = entry["cleanup_plan_fingerprint_sha256"]
        if (
            name not in live
            or name in observed
            or resource != live[name]["resource_reference_sha256"]
            or resource in resources
            or not isinstance(cleanup_plan, str)
            or not HEX_DIGEST(cleanup_plan)
            or cleanup_plan == live[name]["plan_fingerprint_sha256"]
            or entry["status"] != "confirmed-removed"
        ):
            raise EvidenceError("cleanup ledger is incomplete or inconsistent")
        observed.add(name)
        resources.add(resource)
    if observed != set(live):
        raise EvidenceError("cleanup ledger scenario set is incomplete")


def validate_evidence(value: object, policy: dict) -> None:
    if not isinstance(value, dict):
        raise EvidenceError("evidence root is invalid")
    exact_keys(
        value,
        {
            "format",
            "release",
            "source_commit",
            "run_id",
            "resource_prefix",
            "operator_approval",
            "disposable_scope",
            "production_resources_present",
            "executed_in_ci",
            "currency",
            "approved_cost_minor",
            "observed_cost_minor",
            "operator_reference",
            "cleanup_reviewer_reference",
            "credentials_revoked",
            "billing_reviewed",
            "scenarios",
            "cleanup_ledger",
            "final_inventory",
        },
        "evidence",
    )
    if value["format"] != 1 or value["release"] != policy["release"]:
        raise EvidenceError("evidence version is invalid")
    if not isinstance(value["source_commit"], str) or not HEX_COMMIT(value["source_commit"]):
        raise EvidenceError("source commit is invalid")
    if not isinstance(value["run_id"], str) or not RUN_ID(value["run_id"]):
        raise EvidenceError("run identifier is invalid")
    if value["resource_prefix"] != value["run_id"] + "-":
        raise EvidenceError("resource prefix is not unique to the run")
    if (
        value["operator_approval"] != "approved-v0.100-controlled-mutations"
        or value["disposable_scope"] is not True
        or value["production_resources_present"] is not False
        or value["executed_in_ci"] is not False
        or value["credentials_revoked"] is not True
        or value["billing_reviewed"] is not True
        or value["currency"] != "EUR"
    ):
        raise EvidenceError("operational acknowledgements are incomplete")
    for field in ("approved_cost_minor", "observed_cost_minor"):
        if not isinstance(value[field], int) or isinstance(value[field], bool) or value[field] < 0:
            raise EvidenceError("cost envelope is invalid")
    if value["observed_cost_minor"] > value["approved_cost_minor"]:
        raise EvidenceError("observed cost exceeds approval")
    operator = value["operator_reference"]
    reviewer = value["cleanup_reviewer_reference"]
    if (
        not isinstance(operator, str)
        or not isinstance(reviewer, str)
        or not HEX_DIGEST(operator)
        or not HEX_DIGEST(reviewer)
        or operator == reviewer
    ):
        raise EvidenceError("operator and cleanup reviewer are not independent")
    actual = value["scenarios"]
    if not isinstance(actual, list) or len(actual) != len(policy["scenarios"]):
        raise EvidenceError("scenario coverage is incomplete")
    by_name: dict[str, dict] = {}
    for scenario in actual:
        if not isinstance(scenario, dict) or not isinstance(scenario.get("name"), str):
            raise EvidenceError("scenario entry is invalid")
        if scenario["name"] in by_name:
            raise EvidenceError("scenario entry is duplicated")
        by_name[scenario["name"]] = scenario
    if set(by_name) != {item["name"] for item in policy["scenarios"]}:
        raise EvidenceError("scenario set is invalid")
    for expected in policy["scenarios"]:
        validate_scenario(
            by_name[expected["name"]],
            expected,
            policy["maximum_attempts_per_scenario"],
        )
    live_resources = [
        scenario["resource_reference_sha256"]
        for scenario in actual
        if scenario["resource_reference_sha256"] is not None
    ]
    if len(live_resources) != len(set(live_resources)):
        raise EvidenceError("resource references are not unique")
    if sum(item["observed_cost_minor"] for item in actual) != value["observed_cost_minor"]:
        raise EvidenceError("scenario and aggregate costs disagree")
    validate_cleanup_ledger(value["cleanup_ledger"], by_name)
    validate_inventory(value["final_inventory"], reviewer)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("evidence", type=Path)
    parser.add_argument("--policy", type=Path, default=POLICY_PATH)
    arguments = parser.parse_args()
    try:
        policy = load_policy(arguments.policy)
        evidence = read_json(arguments.evidence, policy["maximum_evidence_bytes"])
        validate_evidence(evidence, policy)
    except EvidenceError as error:
        print(f"controlled mutation: {error}", file=sys.stderr)
        return 1
    print("Controlled-mutation evidence is complete, bounded, and cleanup-closed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
