#!/usr/bin/env python3
"""Regression tests for release-plan contract validation."""

from pathlib import Path
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parent.parent
CHECKER = ROOT / "scripts" / "check-release-plan-structure.py"


def section(version: str, gate: str) -> str:
    return f"""### {version} - Test Release

Goal: provide a concrete goal.

Deliverables: provide concrete deliverables.

Verification: run `{gate}`.

Stop gate: `{version} implementation stop reached. Run pentest for this exact commit.`
"""


def run(path: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(CHECKER), str(path)],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )


def reject(path: Path, text: str) -> None:
    path.write_text(text, encoding="utf-8")
    result = run(path)
    assert result.returncode == 1, result


def main() -> None:
    repository_path = ROOT / "docs" / "RELEASE_PLAN.md"
    repository = run(repository_path)
    assert repository.returncode == 0, repository
    repository_text = repository_path.read_text(encoding="utf-8")

    first = section("v0.1.0", "scripts/release_0_1_gate.sh")
    second = section("v0.2.0", "scripts/release_0_2_gate.sh")
    with tempfile.TemporaryDirectory() as temporary:
        path = Path(temporary) / "RELEASE_PLAN.md"

        reject(path, first.replace("Goal:", "Purpose:"))
        reject(
            path,
            first.replace(
                "Deliverables: provide concrete deliverables.\n\n"
                "Verification: run `scripts/release_0_1_gate.sh`.",
                "Verification: run `scripts/release_0_1_gate.sh`.\n\n"
                "Deliverables: provide concrete deliverables.",
            ),
        )
        reject(path, first.replace("release_0_1_gate.sh", "release_0_2_gate.sh"))
        reject(path, first.replace(" for this exact commit", " before release"))
        reject(
            path,
            first.replace("v0.1.0 implementation", "v0.2.0 implementation")
            + "\nLater planning still references v0.1.0.\n",
        )
        reject(
            path,
            first.replace(
                "Stop gate: `v0.1.0 implementation stop reached. Run pentest "
                "for this exact commit.`",
                "Stop gate:\n\n```text\nunterminated",
            ),
        )
        reject(
            path,
            first + section("v0.3.0", "scripts/release_0_3_gate.sh"),
        )
        reject(
            path,
            repository_text.replace(
                "v0.51.0 implementation stop reached. Complete the security review",
                "v0.51.0 implementation stop reached. Run pentest",
            ),
        )
        reject(
            path,
            repository_text.replace(
                "v0.55.0 implementation stop reached. Run the cumulative pentest",
                "v0.55.0 implementation stop reached. Run pentest",
            ),
        )
        reject(
            path,
            repository_text.replace(
                "v0.56.0 implementation stop reached. Complete the pentest",
                "v0.56.0 implementation stop reached. Complete the security review",
            ),
        )
        reject(
            path,
            repository_text.replace(
                "defer crates.io publication to v0.100.0",
                "defer crates.io publication to v1.0.0",
            ),
        )

        path.write_text(first + second, encoding="utf-8")
        valid_fixture = run(path)
        assert valid_fixture.returncode == 0, valid_fixture

    print("13 release plan structure tests passed.")


if __name__ == "__main__":
    main()
