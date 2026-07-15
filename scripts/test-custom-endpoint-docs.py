#!/usr/bin/env python3
"""Regression tests for custom endpoint documentation warnings."""

from pathlib import Path
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parent.parent
CHECKER = ROOT / "scripts" / "check-custom-endpoint-docs.py"
README = ROOT / "crates" / "cloud-sdk-reqwest" / "README.md"


def run(path: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(CHECKER), str(path)],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )


def main() -> None:
    repository = run(README)
    assert repository.returncode == 0, repository

    source = README.read_text(encoding="utf-8")
    with tempfile.TemporaryDirectory() as temporary:
        path = Path(temporary) / "README.md"
        path.write_text(
            source.replace("tenant-controlled input", "untrusted input", 1),
            encoding="utf-8",
        )
        missing_warning = run(path)
        assert missing_warning.returncode == 1, missing_warning

    print("2 custom endpoint documentation tests passed.")


if __name__ == "__main__":
    main()
