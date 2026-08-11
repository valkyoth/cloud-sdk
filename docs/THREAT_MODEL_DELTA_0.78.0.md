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
- Strict JSON numbers retain only their capacity-wiped lexical allocation and
  Booleans use stable `SecretBoxBytes`; the parser tree no longer retains an
  ordinary `u64`, `i64`, `f64`, or `bool` payload beside protected text.
- IDs, addresses, subnets, dates, status, cancellation state, and capability
  flags move into non-`Copy`, stable-allocation-backed `SecretBoxBytes` owners
  that volatile-clear their complete allocations on drop. Moving an SDK model
  transfers only allocation metadata and does not relocate classified bytes.
- Robot identity and linked Storage Box decimals copy directly from protected
  lexical storage into stable owners. Address, subnet, and date parsers use
  bounded byte/word scratch that volatile-clears on every success and error;
  request paths copy protected decimal digits without reconstructing an SDK
  scalar.
- Address, subnet, date, and identity inspection is closure-scoped; status and
  capability checks borrow their protected owner. Callers that retain scalar
  copies assume responsibility for clearing or containing those copies.
- Every server model, nested classified value, and request diagnostic is
  static and redacted.
- Duplicate detection sorts only public vector indices and compares protected
  values in place. It creates no copied topology or identity keys, preserves
  provider order, and bounds work to `O(n log n)`.
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
introduced. Robot server operations now require `alloc` for stable classified
storage; the crate remains `no_std`, runtime-free, filesystem-free, clock-free,
and unsafe-free.
