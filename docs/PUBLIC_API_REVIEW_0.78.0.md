# v0.78.0 Public API Review

Status: remediation complete; final retest required.

Scope: changes from signed v0.77.0 through the v0.78.0 implementation stop.

## Allocation-Gated Requests

With `alloc`, `cloud_sdk_hetzner::robot` adds `RobotServerListRequest`,
`RobotServerGetRequest`, and `RobotServerUpdateRequest`. Canonical paths accept
only a positive, fallibly allocated `RobotServerNumber` whose classified bytes
remain at one stable address; deprecated IPv4 aliases are unavailable. Request
preparation itself remains allocation-free after constructing the identity.
Rename is represented by `RobotServerUpdateIntent::Rename` and a validated
`RobotServerName`, so an empty update cannot be constructed.

All requests implement `PrepareOperation` and return provider-neutral
`PreparedRequest` values bound to the official Robot endpoint, Robot service,
Basic-auth scope, explicit impact and retry metadata, JSON success policy, and
optional-JSON error wire policy. POST uses the new provider-neutral
`ContentType::FORM_URLENCODED` constant.

## Serde Response Models

The existing `serde` feature adds `RobotServerList`, `RobotServer`,
`RobotServerSummary`, `RobotServerCapabilities`, `RobotServerStatus`,
`RobotServerDate`, `RobotServerSubnet`, and `RobotStorageBoxNumber`.
`decode_robot_server_list` and `decode_robot_server` consume only an already
checked success view. Request-owned decode methods consume a
`CheckedResponseGuard`, clear caller storage, and bind detail identity to the
requested server number.

Provider strings use protected closure-scoped access. Operationally sensitive
IDs, dates, addresses, subnets, status, cancellation state, and capabilities
use non-`Copy`, stable-allocation-backed owners with static redacted
diagnostics. Moving an owner transfers allocation metadata rather than copying
classified bytes. Decoder-owned number and Boolean representations also avoid
retained ordinary scalar payloads; Robot decimal path encoding reads the
protected owner directly, while topology/date validation uses bounded
clear-on-drop scratch. Internal protected parsing preserves allocation failure
as the public `RobotServerDecodeError::Allocation` category, and protected
Booleans transfer directly into final storage. Numeric, date, address, and
subnet inspection is closure-scoped; nullable subnets preserve `None`
separately from an empty list.
`linked_storagebox` is optional because the official output table and update
example disagree about its presence.

## Semver And Publication

This is additive pre-1.0 provider API. `cloud-sdk` source advances to v0.78.0;
`cloud-sdk-hetzner` remains package version 0.42.0 while cumulative code waits
for v0.80.0. No package is selected for v0.78 publication.

## Explicit Non-Claims

v0.78 does not add a Robot high-level client, authorization-header encoding,
network execution, automatic retry, cancellation, IP management, or live
mutation. Those remain assigned to later reviewed milestones.
