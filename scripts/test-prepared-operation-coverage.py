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


def row(operation: str, *, deprecated: str = "no") -> str:
    status = "deferred-deprecated" if deprecated == "yes" else "implemented"
    return (
        f"| cloud | Test | POST | `/test` | `{operation}` | `test` | no | no | "
        f"none | {deprecated} | {status} |\n"
    )


def run(
    directory: Path,
    *,
    endpoints: str = 'const OPS: &[&str] = &["read_test", "write_test"];',
    bodies: str = 'const OPS: &[&str] = &["write_test"];',
) -> subprocess.CompletedProcess[str]:
    matrix = directory / "matrix.md"
    matrix.write_text(HEADER + row("read_test") + row("write_test"), encoding="utf-8")
    body_lock = directory / "bodies.txt"
    body_lock.write_text("write_test\n", encoding="ascii")
    endpoint_dir = directory / "prepared"
    body_dir = endpoint_dir / "bodies"
    body_dir.mkdir(parents=True, exist_ok=True)
    (directory / "prepared.rs").write_text(endpoints, encoding="utf-8")
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

        missing_endpoint = run(directory, endpoints='const OPS: &[&str] = &["write_test"];')
        assert missing_endpoint.returncode == 1, missing_endpoint
        assert "missing endpoint adapters: read_test" in missing_endpoint.stderr

        missing_body = run(directory, bodies='const OPS: &[&str] = &[];')
        assert missing_body.returncode == 1, missing_body
        assert "missing body adapters: write_test" in missing_body.stderr

        line_comments = run(
            directory,
            endpoints='// "read_test"\nconst OPS: &[&str] = &["write_test"];',
        )
        assert line_comments.returncode == 1, line_comments
        assert "missing endpoint adapters: read_test" in line_comments.stderr

        block_comments = run(
            directory,
            bodies='/* "write_test" */ const OPS: &[&str] = &[];',
        )
        assert block_comments.returncode == 1, block_comments
        assert "missing body adapters: write_test" in block_comments.stderr

        test_only = run(
            directory,
            endpoints=(
                'const OPS: &[&str] = &["write_test"];\n'
                '#[cfg(test)] mod tests { const OP: &str = "read_test"; }'
            ),
        )
        assert test_only.returncode == 1, test_only
        assert "test-only code is forbidden" in test_only.stderr

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

    print("7 prepared-operation coverage tests passed.")


if __name__ == "__main__":
    main()
