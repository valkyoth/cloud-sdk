# Versioning And Error Policy

## Workspace Releases

Repository tags follow the `cloud-sdk` facade version. The facade always moves
to the tag version. Every other published crate is independently versioned:

- real public code changes receive an appropriate pre-1.0 minor increment;
- dependency-only changes receive a patch increment;
- unchanged crates are not published;
- one provider maps to one provider crate.

`release-crates.toml` is the machine-checked publication plan and must describe
the reason and change class for every crate before a release tag is created.

## Pre-1.0 Compatibility

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
