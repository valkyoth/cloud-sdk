#!/usr/bin/env python3
"""Negative tests for manifest/lock/document admission binding."""
from pathlib import Path
import tempfile
import check_admission_evidence as checker

FILES = (
    "Cargo.toml", "Cargo.lock", "tools/prepared-coverage-check/Cargo.toml",
    "tools/prepared-coverage-check/Cargo.lock", "docs/dependency-admission-rustix.md",
    "docs/dependency-admission-prepared-coverage.md",
)


def main():
    checker.validate(checker.ROOT)
    cases = [
        (FILES[4], "- Version: `1.1.5`", "- Version: `1.1.4`"),
        (FILES[4], "891efaba", "00000000"),
        (FILES[5], "| `syn` | `3.0.9`", "| `syn` | `3.0.4`"),
        (FILES[5], "d78c8dee", "00000000"),
        (FILES[5], "0662b098", "00000000"),
        (FILES[5], "6c622058", "00000000"),
        (FILES[5], "| `saphyr`", "| `other`"),
        (FILES[0], 'version = "=1.1.5"', 'version = "=1.1.4"'),
        (FILES[2], 'version = "=3.0.9"', 'version = "=3.0.4"'),
        (FILES[2], 'default-features = false', 'default-features = true'),
        (FILES[2], '"parsing", "visit"', '"parsing"'),
        (FILES[3], 'checksum = "d78c8dee', 'checksum = "00000000'),
    ]
    for filename, before, after in cases:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name in FILES:
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                text = (checker.ROOT / name).read_text()
                if name == filename:
                    assert before in text
                    text = text.replace(before, after, 1)
                path.write_text(text)
            try:
                checker.validate(root)
            except ValueError:
                pass
            else:
                raise AssertionError(f"accepted stale admission: {filename} {before}")
    with tempfile.TemporaryDirectory() as directory:
        try:
            checker.validate(Path(directory))
        except OSError:
            pass
        else:
            raise AssertionError("missing evidence accepted")
    print("13 admission-evidence rejection cases passed.")


if __name__ == "__main__":
    main()
