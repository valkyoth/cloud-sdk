# cloud-sdk 1.1.2

Maintenance release against `v1.1.1`. The [pentest and remediation retest](../security/pentest/v1.1.2.md)
closed the invitation target-buffer finding with no outstanding findings.

All six packages target 1.1.2. Rust 1.100.0 is the development compiler and
joins the CI compatibility matrix. The minimum supported compiler is Rust 1.92.

The optional transport updates AWS-LC to 1.19.0/0.47.0, bytes to 1.12.2
and WebPKI roots to 1.0.12. Shared dependencies include sanitization 2.2.0,
base64-ng 2.1.1 and rustix 1.1.6. The isolated coverage tool updates Saphyr
and its parser to 0.3.4 and syn to 3.0.10. See the [four-lockfile dependency
review](../docs/DEPENDENCY_REVIEW.md#v112).

CI uses cargo-deny 0.22.0 and cargo-audit 0.23.0. Fuzzing uses the November
20 nightly. The checkout action's SHA was verified against upstream v7.0.1.

## Provider changes (2026-11-20)

- Hetzner Cloud: 21 existing operation fingerprints changed, with no operations
  added or removed. Updated request and response models for the
  [volume target changes](https://docs.hetzner.cloud/changelog#2026-10-27-volumes-allow-generic-resource-targets)
  and [removed deprecated field](https://docs.hetzner.cloud/changelog#2026-11-02-removed-the-deprecated-field).
  The Console specification is unchanged.
- Hetzner Robot: the two IP/subnet traffic-warning mutations and associated
  fields are scheduled for removal on January 12, 2027. Both calls remain
  available; the reviewed inventory records their deprecation separately
  from the 16 excluded Storage Box routes.
- crates.io: the [OpenAPI specification](https://crates.io/api/openapi.json)
  adds invitation listing and update by invitation ID, and deprecates the
  previous handler by crate ID. New single-attempt permits support filtered
  reads and checked accept, decline and revoke responses in all three execution
  modes. Token metadata exposes the new scope without adding a variant to the
  existing public enum. Cargo reference and policy representations changed;
  the seven Cargo overlaps and rate policy did not.

The live provider checks do not require credentials or perform mutations.
