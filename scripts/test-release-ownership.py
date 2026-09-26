#!/usr/bin/env python3
"""Initial-publication absence is distinct from ownership/network failure."""

import importlib.util
from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location(
    "governance", Path(__file__).with_name("check_release_governance.py")
)
checker = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(checker)


class OwnershipTests(unittest.TestCase):
    config = {"ownership": {"required_crates_io_owner": "eldryoth"},
              "packages": {"publishable": ["cloud-sdk-cratesio"]}}
    missing = ("the remote server responded with an error (status 404 Not Found): "
               "crate `cloud-sdk-cratesio` does not exist")

    def check(self, previous, output=None, error=None):
        plan = {"crates": {"cloud-sdk-cratesio": {"previous_version": previous}}}
        with patch.object(checker, "load_toml", return_value=plan), \
                patch.object(checker.subprocess, "check_output", return_value=output, side_effect=error) as call:
            checker.check_live_crates_io(self.config)
            call.assert_called_once_with(
                ["cargo", "owner", "--list", "cloud-sdk-cratesio"],
                cwd=checker.ROOT, text=True, stderr=subprocess.PIPE,
            )

    def test_initial_absence_only(self):
        error = subprocess.CalledProcessError(101, [], stderr=self.missing)
        self.check("none", error=error)
        for previous in (None, "1.0.0", "unknown"):
            with self.assertRaises(subprocess.CalledProcessError):
                self.check(previous, error=error)

    def test_initial_publication_does_not_hide_other_failures(self):
        for stderr in ("timeout", "403 Forbidden", "500 Service Unavailable", "404 Not Found",
                       self.missing.replace("cloud-sdk-cratesio", "another-crate")):
            with self.assertRaises(subprocess.CalledProcessError):
                self.check("none", error=subprocess.CalledProcessError(101, [], stderr=stderr))
        with self.assertRaises(OSError):
            self.check("none", error=OSError("connection failed"))

    def test_existing_initial_package_requires_the_expected_owner(self):
        self.check("none", output="eldryoth (owner)\n")
        self.check("1.0.0", output="eldryoth (owner)\n")
        for previous in ("none", "1.0.0"):
            for output in ("", "someone-else\n", "eldryoth-imposter\n"):
                with self.assertRaises(checker.GovernanceError):
                    self.check(previous, output=output)


if __name__ == "__main__":
    unittest.main()
