# v0.77.0 Threat Model Delta

Status: implementation complete; pentest required.

## New Boundary

v0.77 admits bounded Robot error response bytes after transport commitment and
classifies source-locked authentication, invalid-input, quota, provider, and
maintenance behavior. Provider error text and form-field names may contain
sensitive account context.

## Threats And Controls

### Authentication Retry And Lockout

- HTTP 401 is a bodyless nominal `AuthenticationRejected` variant.
- Its retry disposition is always `Never` and automatic retry is unavailable.
- Unknown statuses and codes return decoder errors rather than a generic
  transient category.
- Only a separate `DeliveryClassified` adapter failure can construct
  `TransientTransport`.
- Credential-generation rejection remains an explicit caller transition
  against the exact v0.76 attempt; decoding alone does not mutate credentials.

### Malformed Or Hostile Provider Data

- Bodyful errors require admitted JSON and are limited to 64 KiB before parse.
- The strict parser bounds depth, nodes, fields, arrays, strings, and numbers
  and rejects duplicate keys, invalid UTF-8, invalid syntax, and trailing data.
- Exact envelopes reject unknown fields, wrong types, bad nullability, and
  HTTP-to-JSON status mismatch.
- Invalid-input field arrays retain at most 256 entries, each at most 1,024
  bytes; messages retain at most 16,384 bytes.
- Quota maximum and interval must be nonzero unsigned integers.
- Local parser allocation failure remains distinct from malformed provider
  data so resource exhaustion cannot be misreported as hostile input.

### Data Lifetime And Diagnostics

- Parsed strings use protected allocations and clear complete capacity on
  drop. Admitted messages and input names move into `SensitiveText` without a
  second unprotected string copy.
- Debug output redacts text and reports only finite enums, counts, and public
  numeric quota metadata.
- Display and error chains contain only static payload-free text.
- Caller response storage remains owned and cleared by the existing response
  buffer boundary.

## Unchanged Boundaries

Default crates remain `no_std`, allocation-free, transport-free, runtime-free,
filesystem-free, clock-free, and unsafe-free. The decoder is opt-in under the
existing provider `serde` feature. No Robot network execution is added.
