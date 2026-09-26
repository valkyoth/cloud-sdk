#!/usr/bin/env python3
"""Exercise clone provenance, clean-tree rejection and archive comparison."""

from pathlib import Path
import subprocess
import tempfile

from check_candidate_reproduction import PACKAGES, package_command, reproduce


def main():
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary)
        subprocess.run(["git", "init", "-q", str(root)], check=True)
        (root / "fixture").write_text("reviewed\n", encoding="ascii")
        subprocess.run(["git", "add", "fixture"], cwd=root, check=True)
        subprocess.run(["git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                        "-c", "commit.gpgsign=false", "commit", "-qm", "fixture"], cwd=root, check=True)
        clones = []

        def builder(clone, target):
            assert (clone / "fixture").read_text("ascii") == "reviewed\n"
            assert not target.exists()
            clones.append(clone)
            return {name: ("digest", {"member"}) for name in PACKAGES}

        reproduce(root, builder)
        assert len(clones) == 2 and clones[0] != clones[1]
        assert all(not path.exists() for path in clones)
        for behavior in ("mismatch", "missing", "dirty-clone", "dirty-root"):
            calls = []

            def bad(clone, target):
                calls.append(clone)
                result = builder(clone, target)
                if behavior == "mismatch" and len(calls) == 2:
                    result[PACKAGES[0]] = ("different", {"member"})
                if behavior == "missing":
                    result.pop(PACKAGES[0])
                if behavior == "dirty-clone":
                    (clone / "unexpected").write_text("dirty", encoding="ascii")
                if behavior == "dirty-root":
                    (root / "unexpected").write_text("dirty", encoding="ascii")
                return result

            try:
                reproduce(root, bad)
            except ValueError:
                pass
            else:
                raise AssertionError(behavior)
            (root / "unexpected").unlink(missing_ok=True)
        (root / "untracked").write_text("dirty", encoding="ascii")
        try:
            reproduce(root, builder)
        except ValueError:
            pass
        else:
            raise AssertionError("dirty root accepted")
        for name in PACKAGES:
            command = package_command(name)
            assert command[:6] == ["cargo", "package", "--locked", "--offline", "--no-verify", "--all-features"]
            assert "--allow-dirty" not in command
            assert "publish" not in command
    print("6 candidate reproduction regression groups passed.")


if __name__ == "__main__":
    main()
