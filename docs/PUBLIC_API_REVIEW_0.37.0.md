# v0.37.0 Public API Review

Date: 2026-07-28

Scope: response-buffer admission, transport commitment, checked response
lifetimes, adapters, testkit, and Hetzner checked decoding.

## Decision

The transport no longer returns a caller-constructible response containing an
arbitrary body slice. Core creates a `ResponseBuffer` over the complete caller
storage, clears it before use, and lends a sealed `ResponseWriter` for only the
operation-admitted prefix. A transport can write that prefix and commit status,
bounded metadata, and initialized length exactly once.

`TransportResponse` remains a public read-only view because adapters, provider
decoders, test helpers, and callers need common inspection. Its fields and
constructor are private. Core creates it only from writer commitment and the
same admitted storage.
The external-caller regression fixture supplies every field and must fail with
the private-field diagnostic, not a missing-field diagnostic.

## Ownership And Lifetimes

`ResponseBuffer` owns the sanitizer reference and writer. The writer does not
contain the sanitizer. This allows an asynchronous transport future to borrow
only the writer while preserving sequential transports that intentionally are
not `Sync`.

`ResponsePolicy::validate` consumes the buffer and returns
`CheckedResponseGuard`. Borrowed checked decoding is available only through a
higher-ranked closure. `decode_owned` evaluates the decoder, drops the guard,
and then returns the owned result. Compile-fail tests prove neither raw nor
checked response bytes can escape these closures.

## Failure Semantics

- An initialized length above the admitted prefix is rejected before commit.
- A second commit and mutable access after commit are rejected.
- Adapters reject a precommitted writer before request transmission.
- Testkit rejects a precommitted writer without consuming an exchange.
- A transport that returns success without commit fails response policy.
- Endpoint, transport, policy, and decode errors drop the cleanup owner.
- `ResponseWriterError`, reqwest `TransportError::ResponseCommitFailed`,
  testkit `MockError::ResponseWriterRejected`, and Hetzner
  `HetznerDecodeError::ResponseWriter` have static payload-free diagnostics.

The writer does not attempt to infer how many bytes a failing transport wrote.
Dropping `ResponseBuffer` sanitizes the complete supplied storage.

## Compatibility

This is an intentional pre-1.0 breaking release:

- blocking and async `send` return `Result<(), E>` and accept
  `&mut ResponseWriter`;
- `TransportResponse` public constructors and builder methods are removed;
- response metadata is committed through `ResponseMetadata`;
- response policy and prepared execution consume `ResponseBuffer`;
- prepared execution returns `CheckedResponseGuard`;
- Hetzner checked decoding consumes `ResponseBuffer`.

All provider, reqwest, testkit, examples, live smoke, and fuzz call sites are
migrated together. See [`MIGRATION_0.37.0.md`](MIGRATION_0.37.0.md).

## Deferred Work

v0.37 uses the existing explicit `ResponseStorageSanitizer` contract. v0.38
adds the single audited non-elidable core cleanup primitive, complete retained
sensitive metadata ownership, atomic cleanup-owning transfer, and precise
platform lifecycle evidence. v0.37 does not claim complete process-allocation
cleanup or immunity to `mem::forget`, process abort, or external copies.
