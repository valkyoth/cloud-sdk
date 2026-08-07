# v0.62.0 Rejected Abstractions

## Provider Enum In Core

Rejected. Provider and service identities remain marker-owned and extensible;
adding Hetzner response types to `cloud-sdk` would freeze provider knowledge
into the neutral crate.

## One Generic Resource Map

Rejected. Identity-only fallback remains useful for the uncompleted operation
set, but the freeze slices use source-complete typed fields. Untyped maps would
hide required-field, nullability, enum, and secret-ownership mistakes.

## A Second Response Decoder

Rejected. Selected models enter through the existing `CheckedResponseGuard`,
quota/error policy, duplicate-rejecting protected tree, and operation binding.
Storage adds incremental admission before that same path rather than bypassing
it with a separate parser contract.

## Provider-Specific Neutral Hooks

Rejected. Neither OVHcloud, Robot, nor the Hetzner slices justify new core
callbacks, enum variants, authentication exceptions, or executor traits.

## Unprotected PEM Or Zonefile Strings

Rejected. Both outputs can contain operationally sensitive data and multiline
content. They remain in cleanup-owning storage with closure-scoped access and
redacted diagnostics.
