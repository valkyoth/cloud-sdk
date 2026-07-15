#!/usr/bin/env python3
"""Regression tests for prepared-operation source coverage."""

from __future__ import annotations

import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "scripts" / "check_prepared_operation_coverage.py"
HEADER = """# Matrix

## Operations

| API | Group | Method | Path | Operation | Owner | Pagination | Sorting | Action | Deprecated | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
"""
ENDPOINTS = """endpoint_wire!(
    TestEndpoint,
    endpoint => (),
    (),
    match endpoint {
        TestEndpoint::Read => "read_test",
        TestEndpoint::Write => "write_test",
    },
    false,
    ()
);"""
BODIES = 'body_wire!(WriteRequest, request => (), "write_test", write_test);'


def row(operation: str, *, deprecated: str = "no") -> str:
    status = "deferred-deprecated" if deprecated == "yes" else "implemented"
    return (
        f"| cloud | Test | POST | `/test` | `{operation}` | `test` | no | no | "
        f"none | {deprecated} | {status} |\n"
    )


def run(
    directory: Path,
    *,
    endpoints: str = ENDPOINTS,
    bodies: str = BODIES,
) -> subprocess.CompletedProcess[str]:
    matrix = directory / "matrix.md"
    matrix.write_text(HEADER + row("read_test") + row("write_test"), encoding="utf-8")
    body_lock = directory / "bodies.txt"
    body_lock.write_text("write_test\n", encoding="ascii")
    endpoint_dir = directory / "prepared"
    body_dir = endpoint_dir / "bodies"
    body_dir.mkdir(parents=True, exist_ok=True)
    (directory / "prepared.rs").write_text(endpoints, encoding="utf-8")
    (endpoint_dir / "bodies.rs").write_text("", encoding="utf-8")
    (body_dir / "test.rs").write_text(bodies, encoding="utf-8")
    return subprocess.run(
        [
            str(CHECKER),
            "--matrix",
            str(matrix),
            "--endpoints",
            str(endpoint_dir),
            "--bodies",
            str(body_dir),
            "--body-lock",
            str(body_lock),
            "--expected-active",
            "2",
            "--expected-bodies",
            "1",
        ],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )


def main() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        directory = Path(temporary)
        complete = run(directory)
        assert complete.returncode == 0, complete
        assert "2 endpoints and 1 request bodies" in complete.stdout

        missing_endpoint = run(
            directory,
            endpoints=(
                "endpoint_wire!(TestEndpoint, endpoint => (), (), match endpoint { "
                'TestEndpoint::Write => "write_test" }, false, ());'
            ),
        )
        assert missing_endpoint.returncode == 1, missing_endpoint
        assert "missing endpoint adapters: read_test" in missing_endpoint.stderr

        missing_body = run(directory, bodies="const NO_ADAPTERS: &[&str] = &[];")
        assert missing_body.returncode == 1, missing_body
        assert "missing body adapters: write_test" in missing_body.stderr

        line_comments = run(
            directory,
            endpoints=(
                '// endpoint_wire!(Fake, value => (), (), "read_test", false, ());\n'
                'endpoint_wire!(Real, value => (), (), match value { '
                'Real::Write => "write_test" }, false, ());'
            ),
        )
        assert line_comments.returncode == 1, line_comments
        assert "missing endpoint adapters: read_test" in line_comments.stderr

        block_comments = run(
            directory,
            bodies=(
                '/* body_wire!(Fake, value => (), "write_test", write); */ '
                "const NO_ADAPTERS: &[&str] = &[];"
            ),
        )
        assert block_comments.returncode == 1, block_comments
        assert "missing body adapters: write_test" in block_comments.stderr

        test_only = run(
            directory,
            endpoints=(
                'endpoint_wire!(Real, value => (), (), match value { '
                'Real::Write => "write_test" }, false, ());\n'
                '#[cfg(test)] endpoint_wire!('
                'TestOnly, value => (), (), "read_test", false, ());'
            ),
        )
        assert test_only.returncode == 1, test_only
        assert "conditionally compiled prepared evidence is forbidden" in test_only.stderr

        standalone_constants = run(
            directory,
            endpoints=(
                'const CLAIMED: &str = "read_test";\n'
                'endpoint_wire!(Real, value => (), (), match value { '
                'Real::Write => "write_test" }, false, ());'
            ),
        )
        assert standalone_constants.returncode == 1, standalone_constants
        assert "missing endpoint adapters: read_test" in standalone_constants.stderr

        duplicate_adapter = run(
            directory,
            endpoints=(
                ENDPOINTS
                + '\nendpoint_wire!('
                'Duplicate, value => (), (), match value { '
                'Duplicate::Read => "read_test" }, false, ());'
            ),
        )
        assert duplicate_adapter.returncode == 1, duplicate_adapter
        assert "ambiguous endpoint adapter keys: read_test" in duplicate_adapter.stderr

        unknown_adapter = run(
            directory,
            bodies=(
                BODIES
                + '\nbody_component!('
                'UnknownRequest, "unknown_test", write_unknown);'
            ),
        )
        assert unknown_adapter.returncode == 1, unknown_adapter
        assert "unknown body adapter keys: unknown_test" in unknown_adapter.stderr

        cfg_attr = run(
            directory,
            endpoints=(
                '#[cfg_attr(not(any()), cfg(any()))]\n'
                + ENDPOINTS
            ),
        )
        assert cfg_attr.returncode == 1, cfg_attr
        assert "conditionally compiled prepared evidence is forbidden" in cfg_attr.stderr

        nested_comment = run(
            directory,
            endpoints=(
                '/* outer /* inner */ endpoint_wire!('
                'Fake, value => (), (), match value { '
                'Fake::Read => "read_test" }, false, ()); */\n'
                'endpoint_wire!(Real, value => (), (), match value { '
                'Real::Write => "write_test" }, false, ());'
            ),
        )
        assert nested_comment.returncode == 1, nested_comment
        assert "missing endpoint adapters: read_test" in nested_comment.stderr

        raw_string = run(
            directory,
            endpoints=(
                'const CLAIMED: &str = r#"endpoint_wire!('
                'Fake, value => (), (), match value { '
                'Fake::Read => \\"read_test\\" }, false, ());"#;\n'
                'endpoint_wire!(Real, value => (), (), match value { '
                'Real::Write => "write_test" }, false, ());'
            ),
        )
        assert raw_string.returncode == 1, raw_string
        assert "missing endpoint adapters: read_test" in raw_string.stderr

        discarded_literal = run(
            directory,
            endpoints=(
                'endpoint_wire!(Fake, value => (), (), '
                '{ let _ = "read_test"; "write_test" }, false, ());'
            ),
        )
        assert discarded_literal.returncode == 1, discarded_literal
        assert "must be an explicit match" in discarded_literal.stderr

        helper_expression = run(
            directory,
            endpoints=(
                'fn key(_: Fake) -> &\'static str { "write_test" }\n'
                'endpoint_wire!(Fake, value => (), (), key(value), false, ());\n'
                'const CLAIMED: &str = "read_test";'
            ),
        )
        assert helper_expression.returncode == 1, helper_expression
        assert "must be an explicit match" in helper_expression.stderr

        duplicate_lock = directory / "bodies.txt"
        duplicate_lock.write_text("write_test\nwrite_test\n", encoding="ascii")
        command = [
            str(CHECKER),
            "--matrix",
            str(directory / "matrix.md"),
            "--endpoints",
            str(directory / "prepared"),
            "--bodies",
            str(directory / "prepared" / "bodies"),
            "--body-lock",
            str(duplicate_lock),
            "--expected-active",
            "2",
            "--expected-bodies",
            "1",
        ]
        duplicate = subprocess.run(command, cwd=ROOT, check=False, capture_output=True, text=True)
        assert duplicate.returncode == 1, duplicate
        assert "duplicate body operation" in duplicate.stderr

    print("15 prepared-operation coverage tests passed.")


if __name__ == "__main__":
    main()
