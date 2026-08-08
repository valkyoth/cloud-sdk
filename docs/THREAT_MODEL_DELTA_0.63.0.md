# v0.63.0 Threat Model Delta

Status: implementation review complete; pentest required.

## New Surface

The checked decoder now allocates and exposes complete ordinary Cloud response
trees rather than selected identity fields. Upstream responses can therefore
exercise more nested values, lists, strings, and unknown additive fields.

## Controls

- The existing 8 MiB wire, 65,536-node aggregate, 256-field-per-object, depth,
  duplicate-key, and checked-allocation limits remain mandatory.
- Every source-known path is checked for presence, nullability, JSON type, and
  documented numeric, Unicode-scalar string, list, format, and pattern
  constraints before public conversion. Unsupported security constraints stop
  deterministic schema generation rather than being silently discarded.
- Public strings reject controls and invisible bidirectional formatting, and
  field names are non-empty, bounded, sorted, and duplicate-free.
- Integer values remain integer values; provider IDs are never converted
  through `f64` and every resource ID must be positive.
- Unknown fields and enum values remain bounded and visible. They cannot alter
  the selected resource family or bypass validation of known fields.
- Ordinary Cloud and pricing diagnostics redact all field content, including
  identifiers and unknown future values. Infallible recursive `Clone` is not
  implemented; explicit `try_clone` methods preserve checked allocation.
- The generated contract and fixtures must reproduce from the exact pinned
  upstream specification during live drift checks.
- The optional Basic-auth encoder moves to exact `base64-ng 2.0.1` with default
  features disabled. Existing exact output, source-clearing, aggregate-bound,
  feature-boundary, and transport tests continue to cover its use.

## Unchanged Boundaries

These ordinary Cloud resources do not contain documented credentials or secret
outputs, but their names, labels, addresses, topology, placement, and unknown
future fields are operationally sensitive. They remain ordinary caller-readable
strings rather than secret-erasing storage, so applications requiring erasure
must copy selected values into protected storage and bound concurrent retained
responses. The SDK's allocation ceilings are per response, not a process-wide
memory quota.

Existing zonefile, certificate, password, and provider-error protected storage
remains unchanged. The default graph remains transport-free and `no_std`; no
network or retry behavior is introduced. The encoder update does not change the
public authentication contract.
