# Public API Review 0.89.0

Status: implementation stop; pentest required.

## Added Surface

`cloud_sdk_hetzner::robot` now exposes all eight active Robot firewall and
firewall-template request types, bounded ordered rules, canonical IPv4
selectors and port ranges, exact protocol/action/status values, protected
template names, non-zero template IDs, typed prepared and checked associations,
strict owned firewall/template/list models, operation-specific failures, and
request-bound mutation/destructive permits.

Inline replacement and template application are distinct public enum variants.
Raw response decoders, strict JSON helpers, and form assembly remain
crate-private. Returned names, addresses, selectors, ports, and TCP flags use
protected owned storage with redacted diagnostics and closure-scoped access.

## Compatibility

This is additive pre-1.0 API. `cloud-sdk` advances to `0.89.0` for source and
tag identity. `cloud-sdk-hetzner` remains `0.44.0` until the v0.90 cumulative
publication checkpoint. Existing neutral transport, authentication, client,
retry, and permit behavior is unchanged.

## Review Result

The exact operation type remains part of preparation, response validation,
decoding, and execution authority. Read, mutation, and destructive requests
cannot exchange checked responses or permits. Rule order remains observable
and is never normalized by sorting.
