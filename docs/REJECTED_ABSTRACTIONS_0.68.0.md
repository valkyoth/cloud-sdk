# v0.68.0 Rejected Abstractions

Status: implementation complete; pentest required.

## A Second Operation Registry

Hand-authoring another 208-row policy table would permit drift and duplicate
review. The complete manifest is generated from existing independent source
locks and checked against generated Rust and AST evidence.

## Runtime Reflection

Adding dynamic policy maps to the provider crate would increase runtime state
and weaken nominal operation typing. v0.68 keeps compile-time associations and
adds only build-time review evidence.

## Treating Rejecting Enum Variants As JSON Bodies

Twelve server actions share a request enum but deliberately reject body
serialization. Calling them request-body operations would contradict the API
source. The verifier records their exact set and requires typed body policy to
remain `forbidden`.

## Reintroducing Deprecated Adapters

Deprecated resource-local action and datacenter endpoints are not needed for
complete active coverage. The verifier explicitly rejects them from every
executable registry.

## A New Published Crate

The proof belongs to repository tooling and the existing Hetzner provider.
Splitting it into another package would add release and supply-chain surface
without a consumer-facing capability.
