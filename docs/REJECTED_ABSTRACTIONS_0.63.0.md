# v0.63.0 Rejected Abstractions

Status: release candidate; pentest and final retest passed.

## One Generic Resource Model

A single `Resource { id, name, status }` loses required nested data and lets
different Cloud families appear interchangeable. v0.63 uses dedicated wrapper
types and an exact `CloudResourceKind` instead.

## Closed Enums For Provider Strings

Rejecting every unknown status or category would turn additive upstream enum
values into availability failures. The generated evidence records known values,
while checked models retain bounded future strings after validating type and
text safety.

## Generated Rust Types From Untrusted Live Input

Build-time or release-time code generation from an unreviewed network response
would weaken reproducibility and source review. The repository commits a small
generated field table and fixture file, authenticates the exact specification
digest, and compares deterministic regeneration during the drift gate. Runtime
Rust code remains reviewed source.

## Untyped Serde Values In Public Results

Exposing `serde_json::Value` would tie the provider API to Serde's owned value
model, hide allocation policy, and permit integer coercion mistakes. The public
field tree uses crate-owned bounded value types and fallible conversion.

## Infallible Recursive Cloning

Deriving `Clone` for an accepted 8 MiB, 65,536-node response tree would make a
later application copy capable of aborting on allocation failure. Complete
Cloud trees expose explicit `try_clone` methods that reserve every allocation
fallibly and return `ResponseModelError::Allocation`.

## Payload-Bearing Diagnostics

Resource metadata is not necessarily a credential, but names, labels,
addresses, topology, placement, billing values, and future fields can still be
operationally sensitive. `Debug` therefore reports only bounded shape and
resource kind; complete values require explicit accessors.
