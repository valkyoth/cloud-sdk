# Threat Model Delta 0.92.0

Status: implementation stop; incremental pentest required.

## New Assets

- standard-server, Server Auction, and addon transaction histories;
- transaction IDs, timestamps, comments, server identities, and SSH metadata;
- ordered product snapshots, exact addon prices, and resulting resources.

## New Untrusted Inputs

Robot JSON can contain oversized arrays or text, unknown states or fields,
invalid timestamps and addresses, contradictory server nullability, duplicate
identities, malformed key shapes, partial price pairs, and detail IDs different
from the requested transaction.

## Controls

- list bodies are capped at 4 MiB and detail bodies at 1 MiB;
- transaction, nested product, addon, and resource arrays are capped at 4,096;
- authorized and host key arrays are capped at 64 and reject duplicate
  fingerprints;
- protected transaction IDs and provider text have redacted diagnostics and
  closure-scoped access;
- strict decoders reject unknown fields, malformed calendars, noncanonical
  addresses, invalid decimals, duplicate identities, and partial hourly pairs;
- `ready` server transactions require both server number and address; non-ready
  states forbid both;
- every detail response retains and verifies the exact admitting request;
- request preparation clears both complete buffers on every reachable
  validation and encoding failure before immutable target binding;
- target and prepared-policy construction failures return typed errors instead
  of relying on panic-only invariant branches;
- all six operations are safe read-only `GET` requests with explicit retry
  ownership and no purchase capability;
- every request exposes the same typed 500-per-hour account quota, and the
  documentation forbids treating it as six separate allowances;
- dedicated fuzzing invokes all six decoders for every input and starts from
  source-locked valid list/detail seeds covering keys, hardware, prices,
  resources, timestamps, and state coherence.

## Residual Boundaries

Stable Rust cannot synchronously reborrow and clear a target in a late error
branch when the success branch returns a request borrowing that target
(`rust-lang/rust#54663`). The final target and policy constructors repeat
unchanged, prevalidated values and their typed errors are currently
unreachable. Callers requiring cleanup under future invariant drift must use
`PreparationStorageGuard` and drop or reuse it promptly; raw
`PreparationStorage` callers remain responsible for clearing their buffers.
Unsafe lifetime emulation was rejected for this low-severity residual boundary.

The fixed 30-day list is a provider observation, not an append-only audit log,
pagination source, or proof that older or concurrently changing transactions do
not exist. Transaction success is provider-reported state, not independent
infrastructure attestation. Quota enforcement remains caller-owned and must be
coordinated with other Robot operations sharing the account. Provider text
remains untrusted when rendered.
Compiler-created scalar copies, registers, crash dumps, and allocator/process-
abort behavior remain outside best-effort in-process cleanup.
