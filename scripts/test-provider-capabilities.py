#!/usr/bin/env python3
"""Regression tests for provider capability documentation."""

from pathlib import Path
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parent.parent
CHECKER = ROOT / "scripts" / "check-provider-capabilities.py"


def run(path: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(CHECKER), str(path)],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )


def main() -> None:
    repository = run(ROOT / "crates" / "cloud-sdk-hetzner" / "README.md")
    assert repository.returncode == 0, repository

    source = (ROOT / "crates" / "cloud-sdk-hetzner" / "README.md").read_text(
        encoding="utf-8"
    )
    with tempfile.TemporaryDirectory() as temporary:
        path = Path(temporary) / "README.md"
        path.write_text(source.replace("End-to-end client | Not available", "End-to-end client | Complete"), encoding="utf-8")
        overstated = run(path)
        assert overstated.returncode == 1, overstated

        path.write_text(source.replace("| Capability | Current coverage | Planned completion |", "| Hetzner API area | Supported |"), encoding="utf-8")
        ambiguous = run(path)
        assert ambiguous.returncode == 1, ambiguous

    print("3 provider capability documentation tests passed.")


if __name__ == "__main__":
    main()
