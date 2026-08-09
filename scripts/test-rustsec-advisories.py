#!/usr/bin/env python3
import os
import pathlib
import subprocess
import tempfile


ROOT = pathlib.Path(__file__).resolve().parents[1]
CHECK = ROOT / "scripts" / "check_rustsec_advisories.sh"


def fail(message: str) -> None:
    raise AssertionError(message)


def run_helper(*, fail_at: int | None = None) -> tuple[subprocess.CompletedProcess[str], list[str]]:
    with tempfile.TemporaryDirectory() as temporary:
        temp = pathlib.Path(temporary)
        binary = temp / "bin"
        binary.mkdir()
        log = temp / "audit.log"
        fake = binary / "cargo"
        fake.write_text(
            """#!/usr/bin/env sh
set -eu
printf '%s\\n' "$*" >>"$AUDIT_LOG"
count="$(wc -l <"$AUDIT_LOG")"
if [ -n "${FAIL_AT:-}" ] && [ "$count" -eq "$FAIL_AT" ]; then
    exit 23
fi
database="$3"
if [ "$count" -eq 1 ]; then
    mkdir -p "$database"
    : >"$database/fetched"
else
    test -f "$database/fetched"
fi
""",
            encoding="ascii",
        )
        fake.chmod(0o755)
        environment = os.environ.copy()
        environment["PATH"] = f"{binary}:{environment['PATH']}"
        environment["TMPDIR"] = str(temp)
        environment["AUDIT_LOG"] = str(log)
        if fail_at is not None:
            environment["FAIL_AT"] = str(fail_at)

        result = subprocess.run(
            [str(CHECK)],
            cwd=ROOT,
            env=environment,
            text=True,
            capture_output=True,
            check=False,
        )
        lines = log.read_text(encoding="ascii").splitlines()
        leftovers = list(temp.glob("cloud-sdk-rustsec.*"))
        if leftovers:
            fail(f"temporary advisory database was not removed: {leftovers}")
        return result, lines


def test_success() -> None:
    result, lines = run_helper()
    if result.returncode != 0:
        fail(result.stderr)
    if len(lines) != 4:
        fail(f"expected four audits, got: {lines}")

    arguments = [line.split() for line in lines]
    databases = [parts[2] for parts in arguments]
    if len(set(databases)) != 1:
        fail(f"audits did not share one fresh database: {databases}")
    if "--no-fetch" in arguments[0]:
        fail("the first audit must fetch the fresh database")

    expected_locks = [
        "tests/reqwest-feature-unification/Cargo.lock",
        "fuzz/Cargo.lock",
        "tools/prepared-coverage-check/Cargo.lock",
    ]
    for parts, lockfile in zip(arguments[1:], expected_locks, strict=True):
        if parts[-3:] != ["--no-fetch", "--file", lockfile]:
            fail(f"unexpected follow-up audit: {' '.join(parts)}")


def test_failure_cleanup() -> None:
    result, lines = run_helper(fail_at=2)
    if result.returncode != 23:
        fail(f"audit failure was not propagated: {result.returncode}")
    if len(lines) != 2:
        fail(f"audits continued after failure: {lines}")


def test_integration() -> None:
    workflow = (ROOT / ".github/workflows/ci.yml").read_text(encoding="ascii")
    gate = (ROOT / "scripts/release_0_69_gate.sh").read_text(encoding="ascii")
    invocation = "scripts/check_rustsec_advisories.sh"
    if invocation not in workflow:
        fail("CI does not use the isolated RustSec audit helper")
    if invocation not in gate:
        fail("the v0.69 release gate does not use the isolated audit helper")


def main() -> None:
    test_success()
    test_failure_cleanup()
    test_integration()
    print("3 isolated RustSec advisory regression groups passed.")


if __name__ == "__main__":
    main()
