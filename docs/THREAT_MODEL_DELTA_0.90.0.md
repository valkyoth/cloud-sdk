# Threat Model Delta 0.90.0

Status: release candidate; pentest and final retest passed.

## New Inputs

Robot vSwitch operations introduce provider IDs, protected names, VLAN IDs,
server-number or IP membership selectors, cancellation schedules, server
status, subnet routes, cloud-network routes, and account-wide inventory.

## Controls

- IDs are nonzero, VLANs are restricted to Hetzner's documented
  `4000..=4091`, outbound names use a conservative ASCII profile, and member
  selectors require canonical number or
  allocation-free canonical IP text.
- Bounded provider-observed names outside the outbound profile are protected
  and explicitly quarantined. They cannot be reused as trusted outbound names
  without deliberate revalidation.
- Membership requests are non-empty, bounded, and duplicate-free. Exact
  repeated `server[]` fields are encoded only after complete preflight.
- Update intent cannot be empty. Every sensitive form and complete preparation
  buffer is cleared on failure and on guard drop.
- Strict response decoding rejects unknown, missing, duplicate, or extra
  fields; unknown statuses; malformed canonical addresses; host-bit routes;
  out-of-network gateways; duplicate identities; and excess collections.
- Creation checks the exact requested protected name and VLAN and rejects
  contradictory cancellation or non-empty membership state.
- Mutations are never automatically retryable. Exact request-bound permits
  separate mutation from destructive authority and preserve delivery phase.
- Empty acknowledgements are never promoted into confirmed state. Callers must
  read current detail after successful or uncertain mutations when state
  matters.
- Immutable source evidence binds all seven active operation rows and official
  examples; mutation tests and a dedicated response fuzzer exercise the
  complete admitted response boundary.

## Residual Boundaries

Caller-owned source strings and transport copies remain caller-owned. Robot
supplies no revision or ETag that atomically binds a mutation acknowledgement
to a later detail read. The SDK detects identity and schema contradictions but
cannot prevent another actor from changing provider state between requests.

VLAN admission follows Hetzner's documented `4000..=4091` range; Robot remains
authoritative for account-specific availability and uniqueness.
The SDK cannot prove switching behavior or membership convergence inside the
provider network. Callers must reconcile after uncertain delivery before
repeating a state change.
