# v0.68.0 Public API Review

Status: implementation complete; pentest required.

Scope: changes from signed v0.67.0 through v0.68.0.

## Public API

v0.68 adds no runtime type, trait, method, feature, or dependency. The existing
sealed `HetznerOperation` markers remain the public compile-time contract for
all 208 active pre-Robot operations.

One additional compile-fail example proves that endpoint and body wrappers
bound to different operation markers cannot be assembled. Existing examples
continue to prove query mismatch, response cross-wiring, and direct mutation
execution failure.

## Review Evidence

`docs/TYPED_OPERATION_BINDINGS.tsv` makes every operation's request, response,
error, and execution policy reviewable in one stable row. It is generated from
the independent API fingerprint, association, request-body, response, and
provider-authentication locks. Normal CI then cross-checks it against the Rust
AST prepared registries and exact generated marker source.

## Compatibility

The change is additive evidence. Default and optional feature graphs, wire
behavior, provider response models, and caller code are unchanged.
