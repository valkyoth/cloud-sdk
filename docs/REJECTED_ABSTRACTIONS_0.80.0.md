# v0.80.0 Rejected Abstractions

Status: release candidate; pentest and final retest passed.

## One Free-Form IP Request

Rejected because caller-selected methods, suffixes, and fields could confuse
read, mutation, and destructive policy. Six named request types make each
official operation and response association explicit.

## String Addresses And MACs

Rejected because arbitrary text defers target validation and permits ambiguous
wire identities. Canonical protected IP and lowercase EUI-48 values are
validated before request preparation or response admission.

## Empty Or Map-Based Traffic Updates

Rejected because an empty form has unclear intent and a general map permits
unsupported or repeated fields. `RobotIpTrafficUpdate` starts with one selected
field and exposes only the four source-locked options.

## Automatic MAC Retry

Rejected because generation is non-idempotent and uncertain delivery can leave
contradictory assignment state. Delete also remains retry-denied despite
idempotent semantics; callers must read and reconcile before another mutation.

## Unbound Checked Responses

Rejected because callers could decode one admitted response against another
request. Typed prepared/checked wrappers and permit attempts retain the exact
request through admission and execution.

## High-Level Robot Client

Rejected for this milestone. Endpoint-family completion and client integration
have separate review stops; Robot client execution remains assigned to the
later roadmap.
