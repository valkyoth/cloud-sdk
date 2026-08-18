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
DEFERRED_PENTEST_START = (0, 51, 0)
PER_TAG_PENTEST_START = (0, 56, 0)
EXTRA_PUBLIC_CHECKPOINTS: set[tuple[int, int, int]] = set()


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


def expected_checkpoint(version: Version) -> str:
    next_minor = ((version.minor // 5) + 1) * 5
    return f"v0.{next_minor}.0"


def is_public_checkpoint(version: Version) -> bool:
    current = (version.major, version.minor, version.patch)
    regular = version.major == 0 and version.patch == 0 and version.minor % 5 == 0
    return regular or current in EXTRA_PUBLIC_CHECKPOINTS


def validate_stop_gate(version: Version, contract: str) -> str | None:
    normalized = contract.lower()
    if version.text not in contract:
        return f"{version.text} stop gate names a different version"
    current = (version.major, version.minor, version.patch)
    historical = current < DEFERRED_PENTEST_START
    per_tag = current >= PER_TAG_PENTEST_START
    stable = version.major >= 1
    checkpoint = is_public_checkpoint(version)
    if stable:
        required = ("pentest", "exact commit")
    elif per_tag:
        required = ("pentest", "exact commit", "crates.io")
        if not stable and not checkpoint:
            required += ("defer", expected_checkpoint(version).lower())
    elif historical or checkpoint:
        required = ("pentest", "exact commit")
        if checkpoint and not historical:
            required += ("cumulative", "crates.io")
    else:
        required = (
            "security review",
            "exact commit",
            "defer",
            "crates.io",
            expected_checkpoint(version).lower(),
        )
    missing = tuple(value for value in required if value not in normalized)
    if missing:
        return f"{version.text} stop gate is missing cadence terms {missing}"
    return None


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
        stop_error = validate_stop_gate(section.version, stop_contract)
        if stop_error is not None:
            errors.append(stop_error)

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
