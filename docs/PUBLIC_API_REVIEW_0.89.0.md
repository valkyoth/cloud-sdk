# Public API Review 0.89.0

Status: implementation stop; pentest required.

## Added Surface

`cloud_sdk_hetzner::robot` now exposes all eight active Robot firewall and
firewall-template request types, bounded ordered rules, canonical IPv4
selectors and port ranges, exact protocol/action/status values, protected
template names, non-zero template IDs, typed prepared and checked associations,
strict owned firewall/template/list models, operation-specific failures, and
request-bound mutation/destructive permits.

The reviewed remediation adds complete closure-scoped destination/source port
and TCP-flag access, template summary policy flags, protected exact comparison
helpers, `RobotFirewallTemplateReconciliation`, and
`RobotFirewallTemplateMutationOutcome`. Its unresolved variant owns a
`PendingRobotFirewallTemplate`, and `into_confirmed()` returns that pending
state instead of erasing it. The pending type retains the exact typed mutation
configuration; callers cannot supply replacement intent during confirmation.
Pending confirmation consumes the state together with the matching
name-bearing list summary and verifies identity, protected name, summary flags,
detailed flags, and ordered rules. Detailed template names are optional because
the official create/get/update examples omit the field despite the output table
listing it.

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
and is never normalized by sorting. Missing-protocol port rules remain
representable as documented by Robot, while an explicitly incompatible
protocol remains rejected. Template mutations cannot claim full confirmation
when Robot omits the protected name.
The list and detail observations have no provider revision binding, so callers
must exclude concurrent template mutation or repeat reconciliation after a
possible race.
