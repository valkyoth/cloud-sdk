# Controlled Mutation Qualification

Status: v0.100.0 operator protocol. Offline qualification is automated; live
qualification requires an explicitly approved disposable Hetzner scope and is
never run by CI.

## Purpose

This protocol qualifies representative state-changing SDK paths without
turning the repository, CI, or ordinary test commands into mutation launchers.
It covers:

| Scenario | Service | Live behavior |
| --- | --- | --- |
| placement-group create/delete | Cloud | required, one attempt |
| zone create/delete | DNS | required, one attempt |
| SSH-key create/delete | Security | required, one attempt |
| snapshot create/delete | Console Storage | required, one attempt |
| server rename/restore | Robot | required, one attempt |
| server-order preparation/reconciliation | Robot cost boundary | preparation only; dispatch forbidden |

The cost scenario proves price binding, spending-ceiling enforcement, permit
binding, and uncertain-delivery reconciliation without requiring a real server
purchase. A release qualification must not place a Robot order.

The read-only live-smoke harness remains separate. Its launchers reject every
destructive opt-in and cannot be reused for this protocol.

## Offline Gate

Run the credential-free branch-complete qualification from the repository:

```sh
unset CLOUD_SDK_HETZNER_TOKEN_FILE
unset CLOUD_SDK_HETZNER_ROBOT_USERNAME_FILE
unset CLOUD_SDK_HETZNER_ROBOT_PASSWORD_FILE
unset CLOUD_SDK_HETZNER_ALLOW_DESTRUCTIVE
scripts/check_controlled_mutation.sh
```

The gate runs exact typed SDK tests for Cloud, DNS, Security, Console Storage,
Robot mutation, and Robot order reconciliation. Every request uses a mock
provider and the same preparation, permit, transport, response, cleanup, and
reconciliation contracts exposed to applications. The evidence validator has
no networking module, subprocess use, credential input, or environment access.

## Live Preconditions

Do not begin unless every item below is true:

1. The source commit is clean, reviewed, and built without credentials.
2. The project/account is disposable and contains no production resource.
3. A separate operator explicitly approves the exact v0.100 protocol.
4. A unique run ID matches `cloud-sdk-live-v0-100-[a-z0-9-]{8,40}`.
5. Every created resource starts with the run ID plus `-`.
6. Current provider prices, quotas, regions, and account limits were reviewed.
7. An explicit EUR minor-unit spending ceiling is recorded before dispatch.
8. Cloud and Robot credentials are short-lived, least-privilege, and created
   or mounted only after all code and dependencies have stopped building.
9. Retry is disabled. Each scenario receives exactly one dispatch attempt.
10. A second person or independently controlled role is available to verify
    cleanup and final inventories.

The SDK cannot prove that an account is disposable, that a credential has the
intended provider-side scope, or that no production resource exists. Those are
operator responsibilities and must be checked in the provider control plane.

## Execution Rules

Use only official endpoint constructors. Do not accept endpoint, resource,
price, region, or operation selection from tenant-controlled input. Build each
typed operation, review the complete canonical plan, bind the matching permit,
and dispatch once.

For each live scenario:

1. Record a SHA-256 fingerprint of the reviewed canonical plan.
2. Record a SHA-256 reference for the created resource. Do not store the raw
   provider resource ID, credential, secret, response body, or private key in
   evidence.
3. Classify delivery exactly as `response-started` or `possibly-sent`.
4. Reconcile provider state before cleanup. A `possibly-sent` result never
   permits blind replay.
5. Construct and review a distinct cleanup plan and record its fingerprint.
6. Remove or restore the resource, then mark the ledger entry
   `confirmed-removed` only after a provider read confirms the result.

On interruption, timeout, transport error, decode failure, or ambiguous
response, stop forward progress. Reconcile that attempt, finish its cleanup,
and verify inventory before considering another scenario. Never infer
`not-sent` from a generic transport error.

The Robot order scenario is different: prepare the exact billable operation,
observe and bind its current catalog price and spending ceiling, exercise its
permit and reconciliation logic offline, and record `not-sent`,
`withheld-by-policy`, `confirmed-not-applied`, and `not-created`. Do not send
the order.

## Cleanup Ledger

Use one evidence document for the entire run. Its `cleanup_ledger` contains one
entry for each of the five live resources. Resource references must be unique,
must match their scenarios, and must have a distinct cleanup-plan fingerprint.

After all cleanup, the independent reviewer lists resources matching the run
prefix in Cloud, DNS, Security, Console Storage, and Robot. Every
`prefixed_resources` value must be zero and every entry must identify the same
independent cleanup reviewer. Then revoke or rotate all credentials and inspect
the account billing view.

## Evidence Format

The committed release evidence path is:

```text
security/mutation/v0.100.0.json
```

It must be a no-follow regular file containing ASCII JSON no larger than 65,536
bytes and use exactly the fields defined by
[`controlled-mutation-policy.json`](../controlled-mutation-policy.json) and
`scripts/check-controlled-mutation.py`. Validate it with:

```sh
scripts/check-controlled-mutation.py security/mutation/v0.100.0.json
```

The validator rejects duplicate JSON fields at every nesting level, unknown
fields, boolean integer aliases, missing or duplicated scenarios, multiple
attempts, unbound permits, unresolved delivery, incomplete cleanup, reused
resource references, cost drift, CI execution, non-disposable scope, unrevoked
credentials, a non-independent reviewer, and non-empty final inventories.
Diagnostics are static and do not reproduce evidence values. The release gate
validates the exact committed regular-file blob, not the worktree path or a
symlink target, and requires every source control to be a regular Git blob.

This evidence is an operational attestation, not cryptographic proof of remote
provider events and not an independent certification. The validator proves
schema and cross-field consistency. Maintainers must review the provider
control plane, billing view, operator record, and cleanup review independently.
Do not commit screenshots, credentials, provider IDs, response bodies, or
other account-sensitive material.

## CI Exclusion

CI runs only `scripts/check_controlled_mutation.sh`, which rejects all known
Cloud/Robot credential and destructive-opt-in variables before invoking
credential-free mock tests. The checked evidence validator cannot dispatch a
request. The existing read-only root-owned live launchers continue to reject
destructive mode.

Any future executable live mutation runner is a new security boundary. It must
receive its own pentest, credential-free build and sealing procedure, exact
operation allowlist, static credential handling, interruption-safe cleanup,
and explicit CI exclusion before it can replace this manual protocol.
