# v0.65.0 Threat Model Delta

Status: release candidate; pentest and final retest passed.

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
- DNS zone names, nameserver text, labels, RRSet identifiers and names, open RR
  types, record values, and comments acquire cleanup ownership immediately when
  copied during fallible parsing. Private guards clear temporary and partial
  allocations on every error path, then transfer ownership without copying to
  final models that clear on drop. Caller-created copies remain the caller's
  cleanup responsibility.
- Returned TSIG keys must be bounded canonical padded standard Base64. Legacy
  HMAC-MD5 and HMAC-SHA1 values are observable for existing configurations but
  remain unavailable to outbound request constructors, which admit only
  HMAC-SHA256.
- Zone IDs, TTLs, ports, nameserver addresses, list sizes, RR values/comments,
  RRSet IDs, and record collections are bounded before public model creation.
- Zone record counts are limited to a conservative 1,000,000-record operational
  envelope. Hetzner currently documents a default 500-record limit that may be
  raised on request.
- Primary zones cannot retain transfer primaries; secondary zones require at
  least one semantically unique primary IP; and TSIG key/algorithm fields must
  occur together. RRSet records must be nonempty and unique by provider
  identity value.
- TSIG-bearing primary-nameserver, zone, DNS-resource, composite-success, and
  checked-success aggregates do not expose ordinary equality.
- Unknown uppercase RR types are retained as unclassified response text and
  never promoted to a source-known request type.
- Lists and zonefiles receive incremental syntax/limit admission before the
  protected duplicate-rejecting full model parser.

## Residual Boundaries

The provider specification omits maxima for several DNS arrays and strings;
the SDK therefore applies documented conservative limits. The published
[Hetzner DNS limits](https://docs.hetzner.com/networking/dns/overview/#limits)
can change and selected limits can be raised by Hetzner; deployments exceeding
the local record-count envelope require a reviewed SDK update. Live provider
behavior, DNS correctness, TSIG entropy/rotation, zone delegation, billable
effects, and caller cleanup after transport remain operational responsibilities.
