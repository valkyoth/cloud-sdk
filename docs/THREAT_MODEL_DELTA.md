# Threat Model Delta Digest

This document consolidates the pre-1.0 `THREAT_MODEL_DELTA_*` review series. The current checkpoint detail remains here; earlier complete snapshots remain available from their signed Git tags and repository history.

Add future release sections here instead of creating another version-named file. Current policies live in the focused documents linked from each release note and in the release roadmap.

## v0.62.0

**Status:** historical reviewed snapshot  
**Topics:** release-specific review evidence  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.62.0/docs/THREAT_MODEL_DELTA_0.62.0.md)

## v0.63.0

**Status:** release candidate; pentest and final retest passed.  
**Topics:** New Surface; Controls; Unchanged Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.63.0/docs/THREAT_MODEL_DELTA_0.63.0.md)

## v0.64.0

**Status:** release candidate; pentest and final retest passed.  
**Topics:** New Surface; Controls; Unchanged Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.64.0/docs/THREAT_MODEL_DELTA_0.64.0.md)

## v0.65.0

**Status:** release candidate; pentest and final retest passed.  
**Topics:** New Assets; Controls; Residual Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.65.0/docs/THREAT_MODEL_DELTA_0.65.0.md)

## v0.66.0

**Status:** release candidate; pentest and final retest passed.  
**Topics:** New Surface; Controls; Unchanged Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.66.0/docs/THREAT_MODEL_DELTA_0.66.0.md)

## v0.67.0

**Status:** release candidate; pentest and retest passed.  
**Topics:** New Surface; Controls; Unchanged Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.67.0/docs/THREAT_MODEL_DELTA_0.67.0.md)

## v0.68.0

**Status:** implementation complete; pentest required.  
**Topics:** Assurance Gap Closed; Controls; Unchanged Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.68.0/docs/THREAT_MODEL_DELTA_0.68.0.md)

## v0.69.0

**Status:** release candidate; pentest and final retest passed.  
**Topics:** New Boundary; Threats And Controls; # Credential Exfiltration Through Endpoint Confusion; # Cross-Service Operation Confusion; # Mutation Or Billing Bypass; # Buffer Residue And Capacity Confusion; # Unbounded Concurrency Or Hidden Runtime Policy; Unchanged Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.69.0/docs/THREAT_MODEL_DELTA_0.69.0.md)

## v0.70.0

**Status:** release candidate; pentest and final retest passed.  
**Topics:** New Boundary; Threats And Controls; # Classification Drift; # Mutation Or Billing Bypass; # Credential Destination Confusion; # Executor Or Policy Divergence; # Buffer Residue And Cancellation; Unchanged Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.70.0/docs/THREAT_MODEL_DELTA_0.70.0.md)

## v0.71.0

**Status:** release candidate; pentest and final retest passed.  
**Topics:** New Boundary; Threats And Controls; # Classification Drift; # Unauthorized DNS Mutation; # Credential Destination Confusion; # TSIG And Zonefile Exposure; # Cancellation Residue; # Misleading FIPS Claims; Unchanged Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.71.0/docs/THREAT_MODEL_DELTA_0.71.0.md)

## v0.72.0

**Status:** release candidate; pentest and final retest passed.  
**Topics:** New Boundary; Threats And Controls; # Classification Drift; # Unauthorized Key Or Certificate Mutation; # Private-Key Disclosure; # Other Sensitive Request Bodies; # Unsafe Rotation Ordering; # Credential Destination And Cancellation; Unchanged Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.72.0/docs/THREAT_MODEL_DELTA_0.72.0.md)

## v0.73.0

**Status:** release candidate; pentest passed with no findings.  
**Topics:** New Boundary; Threats And Controls; # Classification Drift; # Unauthorized Cost, Mutation, Or Destruction; # Password Disclosure; # Large Or Malformed Responses; # Credential Destination; Unchanged Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.73.0/docs/THREAT_MODEL_DELTA_0.73.0.md)

## v0.74.0

**Status:** release candidate; pentest and final retest passed.  
**Topics:** New Boundary; Threats And Controls; # Incomplete Or Reordered Inventory; # Deprecated Endpoint Revival; # Credential Lockout; # Untrusted Upstream Bytes; # Protocol Ambiguity; Unchanged Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.74.0/docs/THREAT_MODEL_DELTA_0.74.0.md)

## v0.75.0

**Status:** release candidate; pentest and final retest passed.  
**Topics:** New Boundary; Threats And Controls; # Ambiguous Form Encoding; # Partial Or Oversized Output; # Secret Tail Retention; # Capability Overstatement; Unchanged Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.75.0/docs/THREAT_MODEL_DELTA_0.75.0.md)

## v0.76.0

**Status:** release candidate; pentest and final retest passed.  
**Topics:** New Boundary; Threats And Controls; # Credential Cross-Use; # Repeated Rejected Credentials; # Secret Lifetime And Diagnostics; # Live Lockout Testing; Unchanged Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.76.0/docs/THREAT_MODEL_DELTA_0.76.0.md)

## v0.77.0

**Status:** implementation complete; pentest required.  
**Topics:** New Boundary; Threats And Controls; # Authentication Retry And Lockout; # Malformed Or Hostile Provider Data; # Data Lifetime And Diagnostics; Unchanged Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.77.0/docs/THREAT_MODEL_DELTA_0.77.0.md)

## v0.78.0

**Status:** release candidate; pentest and final retest passed.  
**Topics:** New Boundary; Threats And Controls; # Credential Destination And Request Confusion; # Hostile Or Contradictory Responses; # Data Lifetime And Diagnostics; Residual Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.78.0/docs/THREAT_MODEL_DELTA_0.78.0.md)

## v0.79.0

**Status:** release candidate; pentest and final retest passed.  
**Topics:** New Boundary; Threats And Controls; # Wrong Target Or Unintended Destruction; # Hostile Or Contradictory Responses; # Data Lifetime; Residual Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.79.0/docs/THREAT_MODEL_DELTA_0.79.0.md)

## v0.80.0

**Status:** release candidate; pentest and final retest passed.  
**Topics:** New Boundary; Threats And Controls; # Wrong Target Or Mutation; # Hostile Or Contradictory Responses; # Data Lifetime; Residual Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.80.0/docs/THREAT_MODEL_DELTA_0.80.0.md)

## v0.81.0

**Status:** implementation stop; pentest required.  
**Topics:** New Boundary; Threats And Controls; # Wrong Target Or Mutation; # Hostile Or Contradictory Responses; # Data Lifetime; Residual Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.81.0/docs/THREAT_MODEL_DELTA_0.81.0.md)

## v0.82.0

**Status:** implementation stop; pentest required.  
**Topics:** New Boundary; Threats And Controls; # Unauthorized Or Wrong Reset; # Hostile Or Contradictory Responses; # Data Lifetime; Residual Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.82.0/docs/THREAT_MODEL_DELTA_0.82.0.md)

## v0.83.0

**Status:** implementation stop; pentest required.  
**Topics:** New Boundary; Threats And Controls; # Wrong Or Widened Route; # Unauthorized Or Replayed Mutation; # Hostile Or Contradictory Responses; # Data Lifetime; Residual Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.83.0/docs/THREAT_MODEL_DELTA_0.83.0.md)

## v0.84.0

**Status:** release review complete; pentest and final retest passed.  
**Topics:** New Boundary; Threats And Controls; # Wrong Server Or Unsupported Capability; # Stale Or Replayed Authorization; # Hostile Responses And Failure Widening; # Data Lifetime; Residual Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.84.0/docs/THREAT_MODEL_DELTA_0.84.0.md)

## v0.85.0

**Status:** implementation stop; pentest required.  
**Topics:** New Boundary; Threats And Controls; # Wrong Server Or Configuration; # Duplicate Or Ambiguous Mutation; # Hostile Provider Data; # Secret Lifetime; Residual Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.85.0/docs/THREAT_MODEL_DELTA_0.85.0.md)

## v0.86.0

**Status:** implementation stop; pentest required.  
**Topics:** New Boundary; Threats And Controls; # Wrong Address Or PTR; # Duplicate Or Ambiguous Mutation; # Hostile Provider Data; # Credential And Endpoint Scope; Residual Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.86.0/docs/THREAT_MODEL_DELTA_0.86.0.md)

## v0.87.0

**Status:** implementation stop; pentest required.  
**Topics:** New Inputs; Controls; Residual Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.87.0/docs/THREAT_MODEL_DELTA_0.87.0.md)

## v0.88.0

**Status:** implementation stop; pentest required.  
**Topics:** New Inputs; Controls; Residual Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.88.0/docs/THREAT_MODEL_DELTA_0.88.0.md)

## v0.89.0

**Status:** release candidate; pentest and final retest passed.  
**Topics:** New Inputs; Controls; Residual Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.89.0/docs/THREAT_MODEL_DELTA_0.89.0.md)

## v0.90.0

**Status:** release candidate; pentest and final retest passed.  
**Topics:** New Inputs; Controls; Residual Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.90.0/docs/THREAT_MODEL_DELTA_0.90.0.md)

## v0.91.0

**Status:** release candidate; pentest and final retest passed.  
**Topics:** New Assets; New Untrusted Inputs; Controls; Residual Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.91.0/docs/THREAT_MODEL_DELTA_0.91.0.md)

## v0.92.0

**Status:** implementation stop; incremental pentest required.  
**Topics:** New Assets; New Untrusted Inputs; Controls; Residual Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.92.0/docs/THREAT_MODEL_DELTA_0.92.0.md)

## v0.93.0

**Status:** implementation stop; incremental pentest required.  
**Topics:** New Assets And Threats; Controls; Residual Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.93.0/docs/THREAT_MODEL_DELTA_0.93.0.md)

## v0.94.0

**Status:** implementation stop; incremental pentest required.  
**Topics:** New Assets And Threats; Controls; Residual Boundaries  
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.94.0/docs/THREAT_MODEL_DELTA_0.94.0.md)

## v0.95.0


Status: release candidate; pentest and final retest passed.

## New Assets And Threats

v0.95 introduces operator-held Robot username and password files, a staged
live-test executable, two installed launchers, a root-owned manifest, and one
authenticated read-only network request. New threats include credential
exposure to build tooling or CI, launcher or artifact substitution, mixed
Cloud/Robot credentials, same-file aliases, filesystem races, secret retention,
custom endpoint exfiltration, invalid-login lockout, accidental mutation or
ordering, response/resource disclosure, and hidden retries.

## Controls

- Cargo staging rejects all Cloud and Robot credential variables and requires
  a clean reviewed commit. Credentials are provisioned only after the build
  environment exits and the bundle is installed in root-owned non-writable
  paths.
- Manifest format 3 binds the executable, isolated runner, Cloud launcher,
  Robot launcher, and reviewed commit. The runner validates ownership, modes,
  link count, directory trust, and already-open descriptor hashes before
  descriptor execution.
- The Robot launcher has one fixed mode. The runner clears inherited state,
  rejects additional arguments, destructive opt-in, bearer tokens, incomplete
  or mixed credentials, and selects exactly `read_only_robot_server_smoke`.
- Credential files must be separate private regular files in owner-only parent
  directories. Unix opens are descriptor-based with no-follow semantics and
  require effective-user ownership, one link, and owner-only permissions;
  non-Unix live loading fails closed. Oversized or empty values are rejected,
  and both complete source allocations clear on every return.
- Basic authorization is scoped to Hetzner, Robot, and the exact official HTTPS
  endpoint. `RobotClient::official` verifies that destination again.
- The only live request is bodyless `GET /server`. A compiled exact-match
  transport test exercises the shared live execution function and rejects any
  method, target, body, header, endpoint, or dispatch-count change. Static
  source checks remain secondary tripwires for mutation, permits, orders,
  transactions, custom endpoints, and workflow execution. No invalid
  credential or automatic retry is intentionally sent.
- Request, response, header, authorization, and credential storage stays
  bounded and cleanup-owned. Output and errors remain static and payload-free.

## Residual Boundaries

The SDK cannot prove provider-side Robot permissions or credential validity.
Even a read-only executable may hold credentials capable of mutations in other
software. Operators must use the narrowest separate Webservice account
available, verify credentials before the one run without intentionally causing
failed logins, monitor Robot security state, and revoke or rotate both values
afterward.

Root ownership is a local operational trust anchor, not reproducible signed
binary provenance. Filesystem caches, shell input, OS and TLS copies, crash
tooling, swap, process abort, allocator exhaustion, remote logging, provider
availability, and account billing remain operational boundaries. Live success
is point-in-time evidence and does not replace source drift, mocks, fuzzing,
platform qualification, or the controlled-mutation plan.

