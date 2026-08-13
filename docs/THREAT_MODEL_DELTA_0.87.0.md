# Threat Model Delta 0.87.0

Status: implementation stop; pentest required.

## New Inputs

Robot traffic introduces repeated protected target identities, interval text,
dynamic response keys, and provider number tokens. These values may reveal
infrastructure layout and utilization. Request bodies are therefore sensitive,
response models redact values, and caller/transport copies remain outside SDK
cleanup ownership.

## Controls

- Empty, duplicate, excessive, and cross-kind ambiguous targets fail before
  transport. Grouped queries use a tighter target bound derived from parser
  token ceilings.
- Interval text has exact type-specific grammar, component ranges, and ordered
  endpoints. Full calendar validation is intentionally excluded for source
  compatibility; the exact response must still echo the request.
- Read-only POST is admitted only through an explicit core constructor. It
  cannot admit another method or non-read-only metadata and does not weaken
  ordinary POST authorization.
- The response is decoded directly from bounded incremental events. Unknown or
  duplicate fields, shape confusion, hostile chunking, excessive structure,
  negative values, malformed ordinals, noncanonical CIDRs, unrequested targets,
  and response range substitution fail closed.
- Decimal text is bounded and retained exactly. No floating-point conversion
  can turn a large, tiny, or high-precision provider value into another value.

## Residual Boundaries

Robot may omit targets with no data, so absence does not prove zero traffic.
The SDK does not verify `in + out == sum` because the source does not define
rounding guarantees. The provider documents 200 requests/hour; caller policy
must coordinate concurrency and retry. Allocation failure may abort where the
configured allocator does not report failure, consistent with the repository
threat model.
