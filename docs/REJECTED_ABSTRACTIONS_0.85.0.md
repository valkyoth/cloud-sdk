# v0.85.0 Rejected Abstractions

Status: implementation stop; pentest required.

## One Generic Boot Request

Rejected because free method, suffix, and field selection could confuse safe
reads, mutations, and destructive installer activation. Fifteen named types
retain exact operation and response association.

## Deprecated Address And Architecture Inputs

Rejected because the official server-IP routes and request `arch` selectors
are deprecated. Only positive server-number paths and active input fields are
constructible.

## Plain Secret Strings

Rejected because generated passwords and keys would remain in ordinary owned
text. Provider secrets use cleanup-owning protected storage with redacted
diagnostics and closure-scoped access.

## Status-Only Mutation Success

Rejected because `200` alone cannot prove the provider applied the intended
family, selector, language, identity, or inactive state. Every success must
decode and match the exact request.

## Automatic Retry

Rejected because delivery ambiguity can duplicate or reverse boot state, and
installer activation can destroy data. All mutations require explicit caller
reconciliation before another attempt.

## Provider-Specific Boot Permit Layer

Rejected for this milestone because the planned contract is request
preparation and exact checked response association, while no current
authenticated boot discovery grants short-lived execution authority. The
provider-neutral operation metadata remains mandatory and any later permit
must source-lock its evidence and authorization semantics rather than copy an
unrelated Robot family's wrapper.
