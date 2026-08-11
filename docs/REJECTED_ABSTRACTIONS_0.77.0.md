# v0.77.0 Rejected Abstractions

Status: implementation complete; pentest required.

## Unknown-Code Transient Fallback

An unknown code may represent authentication rejection or a newly destructive
provider state. Mapping it to transient would permit unsafe retries. Unknown
status and code values remain hard decoder errors until source-lock review.

## One Generic Robot Error String

A message-only error loses authentication lockout, quota interval, maintenance,
and invalid-input semantics while encouraging payload logging. Public types
retain finite classifications and closure-scope protected text instead.

## Automatic Maintenance Or Quota Retry

Provider delay metadata cannot authorize another attempt. Maintenance and
quota expose caller-policy dispositions; operation metadata, delivery state,
budgets, and explicit policy must still permit retry.

## Parsing Directly With serde_json

The existing strict parser rejects duplicate keys and applies repository-wide
tree bounds while storing strings in protected allocations. A second permissive
Robot parser would weaken those guarantees and duplicate security logic.

## Heap-Boxing The Quota Variant

The neutral `QuotaBucket` has large bounded inline extension storage. Storing
it directly made every failure value unnecessarily large; boxing introduced an
infallible allocation. `RobotQuota` instead stores two validated integers and
creates the neutral bucket fallibly on request.

## Building A Robot Client Early

No active Robot endpoint request model exists before v0.78. Decoding protocol
errors alone is insufficient for safe executable client claims.
