# Migrating Source Users To v0.59.0

v0.59.0 is a source-only milestone. The latest crates.io checkpoint remains
v0.55.0; package publication is deferred to v0.60.0.

## Header Cursor Pagination

Provider integrations can construct `HeaderCursorPolicy` from an
`OperationId`, three source-reviewed header names, and one nonzero page size.
Call `bind` with the complete `PreparedRequest`, then execute the returned
`HeaderCursorSession` through `execute_blocking`, `execute_async`, or
`execute_local_async`. The session adds the initial size header, executes the
exact retained request, and decodes only the response produced by that
execution. `HeaderCursorNext::Complete` means the source-defined next header
was absent.

For continuations, call `observe_history` on the returned
`HeaderCursorContinuation`, then execute it through the matching transport
method. The continuation retains the complete prepared request: method,
target, provider/service, endpoint policy, authentication scope (including
account and tenant), operation metadata, and response policy cannot be
replaced. The normalized endpoint identity observed on the first dispatch is
also retained, and continuation execution rejects a transport reporting any
other identity before dispatch. Raw cursor header emission and response-header
decoding are not public. Existing raw response policies must explicitly admit
the provider's next-cursor header.

`PaginationError` adds `InvalidHeaderPolicy`, `InvalidHeaderState`, and
`InsecureHeaderState`, `OperationMismatch`, `RequestHeaderConflict`, and
`EndpointMismatch`. Exhaustive matches must include these payload-free
variants.

## Schema Validation

The new `cloud_sdk::schema` module provides `SchemaVersion`,
`ReviewedSchemaMajor`, and `ValidationSchemaHeader`. Bind the major to the
exact reviewed source digest and construct an override only for a matching
version. Call `with_validation_header` explicitly during provider-defined
validation workflows; do not install it as an automatic production default.

No default feature, allocator, transport, runtime, clock, or third-party
dependency is added.
