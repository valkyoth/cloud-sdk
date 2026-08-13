# Rejected Abstractions 0.87.0

Status: implementation stop; pentest required.

## Treat Every POST As A Mutation

Rejected because Robot traffic is semantically read-only. Misclassification
would force unrelated mutation permits and make operation metadata inaccurate.

## Treat Every Read-Only POST As Directly Safe

Rejected because changing method-wide policy would let future provider code
bypass mutation authority accidentally. The explicit constructor is narrow and
testable.

## Full-Tree Traffic Decoding

Rejected because dynamic target and period objects can be large. A direct
incremental visitor avoids a second complete JSON tree while preserving all
global parser ceilings and duplicate-key checks.

## Floating-Point Traffic Values

Rejected because binary floating point cannot preserve provider decimal text.
The SDK stores bounded exact lexemes and leaves numerical interpretation to an
explicit caller choice.

## Gregorian Date Enforcement

Rejected for this source boundary because Hetzner's own month example contains
September 31. The SDK enforces exact grammar, component bounds, and ordering,
then binds the response to the exact request.
