# Public API Review 0.90.0

Status: implementation stop reached; pentest required.

## Added Surface

`cloud_sdk_hetzner::robot` now exposes all seven active vSwitch request types,
nonzero vSwitch identities, bounded VLANs, protected names, canonical borrowed
server selectors, bounded duplicate-free membership sets, and a non-empty
update-intent enum.

Protected vSwitch names deliberately admit only ASCII letters, digits, spaces,
hyphens, underscores, and periods, with no leading or trailing space. This
prevents invisible Unicode, mixed-script names, and non-ASCII confusables.
Decoded resources use the separate `RobotVSwitchObservedName`: conforming
provider names report high assurance, while bounded nonconforming provider
names are protected and explicitly quarantined. Quarantined names cannot be
reused as outbound names without deliberate revalidation. The observed type is
opaque, so callers cannot fabricate either state or bypass its bound.

With `serde`, it also exposes exact typed prepared/checked associations, strict
owned summary/detail/list and route models, operation-specific provider failure
codes, canonical/exact plan fingerprints, direct and shared mutation permits,
and distinct destructive permits. Raw JSON decoders, form assembly, and strict
field helpers remain crate-private.

## Compatibility

This is additive pre-1.0 API. `cloud-sdk` advances from the v0.85 public
checkpoint to `0.90.0`; `cloud-sdk-hetzner` advances from `0.44.0` to `0.45.0`.
The neutral reqwest and testkit crates receive dependency-only patch releases
to require the v0.90 core without changing behavior. Sanitization remains
unchanged.

## Review Result

The exact operation and request remain part of preparation, response
validation, decoding, and authority. Create responses must match the requested
high-assurance name and VLAN. VLANs use Hetzner's documented `4000..=4091`
range. List and detail responses reject duplicate identities,
noncanonical networks, gateways outside their networks, unknown status text,
and excess collections without dropping the inventory solely because a
bounded provider name falls outside the outbound profile.

Update, cancellation, attach, and detach deliberately decode only the official
empty acknowledgement. They do not expose a misleading resource snapshot.
Callers must perform a later detail read when confirmation matters and must
handle the absence of a provider revision binding between observations.
