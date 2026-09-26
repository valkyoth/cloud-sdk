#!/usr/bin/env python3
"""Regression checks for fail-closed execution coverage, without network access."""
import contextlib
import io
import sys
import tempfile
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import check_cratesio_execution_coverage as checker

SCOPE = "classification\toperation_id\tmethod\tpath\nincluded\tlist_fixture\tGET\t/api/v1/fixture\n"


def rejected(call):
    try:
        call()
    except ValueError:
        return
    raise AssertionError("invalid evidence accepted")


def main():
    checks = (checker.ROOT / "scripts/checks.sh").read_text().splitlines()
    release = (checker.ROOT / "scripts/release_1_1_gate.sh").read_text().splitlines()
    assert "python3 scripts/check_cratesio_execution_coverage.py --check-routes" in checks
    assert "python3 scripts/test-cratesio-execution-coverage.py" in checks
    assert "python3 scripts/check_cratesio_execution_coverage.py" in release
    assert not any("check_cratesio_execution_coverage.py --report" in line for line in release)
    inventory = checker.rows(SCOPE)
    complete = "\n".join(f"{checker.MARKER}list_fixture:{mode}" for mode in sorted(checker.MODES))
    assert checker.coverage(inventory, complete) == {}
    assert checker.coverage(inventory, "") == {"list_fixture": ["blocking", "local", "send"]}
    assert checker.coverage(inventory, f"{checker.MARKER}list_fixture:blocking\n" * 3) == {
        "list_fixture": ["local", "send"]}
    for payload in ("unknown:send", "list_fixture:fake", "list_fixture:send:extra", ""):
        rejected(lambda: checker.coverage(inventory, checker.MARKER + payload))
    rejected(lambda: checker.rows(SCOPE + SCOPE.splitlines()[-1] + "\n"))
    rejected(lambda: checker.rows(SCOPE.replace("list_fixture", 'bad"code')))
    rejected(lambda: checker.rows(SCOPE.replace("GET", "BOGUS")))
    rejected(lambda: checker.rows(SCOPE.splitlines()[0]))
    changed = checker.rows(SCOPE + "included\tnew_fixture\tPOST\t/api/v1/new_fixture\n")
    assert "new_fixture" in checker.coverage(changed, complete)
    assert checker.render(changed) != checker.render(inventory)

    with tempfile.TemporaryDirectory() as directory:
        scope, table = Path(directory) / "scope.tsv", Path(directory) / "table.rs"
        scope.write_text(SCOPE, encoding="ascii")
        table.write_text(checker.render(inventory), encoding="ascii")
        with patch.object(checker, "SCOPE", scope), patch.object(checker, "TABLE", table):
            for args, output, exit_code, expected in [
                ([], complete, 0, 0), ([], "", 0, 1), (["--report"], "", 0, 0),
                ([], complete, 1, None), (["--report"], complete, 1, None),
            ]:
                with patch.object(sys, "argv", ["coverage", *args]), contextlib.redirect_stdout(io.StringIO()), \
                        patch.object(checker.subprocess, "run", return_value=SimpleNamespace(
                            stdout=output, returncode=exit_code)) as run:
                    if expected is None:
                        rejected(checker.main)
                    else:
                        assert checker.main() == expected
                    command = run.call_args.args[0]
                    assert command == ["cargo", "test", "--locked", "-p", "cloud-sdk-cratesio",
                        "--all-features", "--lib", "--", "--nocapture", "--test-threads=1"]
            table.write_text("stale", encoding="ascii")
            with patch.object(sys, "argv", ["coverage"]), patch.object(checker.subprocess, "run") as run:
                rejected(checker.main)
                run.assert_not_called()
    print("Execution coverage parser, command, freshness and failure regressions passed.")


if __name__ == "__main__":
    main()
