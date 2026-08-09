# FIPS Deferment

Status: active project boundary.

FIPS support is not part of the cloud-sdk 1.0 scope. It is deferred until Brynja
has a stable reviewed API and can bind an exact cryptographic module to its
issued validation certificate, approved operating environments, build inputs,
security policy, and runtime evidence.

The earlier experimental `cloud-sdk-reqwest` AWS-LC FIPS feature has been
retired. Active manifests, lockfiles, source, packages, and CI contain no FIPS
transport implementation or `aws-lc-fips-sys` dependency. Historical release
notes, assessments, and admission records remain available as evidence for the
tags that previously exposed that experiment; they are not current support
claims.

## Future Admission Conditions

A future cloud-sdk release may add an optional Brynja adapter only after all of
these conditions are met:

- Brynja's non-FIPS transport and its separately packaged FIPS boundary are
  stable enough for downstream integration.
- The exact cryptographic module version matches an applicable active
  certificate; version-family or runtime-flag equivalence is insufficient.
- Supported operating environments, toolchain and build provenance, entropy,
  roots, revocation policy, and operational requirements are explicit.
- Unsupported targets and configurations fail closed before credentials or
  requests are used.
- The adapter remains non-default and does not enter the portable `no_std`
  provider graph.
- Dependency, package, platform, runtime, security-review, and pentest evidence
  covers the complete selected graph.

Until those conditions are satisfied, cloud-sdk makes no FIPS compliance claim
for a crate, application, deployment, operating environment, or organization.
Applications that require validated cryptography must select and qualify their
own transport outside cloud-sdk.

## Enforcement

`scripts/check_fips_deferred.py` rejects the retired feature or dependency in
active manifests, locks, reqwest source, CI, and public README files. Its
regression tests run in the complete repository gate.
