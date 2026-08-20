# Versioning And Error Policy

## Workspace Releases

Repository tags follow the `cloud-sdk` facade version. The facade always moves
to the tag version and keeps ordinary `vX.Y.Z` tag names. The completed pre-1.0
train used five-minor publication checkpoints:

- every version receives an incremental pentest against the preceding tag and
  a signed tag;
- intermediate versions are not published to crates.io;
- versions divisible by five receive crates.io publication after their own
  incremental pentest;
- v1.0.0 receives an independent full-project pentest and publication;
- material security or compatibility needs may create an exceptional earlier
  publication checkpoint.

For v1.0.0, all five public crates move together to `1.0.0` without runtime or
dependency behavior changes. After v1.0, every published crate is independently
versioned:

- real public code changes receive an appropriate SemVer increment;
- dependency-only changes receive a patch increment;
- unchanged crates are not published;
- one provider maps to one provider crate.

Supporting crates retain their latest published versions during intermediate
tags. At a public checkpoint, accumulated package-tree comparison against the
previous public tag determines which crates must move: dependencies publish
first and the facade publishes last. `release-crates.toml` records the stage,
public baseline, immediate review baseline, complete milestone list, reason,
and change class for every crate.

## Compatibility

Before `1.0.0`, minor releases may contain necessary public API changes. Such
changes must be deliberate, documented in release notes and a migration guide,
and covered by compile-checked examples or tests. Security fixes may require a
narrower compatibility break when preserving an unsafe API would retain the
problem.

At `1.0.0`, ordinary incompatible public API changes require a new major
version. Provider-side removals and security emergencies still follow the
deprecation and security policies rather than silently preserving unsafe or
nonfunctional behavior.

## Error Contract

Public first-party error values implement `core::fmt::Display` and
`core::error::Error` under the MSRV. Display messages are static and
payload-free. They never include request targets, bodies, credentials,
provider messages, customer identifiers, or tenant-controlled input.

Error variants should describe the invalid field or policy where that
distinction is stable. Required fields should normally be represented by direct
constructor arguments, making missing-input variants unnecessary. Nested
errors may retain structured causes for programmatic matching, but their
`Display` implementation must not delegate to potentially sensitive payloads.
