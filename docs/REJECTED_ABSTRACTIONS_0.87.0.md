# Rejected Abstractions 0.87.0

Status: implementation stop; pentest required.

## Treat Every POST As A Mutation

Rejected because Robot traffic is semantically read-only. Misclassification
would force unrelated mutation permits and make operation metadata inaccurate.

## Treat Every Read-Only POST As Directly Safe

Rejected because changing method-wide policy would let future provider code
bypass mutation authority accidentally.

## Caller-Declared Read-Only POST Approval

Rejected because a generic public constructor cannot establish the semantics
of a caller-selected provider, endpoint, operation ID, method, or target. The
closed `ApprovedReadOnlyPostQuery` registry validates the complete reviewed
Robot traffic identity and installs an operation ID that later builder calls
cannot replace.

## Full-Tree Traffic Decoding

Rejected because dynamic target and period objects can be large. A direct
incremental visitor avoids a second complete JSON tree while preserving all
global parser ceilings and duplicate-key checks.

## Repeated Linear Target Binding

Rejected because pairwise duplicate checks and per-result scans multiply work
at the 4,092-target bound. Canonical sorting, adjacent duplicate rejection,
binary lookup, and a bounded seen bitmap keep the operation within `O(n log n)`.

## Floating-Point Traffic Values

Rejected because binary floating point cannot preserve provider decimal text.
The SDK stores bounded exact lexemes and leaves numerical interpretation to an
explicit caller choice.

## Gregorian Date Enforcement

Rejected for this source boundary because Hetzner's own month example contains
September 31. The SDK enforces exact grammar, component bounds, and ordering,
then binds the response to the exact request.
