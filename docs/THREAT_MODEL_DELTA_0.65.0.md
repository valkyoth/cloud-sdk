# v0.65.0 Threat Model Delta

Status: implementation complete; incremental pentest required.

## New Assets

- DNS zone topology, delegation state, record values, comments, and zonefiles.
- TSIG keys returned for secondary-zone transfer configuration.
- Provider-owned RR type strings that may expand before the local source lock.

## Controls

- The generated schema lock now validates every current zone and RRSet field,
  required marker, type, nullability, numeric bound, item bound, string bound,
  format, and known enum set. Root union branches must share the reviewed shape.
- TSIG keys move from protected parser storage into `SensitiveText` without an
  ordinary heap-string copy, use closure-scoped access, clear on drop, and stay
  redacted from all DNS diagnostics.
- Returned TSIG keys must be bounded canonical padded standard Base64. Legacy
  HMAC-MD5 and HMAC-SHA1 values are observable for existing configurations but
  remain unavailable to outbound request constructors, which admit only
  HMAC-SHA256.
- Zone IDs, TTLs, ports, nameserver addresses, list sizes, RR values/comments,
  RRSet IDs, and record collections are bounded before public model creation.
- Primary zones cannot silently retain transfer primaries. RRSet records must
  be nonempty and unique by provider identity value.
- Unknown uppercase RR types are retained as unclassified response text and
  never promoted to a source-known request type.
- Lists and zonefiles receive incremental syntax/limit admission before the
  protected duplicate-rejecting full model parser.

## Residual Boundaries

The provider specification omits maxima for several DNS arrays and strings;
the SDK therefore applies documented conservative limits. Live provider
behavior, DNS correctness, TSIG entropy/rotation, zone delegation, billable
effects, and caller cleanup after transport remain operational responsibilities.
