# v0.62.0 Threat Model Delta

The neutral freeze adds no new network or credential boundary. It increases
provider response exposure in three areas:

- certificate PEM and managed-certificate errors are attacker-controlled
  provider text and must remain protected and redacted;
- Storage Box lists can be large and structurally amplifying, so they pass
  bounded incremental admission before duplicate-rejecting model decoding;
- all dynamic JSON and typed-model collections reserve fallibly, protected
  strings grow fallibly, secret-bearing models adopt that storage directly,
  quota metadata remains inline, and allocation failure returns a payload-free
  error;
- JSON objects admit at most 256 fields, sort once before duplicate detection,
  and use binary lookup rather than attacker-amplifiable linear scans;
- numbered response arrays are rejected before typed allocation when their
  item count exceeds validated `per_page` metadata;
- DNS and security operations must retain exact service identity so a response
  cannot be decoded under a sibling Cloud API scope;
- Storage management operations require the source-locked bearer token class,
  never Storage Box data-plane Basic credentials.

Controls include source-required fields, bounded arrays/maps/text, finite
coordinates, exact booleans and enums, coherent pagination, multiline secret
validation limited to tab/CR/LF, protected parser strings, and checked response
policy before model use. Remaining resource completion is explicitly deferred
to v0.63.0-v0.67.0 and is not claimed by these slices. Storage decoding still
performs bounded incremental admission before constructing the protected DOM;
the DOM and final typed model can overlap until decoding returns, but every
dynamic allocation on that path is fallible and the complete input remains
capped at 8 MiB. Direct event-to-model decoding remains a later optimization.

Executed read responses can enter typed provider decoding through
`decode_associated_checked_response`. Typed execution returns an
`AssociatedCheckedResponse<O>` that brands the guard with its originating
operation. The decoder consumes that same `O`, retains admitted quota headers,
clears caller response storage after owned decoding, and does not expose
reusable raw response bytes or permit cross-operation response pairing.
