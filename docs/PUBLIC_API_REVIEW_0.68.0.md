# v0.68.0 Public API Review

Status: implementation complete; pentest required.

Scope: changes from signed v0.67.0 through v0.68.0.

## Public API

v0.68 adds the public `ResponseIdentityClass` enum and adds `path_template()`,
`success_root()`, `success_required()`, and `response_identity()` accessors to
`OperationDescriptor`. The sealed
`HetznerOperation` markers remain the public compile-time contract for all 208
active pre-Robot operations.

One additional compile-fail example proves that endpoint and body wrappers
bound to different operation markers cannot be assembled. Existing examples
continue to prove query mismatch, response cross-wiring, and direct mutation
execution failure.

## Review Evidence

`docs/TYPED_OPERATION_BINDINGS.tsv` makes every operation's request, response,
error, and execution policy reviewable in one stable row. It is generated from
the independent API fingerprint, association, request-body, response, and
provider-authentication and response-identity locks. Normal CI cross-checks it
against the Rust AST prepared registries, exact generated marker source, and
all 28 policy values emitted from compiled Rust descriptors and associated
marker labels. Written endpoint paths are validated as `RequestPath` values
and against those descriptor templates before preparation succeeds.

## Compatibility

The public additions are backward compatible. Correct endpoint wire behavior,
default and optional feature graphs, provider response models, and caller code
are unchanged. A mismatched internal path encoder, including one that emits a
raw or encoded query/fragment delimiter, now fails closed.
