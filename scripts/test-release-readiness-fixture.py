#!/usr/bin/env python3
"""Install deterministic release metadata used by readiness shell tests."""

from __future__ import annotations

import sys
from pathlib import Path


version, stage, baseline = sys.argv[1:4]
exceptional = len(sys.argv) == 5 and sys.argv[4] == "true"
milestones = f'["{version}"]' if version != baseline else "[]"
publish = "true" if stage == "public" else "false"
Path("release-crates.toml").write_text(
    f'''[release]
version = "{version}"
milestone = "{version}"
baseline = "{baseline}"
cumulative_milestones = {milestones}
policy = "independent"
stage = "{stage}"
exceptional = {str(exceptional).lower()}
exception_reason = "{'Fixture exceptional assessment.' if exceptional else ''}"

[crates."cloud-sdk"]
previous_version = "{baseline}"
version = "{version}"
change = "code"
publish = {publish}
reason = "fixture"
''',
    encoding="ascii",
)
