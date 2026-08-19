#!/usr/bin/env python3
"""Regression tests for immutable crates.io README wording."""

from __future__ import annotations

from pathlib import Path
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parent.parent
CHECKER = ROOT / "scripts" / "check_publishable_readmes.sh"


def run(*paths: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(CHECKER), *(str(path) for path in paths)],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )


def test_repository_readmes() -> None:
    result = run()
    assert result.returncode == 0, result


def test_stable_wording() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        readme = Path(temporary) / "README.md"
        readme.write_text(
            "Published versions are on crates.io. Every candidate needs a pentest.\n",
            encoding="ascii",
        )
        result = run(readme)
        assert result.returncode == 0, result


def test_development_status_is_rejected() -> None:
    phrases = (
        "Status: implementation stop reached",
        "Status: pentest required",
        "The latest published release is v0.19.0",
        "The latest published provider release is 0.17.0",
        "The current main branch is preparing the workspace 0.20.0 release",
        "The planned provider 0.17.1 changes metadata",
    )
    with tempfile.TemporaryDirectory() as temporary:
        directory = Path(temporary)
        for index, phrase in enumerate(phrases):
            readme = directory / f"README-{index}.md"
            readme.write_text(phrase + "\n", encoding="ascii")
            result = run(readme)
            assert result.returncode == 1, (phrase, result)
            assert "development-only release status" in result.stderr


def test_missing_file_is_rejected() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        missing = Path(temporary) / "missing.md"
        result = run(missing)
        assert result.returncode == 2, result
        assert "missing file" in result.stderr


def test_first_party_dependency_versions_are_exact() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        readme = Path(temporary) / "README.md"
        readme.write_text(
            """```toml
[dependencies]
cloud-sdk = "0.99.0"
cloud-sdk-reqwest = { version = "0.37.0" }
```
""",
            encoding="ascii",
        )
        result = run(readme)
        assert result.returncode == 1, result
        assert "cloud-sdk must use version 0.100.0" in result.stderr

        readme.write_text(
            """```toml
[dependencies]
cloud-sdk = "0.100.0"
cloud-sdk-reqwest = { version = "0.37.0" }
```
""",
            encoding="ascii",
        )
        result = run(readme)
        assert result.returncode == 0, result


def test_invalid_toml_fence_is_rejected() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        readme = Path(temporary) / "README.md"
        readme.write_text(
            "```toml\n[dependencies\ncloud-sdk = \"0.100.0\"\n```\n",
            encoding="ascii",
        )
        result = run(readme)
        assert result.returncode == 1, result
        assert "TOML example is invalid" in result.stderr


def test_commonmark_toml_fences_are_enforced() -> None:
    fences = (
        ("```TOML", "```"),
        ("~~~toml", "~~~"),
        ('```toml title="Cargo.toml"', "```"),
        ("````toml", "`````"),
        ('```{.toml title="Cargo.toml"}', "```"),
    )
    with tempfile.TemporaryDirectory() as temporary:
        readme = Path(temporary) / "README.md"
        for opening, closing in fences:
            readme.write_text(
                f"{opening}\n[dependencies]\ncloud-sdk = \"0.99.0\"\n{closing}\n",
                encoding="ascii",
            )
            result = run(readme)
            assert result.returncode == 1, (opening, result)
            assert "cloud-sdk must use version 0.100.0" in result.stderr


def test_unterminated_toml_variants_fail_closed() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        readme = Path(temporary) / "README.md"
        for opening in ("```TOML", "~~~toml", '```toml title="Cargo.toml"'):
            readme.write_text(
                f"{opening}\n[dependencies]\ncloud-sdk = \"0.99.0\"\n",
                encoding="ascii",
            )
            result = run(readme)
            assert result.returncode == 1, (opening, result)
            assert "unterminated TOML fence" in result.stderr


def main() -> None:
    test_repository_readmes()
    test_stable_wording()
    test_development_status_is_rejected()
    test_missing_file_is_rejected()
    test_first_party_dependency_versions_are_exact()
    test_invalid_toml_fence_is_rejected()
    test_commonmark_toml_fences_are_enforced()
    test_unterminated_toml_variants_fail_closed()
    print("8 publishable README regression groups passed.")


if __name__ == "__main__":
    main()
