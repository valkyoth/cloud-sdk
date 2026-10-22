"""Release-stage and stable maintenance version-policy regressions."""

import release_crates


def assert_fails(expected, func, *args):
    try:
        func(*args)
    except RuntimeError as error:
        assert expected in str(error), error
    else:
        raise AssertionError("invalid release plan accepted")


def test_internal_facade_advances_without_publication() -> None:
    entry = {
        "previous_version": "0.50.0",
        "version": "0.51.0",
        "change": "code",
        "publish": False,
        "reason": "test",
    }
    release_crates.validate_plan_entry("cloud-sdk", entry, "0.51.0", "internal")


def test_internal_support_crate_retains_published_version() -> None:
    entry = {
        "previous_version": "0.38.0",
        "version": "0.38.0",
        "change": "code",
        "publish": False,
        "reason": "carry to checkpoint",
    }
    release_crates.validate_plan_entry(
        "cloud-sdk-hetzner", entry, "0.51.0", "internal"
    )


def test_internal_stage_rejects_publication() -> None:
    entry = {
        "previous_version": "0.38.0",
        "version": "0.38.0",
        "change": "code",
        "publish": True,
        "reason": "test",
    }
    assert_fails(
        "cannot publish at internal stage",
        release_crates.validate_plan_entry,
        "cloud-sdk-hetzner",
        entry,
        "0.51.0",
        "internal",
    )


def test_candidate_packages_share_version_without_publication() -> None:
    entry = {
        "previous_version": "1.0.0",
        "version": "1.1.0",
        "change": "metadata",
        "publish": False,
        "reason": "development candidate",
    }
    release_crates.validate_plan_entry(
        "cloud-sdk-hetzner", entry, "1.1.0", "candidate"
    )


def test_candidate_stage_rejects_publication_and_stale_versions() -> None:
    entry = {
        "previous_version": "1.0.0",
        "version": "1.1.0",
        "change": "metadata",
        "publish": True,
        "reason": "development candidate",
    }
    assert_fails(
        "cannot publish at candidate stage",
        release_crates.validate_plan_entry,
        "cloud-sdk-hetzner",
        entry,
        "1.1.0",
        "candidate",
    )
    entry["publish"] = False
    entry["version"] = "1.0.0"
    assert_fails(
        "must match candidate version 1.1.0",
        release_crates.validate_plan_entry,
        "cloud-sdk-hetzner",
        entry,
        "1.1.0",
        "candidate",
    )


def test_provider_code_changes_use_next_independent_minor() -> None:
    entry = {
        "previous_version": "0.7.0",
        "version": "0.8.0",
        "change": "code",
        "publish": True,
        "reason": "test",
    }
    release_crates.validate_plan_entry("cloud-sdk-hetzner", entry, "0.19.0")


def test_stable_maintenance_accepts_only_next_patch_on_same_line() -> None:
    entry = {
        "previous_version": "1.1.0", "version": "1.1.1",
        "change": "code", "publish": True, "reason": "maintenance fix",
    }
    release_crates.validate_plan_entry("cloud-sdk-hetzner", entry, "1.1.1")
    for previous, version, release in [
        ("1.1.0", "1.1.2", "1.1.2"),
        ("1.0.0", "1.1.1", "1.1.1"),
        ("0.1.0", "0.1.1", "0.1.1"),
    ]:
        assert_fails(
            "independent crate version must be",
            release_crates.validate_plan_entry,
            "cloud-sdk-hetzner",
            {**entry, "previous_version": previous, "version": version},
            release,
        )
