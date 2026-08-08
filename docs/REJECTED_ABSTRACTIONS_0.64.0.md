# v0.64.0 Rejected Abstractions

Status: release candidate; pentest and final retest passed.

## Binary Floating-Point Public Results

Converting provider number tokens to `f64` loses decimal identity and can alter
timestamps or sampling steps. v0.64 retains a bounded exact token while still
using finite numeric checks during admission.

## Flattened Composite Actions

Combining `action`, `actions`, and `next_actions` makes source semantics and
workflow ordering ambiguous. The public result now preserves each field.

## Null As Absence

Skipping nullable secret fields prevents callers from distinguishing a missing
field from an explicit provider null. v0.64 records null independently without
allocating or exposing a fake empty secret.

## Closed Error-Code Rejection

Rejecting a future provider error code would turn additive evolution into an
availability failure. Collapsing it to `Unknown` alone loses remediation data.
The SDK retains bounded exact text while keeping diagnostics redacted.

## Infallible Complete-Metrics Clone

One response can contain thousands of points. An infallible recursive clone
could abort on allocation failure, so complete metrics expose only `try_clone`.
