# Migrating To v0.31.0

`cloud-sdk` `0.31.0` and `cloud-sdk-hetzner` `0.24.0` add checked Hetzner
response decoding. Existing preparation and transport APIs remain source
compatible.

## Prepared Operation Identifiers

Hetzner-prepared requests now carry a validated static `OperationId`. Custom
provider integrations that construct `PreparedRequest` directly can opt in
with `PreparedRequest::with_operation_id`. The unchanged constructor leaves the
identifier absent so provider-neutral callers are not forced to invent one.

## Checked Decoding

Enable `cloud-sdk-hetzner/serde` and pass the exact `PreparedRequest` plus the
transport's `TransportResponse` to `serde::decode_response`. The decoder:

- verifies the Hetzner service and source-locked operation binding;
- enforces the prepared success status, content type, body shape, and size;
- rejects duplicate keys, malformed JSON, excessive nesting, and oversized
  containers before model conversion;
- returns `HetznerSuccess` for successful responses or a redacted typed
  `HetznerApiError` for provider error statuses.

Resource success values expose validated common identity and state fields in
this release. Provider-complete field models remain planned before `1.0.0`.

Direct serde_json decoding is no longer the documented integration path
because it cannot prove that response bytes passed the operation's prepared
policy.

## Allocation Boundary

The default provider graph remains allocation-free and `no_std`. The optional
`serde` feature now admits serde_json `1.0.150` with default features disabled
and its `alloc` feature enabled, plus `cloud-sdk-sanitization` `0.14.0` for
owned secret cleanup. It does not enable `std`.

## Sensitive Responses

Root passwords, console passwords, WebSocket console URLs, API error messages,
and zonefiles remain redacted from diagnostics. Closure-scoped accessors expose
the values when needed. Every JSON string value enters volatile-clearing
`SecretString` storage during parsing, including strings later discarded by
duplicate, trailing-document, required-field, or model-validation errors.
Composite secrets and zonefiles move that existing allocation into the public
model without another plaintext copy. Cloning a response shares the protected
allocation rather than copying plaintext; the allocation is cleared after the
final clone drops. The SDK does not clear caller-owned transport response
storage; sanitize that complete buffer after the decoded value is no longer
needed.
