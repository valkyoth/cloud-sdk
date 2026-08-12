# v0.81.0 Rejected Abstractions

Status: implementation stop; pentest required.

## Require Host Bits To Be Clear

Rejected because official Robot examples identify IPv4 subnets with host bits
set. The SDK preserves the canonical provider route identity and validates the
returned prefix and gateway; mathematical network and broadcast values are
derived separately.

## Reuse The Single-IP MAC Model

Rejected because subnet MAC responses contain a string mask and a dynamic
address-to-MAC choice map, and DELETE returns a restored default MAC rather
than `null`. A separate model prevents incompatible null and field semantics.

## Implicit MAC Selection

Rejected because PUT requires one target from the provider's current choices.
The request requires a canonical MAC and response association verifies it.

## Unbounded Map Or Generic JSON

Rejected because caller iteration over unvalidated provider JSON would bypass
identity, canonicalization, duplicate-key, allocation, and response-association
controls. The map is typed, nonempty, and bounded to 256 entries.

## Automatic Retry

Rejected for MAC assignment and restoration. Explicit reconciliation is
required after uncertain delivery despite DELETE's idempotent semantics.

## Address-Only Default Restoration

Rejected because a subnet address cannot prove which MAC is the assigned
server's default. Restoration authority consumes checked subnet and MAC
snapshots and binds the assigned server-to-MAC mapping into the request.

## Global Subnet Error Admission

Rejected because a finite code union without operation association would admit
valid provider text in the wrong control-plane context. Each request type
admits only its documented status and code combinations.
