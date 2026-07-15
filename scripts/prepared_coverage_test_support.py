"""Fixtures for prepared-operation coverage gate regressions."""

from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "scripts" / "check_prepared_operation_coverage.py"
LOCKS = ROOT / "tools" / "prepared-coverage-check" / "locks"
ENDPOINT_DEFINITIONS = (LOCKS / "endpoints.rs").read_text(encoding="utf-8")
BODY_DEFINITIONS = (LOCKS / "bodies.rs").read_text(encoding="utf-8")
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
    """Build one API-matrix fixture row."""
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
    endpoint_definitions: str = ENDPOINT_DEFINITIONS,
    body_definitions: str = BODY_DEFINITIONS,
    crate_root: str = "pub mod prepared;\n",
    prepared_root: str = "mod endpoints;\nmod bodies;\n",
    endpoint_modules: str = "mod test;\n",
    body_modules: str = "mod test;\n",
    extra_endpoint_files: dict[str, str] | None = None,
    write_endpoint_module: bool = True,
) -> subprocess.CompletedProcess[str]:
    """Run the coverage gate against one isolated canonical module graph."""
    matrix = directory / "matrix.md"
    matrix.write_text(HEADER + row("read_test") + row("write_test"), encoding="utf-8")
    body_lock = directory / "bodies.txt"
    body_lock.write_text("write_test\n", encoding="ascii")
    prepared_dir = directory / "prepared"
    endpoint_dir = prepared_dir / "endpoints"
    body_dir = prepared_dir / "bodies"
    endpoint_dir.mkdir(parents=True, exist_ok=True)
    body_dir.mkdir(parents=True, exist_ok=True)
    for source_file in endpoint_dir.glob("*.rs"):
        source_file.unlink()
    for source_file in body_dir.glob("*.rs"):
        source_file.unlink()
    (directory / "lib.rs").write_text(crate_root, encoding="utf-8")
    (directory / "prepared.rs").write_text(prepared_root, encoding="utf-8")
    (prepared_dir / "endpoints.rs").write_text(
        endpoint_definitions + endpoint_modules,
        encoding="utf-8",
    )
    if write_endpoint_module:
        (endpoint_dir / "test.rs").write_text(endpoints, encoding="utf-8")
    else:
        (endpoint_dir / "test.rs").unlink(missing_ok=True)
    for name, source in (extra_endpoint_files or {}).items():
        (endpoint_dir / name).write_text(source, encoding="utf-8")
    (prepared_dir / "bodies.rs").write_text(
        body_definitions + body_modules,
        encoding="utf-8",
    )
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
