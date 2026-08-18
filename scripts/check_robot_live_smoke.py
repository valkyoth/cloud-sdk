#!/usr/bin/env python3
"""Secondary source tripwire for the read-only Robot live smoke."""

from __future__ import annotations

from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parent.parent
ROBOT_ENVIRONMENT = (
    "CLOUD_SDK_HETZNER_ROBOT_USERNAME_FILE",
    "CLOUD_SDK_HETZNER_ROBOT_PASSWORD_FILE",
)


def fail(message: str) -> None:
    raise ValueError(message)


def checked_text(
    label: str, text: str, required: tuple[str, ...], forbidden: tuple[str, ...]
) -> None:
    for value in required:
        if value not in text:
            fail(f"{label}: missing required contract {value!r}")
    for value in forbidden:
        if value in text:
            fail(f"{label}: forbidden live capability {value!r}")


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def check_repository(root: Path) -> None:
    robot = read(root / "crates/cloud-sdk-hetzner/tests/live_smoke/robot.rs")
    checked_text(
        "Robot probe",
        robot,
        (
            "ROBOT_API_BASE_URL",
            "official_robot_endpoint_policy",
            "RobotServerListRequest::new()",
            ".execute_blocking(&request, lease)",
            "RobotClientResponse::Success(_)",
        ),
        (
            "Method::Post",
            "Method::Put",
            "Method::Delete",
            "Mutation",
            "Permit",
            "Order",
            "Transaction",
            "RobotServerGetRequest",
        ),
    )
    checked_text(
        "Robot exact-wire regression",
        robot,
        (
            "fn robot_live_probe_has_exact_read_only_wire_contract()",
            'RequestTarget::new("/server")',
            "ExpectedRequest::new(Method::Get, target)",
            "run_server_probe_with_client(&client)",
            "client.transport().is_complete()",
        ),
        (),
    )

    integration = read(root / "crates/cloud-sdk-hetzner/tests/live_smoke.rs")
    checked_text(
        "Robot integration test",
        integration,
        (
            'ignore = "requires explicit opt-in and private Robot Webservice credential files"',
            "fn read_only_robot_server_smoke()",
            "robot::run_read_only_server_probe()",
        ),
        (),
    )

    runner = read(root / "scripts/hetzner-live-smoke-runner.py")
    checked_text(
        "root-owned runner",
        runner,
        (
            'ROBOT_MODE = "robot-read-only"',
            'test = "read_only_robot_server_smoke"',
            '"--exact"',
            "ROBOT_USERNAME_ENV: username_file",
            "ROBOT_PASSWORD_ENV: password_file",
            'fields["format"] != "3"',
            "ROBOT_LAUNCHER, 0o555",
        ),
        ("read_only_robot_server_smoke,",),
    )

    launcher = read(root / "scripts/cloud-sdk-hetzner-robot-smoke")
    checked_text(
        "Robot launcher",
        launcher,
        ("/usr/bin/python3 -I -S", "runner.py --robot-read-only"),
        ("$@", "${", "CLOUD_SDK_HETZNER_ROBOT_"),
    )

    wrapper = read(root / "scripts/smoke_hetzner_live.sh")
    checked_text(
        "credential-free staging",
        wrapper,
        ROBOT_ENVIRONMENT
        + (
            "cloud-sdk-hetzner-robot-smoke",
            "robot_launcher_sha256",
            "format=3",
        ),
        (),
    )

    workflow_root = root / ".github/workflows"
    for path in sorted(workflow_root.glob("*")):
        if not path.is_file() or path.suffix not in {".yml", ".yaml"}:
            continue
        text = read(path)
        for name in ROBOT_ENVIRONMENT:
            if name in text:
                fail(f"GitHub workflow contains Robot credential variable: {path.name}")
        if "cloud-sdk-hetzner-robot-smoke" in text or "robot-read-only" in text:
            fail(f"GitHub workflow gained Robot live execution: {path.name}")


def main() -> int:
    try:
        check_repository(ROOT)
    except (OSError, ValueError) as error:
        print(f"Robot live smoke contract: {error}", file=sys.stderr)
        return 1
    print("Robot live smoke secondary source tripwire passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
