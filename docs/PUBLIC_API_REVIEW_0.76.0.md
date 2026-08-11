# v0.76.0 Public API Review

Status: implementation complete; pentest required.

Scope: changes from signed v0.75.0 through the v0.76.0 implementation stop.

## Provider-Neutral Addition

`cloud_sdk::authentication` adds a caller-owned lockout lifecycle:

- `SharedCredentialAttemptState` stores one atomic current generation and open
  or rejected state without allocation, `std`, a clock, or an executor;
- `CredentialAttempt<'a>` proves which exact state owner and open generation
  began an execution without an allocated identifier;
- with `alloc`, `OwnedCredentialAttemptState` and
  `OwnedCredentialAttempt` retain the same opaque owner identity across task
  boundaries without borrowing a credential owner or allocating per attempt;
- `CredentialAttemptGeneration` is bounded, nonzero, monotonic, and never
  wraps;
- `CredentialReconfirmation` makes unchanged-credential reuse an explicit
  caller action; and
- `CredentialAttemptError` and `CredentialAttemptStatus` expose payload-free
  state and transition results.

Concurrent attempts may share one open generation. Authentication rejection
idempotently closes that generation. Stale attempts cannot close replacement
credentials, attempts from another state fail with `ForeignState` even when
their generations match, reconfirmation is rejected while a generation
remains open, and exhaustion cannot wrap into an older identity. Neither
borrowed nor owned attempts implement `Hash`, so owner addresses are not
exposed to caller-supplied hashers.

## Hetzner Robot Addition

The provider owns `ROBOT_SERVICE_ID`, `RobotService`,
`ROBOT_API_BASE_URL`, and an exact official Robot endpoint policy. With the
existing `alloc` feature, `cloud_sdk_hetzner::robot` also exposes:

- `RobotCredentials`, a non-`Clone` protected username/password owner;
- `RobotCredentialScope`, fixed to Hetzner, Robot, and
  `https://robot-ws.your-server.de/`;
- `RobotCredentialAttempt`, `RobotCredentialError`,
  `RobotCredentialStateError`, and `RobotCredentialRotationError`; and
- mutable-byte and cleanup-guard ingestion and rotation methods.

Secret text is available only inside `try_with_attempt`. Its closure cannot
return a borrow, and the issuing owner plus generation are revalidated
immediately before access. Robot attempts own an opaque shared lineage, can
move into owned tasks, and do not prevent credential rotation while a request
is outstanding. There is no unrestricted `as_str`, username, password,
authorization-header, or conversion API.

## Semver And Publication

This is a pre-1.0 additive API. `cloud-sdk` source advances to v0.76.0.
`cloud-sdk-hetzner` remains package version 0.42.0 while code accumulates for
the v0.80 public checkpoint. No crate is selected for v0.76 publication.

## Explicit Non-Claims

v0.76 does not encode Basic authorization, create a Robot request, classify a
401 response, retry, paginate, poll, send network traffic, or prove live
credentials. Typed Robot protocol errors begin in v0.77 and resource
operations begin in v0.78. Live evidence never intentionally submits invalid
credentials.
