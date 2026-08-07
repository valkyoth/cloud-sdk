# v0.59.0 Public API Review

Date: 2026-08-07

Scope: provider-neutral header cursor decoding and reviewed schema-version
validation.

## Cursor API

`HeaderCursorPolicy` owns an `OperationId`, validated borrowed header names,
and a nonzero page size. It emits request headers inside a closure, marks
continuation cursors sensitive, and clears decimal scratch. `decode_next` reads
bounded raw `ResponseHeaders`, treats next-header absence as terminal, and
transfers a present request-safe cursor into cleanup-owning caller storage.
The returned `HeaderCursorContinuation` retains the decoding policy and emits
only that operation's next request headers; it exposes history observation but
not raw cursor access.

The API does not allocate, expose cursor text outside closure scope, infer
continuation from response bodies, select a digest, or execute transport.
`CursorHistory` remains the exact cycle and collision boundary. New
`PaginationError` variants are static and payload-free.

## Schema API

`SchemaVersion` represents canonical nonzero-major `major.minor` values.
`ReviewedSchemaMajor` binds an admitted major to exact source-lock evidence.
`ValidationSchemaHeader` rejects an unreviewed major and exposes only an
explicit validation encoder with cleanup-owning scratch.

The validation-only type name and method make provider override intent
visible. Account configuration, migration timing, and default schema selection
remain provider and caller responsibilities.

## Compatibility

The additions are provider-neutral and `no_std`. Existing APIs keep their
signatures except exhaustive `PaginationError` matches, which must admit the
three new variants. No default feature or dependency graph changes.
