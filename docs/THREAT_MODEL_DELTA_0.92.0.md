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
- request preparation is transactional and clears caller storage on failure;
- all six operations are safe read-only `GET` requests with explicit retry
  ownership and no purchase capability.

## Residual Boundaries

The fixed 30-day list is a provider observation, not an append-only audit log,
pagination source, or proof that older or concurrently changing transactions do
not exist. Transaction success is provider-reported state, not independent
infrastructure attestation. Provider text remains untrusted when rendered.
Compiler-created scalar copies, registers, crash dumps, and allocator/process-
abort behavior remain outside best-effort in-process cleanup.
