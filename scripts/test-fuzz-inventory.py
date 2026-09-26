#!/usr/bin/env python3
"""Regression coverage for inventory bypasses and mandatory all-target Clippy."""

import copy
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib

from check_fuzz_inventory import ROOT, TARGETS, validate


def rejects(manifest):
    try:
        validate(manifest)
    except ValueError:
        pass
    else:
        raise AssertionError("unreviewed fuzz inventory accepted")


def inventory_cases():
    manifest = tomllib.loads((ROOT / "fuzz/Cargo.toml").read_text())
    validate(manifest)
    for position in (0, 10, len(TARGETS)):
        changed = copy.deepcopy(manifest)
        changed["bin"].insert(position, dict(changed["bin"][0], name="unreviewed"))
        rejects(changed)
    for field, value in (
        ("path", "fuzz_targets/buffer_writers.rs"),
        ("path", "../outside.rs"), ("name", "renamed"),
        ("test", True), ("bench", True), ("doc", True),
    ):
        changed = copy.deepcopy(manifest)
        changed["bin"][1][field] = value
        rejects(changed)
    changed = copy.deepcopy(manifest)
    changed["bin"].reverse()
    rejects(changed)
    changed = copy.deepcopy(manifest)
    changed["bin"].pop()
    rejects(changed)
    changed = copy.deepcopy(manifest)
    del changed["bin"][0]["path"]
    rejects(changed)
    changed = copy.deepcopy(manifest)
    changed["bin"][0]["required-features"] = ["skip"]
    rejects(changed)
    for value in (None, True):
        changed = copy.deepcopy(manifest)
        changed["package"]["autobins"] = value
        rejects(changed)


def shell_cases():
    # Only external Cargo/Git/AWS-LC work is stubbed. Inventory and shell gate
    # are the real files; test their rejection ordering and Clippy arguments.
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        (root / "scripts").mkdir()
        (root / "bin").mkdir()
        for name in ("check_fuzz_inventory.py", "check_fuzz_harness.sh"):
            shutil.copyfile(ROOT / "scripts" / name, root / "scripts" / name)
        checker = root / "scripts/check-fuzz-aws-lc-tree.py"
        checker.write_text("#!/bin/sh\ncat >/dev/null\n")
        checker.chmod(0o755)
        for name in TARGETS:
            source = root / f"fuzz/fuzz_targets/{name}.rs"
            source.parent.mkdir(parents=True, exist_ok=True)
            source.write_text("// fixture\n")
            seed = root / f"fuzz/seeds/{name}/seed"
            seed.parent.mkdir(parents=True, exist_ok=True)
            seed.write_text("fixture\n")
        cargo = root / "bin/cargo"
        cargo.write_text(
            "#!/usr/bin/env python3\n"
            "import json, os, sys\n"
            "with open(os.environ['CALL_LOG'], 'a') as log:\n"
            "    log.write(json.dumps(sys.argv[1:]) + '\\n')\n"
            "if sys.argv[1] == 'clippy':\n"
            "    raise SystemExit(int(os.environ['CLIPPY_EXIT']))\n"
        )
        cargo.chmod(0o755)
        git = root / "bin/git"
        git.write_text("#!/bin/sh\nexit 0\n")
        git.chmod(0o755)
        log = root / "calls"
        environment = {
            "PATH": str(root / "bin") + os.pathsep + os.environ["PATH"],
            "CALL_LOG": str(log), "CLIPPY_EXIT": "23",
        }
        original = (ROOT / "fuzz/Cargo.toml").read_text()
        manifest = root / "fuzz/Cargo.toml"
        for content in (
            original.replace("[[bin]]", '[[bin]]\nname = "unreviewed"\n'
                             'path = "fuzz_targets/unreviewed.rs"\n\n[[bin]]', 1),
            original.replace('path = "fuzz_targets/checked_response.rs"',
                             'path = "fuzz_targets/buffer_writers.rs"', 1),
            "invalid TOML [",
        ):
            assert content != original
            manifest.write_text(content)
            result = subprocess.run(["sh", "scripts/check_fuzz_harness.sh", "--metadata"],
                                    cwd=root, env=environment, capture_output=True)
            assert result.returncode != 0
            assert not log.exists(), "inventory rejection must precede Cargo"
        manifest.write_text(original)
        for code in (23, 0):
            environment["CLIPPY_EXIT"] = str(code)
            log.write_text("")
            result = subprocess.run(["sh", "scripts/check_fuzz_harness.sh", "--metadata"],
                                    cwd=root, env=environment, capture_output=True)
            assert result.returncode == code, result.stderr
            calls = [json.loads(line) for line in log.read_text().splitlines()]
            assert calls[3] == ["clippy", "--locked", "--manifest-path",
                                "fuzz/Cargo.toml", "--all-targets", "--", "-D", "warnings"]
            if code:
                assert len(calls) == 4, "Clippy failure must stop before tests"
            else:
                assert calls[4] == ["test", "--locked", "--manifest-path",
                                    "fuzz/Cargo.toml", "--tests"]
                assert len(calls) == 5


if __name__ == "__main__":
    inventory_cases()
    shell_cases()
    print("Fuzz inventory mutations and shell Clippy enforcement regressions passed.")
