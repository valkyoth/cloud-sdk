# v0.67.0 Threat Model Delta

Status: implementation stop; incremental pentest required.

## New Surface

Checked decoding now retains complete Console Storage Box, type, price,
snapshot, and subaccount responses. This includes provider-returned account
usernames, service hosts, system identifiers, home directories, descriptions,
labels, usage counters, pricing, lifecycle state, and access settings.

## Controls

- Deterministic evidence combines the exact pinned Cloud and Console
  specifications. Every selected model field, type, nullability rule, numeric
  bound, pattern, format, and enum is regenerated and compared in CI.
- All Console response routes first pass bounded incremental JSON admission.
  Model parsing then enforces exact required fields, canonical UTC timestamps,
  positive source identifiers, bounded prices and resource lists, and
  pagination coherence.
- Initializing boxes require unavailable username, server, system, and snapshot
  plan fields to remain null. Active and locked boxes require the initialized
  identity fields. Contradictory states fail closed.
- Snapshot descriptions and subaccount home directories enforce reviewed
  source character and length policy. Invalid controls, leading absolute home
  paths, excessive collections, malformed decimals, and invalid schedule
  values fail before public model construction.
- Provider-returned dynamic text is transferred into cleanup-owning storage.
  Fallible parse temporaries and completed models sanitize allocations on drop;
  aggregate diagnostics redact values.
- Create composites distinguish complete boxes from source-documented partial
  snapshot/subaccount references instead of pretending all create responses
  have one generic resource shape.
- Dedicated tests cover late failures, exact limits, multi-chunk large lists,
  source fixtures, vertical execution, named fuzz seeds, and credential-gated
  read-only box/type probes.

## Unchanged Boundaries

No response model contains a Storage Box password. Password-bearing action
outputs remain in the existing `SensitiveText` composite boundary. Callers are
still responsible for clearing copies made from protected inspection closures.
The default crate graph remains transport-free and `no_std`.
