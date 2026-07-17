#!/usr/bin/env python3
"""Validate the structure and pentest exit of every planned release."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parent.parent
DEFAULT_PLAN = ROOT / "docs" / "RELEASE_PLAN.md"
HEADING = re.compile(
    r"^### (v(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)) - .+$",
    re.MULTILINE,
)
FIELD = re.compile(r"^(Goal|Deliverables|Verification|Stop gate):(.*)$", re.MULTILINE)
FIELD_ORDER = ("Goal", "Deliverables", "Verification", "Stop gate")


@dataclass(frozen=True, order=True)
class Version:
    major: int
    minor: int
    patch: int

    @property
    def text(self) -> str:
        return f"v{self.major}.{self.minor}.{self.patch}"

    @property
    def gate(self) -> str:
        suffix = f"{self.major}_{self.minor}"
        if self.patch != 0:
            suffix += f"_{self.patch}"
        return f"scripts/release_{suffix}_gate.sh"


@dataclass(frozen=True)
class Section:
    version: Version
    text: str


def parse_sections(text: str) -> list[Section]:
    matches = list(HEADING.finditer(text))
    if not matches:
        raise ValueError("no release sections found")

    sections = []
    for index, match in enumerate(matches):
        end = matches[index + 1].start() if index + 1 < len(matches) else len(text)
        sections.append(
            Section(
                Version(
                    int(match.group("major")),
                    int(match.group("minor")),
                    int(match.group("patch")),
                ),
                text[match.start() : end],
            )
        )
    return sections


def is_successor(previous: Version, current: Version) -> bool:
    if current.major == previous.major:
        next_patch = (
            current.minor == previous.minor
            and current.patch == previous.patch + 1
        )
        next_minor = (
            current.minor == previous.minor + 1
            and current.patch == 0
        )
        return next_patch or next_minor
    return (
        current.major == previous.major + 1
        and current.minor == 0
        and current.patch == 0
    )


def field_content(section: str, fields: list[re.Match[str]], index: int) -> str:
    match = fields[index]
    end = fields[index + 1].start() if index + 1 < len(fields) else len(section)
    return f"{match.group(2)}\n{section[match.end():end]}".strip()


def stop_gate_contract(content: str) -> str:
    if content.startswith("```"):
        match = re.match(r"^```[^\n]*\n(.*?)\n```", content, re.DOTALL)
        if match is None:
            raise ValueError("stop gate has an unterminated fenced block")
        return match.group(1).strip()
    return content.splitlines()[0].strip()


def validate(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    sections = parse_sections(text)
    errors: list[str] = []

    if sections[0].version != Version(0, 1, 0):
        errors.append("release sequence must begin at v0.1.0")

    for previous, current in zip(sections, sections[1:]):
        if not is_successor(previous.version, current.version):
            errors.append(
                f"{current.version.text} is not the immediate successor of "
                f"{previous.version.text}"
            )

    for section in sections:
        fields = list(FIELD.finditer(section.text))
        names = tuple(match.group(1) for match in fields)
        if names != FIELD_ORDER:
            errors.append(
                f"{section.version.text} fields are {names!r}, expected "
                f"{FIELD_ORDER!r}"
            )
            continue

        contents = [field_content(section.text, fields, index) for index in range(4)]
        for name, content in zip(FIELD_ORDER, contents):
            if not content:
                errors.append(f"{section.version.text} has an empty {name} field")

        verification = contents[2]
        if section.version.gate not in verification:
            errors.append(
                f"{section.version.text} verification must call "
                f"{section.version.gate}"
            )

        try:
            stop_contract = stop_gate_contract(contents[3])
        except ValueError as error:
            errors.append(f"{section.version.text} {error}")
            continue
        stop_gate = stop_contract.lower()
        if section.version.text not in stop_contract:
            errors.append(
                f"{section.version.text} stop gate names a different version"
            )
        if "pentest" not in stop_gate or "exact commit" not in stop_gate:
            errors.append(
                f"{section.version.text} stop gate must require an exact-commit "
                "pentest"
            )

    if errors:
        raise ValueError("\n".join(errors))
    return len(sections)


def main() -> int:
    path = Path(sys.argv[1]) if len(sys.argv) == 2 else DEFAULT_PLAN
    if len(sys.argv) > 2:
        print("usage: check-release-plan-structure.py [RELEASE_PLAN]", file=sys.stderr)
        return 2
    try:
        count = validate(path)
    except (OSError, UnicodeError, ValueError) as error:
        print(f"release plan structure: {error}", file=sys.stderr)
        return 1
    print(f"{count} release plan contracts passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
