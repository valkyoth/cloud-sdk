# v0.77.0 Public API Review

Status: implementation complete; pentest required.

Scope: changes from signed v0.76.0 through the v0.77.0 implementation stop.

## Hetzner Robot Additions

With the existing `serde` feature, `cloud_sdk_hetzner::robot` adds:

- `decode_robot_failure`, which consumes an admitted `TransportResponse` and
  caller-owned `ResponseDecodeWorkspace`;
- `RobotFailure` and finite `RobotFailureCategory` variants for authentication,
  invalid input, quota, maintenance, provider, and explicit transport failure;
- `RobotRetryDisposition`, with authentication, invalid input, and provider
  errors fixed to `Never`;
- `RobotInvalidInput`, `RobotQuota`, and `RobotProviderError`, whose provider
  text is available only through closure-scoped protected access;
- `RobotTransientTransport`, created only from `DeliveryClassified`; and
- payload-free `RobotDecodeError` diagnostics and public aggregate bounds.

`RobotQuota::quota_bucket` returns an exhausted provider-neutral
`robot-global` bucket with the source-locked maximum and relative interval.
It does not allocate and returns the neutral `QuotaError` if that fixed model
ever becomes incoherent.

## Fail-Closed Shape

The response decoder cannot construct a transient transport failure from
provider bytes. It admits only bodyless 401 and 503 responses plus exact JSON
400, 403, and 404 envelopes with source-locked codes. Unknown status, code,
field, duplicate key, status mismatch, content type, or bound failure returns
`RobotDecodeError`.

## Semver And Publication

This is a pre-1.0 additive provider API. `cloud-sdk` source advances to
v0.77.0 without a neutral API change. `cloud-sdk-hetzner` remains package
version 0.42.0 while code accumulates for v0.80.0. No package is selected for
v0.77 publication.

## Explicit Non-Claims

v0.77 does not prepare a Robot endpoint operation, encode Basic authorization,
send a request, close a credential generation automatically, retry, paginate,
or prove live behavior. Server operations begin in v0.78. Live evidence never
intentionally submits invalid credentials.
