#!/usr/bin/env python3
"""Regression tests for the bounded controlled-mutation evidence protocol."""

from __future__ import annotations

import copy
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "scripts" / "check-controlled-mutation.py"
POLICY = ROOT / "controlled-mutation-policy.json"
DIGEST_A = "a" * 64
DIGEST_B = "b" * 64
DIGEST_C = "c" * 64


def load_checker():
    specification = importlib.util.spec_from_file_location("controlled_mutation", CHECKER)
    assert specification is not None and specification.loader is not None
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def valid_evidence(policy: dict) -> dict:
    scenarios = []
    cleanup_ledger = []
    for index, expected in enumerate(policy["scenarios"]):
        if expected["live_dispatch"] == "required":
            resource = f"{index + 1:064x}"
            scenario = {
                "name": expected["name"],
                "service": expected["service"],
                "classification": expected["classification"],
                "plan_fingerprint_sha256": DIGEST_A,
                "permit_bound": True,
                "attempts": 1,
                "delivery": "response-started",
                "outcome": "applied",
                "reconciliation": "confirmed-applied",
                "cleanup": "confirmed-removed",
                "resource_reference_sha256": resource,
                "observed_cost_minor": 0,
            }
            cleanup_ledger.append(
                {
                    "scenario": expected["name"],
                    "resource_reference_sha256": resource,
                    "cleanup_plan_fingerprint_sha256": f"{index + 101:064x}",
                    "status": "confirmed-removed",
                }
            )
        else:
            scenario = {
                "name": expected["name"],
                "service": expected["service"],
                "classification": expected["classification"],
                "plan_fingerprint_sha256": DIGEST_A,
                "permit_bound": True,
                "attempts": 1,
                "delivery": "not-sent",
                "outcome": "withheld-by-policy",
                "reconciliation": "confirmed-not-applied",
                "cleanup": "not-created",
                "resource_reference_sha256": None,
                "observed_cost_minor": 0,
            }
        scenarios.append(scenario)
    return {
        "format": 1,
        "release": "0.100.0",
        "source_commit": "d" * 40,
        "run_id": "cloud-sdk-live-v0-100-20260819-a1b2c3d4",
        "resource_prefix": "cloud-sdk-live-v0-100-20260819-a1b2c3d4-",
        "operator_approval": "approved-v0.100-controlled-mutations",
        "disposable_scope": True,
        "production_resources_present": False,
        "executed_in_ci": False,
        "currency": "EUR",
        "approved_cost_minor": 0,
        "observed_cost_minor": 0,
        "operator_reference": DIGEST_B,
        "cleanup_reviewer_reference": DIGEST_C,
        "credentials_revoked": True,
        "billing_reviewed": True,
        "scenarios": scenarios,
        "cleanup_ledger": cleanup_ledger,
        "final_inventory": [
            {
                "service": service,
                "prefixed_resources": 0,
                "verified_by": DIGEST_C,
            }
            for service in ("cloud", "dns", "security", "storage", "robot")
        ],
    }


def assert_rejected(checker, evidence: dict, policy: dict) -> None:
    try:
        checker.validate_evidence(evidence, policy)
    except checker.EvidenceError:
        return
    raise AssertionError("invalid controlled-mutation evidence was accepted")


def mutate(evidence: dict, callback) -> dict:
    changed = copy.deepcopy(evidence)
    callback(changed)
    return changed


def replace_path(value: dict, path: tuple[object, ...], replacement: object) -> dict:
    changed = copy.deepcopy(value)
    target = changed
    for segment in path[:-1]:
        target = target[segment]
    target[path[-1]] = replacement
    return changed


def test_policy_and_valid_evidence(checker) -> tuple[dict, dict]:
    policy = checker.load_policy()
    evidence = valid_evidence(policy)
    checker.validate_evidence(evidence, policy)
    return policy, evidence


def test_global_fail_closed_paths(checker, policy: dict, evidence: dict) -> None:
    mutations = (
        lambda value: value.update(format=2),
        lambda value: value.update(format=True),
        lambda value: value.update(release="0.99.0"),
        lambda value: value.update(source_commit="not-a-commit"),
        lambda value: value.update(run_id="generic"),
        lambda value: value.update(resource_prefix="cloud-sdk-live-v0-100-shared-"),
        lambda value: value.update(operator_approval="yes"),
        lambda value: value.update(disposable_scope=False),
        lambda value: value.update(production_resources_present=True),
        lambda value: value.update(executed_in_ci=True),
        lambda value: value.update(currency="USD"),
        lambda value: value.update(approved_cost_minor=-1),
        lambda value: value.update(observed_cost_minor=1),
        lambda value: value.update(cleanup_reviewer_reference=DIGEST_B),
        lambda value: value.update(credentials_revoked=False),
        lambda value: value.update(billing_reviewed=False),
        lambda value: value.update(extra=True),
        lambda value: value["scenarios"].pop(),
        lambda value: value["scenarios"].append(copy.deepcopy(value["scenarios"][0])),
        lambda value: value["cleanup_ledger"].pop(),
        lambda value: value["cleanup_ledger"][0].update(status="pending"),
        lambda value: value["cleanup_ledger"][0].update(
            resource_reference_sha256=DIGEST_A
        ),
        lambda value: value["cleanup_ledger"][0].update(
            cleanup_plan_fingerprint_sha256=DIGEST_A
        ),
        lambda value: value["final_inventory"].pop(),
        lambda value: value["final_inventory"][0].update(prefixed_resources=1),
        lambda value: value["final_inventory"][0].update(prefixed_resources=False),
        lambda value: value["final_inventory"][0].update(verified_by=DIGEST_A),
    )
    for callback in mutations:
        assert_rejected(checker, mutate(evidence, callback), policy)


def test_live_scenario_fail_closed_paths(checker, policy: dict, evidence: dict) -> None:
    index = 0
    mutations = (
        lambda value: value["scenarios"][index].update(service="robot"),
        lambda value: value["scenarios"][index].update(classification="read-only"),
        lambda value: value["scenarios"][index].update(plan_fingerprint_sha256="short"),
        lambda value: value["scenarios"][index].update(permit_bound=False),
        lambda value: value["scenarios"][index].update(attempts=0),
        lambda value: value["scenarios"][index].update(attempts=2),
        lambda value: value["scenarios"][index].update(attempts=True),
        lambda value: value["scenarios"][index].update(delivery="not-sent"),
        lambda value: value["scenarios"][index].update(outcome="unknown"),
        lambda value: value["scenarios"][index].update(reconciliation="unresolved"),
        lambda value: value["scenarios"][index].update(cleanup="pending"),
        lambda value: value["scenarios"][index].update(resource_reference_sha256=None),
        lambda value: value["scenarios"][index].update(observed_cost_minor=-1),
        lambda value: value["scenarios"][index].update(extra=True),
    )
    for callback in mutations:
        assert_rejected(checker, mutate(evidence, callback), policy)

    reconciled = copy.deepcopy(evidence)
    reconciled["scenarios"][index]["delivery"] = "possibly-sent"
    checker.validate_evidence(reconciled, policy)


def test_cost_scenario_never_dispatches(checker, policy: dict, evidence: dict) -> None:
    index = len(evidence["scenarios"]) - 1
    mutations = (
        lambda value: value["scenarios"][index].update(delivery="response-started"),
        lambda value: value["scenarios"][index].update(outcome="applied"),
        lambda value: value["scenarios"][index].update(reconciliation="confirmed-applied"),
        lambda value: value["scenarios"][index].update(cleanup="confirmed-removed"),
        lambda value: value["scenarios"][index].update(resource_reference_sha256=DIGEST_A),
        lambda value: value["scenarios"][index].update(observed_cost_minor=1),
    )
    for callback in mutations:
        assert_rejected(checker, mutate(evidence, callback), policy)


def test_bounded_file_boundary(checker, policy: dict, evidence: dict) -> None:
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary)
        valid = root / "valid.json"
        valid.write_text(json.dumps(evidence), encoding="ascii")
        checker.validate_evidence(
            checker.read_json(valid, policy["maximum_evidence_bytes"]), policy
        )
        oversized = root / "oversized.json"
        oversized.write_bytes(b" " * (policy["maximum_evidence_bytes"] + 1))
        try:
            checker.read_json(oversized, policy["maximum_evidence_bytes"])
        except checker.EvidenceError:
            pass
        else:
            raise AssertionError("oversized evidence was accepted")
        non_ascii = root / "non-ascii.json"
        non_ascii.write_bytes(b'{"value":"\xc3\xa5"}')
        try:
            checker.read_json(non_ascii, policy["maximum_evidence_bytes"])
        except checker.EvidenceError:
            pass
        else:
            raise AssertionError("non-ASCII evidence was accepted")

        link = root / "evidence-link.json"
        link.symlink_to(valid)
        for rejected in (link, root):
            try:
                checker.read_json(rejected, policy["maximum_evidence_bytes"])
            except checker.EvidenceError:
                pass
            else:
                raise AssertionError("non-regular evidence was accepted")

        fifo = root / "evidence-fifo.json"
        os.mkfifo(fifo)
        try:
            checker.read_json(fifo, policy["maximum_evidence_bytes"])
        except checker.EvidenceError:
            pass
        else:
            raise AssertionError("FIFO evidence was accepted")


def test_strict_json_rejects_duplicates(checker, evidence: dict) -> None:
    raw = json.dumps(evidence).encode("ascii")
    duplicates = (
        b'"format": 1',
        b'"attempts": 1',
        b'"status": "confirmed-removed"',
        b'"prefixed_resources": 0',
    )
    for field in duplicates:
        assert field in raw
        changed = raw.replace(field, field + b", " + field, 1)
        try:
            checker.parse_json(changed)
        except checker.EvidenceError:
            pass
        else:
            raise AssertionError("duplicate JSON field was accepted")

    try:
        checker.parse_json(b'{"format": NaN}')
    except checker.EvidenceError:
        pass
    else:
        raise AssertionError("non-standard JSON number was accepted")


def test_policy_integer_types_are_exact(checker, policy: dict) -> None:
    for field in (
        "format",
        "maximum_evidence_bytes",
        "maximum_attempts_per_scenario",
    ):
        changed = copy.deepcopy(policy)
        changed[field] = True
        try:
            checker.validate_policy(changed)
        except checker.EvidenceError:
            pass
        else:
            raise AssertionError("boolean policy integer was accepted")


def test_every_scalar_type_confusion_fails_closed(
    checker, policy: dict, evidence: dict
) -> None:
    policy_paths = [
        (field,)
        for field in policy
        if field != "scenarios"
    ]
    for index, scenario in enumerate(policy["scenarios"]):
        policy_paths.extend(("scenarios", index, field) for field in scenario)
    for path in policy_paths:
        for replacement in ([], {}, None):
            changed = replace_path(policy, path, replacement)
            try:
                checker.validate_policy(changed)
            except checker.EvidenceError:
                pass
            else:
                raise AssertionError("policy scalar type confusion was accepted")

    evidence_paths = [
        (field,)
        for field in evidence
        if field not in {"scenarios", "cleanup_ledger", "final_inventory"}
    ]
    for collection in ("scenarios", "cleanup_ledger", "final_inventory"):
        for index, item in enumerate(evidence[collection]):
            evidence_paths.extend((collection, index, field) for field in item)
    for path in evidence_paths:
        original = evidence
        for segment in path:
            original = original[segment]
        for replacement in ([], {}, None):
            if replacement == original:
                continue
            assert_rejected(checker, replace_path(evidence, path, replacement), policy)


def test_cli_is_static_and_payload_free(policy: dict, evidence: dict) -> None:
    with tempfile.TemporaryDirectory() as temporary:
        path = Path(temporary) / "evidence.json"
        path.write_text(json.dumps(evidence), encoding="ascii")
        accepted = subprocess.run(
            ["python3", str(CHECKER), str(path)],
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
        )
        assert accepted.returncode == 0, accepted
        changed = copy.deepcopy(evidence)
        changed["resource_prefix"] = "secret-sentinel"
        path.write_text(json.dumps(changed), encoding="ascii")
        rejected = subprocess.run(
            ["python3", str(CHECKER), str(path)],
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
        )
        assert rejected.returncode == 1, rejected
        assert "secret-sentinel" not in rejected.stdout + rejected.stderr

        malformed = (
            mutate(
                evidence,
                lambda value: value["scenarios"][0].update(delivery=[]),
            ),
            mutate(
                evidence,
                lambda value: value["cleanup_ledger"][0].update(scenario={}),
            ),
            mutate(
                evidence,
                lambda value: value["final_inventory"][0].update(service=None),
            ),
        )
        for changed in malformed:
            path.write_text(json.dumps(changed), encoding="ascii")
            rejected = subprocess.run(
                ["python3", str(CHECKER), str(path)],
                cwd=ROOT,
                check=False,
                capture_output=True,
                text=True,
            )
            assert rejected.returncode == 1, rejected
            assert "Traceback" not in rejected.stdout + rejected.stderr

        path.write_bytes(b'{"format":' + b"9" * 5_000 + b"}")
        rejected = subprocess.run(
            ["python3", str(CHECKER), str(path)],
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
        )
        assert rejected.returncode == 1, rejected
        assert "Traceback" not in rejected.stdout + rejected.stderr


def test_checker_has_no_network_or_credential_input() -> None:
    source = CHECKER.read_text(encoding="ascii")
    for forbidden in (
        "urllib",
        "requests",
        "socket",
        "subprocess",
        "TOKEN",
        "PASSWORD",
        "USERNAME",
        "os.environ",
    ):
        assert forbidden not in source


def main() -> None:
    checker = load_checker()
    policy, evidence = test_policy_and_valid_evidence(checker)
    test_global_fail_closed_paths(checker, policy, evidence)
    test_live_scenario_fail_closed_paths(checker, policy, evidence)
    test_cost_scenario_never_dispatches(checker, policy, evidence)
    test_bounded_file_boundary(checker, policy, evidence)
    test_strict_json_rejects_duplicates(checker, evidence)
    test_policy_integer_types_are_exact(checker, policy)
    test_every_scalar_type_confusion_fails_closed(checker, policy, evidence)
    test_cli_is_static_and_payload_free(policy, evidence)
    test_checker_has_no_network_or_credential_input()
    print("Controlled-mutation protocol regressions passed.")


if __name__ == "__main__":
    main()
