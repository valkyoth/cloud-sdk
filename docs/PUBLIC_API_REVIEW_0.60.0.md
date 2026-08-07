# v0.60.0 Public API Review

Status: implementation review and pentest complete; release gate required.

Scope: cumulative public API changes from published v0.55.0 through v0.60.0.

## Added Surface

`cloud-sdk 0.60.0` publishes the provider-neutral v0.58 and v0.59 contracts
described in their migration guides and adds the `async_resource` module:

- hard-bound constants for identifiers, text, links, progress, errors, and
  events;
- payload-free `AsyncResourceValidationError`;
- redacted borrowed `AsyncResourceId`, `AsyncResourceText`, and
  `AsyncResourceLink`;
- strict UTC nanosecond, calendar-valid, instant-equal
  `AsyncResourceTimestamp`;
- normalized `AsyncResourceStatus` and bounded progress/error values;
- lifecycle-coherent `AsyncTask` and complete read-only field access;
- exhaustive `AsyncPollDisposition` that cannot collapse caller intervention
  into continued polling;
- generic bounded `AsyncEvent` and `AsyncEventBatch` fixture models.

`cloud-sdk-reqwest 0.33.0` publishes the v0.58 expiry-qualified atomic bearer
rotation and refresh additions. `cloud-sdk-hetzner 0.39.1` and
`cloud-sdk-testkit 0.29.1` only update their core dependency.

## Semver And Security Assessment

The new core module is additive. Existing exhaustive `PaginationError`
matches require the v0.59 migration described in
[`MIGRATION_0.59.0.md`](MIGRATION_0.59.0.md). No default feature or dependency
is added.

All asynchronous-resource scalar content is borrowed, bounded, validated, and
redacted from Debug. Optional task links and messages preserve source
optionality, and links are explicitly non-executable. Timestamp ordering and
terminal coherence fail closed. Contradictory success/error snapshots and
waiting-for-input states remain exhaustive non-success polling dispositions.
The model does not parse provider JSON,
own a clock, execute links, poll automatically, or claim an OVHcloud provider.

The public checkpoint is suitable for pentest after the complete repository,
MSRV, source-drift, packaging, SBOM, and v0.60 release gates pass.
