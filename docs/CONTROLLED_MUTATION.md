# Mutation Safety Qualification

Status: stable v1.0 credential-free release control. Live mutation
qualification remains deferred and is not claimed by the release.

## Purpose

The gate exercises representative state-changing SDK paths without giving the
repository, CI, or ordinary test commands access to provider credentials. It
covers exact typed Cloud, DNS, Security, Console Storage, Robot mutation, and
Robot billable-order reconciliation paths.

All requests use mock transports while retaining the same preparation, permit,
transport, response, cleanup, and reconciliation contracts exposed to
applications. The Robot order checks stop before dispatch and cannot purchase a
server.

## Run The Gate

Run the qualification without credential or destructive-opt-in variables:

```sh
unset CLOUD_SDK_HETZNER_TOKEN_FILE
unset CLOUD_SDK_HETZNER_ROBOT_USERNAME_FILE
unset CLOUD_SDK_HETZNER_ROBOT_PASSWORD_FILE
unset CLOUD_SDK_HETZNER_ALLOW_DESTRUCTIVE
scripts/check_controlled_mutation.sh
```

The script rejects those variables before running exact typed SDK tests. It
does not read a credential file, open a network connection, create a resource,
or write live evidence.

## Release Boundary

Stable releases follow the normal release process: implementation stop,
pentest, complete local release gate, GitHub CI and CodeQL, signed tag, and
explicit crates.io publication. No `security/mutation` attestation is required.

Real mutation tests require provider-specific disposable scope, cost approval,
credential handling, interruption-safe cleanup, and independent inventory
review. Those operational controls are not implemented as repository release
tooling and are deferred to a separately reviewed future milestone. A future
live runner or mandatory attestation would be a new security boundary and must
receive its own threat model, tests, pentest, and release-plan change.

## CI Exclusion

The existing read-only live-smoke artifact and launchers remain incapable of
mutation. No destructive command may infer consent from a token's permission,
reuse the read-only wrapper, accept an empty or generic resource prefix, or run
in CI.
