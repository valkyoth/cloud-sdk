# v0.61.0 Public API Review

Status: release review complete; pentest and final retest passed.

Scope: changes from signed and published v0.60.0 through v0.61.0.

## Public Surface

No public library item, feature, default dependency, or provider support claim
is added. The only Rust package added is the exact `publish = false`
`ovhcloud-v2-probe` repository harness. It depends on existing neutral crates
and keeps all provider-specific identities, routes, fixtures, and live
configuration outside reusable production modules.

`cloud-sdk` advances to source version 0.61.0 so the signed milestone matches
the tag. Published supporting crate versions remain unchanged, and no package
is selected for crates.io.

## Security Assessment

All ten operations are source-locked `GET` requests with read-only, safe,
no-known-cost, and never-retry metadata. Deterministic fixtures execute through
blocking, Send-async, and local-async prepared response validation. The
optional live smoke is ignored, EU-only, exact-endpoint bound, least-privilege,
bounded, and protected on Unix by private token-file, opened-file identity,
and complete partial-read cleanup controls. Non-Unix live execution fails
closed until equivalent platform checks exist.

Release tooling ignores only workspace members whose Cargo metadata explicitly
reports `publish = false`; regression coverage proves that publishable members
remain the exact five-crate release order. A separately supported OVHcloud
provider still requires its own source lock, scope, crate, and review plan.
