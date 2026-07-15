#!/usr/bin/env python3
"""Regression tests for prepared-operation source coverage."""

from __future__ import annotations

import subprocess
import tempfile
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
    endpoint_modules: str = "mod test;\n",
    body_modules: str = "mod test;\n",
    extra_endpoint_files: dict[str, str] | None = None,
    write_endpoint_module: bool = True,
) -> subprocess.CompletedProcess[str]:
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


def main() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        directory = Path(temporary)
        complete = run(directory)
        assert complete.returncode == 0, complete
        assert "2 endpoints and 1 request bodies" in complete.stdout

        removed_module = run(directory, endpoint_modules="")
        assert removed_module.returncode == 1, removed_module
        assert "orphan prepared module source: test.rs" in removed_module.stderr

        orphan_module = run(
            directory,
            extra_endpoint_files={"orphan.rs": "const UNUSED: () = ();\n"},
        )
        assert orphan_module.returncode == 1, orphan_module
        assert "orphan prepared module source: orphan.rs" in orphan_module.stderr

        redirected_module = run(
            directory,
            endpoint_modules='#[path = "redirected.rs"] mod test;\n',
        )
        assert redirected_module.returncode == 1, redirected_module
        assert "prepared module test cannot have attributes" in redirected_module.stderr

        inline_module = run(
            directory,
            endpoint_modules="mod test { const INLINE: () = (); }\n",
        )
        assert inline_module.returncode == 1, inline_module
        assert "must be an external module declaration" in inline_module.stderr

        missing_module_file = run(directory, write_endpoint_module=False)
        assert missing_module_file.returncode == 1, missing_module_file
        assert "declared prepared module test has no source file" in missing_module_file.stderr

        duplicate_module = run(directory, endpoint_modules="mod test;\nmod test;\n")
        assert duplicate_module.returncode == 1, duplicate_module
        assert "duplicate prepared module declaration: test" in duplicate_module.stderr

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

        inner_cfg = run(directory, endpoints="#![cfg(any())]\n" + ENDPOINTS)
        assert inner_cfg.returncode == 1, inner_cfg
        assert "conditionally compiled prepared evidence is forbidden" in inner_cfg.stderr

        inner_cfg_attr = run(
            directory,
            bodies="#![cfg_attr(not(any()), cfg(any()))]\n" + BODIES,
        )
        assert inner_cfg_attr.returncode == 1, inner_cfg_attr
        assert "conditionally compiled prepared evidence is forbidden" in inner_cfg_attr.stderr

        inner_macro_use = run(directory, endpoints="#![macro_use]\n" + ENDPOINTS)
        assert inner_macro_use.returncode == 1, inner_macro_use
        assert "macro imports are forbidden" in inner_macro_use.stderr

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

        namespaced_endpoint = run(
            directory,
            endpoints=(
                "decoy::endpoint_wire!(Fake, value => (), (), match value { "
                'Fake::Read => "read_test" }, false, ());\n'
                "endpoint_wire!(Real, value => (), (), match value { "
                'Real::Write => "write_test" }, false, ());'
            ),
        )
        assert namespaced_endpoint.returncode == 1, namespaced_endpoint
        assert "must use an unqualified path" in namespaced_endpoint.stderr

        namespaced_body = run(
            directory,
            bodies=(
                'decoy::body_wire!(Fake, value => (), "write_test", write);'
            ),
        )
        assert namespaced_body.returncode == 1, namespaced_body
        assert "must use an unqualified path" in namespaced_body.stderr

        inline_fake_trait = run(
            directory,
            endpoints=(
                "mod decoy { trait EndpointWire { fn operation_key(self) -> &'static str; } "
                "struct Fake; impl EndpointWire for Fake { "
                'fn operation_key(self) -> &\'static str { "read_test" } } }\n'
                "endpoint_wire!(Real, value => (), (), match value { "
                'Real::Write => "write_test" }, false, ());'
            ),
        )
        assert inline_fake_trait.returncode == 1, inline_fake_trait
        assert "inline modules are forbidden" in inline_fake_trait.stderr

        imported_adapter = run(
            directory,
            endpoints="use decoy::endpoint_wire;\n" + ENDPOINTS,
        )
        assert imported_adapter.returncode == 1, imported_adapter
        assert "imports and aliases are forbidden" in imported_adapter.stderr

        glob_import = run(
            directory,
            endpoints="use decoy::*;\n" + ENDPOINTS,
        )
        assert glob_import.returncode == 1, glob_import
        assert "imports and aliases are forbidden" in glob_import.stderr

        macro_use = run(
            directory,
            endpoints=(
                "#[macro_use] mod decoy { "
                "macro_rules! endpoint_wire { ($($tokens:tt)*) => {}; } }\n"
                + ENDPOINTS
            ),
        )
        assert macro_use.returncode == 1, macro_use
        assert "macro imports are forbidden" in macro_use.stderr

        local_adapter = run(
            directory,
            endpoints=(
                "macro_rules! endpoint_wire { ($($tokens:tt)*) => {}; }\n" + ENDPOINTS
            ),
        )
        assert local_adapter.returncode == 1, local_adapter
        assert "macro shadowing is forbidden" in local_adapter.stderr

        generated_adapter = run(
            directory,
            endpoints=(
                "macro_rules! install_decoy { () => { "
                "macro_rules! endpoint_wire { ($($tokens:tt)*) => {}; } }; }\n"
                "install_decoy!();\n"
                + ENDPOINTS
            ),
        )
        assert generated_adapter.returncode == 1, generated_adapter
        assert "macro shadowing is forbidden" in generated_adapter.stderr

        fake_trait = run(
            directory,
            endpoints=(
                "trait EndpointWire { fn operation_key(self) -> &'static str; } "
                "struct Fake; impl EndpointWire for Fake { "
                'fn operation_key(self) -> &\'static str { "read_test" } }\n'
                "endpoint_wire!(Real, value => (), (), match value { "
                'Real::Write => "write_test" }, false, ());'
            ),
        )
        assert fake_trait.returncode == 1, fake_trait
        assert "local EndpointWire trait definitions are forbidden" in fake_trait.stderr

        duplicate_endpoint_definition = run(
            directory,
            endpoint_definitions=ENDPOINT_DEFINITIONS + ENDPOINT_DEFINITIONS,
        )
        assert duplicate_endpoint_definition.returncode == 1, duplicate_endpoint_definition
        assert "duplicate impl_endpoint_prepare definition" in duplicate_endpoint_definition.stderr

        no_op_endpoint_definition = run(
            directory,
            endpoint_definitions=(
                "macro_rules! impl_endpoint_prepare { ($($tokens:tt)*) => {}; }\n"
                "macro_rules! endpoint_wire { ($($tokens:tt)*) => {}; }\n"
                "macro_rules! query_wire { ($($tokens:tt)*) => {}; }\n"
            ),
        )
        assert no_op_endpoint_definition.returncode == 1, no_op_endpoint_definition
        assert "differs from its source lock" in no_op_endpoint_definition.stderr

        duplicate_body_definition = run(
            directory,
            body_definitions=BODY_DEFINITIONS + BODY_DEFINITIONS,
        )
        assert duplicate_body_definition.returncode == 1, duplicate_body_definition
        assert "duplicate body_wire definition" in duplicate_body_definition.stderr

        no_op_body_definition = run(
            directory,
            body_definitions=(
                "macro_rules! body_wire { ($($tokens:tt)*) => {}; }\n"
                "macro_rules! body_component { ($($tokens:tt)*) => {}; }\n"
            ),
        )
        assert no_op_body_definition.returncode == 1, no_op_body_definition
        assert "differs from its source lock" in no_op_body_definition.stderr

        duplicate_lock = directory / "bodies.txt"
        duplicate_lock.write_text("write_test\nwrite_test\n", encoding="ascii")
        command = [
            str(CHECKER),
            "--matrix",
            str(directory / "matrix.md"),
            "--endpoints",
            str(directory / "prepared" / "endpoints"),
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

    print("37 prepared-operation coverage tests passed.")


if __name__ == "__main__":
    main()
