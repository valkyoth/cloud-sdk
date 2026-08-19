#!/usr/bin/env python3
"""Validate first-party versions in publishable README TOML examples."""

from __future__ import annotations

from pathlib import Path
import sys
import tomllib


ROOT = Path(__file__).resolve().parent.parent
DEPENDENCY_TABLES = {"dependencies", "dev-dependencies", "build-dependencies"}


class ReadmeDependencyError(Exception):
    """Static README dependency validation failure."""


def release_context() -> tuple[dict[str, str], set[str]]:
    plan = tomllib.loads((ROOT / "release-crates.toml").read_text(encoding="ascii"))
    crates = plan["crates"]
    versions: dict[str, str] = {}
    selected: set[str] = set()
    for name, package in crates.items():
        version = package.get("version")
        if not isinstance(version, str) or not isinstance(package.get("publish"), bool):
            raise ReadmeDependencyError("release package metadata is invalid")
        versions[name] = version
        if package["publish"]:
            selected.add(name)
    return versions, selected


def readme_owner(path: Path) -> str | None:
    resolved = path.resolve()
    if resolved == (ROOT / "README.md").resolve():
        return "cloud-sdk"
    try:
        relative = resolved.relative_to((ROOT / "crates").resolve())
    except ValueError:
        return None
    if len(relative.parts) != 2 or relative.name != "README.md":
        return None
    manifest_path = resolved.parent / "Cargo.toml"
    manifest = tomllib.loads(manifest_path.read_text(encoding="ascii"))
    name = manifest["package"]["name"]
    if not isinstance(name, str):
        raise ReadmeDependencyError("README package identity is invalid")
    return name


def toml_blocks(path: Path) -> list[tuple[int, str]]:
    lines = path.read_text(encoding="utf-8").splitlines()
    blocks: list[tuple[int, str]] = []
    start: int | None = None
    content: list[str] = []
    for number, line in enumerate(lines, start=1):
        if start is None:
            if line.strip() == "```toml":
                start = number
                content = []
            continue
        if line.strip() == "```":
            blocks.append((start, "\n".join(content) + "\n"))
            start = None
            content = []
        else:
            content.append(line)
    if start is not None:
        raise ReadmeDependencyError("README contains an unterminated TOML fence")
    return blocks


def dependency_tables(value: object):
    if not isinstance(value, dict):
        return
    for name, item in value.items():
        if name in DEPENDENCY_TABLES:
            if not isinstance(item, dict):
                raise ReadmeDependencyError("dependency table is invalid")
            yield item
        yield from dependency_tables(item)


def dependency_identity(name: str, specification: object) -> tuple[str, str | None]:
    if isinstance(specification, str):
        return name, specification
    if isinstance(specification, dict):
        package = specification.get("package", name)
        version = specification.get("version")
        if not isinstance(package, str) or (
            version is not None and not isinstance(version, str)
        ):
            raise ReadmeDependencyError("dependency specification is invalid")
        return package, version
    raise ReadmeDependencyError("dependency specification is invalid")


def validate_readme(
    path: Path, versions: dict[str, str], selected: set[str]
) -> None:
    owner = readme_owner(path)
    if owner is not None and owner not in selected:
        return
    for line, block in toml_blocks(path):
        try:
            parsed = tomllib.loads(block)
        except tomllib.TOMLDecodeError as error:
            raise ReadmeDependencyError(
                f"{path}:{line}: TOML example is invalid"
            ) from error
        for table in dependency_tables(parsed):
            for name, specification in table.items():
                package, version = dependency_identity(name, specification)
                if package not in versions:
                    continue
                expected = versions[package]
                if version != expected:
                    raise ReadmeDependencyError(
                        f"{path}:{line}: {package} must use version {expected}"
                    )


def main() -> int:
    try:
        versions, selected = release_context()
        for raw_path in sys.argv[1:]:
            validate_readme(Path(raw_path), versions, selected)
    except (OSError, UnicodeError, KeyError, ReadmeDependencyError) as error:
        print(f"publishable README check: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
