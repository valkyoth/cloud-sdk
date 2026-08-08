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
  documented numeric, string, and list bounds before public conversion.
- Public strings reject controls and invisible bidirectional formatting, and
  field names are non-empty, bounded, sorted, and duplicate-free.
- Integer values remain integer values; provider IDs are never converted
  through `f64` and every resource ID must be positive.
- Unknown fields and enum values remain bounded and visible. They cannot alter
  the selected resource family or bypass validation of known fields.
- The generated contract and fixtures must reproduce from the exact pinned
  upstream specification during live drift checks.

## Unchanged Boundaries

These ordinary Cloud resources do not contain credentials or documented secret
outputs. Existing zonefile, certificate, password, and provider-error protected
storage remains unchanged. The default graph remains transport-free and
`no_std`; no network or retry behavior is introduced.
