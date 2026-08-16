# Threat Model Delta 0.93.0

Status: implementation stop; incremental pentest required.

## New Assets And Threats

v0.93 introduces authority capable of creating billable Robot resources.
Protected assets now include the exact catalog price observation, spending
ceiling, account identity, request body, replay budget, reconciliation
identity, and uncertain-delivery state.

Primary threats are stale or substituted catalog intent, arithmetic overflow,
price/currency/account confusion, unauthorized replay, retry after an
indeterminate send, response substitution, sensitive body retention, and an
accidental real purchase from CI.

## Controls

- Exact bounded catalog decimals are converted to scale-4 integers. Gross
  recurring and setup amounts and standard-addon quantities are checked for
  overflow and against an explicit ceiling.
- Plan fingerprints cover request bytes, cost, currency, account, official
  endpoint, validity, context, replay, attempts, and reconciliation identity.
- Sensitive form bodies require collision-resistant digest fingerprinting and
  cleanup-owned preparation storage.
- Requests are non-idempotent, may incur cost, and have retry eligibility
  `Never`.
- Proven-not-sent recovery is distinct from possibly-sent reconciliation.
  Matching transactions fail closed; proof is bound to the exact request
  instance and rechecked with a fresh matching subject and idempotency key.
- Strict `201` decoding verifies the observable product, selector, location,
  addon, and server identity before returning a transaction.
- The source checker scans GitHub workflows and live-smoke entry points and
  rejects every billable Robot route.

## Residual Boundaries

Robot transaction lists cover only the preceding 30 days and carry no
revision, causal token, price, account, or request idempotency key. Callers must
fetch reconciliation evidence after the uncertain attempt, account for
provider eventual consistency, prevent concurrent indistinguishable orders,
and escalate ambiguity to an operator. The SDK deliberately permits false
positive matching because false negative matching could duplicate a charge.

Catalog freshness is caller-observed. Permit expiry bounds authority lifetime
but does not turn a catalog snapshot into a provider quote. Provider-side
pricing, taxes, billing intervals, manual processing, and availability remain
Hetzner boundaries. Process abort, allocator exhaustion, transport copies, and
caller-owned source cleanup remain covered by the repository threat model.
