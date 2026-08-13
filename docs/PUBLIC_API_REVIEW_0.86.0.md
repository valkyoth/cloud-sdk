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

Set and update require request-bound `AuthorizedRobotRdnsMutation` evidence.
Delete requires `AuthorizedRobotRdnsDestructive` evidence. Direct and shared
permits retain the exact request fingerprint and expire before dispatch rather
than authorizing a resource class generally.

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

`RobotRdnsFailureCode` narrows only the source-locked failures admitted for the
specific reverse-DNS operation and status.

## Explicit Non-Claims

The SDK does not perform network I/O by default, resolve DNS, prove address
ownership, verify that public resolvers observe a new PTR, retry uncertain
mutations, accept tenant-controlled endpoints, or provide a high-level Robot
client. Lowercase host syntax is a canonical SDK input policy, not a claim that
the provider validates DNS ownership or reachability.
