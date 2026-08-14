# Rejected Abstractions 0.91.0

Status: implementation stop; pentest required.

## Floating-Point Prices

Rejected because binary floating point cannot preserve Robot's exact decimal
wire values and can make approval comparisons ambiguous. Prices retain exact
protected text plus a bounded integer coefficient and scale.

## Executable Catalog Plans

Rejected because catalog reads do not provide durable price or availability
authority. Plan types intentionally cannot prepare a transport request. The
billable v0.93 operation must introduce separately reviewed, expiring, request-
bound cost permits and current-price confirmation.

## Generic Untyped Catalog JSON

Rejected because callers would need to remember field sets, list limits,
decimal rules, identity checks, and net/gross coherence. Each operation has a
typed prepared/checked association and a strict bounded model.

## One Universal Product Model

Rejected because standard servers, Server Auction products, and per-server
addons have materially different identities, fields, price shapes, and
selection semantics. Sharing exact value and price primitives retains useful
consistency without erasing provider distinctions.

## Automatic Retry

Rejected as a default even though these operations are reads. Retry ownership
remains explicit so callers bound request volume and account for Robot's
documented 500-request/hour quota.
