# v0.65.0 Rejected Abstractions

Status: implementation complete; incremental pentest required.

## Generic DNS Resources

Keeping zones and RRSets as generic ID/name/status records was rejected because
it discarded delegation, protection, TTL, record, and secret-bearing fields and
made checked decoding overstate practical support.

## Unfiltered Field Trees

Reusing `CloudObject` for a complete zone was rejected because it would copy a
returned `tsig_key` into an ordinary `String`. DNS receives dedicated typed
models so secret fields can move directly into protected storage.

## Closed Response RR Types

Reusing only the request-side `RrsetType` enum was rejected. A provider-added
record type is safe to observe but not safe to assign existing request
semantics, so `DnsRrsetType` preserves bounded raw text and returns `None` from
`known` until the source lock is reviewed.

## Implicit Legacy TSIG Support

Expanding outbound constructors to HMAC-MD5 or HMAC-SHA1 was rejected. Response
models must describe existing provider state, while creation and mutation keep
the stronger HMAC-SHA256 policy.

## New Base64 Dependency

Adding a decoder solely for TSIG response syntax was rejected. Bounded
canonical representation checks are small, allocation-free, and already match
the request-side policy without expanding the provider dependency graph.
