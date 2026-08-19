#!/usr/bin/env python3
"""Regression tests for release governance source controls."""

from __future__ import annotations

import copy
import importlib.util
import shutil
import tempfile
from pathlib import Path
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "check_release_governance.py"
CHECKOUT = "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1"

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
        f"    runs-on: ubuntu-latest\n    steps:\n      - uses: {CHECKOUT} # v7.0.1\n"
    )
    checker.check_workflow(path)


def test_unpinned_action_fails() -> None:
    path = workflow(
        "name: Test\npermissions:\n  contents: read\njobs:\n  test:\n"
        "    steps:\n      - uses: actions/checkout@v7\n"
    )
    assert_fails("not explicitly approved", lambda: checker.check_workflow(path))


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
    assert_fails("not explicitly approved", lambda: checker.check_workflow(path))


def test_aliases_are_rejected_before_dom_expansion() -> None:
    path = workflow(
        "name: Test\npermissions: {contents: read}\n"
        "template: &privileged {permissions: write-all, steps: []}\n"
        "jobs: {test: *privileged}\n"
    )
    assert_fails("anchors or aliases", lambda: checker.check_workflow(path))


def test_alias_expansion_bomb_is_rejected_before_dom_expansion() -> None:
    path = workflow(
        "name: Test\npermissions: {contents: read}\n"
        "a: &a [x, x, x, x, x, x, x, x, x, x]\n"
        "b: &b [*a, *a, *a, *a, *a, *a, *a, *a, *a, *a]\n"
        "c: &c [*b, *b, *b, *b, *b, *b, *b, *b, *b, *b]\n"
        "jobs: {test: {steps: []}}\n"
    )
    assert len(path.read_bytes()) < 1024
    assert_fails("anchors or aliases", lambda: checker.check_workflow(path))


def test_yaml_merge_key_fails_closed() -> None:
    path = workflow(
        "name: Test\npermissions: {contents: read}\n"
        "jobs:\n  test: {<<: {permissions: write-all}, steps: []}\n"
    )
    assert_fails("YAML merge key", lambda: checker.check_workflow(path))


def test_excessive_yaml_depth_fails_before_dom_construction() -> None:
    nested = "[" * 65 + "x" + "]" * 65
    path = workflow(
        "name: Test\npermissions: {contents: read}\n"
        f"extra: {nested}\njobs: {{test: {{steps: []}}}}\n"
    )
    assert_fails("nesting depth", lambda: checker.check_workflow(path))


def test_excessive_yaml_events_fail_before_dom_construction() -> None:
    path = workflow(
        "name: Test\npermissions: {contents: read}\nextra: ["
        + ",".join("x" for _ in range(10_001))
        + "]\njobs: {test: {steps: []}}\n"
    )
    assert len(path.read_bytes()) < 1024 * 1024
    assert_fails("too many YAML events", lambda: checker.check_workflow(path))


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
    assert_fails("not explicitly approved", lambda: checker.check_workflow(path))


def test_environment_constructed_publish_command_fails() -> None:
    path = workflow(
        "name: Test\npermissions: {contents: read}\njobs:\n  test:\n"
        "    steps:\n      - env: {OPERATION: publish}\n"
        '        run: cargo "$OPERATION"\n'
    )
    assert_fails("not explicitly approved", lambda: checker.check_workflow(path))


def test_execution_modifiers_fail() -> None:
    cases = (
        ("defaults: {run: {shell: 'bash -c \"cargo publish; bash {0}\"'}}\n", "forbidden"),
        ("jobs: {test: {defaults: {run: {shell: bash}}, steps: []}}\n", "forbidden"),
        ("jobs: {test: {container: rust:latest, steps: []}}\n", "forbidden"),
        ("jobs: {test: {services: {db: {image: postgres}}, steps: []}}\n", "forbidden"),
        ("jobs: {test: {steps: [{working-directory: /tmp, run: scripts/checks.sh}]}}\n", "not explicitly approved"),
        ("jobs: {test: {steps: [{shell: 'bash -c {0}', run: scripts/checks.sh}]}}\n", "custom shell"),
    )
    prefix = "name: Test\npermissions: {contents: read}\n"
    for body, expected in cases:
        assert_fails(expected, lambda body=body: checker.check_workflow(workflow(prefix + body)))


def test_secret_context_fails() -> None:
    expressions = (
        "${{secrets.CRATES_IO}}",
        "${{ secrets['CRATES_IO'] }}",
        "${{ github['token'] }}",
        "${{ github.token }}",
        "${{ matrix.unreviewed }}",
    )
    prefix = "name: Test\npermissions: {contents: read}\njobs:\n  test:\n    steps:\n"
    for expression in expressions:
        body = f"      - env: {{TOKEN: \"{expression}\"}}\n        run: scripts/checks.sh\n"
        assert_fails(
            "expression is not explicitly approved",
            lambda body=body: checker.check_workflow(workflow(prefix + body)),
        )


def test_unapproved_pinned_action_fails() -> None:
    path = workflow(
        "name: Test\npermissions: {contents: read}\njobs:\n  test:\n    steps:\n"
        "      - uses: attacker/publish-action@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n"
    )
    assert_fails("not explicitly approved", lambda: checker.check_workflow(path))
    alternate = workflow(
        "name: Test\npermissions: {contents: read}\njobs:\n  test:\n    steps:\n"
        f"      - uses: {CHECKOUT}\n        with: {{repository: attacker/repository}}\n"
    )
    assert_fails("action inputs", lambda: checker.check_workflow(alternate))


def test_github_environment_fails() -> None:
    path = workflow(
        "name: Test\npermissions: {contents: read}\njobs:\n"
        "  test: {environment: release, steps: []}\n"
    )
    assert_fails("GitHub environments", lambda: checker.check_workflow(path))


def test_unapproved_runners_fail() -> None:
    prefix = "name: Test\npermissions: {contents: read}\njobs:\n  test:\n"
    for body in (
        "    runs-on: self-hosted\n    steps: []\n",
        "    runs-on: [self-hosted, linux]\n    steps: []\n",
        "    runs-on: '${{ matrix.os }}'\n    strategy: {matrix: {os: [self-hosted]}}\n    steps: []\n",
        "    runs-on: '${{ matrix.os }}'\n    strategy: {matrix: {os: [ubuntu-latest], include: [{os: self-hosted}]}}\n    steps: []\n",
    ):
        assert_fails("runner", lambda body=body: checker.check_workflow(workflow(prefix + body)))

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


def live_fixture() -> tuple[dict, dict[str, object]]:
    config = checker.load_toml(checker.CONFIG)
    repository = config["policy"]["repository"]
    ruleset = {
        "id": 7,
        "name": config["policy"]["branch_ruleset_name"],
        "target": "branch",
        "enforcement": "active",
    }
    details = {
        "conditions": {"ref_name": {"include": ["~DEFAULT_BRANCH"]}},
        "bypass_actors": [
            {
                "actor_id": None,
                "actor_type": "OrganizationAdmin",
                "bypass_mode": "always",
            },
            {"actor_id": 1921261, "actor_type": "User", "bypass_mode": "always"},
        ],
        "rules": [
            {"type": name}
            for name in (
                "creation",
                "deletion",
                "non_fast_forward",
                "required_linear_history",
                "required_signatures",
                "update",
            )
        ]
        + [
            {
                "type": "pull_request",
                "parameters": {
                    "required_approving_review_count": 1,
                    "require_code_owner_review": True,
                    "dismiss_stale_reviews_on_push": True,
                    "require_last_push_approval": True,
                },
            },
            {
                "type": "code_scanning",
                "parameters": {
                    "code_scanning_tools": [
                        {
                            "tool": "CodeQL",
                            "security_alerts_threshold": "all",
                            "alerts_threshold": "all",
                        }
                    ]
                },
            },
        ],
    }
    responses = {
        f"repos/{repository}": {
            "default_branch": "main",
            "security_and_analysis": {
                "secret_scanning": {"status": "disabled"},
                "secret_scanning_push_protection": {"status": "disabled"},
            },
        },
        f"repos/{repository}/rulesets": [ruleset],
        f"repos/{repository}/rulesets/7": details,
        f"repos/{repository}/actions/permissions/workflow": {
            "default_workflow_permissions": "read",
            "can_approve_pull_request_reviews": False,
        },
        f"repos/{repository}/code-scanning/default-setup": {
            "state": "configured",
            "query_suite": "default",
            "schedule": "weekly",
            "languages": ["actions", "python", "rust"],
        },
        f"repos/{repository}/actions/permissions": {
            "enabled": True,
            "allowed_actions": "all",
            "sha_pinning_required": False,
        },
        f"repos/{repository}/actions/runners": {"total_count": 0, "runners": []},
    }
    return config, responses


def test_live_github_baseline_passes() -> None:
    config, responses = live_fixture()
    checker.check_live_github(config, lambda endpoint: responses[endpoint])


def test_every_documented_live_setting_drift_fails() -> None:
    config, baseline = live_fixture()
    repository = config["policy"]["repository"]
    cases = (
        (
            f"repos/{repository}",
            lambda value: value.update(default_branch="trunk"),
            "default branch",
        ),
        (
            f"repos/{repository}/rulesets/7",
            lambda value: value.update(bypass_actors=[]),
            "bypass actors",
        ),
        (
            f"repos/{repository}/rulesets",
            lambda value: value[0].update(name="replacement ruleset"),
            "uniquely active",
        ),
        (
            f"repos/{repository}/rulesets",
            lambda value: value.append(
                {"id": 8, "name": "tags", "target": "tag", "enforcement": "active"}
            ),
            "tag ruleset state",
        ),
        (
            f"repos/{repository}/actions/permissions",
            lambda value: value.update(allowed_actions="selected"),
            "allowed_actions",
        ),
        (
            f"repos/{repository}/actions/permissions",
            lambda value: value.update(enabled=False),
            "enabled",
        ),
        (
            f"repos/{repository}/actions/permissions",
            lambda value: value.update(sha_pinning_required=True),
            "sha_pinning_required",
        ),
        (
            f"repos/{repository}/actions/runners",
            lambda value: value.update(total_count=1),
            "self-hosted runner",
        ),
        (
            f"repos/{repository}",
            lambda value: value["security_and_analysis"]["secret_scanning"].update(
                status="enabled"
            ),
            "secret_scanning",
        ),
        (
            f"repos/{repository}",
            lambda value: value["security_and_analysis"][
                "secret_scanning_push_protection"
            ].update(status="enabled"),
            "secret_scanning_push_protection",
        ),
        (
            f"repos/{repository}/actions/permissions/workflow",
            lambda value: value.update(can_approve_pull_request_reviews=True),
            "review approval",
        ),
        (
            f"repos/{repository}/code-scanning/default-setup",
            lambda value: value.update(state="not-configured"),
            "CodeQL state",
        ),
        (
            f"repos/{repository}/code-scanning/default-setup",
            lambda value: value.update(query_suite="security-extended"),
            "query suite",
        ),
        (
            f"repos/{repository}/code-scanning/default-setup",
            lambda value: value.update(schedule="monthly"),
            "CodeQL schedule",
        ),
        (
            f"repos/{repository}/code-scanning/default-setup",
            lambda value: value.update(languages=["python", "rust"]),
            "CodeQL languages",
        ),
    )
    for endpoint, mutate, expected in cases:
        responses = copy.deepcopy(baseline)
        mutate(responses[endpoint])
        assert_fails(
            expected,
            lambda responses=responses: checker.check_live_github(
                config, lambda endpoint: responses[endpoint]
            ),
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
        test_aliases_are_rejected_before_dom_expansion,
        test_alias_expansion_bomb_is_rejected_before_dom_expansion,
        test_yaml_merge_key_fails_closed,
        test_excessive_yaml_depth_fails_before_dom_construction,
        test_excessive_yaml_events_fail_before_dom_construction,
        test_unlisted_yaml_workflow_fails_inventory,
        test_publish_command_fails,
        test_environment_constructed_publish_command_fails,
        test_execution_modifiers_fail,
        test_secret_context_fails,
        test_unapproved_pinned_action_fails,
        test_github_environment_fails,
        test_unapproved_runners_fail,
        test_flow_style_release_trigger_fails,
        test_incomplete_recovery_runbook_fails,
        test_publisher_with_git_push_fails,
        test_live_github_baseline_passes,
        test_every_documented_live_setting_drift_fails,
        test_current_repository_policy_passes,
    )
    for test in tests:
        test()
    print(f"{len(tests)} release governance regression tests passed.")


if __name__ == "__main__":
    main()
