#!/usr/bin/env python3
"""Regression tests for release governance source controls."""

from __future__ import annotations

import importlib.util
import shutil
import tempfile
from pathlib import Path
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "check_release_governance.py"


def load_checker():
    spec = importlib.util.spec_from_file_location("release_governance", SCRIPT)
    if spec is None or spec.loader is None:
        raise RuntimeError("could not load release governance checker")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


checker = load_checker()


def assert_fails(expected: str, operation) -> None:
    try:
        operation()
    except checker.GovernanceError as error:
        assert expected in str(error), error
        return
    raise AssertionError("expected governance failure")


def workflow(text: str) -> Path:
    path = Path(tempfile.mkdtemp()) / "workflow.yml"
    path.write_text(text, encoding="utf-8")
    return path


def source_governance_fixture() -> tuple[Path, dict]:
    root = Path(tempfile.mkdtemp())
    workflows = root / ".github" / "workflows"
    workflows.mkdir(parents=True)
    allowed = workflows / "allowed.yml"
    allowed.write_text(
        "name: Test\npermissions: {contents: read}\njobs:\n"
        "  test: {runs-on: ubuntu-latest, steps: []}\n",
        encoding="utf-8",
    )
    config = {"workflows": {"files": ["allowed.yml"]}}
    return root, config


def test_pinned_read_only_workflow_passes() -> None:
    path = workflow(
        "name: Test\npermissions:\n  contents: read\njobs:\n  test:\n"
        "    steps:\n      - uses: actions/checkout@" + "a" * 40 + " # v1\n"
    )
    checker.check_workflow(path)


def test_unpinned_action_fails() -> None:
    path = workflow(
        "name: Test\npermissions:\n  contents: read\njobs:\n  test:\n"
        "    steps:\n      - uses: actions/checkout@v7\n"
    )
    assert_fails("not SHA-pinned", lambda: checker.check_workflow(path))


def test_write_permission_fails() -> None:
    path = workflow(
        "name: Test\npermissions:\n  contents: write\njobs:\n  test:\n"
        "    steps: []\n"
    )
    assert_fails("contents: read", lambda: checker.check_workflow(path))


def test_job_permissions_fail() -> None:
    path = workflow(
        "name: Test\npermissions:\n  contents: read\njobs:\n  test:\n"
        "    permissions:\n      contents: read\n    steps: []\n"
    )
    assert_fails("job-level permissions", lambda: checker.check_workflow(path))


def test_flow_style_write_all_job_permissions_fail() -> None:
    path = workflow(
        "name: Test\npermissions: {contents: read}\njobs:\n"
        "  privileged: {permissions: write-all, runs-on: ubuntu-latest, steps: []}\n"
    )
    assert_fails("job-level permissions", lambda: checker.check_workflow(path))


def test_flow_style_unpinned_action_fails() -> None:
    path = workflow(
        "name: Test\npermissions: {contents: read}\njobs:\n"
        "  test:\n    steps: [{uses: actions/checkout@v7}]\n"
    )
    assert_fails("not SHA-pinned", lambda: checker.check_workflow(path))


def test_aliased_job_permissions_are_checked_semantically() -> None:
    path = workflow(
        "name: Test\npermissions: {contents: read}\n"
        "template: &privileged {permissions: write-all, steps: []}\n"
        "jobs: {test: *privileged}\n"
    )
    assert_fails("job-level permissions", lambda: checker.check_workflow(path))


def test_yaml_merge_key_fails_closed() -> None:
    path = workflow(
        "name: Test\npermissions: {contents: read}\n"
        "template: &privileged {permissions: write-all}\n"
        "jobs:\n  test: {<<: *privileged, steps: []}\n"
    )
    assert_fails("YAML merge key", lambda: checker.check_workflow(path))


def test_unlisted_yaml_workflow_fails_inventory() -> None:
    root, config = source_governance_fixture()
    (root / ".github/workflows/backdoor.yaml").write_text(
        "permissions: write-all\njobs: {}\n", encoding="utf-8"
    )
    expected = set(config["workflows"]["files"])
    assert_fails(
        "workflow inventory differs",
        lambda: checker.check_workflow_inventory(
            root / ".github/workflows", expected
        ),
    )
    shutil.rmtree(root)


def test_publish_command_fails() -> None:
    path = workflow(
        "name: Test\npermissions:\n  contents: read\njobs:\n  test:\n"
        "    steps:\n      - run: cargo publish\n"
    )
    assert_fails("forbidden workflow", lambda: checker.check_workflow(path))


def test_flow_style_release_trigger_fails() -> None:
    path = workflow(
        "name: Test\non: [push, release]\npermissions: {contents: read}\n"
        "jobs: {test: {runs-on: ubuntu-latest, steps: []}}\n"
    )
    assert_fails("forbidden workflow trigger", lambda: checker.check_workflow(path))


def test_incomplete_recovery_runbook_fails() -> None:
    path = workflow("# Release Governance\n## Signer Lifecycle\n")
    assert_fails(
        "missing governance procedure",
        lambda: checker.check_governance_document(path),
    )


def test_publisher_with_git_push_fails() -> None:
    path = workflow('run(["git push origin main"])\n')
    assert_fails(
        "publisher contains forbidden command git push",
        lambda: checker.check_publisher_source(path),
    )


def test_current_repository_policy_passes() -> None:
    config = checker.load_toml(checker.CONFIG)
    checker.check_package_policy(ROOT, config)
    checker.check_source_governance(ROOT, config)


def main() -> None:
    tests = (
        test_pinned_read_only_workflow_passes,
        test_unpinned_action_fails,
        test_write_permission_fails,
        test_job_permissions_fail,
        test_flow_style_write_all_job_permissions_fail,
        test_flow_style_unpinned_action_fails,
        test_aliased_job_permissions_are_checked_semantically,
        test_yaml_merge_key_fails_closed,
        test_unlisted_yaml_workflow_fails_inventory,
        test_publish_command_fails,
        test_flow_style_release_trigger_fails,
        test_incomplete_recovery_runbook_fails,
        test_publisher_with_git_push_fails,
        test_current_repository_policy_passes,
    )
    for test in tests:
        test()
    print(f"{len(tests)} release governance regression tests passed.")


if __name__ == "__main__":
    main()
