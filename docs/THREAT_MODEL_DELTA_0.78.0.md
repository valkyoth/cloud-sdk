# v0.78.0 Threat Model Delta

Status: implementation complete; pentest required.

## New Boundary

v0.78 prepares the first Robot endpoint-family requests and admits bounded
server success responses. Server inventory, names, addresses, topology,
capabilities, billing dates, and cancellation state may be operationally
sensitive.

## Threats And Controls

### Credential Destination And Request Confusion

- Every request binds the exact HTTPS Robot endpoint and Robot service scope.
- Only positive server numbers enter paths; deprecated caller-supplied IP path
  aliases are absent.
- Rename intent is explicit and body serialization fixes the sole field name.
- Method, impact, idempotency, retry eligibility, cost intent, response shape,
  and operation ID are complete before transport execution.
- Preparation clears target and body storage before writing and again on every
  reachable validation or capacity error.

### Hostile Or Contradictory Responses

- Checked success policy requires 200, JSON, a nonempty body, and an 8 MiB
  ceiling before model decoding.
- The strict parser rejects duplicate keys, malformed UTF-8/JSON, excess depth,
  nodes, containers, fields, strings, and numbers.
- List, summary, and detail field sets are exact; only the documented
  `linked_storagebox` source inconsistency is optional.
- IDs are positive; status strings are finite; dates are calendar-valid.
- Address and subnet lists are bounded and duplicate-free. Subnet prefixes
  match their address family and host bits must be clear.
- The main IPv4 address must occur in the assigned single-address list.
- Detail decoding rejects a server number different from the request.

### Data Lifetime And Diagnostics

- Provider text moves from protected parser strings into `SensitiveText`
  without an ordinary unprotected owned copy.
- IDs, addresses, subnets, dates, status, cancellation state, and capability
  flags move into non-`Copy`, byte-backed owners that volatile-clear on drop.
- Address, subnet, date, and identity inspection is closure-scoped; status and
  capability checks borrow their protected owner. Callers that retain scalar
  copies assume responsibility for clearing or containing those copies.
- Every server model, nested classified value, and request diagnostic is
  static and redacted.
- Duplicate detection sorts a cleanup-owning identity scratch allocation once,
  preserving provider order while bounding work to `O(n log n)`.
- Request-owned decode methods consume the checked guard so response and
  decoder workspace storage is cleared before returning the owned model.
- A direct checked-response fuzz target covers list and detail decoders,
  source bounds, duplicate tails, mixed families, prefix edges, invalid dates,
  and identity mismatch.

## Residual Boundaries

The official update example omits `linked_storagebox` while its output table
lists the field. Missing and zero values therefore map to `None`; this choice is
source-locked and must be revisited on upstream drift. Server names are public
request bodies but may still be sensitive to callers, who remain responsible
for clearing any borrowed source storage they classify as confidential.

No authorization header, network request, retry loop, or live mutation is
introduced. Default crates remain `no_std`, runtime-free, filesystem-free,
clock-free, and unsafe-free.
