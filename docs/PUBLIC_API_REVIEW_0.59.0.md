# v0.59.0 Public API Review

Date: 2026-08-07

Scope: provider-neutral header cursor decoding and reviewed schema-version
validation.

## Cursor API

`HeaderCursorPolicy` owns an `OperationId`, validated borrowed header names,
and a nonzero page size. `bind` rejects a `PreparedRequest` with another
operation ID and returns a session retaining the complete request context.
Session execution adds bounded headers, dispatches the retained prepared
request, and decodes only the resulting checked response. A continuation
retains that same prepared request plus the exact normalized endpoint identity
observed on the first dispatch. It exposes execution and history observation,
but no raw cursor or request-header access. Continuation execution rejects a
different transport endpoint before dispatch.

The API does not allocate, expose cursor text, accept a replacement method,
target, provider/service, endpoint policy, authentication scope, operation, or
response policy, infer continuation from response bodies, or select a digest.
Blocking, executor-neutral async, and local async execution share this
boundary. `CursorHistory` remains the exact cycle and collision boundary. New
errors are static and payload-free.

## Schema API

`SchemaVersion` represents canonical nonzero-major `major.minor` values.
`ReviewedSchemaMajor` binds an admitted major to exact source-lock evidence.
`ValidationSchemaHeader` rejects an unreviewed major and exposes only an
explicit validation encoder with cleanup-owning scratch.

The validation-only type name and method make provider override intent
visible. Account configuration, migration timing, and default schema selection
remain provider and caller responsibilities.

## Compatibility

The additions are provider-neutral and `no_std`. The unreleased initial v0.59
raw header methods were replaced by prepared-request-bound execution after
security review. Exhaustive `PaginationError` matches must admit six new
variants. No default feature or dependency graph changes.
