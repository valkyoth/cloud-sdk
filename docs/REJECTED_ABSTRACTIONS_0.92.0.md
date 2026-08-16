# Rejected Abstractions 0.92.0

Status: implementation stop; incremental pentest required.

## Invented Pagination

Rejected because Robot documents one fixed 30-day list response and exposes no
cursor, offset, page, or continuation token. A list wrapper describes only that
snapshot and makes no older-history or completeness claim.

## One Universal Transaction Model

Rejected because standard servers, Server Auction orders, and per-server addon
orders have different product shapes, server-result semantics, exact prices,
and resources. Shared protected values retain consistency without erasing
provider distinctions.

## Shape-Free Detail Decoding

Rejected because a caller could decode one valid transaction under another
request identity. Prepared and checked types retain the exact request and fail
closed on ID substitution.

## Success From State Alone

Rejected for server transactions. `ready` without both resulting server number
and address is contradictory, while a non-ready state carrying either identity
is also rejected.

## Automatic Retry

Rejected even for these reads. Caller policy must bound request volume under
Robot's documented 500-request/hour quota.
