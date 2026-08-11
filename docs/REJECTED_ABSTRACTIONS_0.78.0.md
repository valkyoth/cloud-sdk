# v0.78.0 Rejected Abstractions

Status: implementation complete; pentest required.

## Reusing Cloud JSON Preparation

Rejected because Robot uses another origin, HTTP Basic authentication, form
bodies, bodyless error statuses, and different service identity. Sharing that
assembler would risk bearer/Basic and JSON/form policy confusion.

## Server ID Or IP Union

Rejected because the IP path aliases are explicitly deprecated. The public
path identity is a positive `RobotServerNumber`, so legacy behavior cannot
re-enter through a convenience enum or string reference.

## Optional Update Fields

Rejected because `Option<RobotServerName>` would admit an empty update and
defer required-field failure to runtime. `RobotServerUpdateIntent::Rename`
makes the only current mutation explicit and extensible.

## Unchecked Success Decoder

Rejected because callers could bypass status, media-type, size, and cleanup
policy. Public success functions require `CheckedResponse`, and ergonomic
request methods consume `CheckedResponseGuard`.

## Flattening Nullable Subnets

Rejected because JSON `null` and `[]` are distinct provider states.
`Option<Vec<RobotServerSubnet>>` preserves that distinction.

## Copyable Topology Models

Rejected because server inventory is classified as operationally sensitive.
IDs, addresses, subnets, dates, states, cancellation state, and capabilities
use non-`Copy`, stable-allocation-backed owners instead of inline scalar or
array fields. Moving an SDK model therefore transfers allocation metadata
without creating an abandoned classified inline copy. Public inspection is
scoped and diagnostics are static.

## Copied Duplicate Keys

Rejected because fixed-array identity keys recreate classified topology in
ordinary sortable memory. Duplicate detection sorts public collection indices
and compares stable protected values in place instead.

## Permissive Future Fields And Statuses

Rejected for the first Robot endpoint family. Unknown fields and enum strings
stop decoding until the source lock and model are reviewed together.

## High-Level Robot Client

Rejected for this milestone because credentials, endpoint-family operations,
client execution, and live evidence have separate review stops. Client
integration remains assigned to v0.94.0.
