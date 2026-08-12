# v0.81.0 Public API Review

Status: implementation stop; pentest required.

Scope: changes from signed v0.80.0 through the v0.81.0 implementation stop.

## Allocation-Gated Requests

`cloud_sdk_hetzner::robot` adds six named subnet requests for list, get,
traffic update, MAC get, explicit MAC assignment, and default-MAC restoration.
They reuse `RobotSubnetAddress`, `RobotIpAddress`, and `RobotMacAddress`; no
arbitrary method, route, endpoint, query key, or form key is exposed.

List filtering requires an IPv4 server main address. Traffic updates are
nonempty typed partial updates. MAC assignment requires an explicit canonical
MAC and serializes it as a sensitive bounded form. Every request binds the
official Robot service, origin, Basic scope, operation ID, response policy,
impact, semantics, and retry classification.

## Serde Models And Association

The `serde` feature adds `RobotSubnet`, `RobotSubnetList`, `RobotSubnetMac`,
`RobotSubnetMacOption`, `PreparedRobotSubnet`, and `CheckedRobotSubnet`.
Models expose protected identities through closure-scoped access, retain
nullable server assignment, and expose derived network and IPv4 broadcast
values without changing the provider route identity.

Decoding rejects unknown fields, duplicate or oversized lists, noncanonical
addresses or MACs, invalid family masks, cross-family or off-network gateways,
invalid server identities, empty or oversized MAC maps, a current MAC absent
from its advertised map, response identity mismatch, and mutation outcome
conflicts. It deliberately admits host-bits-set route identities because the
official source demonstrates them.

Traffic updates, MAC assignment, and MAC restoration use request-bound direct
or shared permits. Sensitive bodies require strong-digest fingerprints. PUT
and DELETE deny automatic retry; uncertain delivery requires reconciliation.

## Semver And Publication

This is additive pre-1.0 provider API. `cloud-sdk` source advances to v0.81.0;
`cloud-sdk-hetzner` remains package version 0.43.0 while cumulative code waits
for v0.85.0. No package is selected for v0.81 publication.

## Explicit Non-Claims

v0.81 does not add a Robot high-level client, live mutation, automatic retry,
or undocumented subnet fields. Reset operations remain assigned to v0.82.0.
