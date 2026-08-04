# v0.55.0 Public API Review

Date: 2026-08-04

Scope: cumulative public API changes from v0.50.0 through v0.55.0, with a
focused review of the dynamic testkit boundary.

## Cumulative Core Surface

The public checkpoint adds plan-confirm execution permits, fixed-capacity
client workspaces and the provider-generic client kernel, bounded pagination
and action workflow drivers, and finite opt-in diagnostics. Their detailed API
and security decisions are recorded in the v0.51 through v0.54 public API
reviews and their dedicated guides.

Rust 1.92.0 is the new MSRV established by the client-kernel future structure.
The public documentation and CI matrix no longer claim Rust 1.90 or 1.91.

## Dynamic Testkit Surface

`cloud-sdk-testkit 0.29.0` adds:

- `ProviderFixtureBuilder`, `DynamicResponder`, and `DynamicRequest`;
- `DynamicMockTransport` plus finite configuration and execution errors;
- caller-owned `RequestRecordSlot` and payload-free `RecordedRequest`;
- validated `PaginationScript` and `ActionScript`;
- exact source/sink fault injection; and
- non-terminating `StreamPatternSource` inputs.

The transport reuses the existing private sealed response writer. A builder
error or staging failure leaves the successful sequence and record set
unchanged. One atomic in-flight guard rejects overlapping selection instead of
waiting or introducing a lock/runtime dependency.

## Security Review

Dynamic request and transport `Debug` output redacts targets, bodies, headers,
fixtures, and generic errors. Records retain only finite method classification,
target/body lengths, header count, status, and sequence. Extension method tokens
are reduced to `Extension`.

Record capacity is caller-owned, nonzero, clean at construction, and hard
bounded to 1,024 entries. Exhaustion fails closed. Slots have no public reset,
preventing stale evidence from being silently repurposed.

Pagination scripts require an exact complete sequence with stable metadata.
Action scripts require monotonic progress and a single final terminal state.
Custom builders remain available for intentional negative tests.

Stream patterns never terminate themselves and therefore prove core hard
limits. Fault indices are one-based and reject zero. Existing short-write sinks
continue to model partial I/O without allocation.

No API admits network, filesystem, runtime, clock, sleep, automatic retry,
secret storage, or unbounded recording behavior.
