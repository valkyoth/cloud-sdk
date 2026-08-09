# v0.67.0 Rejected Abstractions

Status: implementation stop; incremental pentest required.

## Generic Console Resources

Reducing boxes, types, snapshots, and subaccounts to identifiers discards
access, lifecycle, pricing, usage, protection, location, and identity state.
v0.67 uses dedicated source-derived models and result variants.

## One Shape For Every Create Response

Console box creation returns a complete box, while snapshot and subaccount
creation return partial references alongside actions and sensitive outputs.
`StorageBoxResource` represents these documented differences explicitly rather
than inventing absent fields or retaining an ambiguous generic resource.

## Public Dynamic Fields

Public `String` fields permit accidental unredacted formatting and prevent the
model from owning cleanup policy. Dynamic Console response text is private,
cleanup-owning, and exposed through borrowed read-only accessors.

## Reparse Serialized Paths For Response Identity

Recovering expected identifiers from prepared URL text would duplicate path
parsing and couple security checks to serialization details. Typed preparation
captures identifiers from endpoint values before path encoding and carries an
opaque allocation-free identity alongside the operation marker.

## Structural Equality For Dynamic Models

Derived equality would compare cleanup-owning provider text with ordinary
variable-time string operations. Console aggregates retain explicit scalar
accessors but expose no whole-model equality contract.

## Unbounded Provider Collections

The provider specification does not give every list a practical memory cap.
The SDK retains explicit local maxima and pagination coherence instead of
allocating directly from provider-controlled counts.

## Infallible Boxing For Composite Layout

Boxing the complete `StorageBoxResource::StorageBox` value only to reduce enum
size would introduce an infallible allocation after otherwise fallible,
bounded parsing. The result remains value-owned and carries one narrow
documented Clippy layout exception.

## One Monolithic Storage Model File

Boxes, prices, snapshots, subaccounts, references, and parser helpers have
different responsibilities. Separate focused modules keep each source file
under the repository's 500-line policy without creating another published
crate.
