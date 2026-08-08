# v0.64.0 Threat Model Delta

Status: implementation stop reached; pentest required.

## New Surface

Checked decoding now retains exact numeric tokens, complete Cloud actions,
metric series, separate composite action collections, nullable secret-output
states, and unknown provider error-code text.

## Controls

- Every strict-JSON number token is limited to 128 bytes before allocation and
  retains its exact lexical form. Non-finite values fail admission.
- Action and metric timestamps require calendar-valid uppercase UTC RFC 3339.
  Action and referenced resource IDs remain within `1..=2^53-1`, progress is
  within `0..=100`, and every documented nullable field remains explicit.
- Metrics admit at most 512 named series, 16,384 points per series, and 16,384
  points across the response. Existing wire, node, object, depth, duplicate,
  and checked-allocation limits remain mandatory.
- Metric timestamp sign and step positivity are derived from the exact lexical
  token. Binary-float underflow therefore cannot turn a negative timestamp or
  positive nonzero step into accepted or rejected zero.
- Composite action fields are no longer flattened. Secret outputs use protected
  owned storage, operation-specific nullability, and an accessor that
  distinguishes absent from explicit null.
- Unknown error codes remain available for forward compatibility but are
  limited to bounded ASCII alphanumeric, underscore, hyphen, and period
  machine identifiers. They are redacted with commands, timestamps, resource
  types, metric values, and protected messages from `Debug` output.
- Complete metric copies are fallible and reserve each bounded allocation.
  Allocation failure returns `ResponseModelError::Allocation`.
- Exact pinned operation/schema fingerprints, all-operation fixtures, focused
  adversarial tests, named seed-route assertions, and the dedicated checked and
  borrowed response fuzz paths cover the new parser surface.

## Unchanged Boundaries

The default graph remains transport-free and `no_std`. Exact numeric and
timestamp text is operational response data, not automatically secret-erasing
storage; applications requiring erasure must move selected values into a
protected type. Allocation ceilings remain per response rather than a
process-wide memory quota.
