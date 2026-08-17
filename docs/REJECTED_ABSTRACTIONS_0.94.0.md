# Rejected Abstractions 0.94.0

Status: implementation stop; incremental pentest required.

## Custom Robot Endpoint

A generic endpoint constructor was rejected because it could send real Basic
credentials to an attacker-controlled host. The client accepts only the exact
official Robot endpoint. Provider-neutral custom transports remain separate
from this facade.

## Universal Mutation Permit

Converting every Robot mutation to one generic permit was rejected. Existing
operation-specific reconciliation and billable-order evidence must remain
non-erasable. The generic family is sealed to nine server and boot mutations
that had no stronger permit contract.

## Automatic Retry Or Authentication Replay

Client-owned retry was rejected. Robot authentication rejection can lock a
source IP, and mutations can have indeterminate delivery. A `401` closes the
credential generation; retry requires caller-owned policy plus explicit new or
reconfirmed credentials.

## Synthetic Pager And Action APIs

Robot bounded lists and transaction snapshots do not expose Cloud-style
pagination, and Robot mutations do not expose Cloud action resources. Inventing
those abstractions would overstate provider guarantees. The client performs one
typed call while existing explicit reconciliation workflows retain multi-step
authority.

## Untyped Success Callback

Returning raw response bytes from the facade was rejected. Provider callbacks
receive the cleanup-owning checked guard and must return request-bound typed
success or typed provider failure.

## Unsafe Borrow Extraction

Unsafe lifetime extension was rejected. The guarded provider callback keeps
borrowed outputs inside the client execution lifetime while preserving
`#![forbid(unsafe_code)]`.
