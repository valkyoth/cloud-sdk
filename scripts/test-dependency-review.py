#!/usr/bin/env python3
"""Regression tests for the dependency-review lockfile inventory."""

from __future__ import annotations

import importlib.util
import contextlib
import io
import tempfile
from unittest.mock import patch
from pathlib import Path

SCRIPT = Path(__file__).with_name("check_dependency_review.py")
SPEC = importlib.util.spec_from_file_location("dependency_review", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def lock(packages: list[tuple[str, str]]) -> str:
    body = ['version = 4', '']
    for name, version in packages:
        body.extend(
            [
                '[[package]]',
                f'name = "{name}"',
                f'version = "{version}"',
                '',
            ]
        )
    return "\n".join(body)


def test_change_inventory() -> None:
    previous = MODULE.package_identities(
        lock([("changed", "1.0.0"), ("removed", "1.0.0"), ("multi", "1.0.0")])
    )
    current = MODULE.package_identities(
        lock([("changed", "1.1.0"), ("added", "1.0.0"), ("multi", "2.0.0")])
    )
    assert MODULE.identity_changes(previous, current) == [
        ("added", "-", current["added"]),
        ("changed", previous["changed"], current["changed"]),
        ("multi", previous["multi"], current["multi"]),
        ("removed", previous["removed"], "-"),
    ]


def test_exact_review_rows() -> None:
    changes = [("one", "1.0.0", "2.0.0"), ("two", "-", "1.0.0")]
    review = "\n".join(
        [
            "| Package | Previous | Current | Review |",
            "| --- | --- | --- | --- |",
            "| `Cargo.lock` | `one` | `1.0.0` | `2.0.0` | reviewed |",
        ]
    )
    assert MODULE.missing_rows(changes, review) == [("two", "-", "1.0.0")]
    review += "\n| `Cargo.lock` | `two` | `-` | `1.0.0` | reviewed |"
    assert MODULE.missing_rows(changes, review) == []


def test_review_evidence_is_scoped_to_the_requested_release() -> None:
    historical_row = "| `Cargo.lock` | `changed` | `1.0.0` | `2.0.0` | reviewed |"
    review = "\n".join(
        [
            "## v0.95.0",
            "",
            historical_row,
            "",
            "## v0.96.0",
            "",
            "No dependency changes.",
        ]
    )
    current = MODULE.review_section(review, "0.96.0")
    assert historical_row not in current
    assert MODULE.missing_rows([("changed", "1.0.0", "2.0.0")], current) == [
        ("changed", "1.0.0", "2.0.0")
    ]


def test_missing_or_invalid_release_sections_fail_closed() -> None:
    for version in ("0.97.0", "0.96.0\n## v0.95.0"):
        try:
            MODULE.review_section("## v0.96.0\n\nreviewed\n", version)
        except MODULE.ReviewError:
            pass
        else:
            raise AssertionError("invalid dependency-review section was accepted")


def test_all_graphs_fail_closed() -> None:
    for graph in MODULE.LOCKFILES:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "docs").mkdir()
            for name in MODULE.LOCKFILES:
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(lock([("transitive", "2.0.0" if name == graph else "1.0.0")]))
            document = root / "docs/review.md"
            old = MODULE.package_identities(lock([("transitive", "1.0.0")]))["transitive"]
            new = MODULE.package_identities(lock([("transitive", "2.0.0")]))["transitive"]
            row = f"| `{graph}` | `transitive` | `{old}` | `{new}` | reviewed |"
            def run():
                with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                    return MODULE.main(["baseline", "1.1.0", "docs/review.md"])
            with patch.object(MODULE, "ROOT", root), patch.object(
                MODULE, "previous_lock", return_value=lock([("transitive", "1.0.0")])
            ) as previous:
                document.write_text("## v1.1.0\n" + row)
                assert run() == 0
                assert previous.call_args_list[-4:] == [
                    (("baseline", name),) for name in MODULE.LOCKFILES
                ]
                original = (root / graph).read_text()
                for field in ('checksum = "changed"', 'source = "registry+changed"',
                              'dependencies = ["new-edge"]'):
                    (root / graph).write_text(original + field + "\n")
                    assert run() == 1
                (root / graph).write_text(original)
                for invalid in ("", row.replace(graph, "wrong/Cargo.lock"), row.replace(f"`{graph}` | ", "")):
                    document.write_text("## v1.1.0\n" + invalid)
                    assert run() == 1
                document.write_text("## v1.1.0\n" + row)
                (root / graph).unlink()
                assert run() == 1
                (root / graph).write_text("invalid TOML [")
                assert run() == 1
                (root / graph).write_text(lock([("transitive", "2.0.0")]))
                previous.side_effect = MODULE.ReviewError("missing baseline lock")
                assert run() == 1


def test_complete_identity():
    original = lock([("package", "1.0.0")]) + 'source = "registry+trusted"\nchecksum = "aaa"\ndependencies = ["one"]\n'
    previous = MODULE.package_identities(original)
    for changed in (
        original.replace('"aaa"', '"bbb"'),
        original.replace('registry+trusted', 'registry+other'),
        original.replace('["one"]', '["two"]'),
        original.replace('["one"]', '["one", "two"]'),
        original.replace('dependencies = ["one"]\n', ''),
        original + 'additional = "unreviewed"\n',
    ):
        changes = MODULE.identity_changes(previous, MODULE.package_identities(changed))
        assert len(changes) == 1
        for graph in MODULE.LOCKFILES:
            assert MODULE.missing_rows(changes, "", graph) == changes
            name, old, new = changes[0]
            row = f"| `{graph}` | `{name}` | `{old}` | `{new}` | reviewed |"
            assert MODULE.missing_rows(changes, row, graph) == []
            assert MODULE.missing_rows(changes, row.replace(new, old), graph) == changes
    reordered = original.replace('source = "registry+trusted"\n', '') + 'source = "registry+trusted"\n'
    assert MODULE.package_identities(reordered) == previous
    records = lock([("package", "1.0.0"), ("package", "2.0.0")])
    assert MODULE.package_identities(records) == MODULE.package_identities(
        lock([("package", "2.0.0"), ("package", "1.0.0")]))
    for invalid in ('version = 4', 'package = "invalid"',
                    lock([("package", "1.0.0"), ("package", "1.0.0")])):
        try:
            MODULE.package_identities(invalid)
        except MODULE.ReviewError:
            pass
        else:
            raise AssertionError("invalid identity inventory accepted")


def main() -> None:
    test_change_inventory()
    test_exact_review_rows()
    test_review_evidence_is_scoped_to_the_requested_release()
    test_missing_or_invalid_release_sections_fail_closed()
    test_all_graphs_fail_closed()
    test_complete_identity()
    print("6 dependency-review regression groups passed.")


if __name__ == "__main__":
    main()
