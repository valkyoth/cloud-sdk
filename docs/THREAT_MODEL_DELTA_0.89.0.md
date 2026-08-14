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
  reject controls, bidirectional controls, zero-width formatting, and byte
  order marks.
- Cross-field validation requires IPv4 for address selectors and TCP for flag
  expressions. Robot's official examples permit a port without an explicit
  protocol; explicitly incompatible protocols remain rejected. IPv6 rules
  cannot carry IPv4 selectors, and protocol constraints require an explicit IP
  version.
- Inline-rule and template-ID replacement modes are distinct types. Safe
  construction cannot emit Robot's source-forbidden mixed form.
- Forms are atomically encoded into caller storage, marked sensitive, and
  cleared completely on every preparation failure. Mutation retries are never
  automatic.
- Strict decoding rejects unknown/missing fields, malformed identities and
  states, excessive or duplicate collections, contradictory rules, and
  noncanonical selectors. Protected text is non-copyable and diagnostics are
  redacted. Protected policy comparison uses constant-time text equality and
  fixed-work field traversal.
- Checked responses retain the exact operation and request. Server/template
  identities and complete replacement/create/update/clear outcomes must match
  before success is exposed. When an official detailed template response omits
  the documented name, create/update returns a non-erasable pending type rather
  than claiming complete confirmation. Confirmation consumes that state and
  requires a matching name-bearing list summary, exact identity, protected
  name, policy flags, and detailed ordered rules.
- Mutations require request-bound strong-digest authority; server clear and
  template delete require distinct destructive authority. Permit attempts
  remain delivery-classified and single use.
- Immutable source evidence reconciles all eight active inventory rows, their
  exact 500-per-hour quotas, and digest-bound official response examples.
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

Robot supplies no revision or ETag that atomically binds template-list and
detail observations. Callers must prevent concurrent mutation while collecting
both views or repeat reconciliation after any possible race. The SDK rejects
cross-identity and contradictory observations but cannot prove that no third
party changed the template between provider reads.
