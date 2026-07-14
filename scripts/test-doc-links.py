#!/usr/bin/env python3
"""Regression tests for the local documentation-link gate."""

from __future__ import annotations

import subprocess
import tempfile
from pathlib import Path

CHECKER = Path(__file__).with_name("check_doc_links.py")


def run(root: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["python3", str(CHECKER), "--root", str(root)],
        check=False,
        capture_output=True,
        text=True,
    )


def write(path: Path, value: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(value, encoding="utf-8")


def main() -> None:
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        write(root / "docs" / "guide.md", "# Guide\n")
        write(
            root / "README.md",
            "[guide](docs/guide.md#section) [external](https://example.com/x)\n",
        )
        result = run(root)
        assert result.returncode == 0, result.stderr

        write(root / "README.md", "[valid](docs/guide.md) [missing](docs/no.md)\n")
        result = run(root)
        assert result.returncode == 1, result
        assert "docs/no.md" in result.stderr, result.stderr

        write(root / "README.md", "[escape](../outside.md)\n")
        result = run(root)
        assert result.returncode == 1, result
        assert "escapes repository root" in result.stderr, result.stderr

        write(root / "README.md", "```md\n[fixture](missing.md)\n```\n")
        result = run(root)
        assert result.returncode == 0, result.stderr


if __name__ == "__main__":
    main()
