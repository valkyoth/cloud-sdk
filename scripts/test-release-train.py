#!/usr/bin/env python3
"""Regression tests for staged and cumulative release policy."""

from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import release_train


def release(
    version: str,
    *,
    stage: str,
    baseline: str = "0.50.0",
    review_baseline: str | None = None,
    milestones: tuple[str, ...] = (),
    exceptional: bool = False,
    reason: str = "",
) -> dict:
    return {
        "version": version,
        "milestone": version,
        "baseline": baseline,
        "review_baseline": review_baseline or baseline,
        "cumulative_milestones": list(milestones),
        "policy": "independent",
        "stage": stage,
        "exceptional": exceptional,
        "exception_reason": reason,
    }


def assert_fails(expected: str, function, *args) -> None:
    try:
        function(*args)
    except RuntimeError as error:
        assert expected in str(error), error
        return
    raise AssertionError("expected failure")


def test_anchor_is_public_without_a_cumulative_range() -> None:
    context = release_train.validate_release_context(
        release("0.50.0", stage="public", baseline="0.50.0")
    )
    assert context["anchor"] is True


def test_intermediate_minor_is_internal() -> None:
    context = release_train.validate_release_context(
        release("0.51.0", stage="internal", milestones=("0.51.0",))
    )
    assert context["stage"] == "internal"
    assert not release_train.publication_allowed(context)
    assert release_train.next_checkpoint("0.51.0") == "0.55.0"


def test_post_stable_candidate_is_not_publishable() -> None:
    context = release_train.validate_release_context(
        release(
            "1.1.0",
            stage="candidate",
            baseline="1.0.0",
            review_baseline="1.0.0",
            milestones=("1.1.0",),
        )
    )
    assert context["stage"] == "candidate"
    assert not release_train.publication_allowed(context)


def test_candidate_stage_rejects_pre_stable_and_exceptions() -> None:
    assert_fails(
        "available only after v1.0.0",
        release_train.validate_release_context,
        release("0.51.0", stage="candidate", milestones=("0.51.0",)),
    )
    assert_fails(
        "cannot declare a release exception",
        release_train.validate_release_context,
        release(
            "1.1.0",
            stage="candidate",
            baseline="1.0.0",
            review_baseline="1.0.0",
            milestones=("1.1.0",),
            exceptional=True,
            reason="not permitted",
        ),
    )


def test_review_baseline_is_retained_in_validated_context() -> None:
    context = release_train.validate_release_context(
        release(
            "0.57.0",
            stage="internal",
            baseline="0.55.0",
            review_baseline="0.56.0",
            milestones=("0.56.0", "0.57.0"),
        )
    )
    assert context["review_baseline"] == "0.56.0"


def test_intermediate_patch_keeps_the_same_checkpoint() -> None:
    context = release_train.validate_release_context(
        release(
            "0.52.7",
            stage="internal",
            milestones=("0.51.0", "0.52.0", "0.52.7"),
        )
    )
    assert context["stage"] == "internal"
    assert release_train.next_checkpoint("0.52.7") == "0.55.0"
    assert release_train.next_checkpoint("0.99.0") == "0.100.0"


def test_final_development_milestones_target_the_public_release_candidate() -> None:
    assert release_train.next_checkpoint("0.95.0") == "0.100.0"
    assert release_train.next_checkpoint("0.96.0") == "0.100.0"
    assert release_train.next_checkpoint("0.98.7") == "0.100.0"


def test_v0_100_is_a_scheduled_public_checkpoint() -> None:
    milestones = tuple(f"0.{minor}.0" for minor in range(96, 101))
    context = release_train.validate_release_context(
        release(
            "0.100.0",
            stage="public",
            baseline="0.95.0",
            review_baseline="0.99.0",
            milestones=milestones,
        )
    )
    assert release_train.publication_allowed(context)
    assert not context["exceptional"]


def test_scheduled_checkpoint_requires_public_stage() -> None:
    candidate = release(
        "0.55.0",
        stage="internal",
        milestones=("0.51.0", "0.52.0", "0.53.0", "0.54.0", "0.55.0"),
    )
    assert_fails(
        "internal stage conflicts",
        release_train.validate_release_context,
        candidate,
    )


def test_scheduled_checkpoint_accepts_cumulative_range() -> None:
    candidate = release(
        "0.55.0",
        stage="public",
        milestones=("0.51.0", "0.52.0", "0.53.0", "0.54.0", "0.55.0"),
    )
    context = release_train.validate_release_context(candidate)
    assert context["stage"] == "public"
    assert release_train.publication_allowed(context)
    assert context["baseline"] == "0.50.0"


def test_off_cycle_public_release_requires_reasoned_exception() -> None:
    ordinary = release("0.52.0", stage="public", milestones=("0.52.0",))
    assert_fails(
        "off-cycle public releases must be exceptional",
        release_train.validate_release_context,
        ordinary,
    )
    exceptional = release(
        "0.52.0",
        stage="public",
        milestones=("0.51.0", "0.52.0"),
        exceptional=True,
        reason="Credential-boundary security fix.",
    )
    assert release_train.validate_release_context(exceptional)["exceptional"]


def test_targeted_exception_can_remain_an_internal_tag() -> None:
    exceptional = release(
        "0.53.0",
        stage="internal",
        milestones=("0.51.0", "0.52.0", "0.53.0"),
        exceptional=True,
        reason="Review a material credential boundary without publication.",
    )
    context = release_train.validate_release_context(exceptional)
    assert context["exceptional"]
    assert not release_train.publication_allowed(context)


def test_repository_train_rejects_omitted_patch_tag() -> None:
    plan = release_train.validate_release_context(
        release(
            "0.55.0",
            stage="public",
            milestones=("0.51.0", "0.52.0", "0.55.0"),
        )
    )
    original = release_train.semantic_tags_before
    release_train.semantic_tags_before = lambda _release: (
        "0.50.0",
        "0.51.0",
        "0.52.0",
        "0.52.1",
        "0.54.0",
    )
    try:
        assert_fails(
            "must list every tag",
            release_train.validate_repository_train,
            plan,
        )
    finally:
        release_train.semantic_tags_before = original


def test_public_checkpoint_rejects_lost_package_delta() -> None:
    plan = {
        "stage": "public",
        "anchor": False,
        "baseline": "0.50.0",
        "crates": {"cloud-sdk-hetzner": {"change": "unchanged"}},
    }
    original = release_train.changed_packages
    release_train.changed_packages = lambda _packages, _baseline: {
        "cloud-sdk-hetzner"
    }
    try:
        assert_fails(
            "changed after v0.50.0 but is marked unchanged",
            release_train.validate_cumulative_package_changes,
            {},
            plan,
        )
    finally:
        release_train.changed_packages = original


def test_public_checkpoint_rejects_lost_dependency_delta() -> None:
    packages = {
        "cloud-sdk": {"dependencies": []},
        "cloud-sdk-reqwest": {"dependencies": [{"name": "cloud-sdk"}]},
    }
    plan = {
        "crates": {
            "cloud-sdk": {
                "previous_version": "0.50.0",
                "version": "0.55.0",
                "change": "code",
            },
            "cloud-sdk-reqwest": {
                "previous_version": "0.32.3",
                "version": "0.32.3",
                "change": "unchanged",
            },
        }
    }
    assert_fails(
        "depends on changed internal packages",
        release_train.validate_dependency_closure,
        packages,
        plan,
    )


def test_change_selection_rejects_unmodified_code_package() -> None:
    packages = {"cloud-sdk-hetzner": {"dependencies": []}}
    plan = {
        "crates": {
            "cloud-sdk-hetzner": {
                "previous_version": "0.3.0",
                "version": "0.4.0",
                "change": "code",
            }
        }
    }
    assert_fails(
        "package tree did not change",
        release_train.validate_change_selection,
        packages,
        plan,
        set(),
    )


def test_dependency_selection_requires_changed_dependency() -> None:
    packages = {
        "cloud-sdk-testkit": {"dependencies": [{"name": "cloud-sdk"}]},
        "cloud-sdk": {"dependencies": []},
    }
    plan = {
        "crates": {
            "cloud-sdk": {
                "previous_version": "0.55.0",
                "version": "0.55.0",
                "change": "unchanged",
            },
            "cloud-sdk-testkit": {
                "previous_version": "0.3.0",
                "version": "0.3.1",
                "change": "dependency",
            },
        }
    }
    assert_fails(
        "no internal dependency changed",
        release_train.validate_change_selection,
        packages,
        plan,
        set(),
    )


def main() -> None:
    tests = tuple(
        value
        for name, value in globals().items()
        if name.startswith("test_") and callable(value)
    )
    for test in tests:
        test()
    print(f"{len(tests)} release train tests passed.")


if __name__ == "__main__":
    main()
