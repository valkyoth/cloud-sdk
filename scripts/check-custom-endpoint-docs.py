#!/usr/bin/env python3
"""Check explicit custom credential endpoint naming and nearby warnings."""

from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parent.parent
README = ROOT / "crates" / "cloud-sdk-reqwest" / "README.md"


def validate(readme: Path) -> None:
    text = readme.read_text(encoding="utf-8")
    lines = text.splitlines()
    examples = [index for index, line in enumerate(lines) if "HttpsEndpoint::new_custom(" in line]
    if len(examples) != 2:
        raise ValueError("blocking and async examples must each construct one custom endpoint")
    for index in examples:
        context = "\n".join(lines[max(0, index - 3) : index])
        if "bearer-token destination" not in context or "tenant-controlled input" not in context:
            raise ValueError("custom endpoint example lacks an adjacent credential warning")

    for path in [*ROOT.glob("crates/**/*.rs"), *ROOT.glob("tests/**/*.rs")]:
        if "HttpsEndpoint::new(" in path.read_text(encoding="utf-8"):
            raise ValueError(f"legacy ambiguous endpoint constructor remains in {path}")


def main() -> int:
    readme = Path(sys.argv[1]) if len(sys.argv) == 2 else README
    if len(sys.argv) > 2:
        print("usage: check-custom-endpoint-docs.py [README]", file=sys.stderr)
        return 2
    try:
        validate(readme)
    except (OSError, UnicodeError, ValueError) as error:
        print(f"custom endpoint docs: {error}", file=sys.stderr)
        return 1
    print("Custom endpoint examples have explicit credential warnings.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
