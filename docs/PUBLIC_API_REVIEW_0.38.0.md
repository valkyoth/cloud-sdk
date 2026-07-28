# v0.38.0 Public API Review

Date: 2026-07-28

Scope: mandatory response cleanup, complete checked workspace ownership,
request-identifier policy, retained metadata transfer, and adapter/provider
migration.

## Decision

Core now owns the baseline cleanup guarantee. `ResponseBuffer::new` always
volatile-clears the complete caller storage at admission and drop through
`cloud-sdk-sanitization`. `ResponseStorageSanitizer` remains public only as an
additive platform hook selected with `with_additive_sanitizer`; it cannot
replace either mandatory clear.

The additive hook runs between two mandatory clears. A private final-clear drop
guard covers normal return and unwind from the hook. This design prevents a
no-op or faulty hook from weakening core's guarantee.

## Owned State

`CheckedResponseGuard` owns:

- complete caller response storage;
- bounded response headers and content type;
- protected request-identifier storage;
- fixed decoder scratch;
- cursor staging;
- provider-link staging.

Every byte-bearing owner is non-`Copy` and clears on drop. Response headers and
content type offer deliberately named `retain_copy` methods only where a
transport test or adapter needs a distinct cleanup owner.

`RetainedResponseMetadata` is non-`Copy`, non-`Clone`, redacted, and
closure-accessed. Transfer from protected response metadata clears the source
on success and failure. Any partially initialized destination remains a local
cleanup owner and drops before an error escapes.

## Request-Identifier Policy

Every `OperationMetadata` now selects `RequestIdPolicy::Retain`,
`Protected`, or `Discard`. Policy admission extracts `x-request-id` from
bounded sensitive header storage and either clears it, keeps it under the
checked guard, or permits explicit bounded transfer.

All current Hetzner operations use `Protected`; they expose identifiers only
through guard-scoped closure access. Future operations must make a deliberate
retention decision rather than receiving a permissive default.

## Decoder Contract

`decode_owned_with_workspace` lends mutable scratch only for the decoder call
and drops the full guard before returning its owned result. The existing
`decode_owned` convenience delegates to that path. The Hetzner checked decoder
uses the guard-owned scratch for direct JSON parsing, while independently
decoded provider-error paths create a separate cleanup owner.

## Compatibility

This is an intentional pre-1.0 breaking release:

- `ResponseBuffer::new` takes only storage and body limit;
- additive cleanup uses `with_additive_sanitizer`;
- `OperationMetadata::new` requires request-ID policy;
- `ResponsePolicy::validate` requires request-ID policy;
- response metadata, headers, and content type are not implicitly copied;
- checked decoders may use the new workspace-aware owned decode method.

See [`MIGRATION_0.38.0.md`](MIGRATION_0.38.0.md).

## Exclusions

The API guarantees cleanup ownership, not complete process-memory erasure.
Process abort, leaked guards, `mem::forget`, and copies in TLS, allocators,
kernels, devices, swap, crash dumps, or remote systems remain outside the
boundary. First-party crates contain no unsafe implementation of the primitive;
the reviewed dependency owns that narrow unsafe boundary, so a separate local
Miri claim is not applicable to the volatile implementation itself.
