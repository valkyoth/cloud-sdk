# v0.38.0 Public API Review

Date: 2026-07-28

Scope: mandatory response cleanup, complete checked workspace ownership,
request-identifier policy, retained metadata transfer, and adapter/provider
migration.

## Decision

Core now owns the baseline cleanup guarantee. `ResponseBuffer::new` accepts
distinct caller-owned body and header destinations and volatile-clears both at
admission and drop through `cloud-sdk-sanitization`.
`ResponseStorageSanitizer` remains public only as an additive body-storage
platform hook selected with `with_additive_sanitizer`; it cannot replace
either mandatory clear.

The additive hook runs between two mandatory clears. A private final-clear drop
guard covers normal return and unwind from the hook. This design prevents a
no-op or faulty hook from weakening core's guarantee.

## Owned State

`CheckedResponseGuard` owns:

- complete caller response body and header storage;
- bounded response-header ranges and content type;
- a scalar locator for a protected request identifier in stable header storage;
- fixed decoder scratch;
- cursor staging;
- provider-link staging.

Every byte-bearing owner is non-`Copy` and clears on drop. Header and retained
request-ID bytes never live in by-value arrays: they remain in caller-owned
storage while only non-secret pointers, lengths, and ranges may move.
Response headers offer a deliberately named copy operation only where a test
or adapter supplies a distinct cleanup destination. Response content type is
a borrowed validated view over stable header bytes.

`RetainedResponseMetadata` is non-`Copy`, non-`Clone`, redacted, and
closure-accessed. It wraps caller-owned destination storage. Transfer copies
directly from stable protected header storage, then clears the source on
success and failure. Any partially initialized destination remains under its
cleanup owner before an error escapes.

## Request-Identifier Policy

Every `OperationMetadata` now selects `RequestIdPolicy::Retain`,
`Protected`, or `Discard`. Policy admission extracts `x-request-id` from
bounded sensitive header storage and either clears it, keeps it under the
checked guard, or permits explicit bounded transfer.

The same metadata admission runs before both successful response validation
and provider-error decoding. Extraction removes the complete field from the
visible bounded header table without moving its sensitive bytes. Those bytes
remain at their stable caller-owned address until policy discards them,
retention transfers and clears them, or the response guard drops.

All current Hetzner operations use `Protected`; they expose identifiers only
through guard-scoped closure access. Future operations must make a deliberate
retention decision rather than receiving a permissive default.

## Decoder Contract

`decode_owned_with_workspace` lends mutable scratch only for the decoder call
and drops the full guard before returning its owned result. The existing
`decode_owned` convenience delegates to that path. The Hetzner checked decoder
uses the guard-owned scratch for direct JSON parsing, while independently
decoded provider-error paths create a separate cleanup owner.

The strict JSON tree stores object keys in capacity-wiping protected strings.
Both recognized and ignored extension keys clear their complete allocation on
drop.

## Compatibility

This is an intentional pre-1.0 breaking release:

- `ResponseBuffer::new` takes body storage, body limit, and header storage;
- additive cleanup uses `with_additive_sanitizer`;
- prepared blocking and async execution take separate response-header storage;
- `OperationMetadata::new` requires request-ID policy;
- `ResponsePolicy::validate` requires request-ID policy;
- response metadata contains only non-sensitive scalars, headers are not
  implicitly copied, and response content type is borrowed from them;
- retained metadata requires caller-owned destination storage;
- checked decoders may use the new workspace-aware owned decode method.

See [`MIGRATION_0.38.0.md`](MIGRATION_0.38.0.md).

## Exclusions

The API guarantees cleanup ownership, not complete process-memory erasure.
Process abort, leaked guards, `mem::forget`, and copies in TLS, allocators,
kernels, devices, swap, crash dumps, or remote systems remain outside the
boundary. First-party crates contain no unsafe implementation of the primitive;
the reviewed dependency owns that narrow unsafe boundary, so a separate local
Miri claim is not applicable to the volatile implementation itself.
