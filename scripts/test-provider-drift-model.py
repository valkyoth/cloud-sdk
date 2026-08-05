#!/usr/bin/env python3
"""Adversarial tests for provider drift document validation."""

from __future__ import annotations

import copy
import json
import os
import tempfile
from pathlib import Path

import provider_drift_model as model


ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / "provider-drift" / "providers" / "hetzner.lock.json"
PLUGIN = ROOT / "provider-drift" / "plugins" / "normalized-json-v1.json"


def assert_raises(expected: str, function, *args) -> None:
    try:
        function(*args)
    except model.ModelError as error:
        assert expected in str(error), error
        return
    raise AssertionError("expected ModelError")


def write_json(path: Path, value) -> None:
    path.write_text(json.dumps(value), encoding="utf-8")


def lock_value() -> dict:
    return model.validate_lock(model.read_bounded_json(LOCK, "provider lock"))


def test_repository_documents_are_strict_and_complete() -> None:
    plugin = model.validate_plugin(model.read_bounded_json(PLUGIN, "plugin"))
    lock = lock_value()
    assert plugin["categories"] == list(model.CATEGORIES)
    assert lock["provider"] == "hetzner"
    assert set(lock["contracts"]) == set(model.CATEGORIES)


def test_duplicate_keys_and_unknown_fields_are_rejected() -> None:
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "duplicate.json"
        path.write_text('{"format":"a","format":"b"}', encoding="ascii")
        assert_raises("duplicate key", model.read_bounded_json, path, "fixture")
    value = lock_value()
    value["execute"] = "arbitrary-code"
    assert_raises("fields are incomplete or unsupported", model.validate_lock, value)


def test_symlinks_non_objects_and_oversized_documents_are_rejected() -> None:
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        target = root / "target.json"
        target.write_text("{}", encoding="ascii")
        link = root / "link.json"
        os.symlink(target, link)
        assert_raises("readable regular file", model.read_bounded_json, link, "fixture")
        scalar = root / "scalar.json"
        scalar.write_text("[]", encoding="ascii")
        assert_raises("root must be an object", model.read_bounded_json, scalar, "fixture")
        oversized = root / "oversized.json"
        oversized.write_bytes(b" " * (model.MAX_DOCUMENT_BYTES + 1))
        assert_raises("exceeds", model.read_bounded_json, oversized, "fixture")


def test_malicious_values_fail_closed_before_comparison() -> None:
    value = lock_value()
    value["contracts"]["operations"][0]["values"]["floating"] = 1.5
    assert_raises("floating-point", model.validate_lock, value)
    value = lock_value()
    nested = value["contracts"]["operations"][0]["values"]
    for _depth in range(model.MAX_VALUE_DEPTH + 2):
        nested["nested"] = {}
        nested = nested["nested"]
    assert_raises("nesting depth", model.validate_lock, value)
    value = lock_value()
    value["contracts"]["operations"][0]["values"]["payload"] = "x" * (
        model.MAX_TEXT_BYTES + 1
    )
    assert_raises("oversized text", model.validate_lock, value)


def test_plugin_identity_categories_and_source_urls_are_closed() -> None:
    value = model.read_bounded_json(PLUGIN, "plugin")
    value["categories"] = value["categories"][:-1]
    assert_raises("every category", model.validate_plugin, value)
    value = lock_value()
    value["sources"][0]["url"] = "https://user@example.invalid/spec.json"
    assert_raises("credential-free HTTPS URL", model.validate_lock, value)
    value = lock_value()
    value["sources"][0]["url"] = "http://example.invalid/spec.json"
    assert_raises("credential-free HTTPS URL", model.validate_lock, value)
    value = lock_value()
    value["sources"][0]["url"] = "https://EXAMPLE.invalid/spec.json"
    assert_raises("authority must be lowercase", model.validate_lock, value)
    value = lock_value()
    value["sources"][0]["url"] = "https://example.invalid/spec.json?token=secret"
    assert_raises("credential-free HTTPS URL", model.validate_lock, value)
    value = lock_value()
    value["sources"][0]["url"] = "https://example.invalid/spec.json\n"
    assert_raises("ASCII HTTPS URL", model.validate_lock, value)


def test_integer_and_aggregate_source_bounds_reject_boolean_or_excessive_plans() -> None:
    plugin = model.read_bounded_json(PLUGIN, "plugin")
    plugin["version"] = True
    assert_raises("plugin version", model.validate_plugin, plugin)
    value = lock_value()
    value["sources"][0]["max_bytes"] = True
    assert_raises("outside the hard bound", model.validate_lock, value)
    value = lock_value()
    for index in range(2, 4):
        extra = copy.deepcopy(value["sources"][0])
        extra["id"] = f"source-{index}"
        extra["max_bytes"] = 64 * 1024 * 1024
        value["sources"].append(extra)
    assert_raises("aggregate byte bound", model.validate_lock, value)


def test_duplicate_rows_sources_and_invalid_policy_are_rejected() -> None:
    value = lock_value()
    value["sources"].append(copy.deepcopy(value["sources"][0]))
    assert_raises("duplicate source id", model.validate_lock, value)
    value = lock_value()
    value["contracts"]["schemas"].append(
        copy.deepcopy(value["contracts"]["schemas"][0])
    )
    assert_raises("duplicate id", model.validate_lock, value)
    value = lock_value()
    value["compatibility"]["schemas"]["changed"] = "ignore"
    assert_raises("is invalid", model.validate_lock, value)


def main() -> None:
    tests = tuple(
        value
        for name, value in globals().items()
        if name.startswith("test_") and callable(value)
    )
    for test in tests:
        test()
    print(f"{len(tests)} provider drift model tests passed.")


if __name__ == "__main__":
    main()
