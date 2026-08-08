# v0.66.0 Rejected Abstractions

Status: release candidate; pentest and final retest passed.

## Generic Resource Identities

Reducing certificates or SSH keys to an identifier and name discards security
state, key material, labels, validity, and usage. v0.66 uses dedicated models
inside one narrowly scoped `SecurityResource` dispatcher.

## Public Secret-Bearing Fields

Public fields make accidental formatting and long-lived borrowing too easy.
Protected certificate chains and SSH keys use closure-scoped access, while all
other fields use explicit read-only accessors and redacted diagnostics.

## Open Certificate-State Strings

Treating future issuance or renewal strings as ordinary success can hide a
provider semantic change. v0.66 fails closed until a new state is reviewed and
source-locked.

## Infallible Boxing For Enum Layout

Boxing security results solely to satisfy a layout heuristic would introduce
an infallible allocation after otherwise fallible parsing. The checked result
remains value-owned and documents the narrow Clippy layout exception.

## Certificate Parsing Dependency

A general X.509 package would add a large cryptographic/parser supply-chain
surface when this milestone only needs source-bounded PEM framing. v0.66
validates the documented wire shape without claiming certificate-chain trust
or cryptographic verification.
