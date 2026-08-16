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
- Plan fingerprints cover request bytes, cost, currency, account, opaque
  credential lineage, official endpoint, validity, context, replay, attempts,
  and reconciliation identity. Dispatch requires the same credential binding.
- Sensitive form bodies require collision-resistant digest fingerprinting and
  cleanup-owned preparation storage.
- Requests are non-idempotent, may incur cost, and have retry eligibility
  `Never`.
- Each strong digest can mint exactly one direct cost permit. Proven-not-sent
  recovery is distinct from possibly-sent reconciliation. Matching
  transactions fail closed; proof is bound to the exact request instance and
  credential and rechecked with a fresh matching subject and idempotency key.
- Standard-order addon reconciliation compares bounded multisets, not provider
  response order, and preserves catalog quantities.
- Addon creation responses require the exact catalog type and price. Historical
  reconciliation deliberately uses only the request-bound server number and
  product ID: a changed historical type or price still blocks another order,
  avoiding a false negative that could duplicate a charge.
- Addon RIPE reasons and optional subnet gateways are type-checked against the
  exact catalog product type before form preparation.
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

Catalog and transaction reads used for ordering execute through authenticated
observation methods that capture the credential lineage before dispatch and
reject credential rotation before producing `CredentialObserved<T>`. Catalog
plans require matching product and currency observations, authorization is
derived from the exact request, and reconciliation accepts only a matching
credential-bound transaction snapshot. The observation constructor is not
public, so callers cannot attach unrelated credential evidence afterward.

The conservative addon history match can produce a false positive when an
indistinguishable server/product transaction was created independently. That
outcome requires operator review and is preferred to automatically authorizing
a second potentially billable request.

Catalog freshness remains caller-observed. Permit expiry bounds authority lifetime
but does not turn a catalog snapshot into a provider quote. Provider-side
pricing, taxes, billing intervals, manual processing, and availability remain
Hetzner boundaries. Process abort, allocator exhaustion, transport copies, and
caller-owned source cleanup remain covered by the repository threat model.
