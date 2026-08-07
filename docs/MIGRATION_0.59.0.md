# Migrating Source Users To v0.59.0

v0.59.0 is a source-only milestone. The latest crates.io checkpoint remains
v0.55.0; package publication is deferred to v0.60.0.

## Header Cursor Pagination

Provider integrations can construct `HeaderCursorPolicy` from an
`OperationId`, three source-reviewed header names, and one nonzero page size.
Use `with_initial_request_headers` for the first request. After transport has
retained the admitted next-cursor response header as sensitive metadata, call
`decode_next`; `HeaderCursorNext::Complete` means the source-defined next
header was absent.

For continuations, call `observe_history` on the returned
`HeaderCursorContinuation`, then use its `with_request_headers` method. The
continuation retains the operation and decoding policy, so the cursor cannot
be safely rebound to another operation or header policy. The cursor is never
returned as a plain public string. Existing raw response policies must
explicitly admit the provider's next-cursor header.

`PaginationError` adds `InvalidHeaderPolicy`, `InvalidHeaderState`, and
`InsecureHeaderState`. Exhaustive matches must include these payload-free
variants.

## Schema Validation

The new `cloud_sdk::schema` module provides `SchemaVersion`,
`ReviewedSchemaMajor`, and `ValidationSchemaHeader`. Bind the major to the
exact reviewed source digest and construct an override only for a matching
version. Call `with_validation_header` explicitly during provider-defined
validation workflows; do not install it as an automatic production default.

No default feature, allocator, transport, runtime, clock, or third-party
dependency is added.
