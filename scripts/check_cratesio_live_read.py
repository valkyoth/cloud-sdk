#!/usr/bin/env python3
"""Build the read harness, run offline tests, and prove CI opt-in fails closed."""

import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    command = ["cargo", "test", "--locked", "-p", "cloud-sdk-cratesio",
               "--no-default-features", "--features", "blocking-rustls",
               "--test", "live_read", "--no-run", "--message-format=json"]
    built = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, check=True)
    paths = []
    for line in built.stdout.splitlines():
        artifact = json.loads(line)
        if (artifact.get("reason") == "compiler-artifact"
                and artifact.get("target", {}).get("name") == "live_read"
                and artifact.get("executable")):
            paths.append(artifact["executable"])
    if len(paths) != 1:
        raise RuntimeError("exactly one live-read test binary required")
    binary = paths[0]
    # Deliberately do not pass the caller's credentials, opt-ins or secrets.
    environment = {key: os.environ[key] for key in ("PATH", "SystemRoot") if key in os.environ}
    subprocess.run([binary], env=environment, cwd=ROOT, stdin=subprocess.DEVNULL, check=True)
    for marker in ("CI", "GITHUB_ACTIONS", "GITLAB_CI", "BUILD_BUILDID", "JENKINS_URL"):
        for mode in ("anonymous", "token"):
            configured = {**environment, marker: "false",
                          "CLOUD_SDK_CRATESIO_LIVE": mode,
                          "CLOUD_SDK_CRATESIO_USER_AGENT": "ci-proof/1 (tests@example.org)"}
            if mode == "token":
                configured["CLOUD_SDK_CRATESIO_TOKEN_ACK"] = "isolated-account-read-requests"
            result = subprocess.run([binary, "--ignored", "--exact", "live_read", "--nocapture"],
                                    env=configured, cwd=ROOT, stdin=subprocess.DEVNULL,
                                    capture_output=True, text=True, timeout=10)
            if result.returncode == 0 or "live crates.io execution is forbidden in CI" not in result.stdout + result.stderr:
                raise RuntimeError("live-read CI rejection proof failed")
    print("crates.io live harness: offline fixtures and 10 CI rejection cases passed")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        print(f"crates.io live harness check failed: {type(error).__name__}", file=sys.stderr)
        raise SystemExit(1) from None
