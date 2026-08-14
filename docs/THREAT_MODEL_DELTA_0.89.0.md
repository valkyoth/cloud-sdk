# Threat Model Delta 0.89.0

Status: implementation stop; pentest required.

## New Inputs

Robot firewall operations introduce server identities, protected account
addresses, ordered ingress/egress rules, IP selectors, ports, protocols, TCP
flags, template names and identities, lifecycle state, and security-sensitive
replacement or clear mutations.

## Controls

- Rule directions are independently bounded to 100 entries, preserve provider
  order, and reject exact duplicates without sorting or implicit deduplication.
- IPv4 selectors require canonical host or network text; CIDRs reject host bits.
  Ports require canonical non-zero single values or ascending ranges. Names
  reject controls and directional formatting.
- Cross-field validation requires IPv4 for address selectors, TCP/UDP for port
  selectors, and TCP for flag expressions. IPv6 rules cannot carry IPv4
  selectors, and protocol constraints require an explicit IP version.
- Inline-rule and template-ID replacement modes are distinct types. Safe
  construction cannot emit Robot's source-forbidden mixed form.
- Forms are atomically encoded into caller storage, marked sensitive, and
  cleared completely on every preparation failure. Mutation retries are never
  automatic.
- Strict decoding rejects unknown/missing fields, malformed identities and
  states, excessive or duplicate collections, contradictory rules, and
  noncanonical selectors. Protected text is non-copyable and diagnostics are
  redacted.
- Checked responses retain the exact operation and request. Server/template
  identities and complete replacement/create/update/clear outcomes must match
  before success is exposed.
- Mutations require request-bound strong-digest authority; server clear and
  template delete require distinct destructive authority. Permit attempts
  remain delivery-classified and single use.
- Immutable source evidence reconciles all eight active inventory rows.
  Dedicated decoder fuzzing admits one selector byte plus the complete 2 MiB
  template-list boundary.

## Residual Boundaries

Caller-owned source strings and transport copies remain caller-owned and need
appropriate lifecycle handling. Robot applies firewall changes asynchronously
after returning `in process`; callers must observe subsequent provider state
before depending on enforcement. The SDK validates source syntax and exact
request/response coherence but cannot prove the provider's packet-filtering
implementation or prevent callers from intentionally constructing permissive
ordered policies.
