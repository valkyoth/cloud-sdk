# v0.86.0 Public API Review

Status: implementation stop; pentest required.

Scope: incremental public changes after signed and published v0.85.0.

## Robot Reverse-DNS Requests

`cloud_sdk_hetzner::robot` adds named list, get, set, update-or-create, and
delete request types. Every request fixes the official Robot endpoint, service
identity, Basic scope, method, operation ID, quota, response policy, impact,
idempotency, and retry eligibility.

`RobotIpAddress` supplies canonical protected IPv4/IPv6 identity.
`RobotRdnsName` owns a lowercase DNS name capped at 253 bytes with labels
capped at 63 bytes. Names reject empty labels, a trailing dot, controls,
non-ASCII text, and noncanonical uppercase spelling.

Set and update require a request-bound `RobotRdnsMutationPermit`. Delete
requires a `RobotRdnsDestructivePermit`. Their shared variants retain the exact
request fingerprint and expire before dispatch rather than authorizing a
resource class generally.

## Preparation And Decoding

Each request exposes provider-bound preparation with atomic target/query/form
encoding and failure cleanup. The optional list filter accepts only a
canonical main-server IPv4 address and uses `server_ip`; set and update use the
exact `ptr` form field.

With `serde`, typed prepared and checked wrappers retain exact request
provenance. `RobotRdns` and `RobotRdnsList` expose bounded results. Decoding
rejects unknown or duplicate fields, noncanonical addresses, invalid names,
duplicate list identities, oversized lists, wrong content types, wrong status,
and cross-request identity substitution. Set and update acknowledgements must
echo the exact requested IP and PTR. Delete requires the documented empty
`200` response.

Raw response decoders remain crate-private so callers cannot discard the typed
request provenance. An unfiltered list decodes directly through its checked
wrapper. Because a filtered Robot response does not echo the requested server
association, empty filtered responses remain unverifiable. Non-empty filtered
decoding requires a strictly decoded `RobotIpList`; every returned address must
be assigned to the exact filtered server in that inventory. The result is the
distinct `RobotRdnsFilteredMembership` type, which proves membership only and
cannot be confused with the potentially empty authoritative `RobotRdnsList`.
Verification indexes the bounded inventory once and uses binary search rather
than a cross-product scan.

`RobotRdnsFailureCode` narrows only the source-locked failures admitted for the
specific reverse-DNS operation and status.

## Explicit Non-Claims

The SDK does not perform network I/O by default, resolve DNS, prove address
ownership, verify that public resolvers observe a new PTR, retry uncertain
mutations, accept tenant-controlled endpoints, or provide a high-level Robot
client. Lowercase host syntax is a canonical SDK input policy, not a claim that
the provider validates DNS ownership or reachability.
