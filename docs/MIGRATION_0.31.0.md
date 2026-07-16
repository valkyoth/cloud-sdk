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

Provider and action error messages no longer expose an ordinary borrowed
`&str`. Use `HetznerApiError::try_with_message` or
`ActionResultError::try_with_message` so clones continue sharing one protected
allocation:

```rust
let contains_rate_limit = error
    .try_with_message(|message| message.contains("rate"))
    .unwrap_or(false);
```

## Allocation Boundary

The default provider graph remains allocation-free and `no_std`. The optional
`serde` feature now admits serde_json `1.0.150` with default features disabled
and its `alloc` feature enabled for public Serde request/envelope APIs, plus
`cloud-sdk-sanitization` `0.14.0` for owned secret cleanup. Checked response
admission uses a private direct parser and adds no further dependency. It does
not enable `std`.

## Sensitive Responses

Root passwords, console passwords, WebSocket console URLs, API error messages,
action error messages, and zonefiles remain redacted from diagnostics.
Closure-scoped accessors expose the values when needed. Every JSON string value,
including escaped text, is decoded directly into volatile-clearing
`SecretString` storage rather than an ordinary parser scratch allocation.
Strings later discarded by duplicate, trailing-document, required-field, or
model-validation errors are therefore cleared. Sensitive public models move
that existing allocation without another plaintext copy. Cloning a response
shares the protected allocation rather than copying plaintext; the allocation
is cleared after the final clone drops. The SDK does not clear caller-owned
transport response storage; sanitize that complete buffer after the decoded
value is no longer needed.
