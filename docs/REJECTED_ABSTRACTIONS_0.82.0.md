# v0.82.0 Rejected Abstractions

Status: implementation stop; pentest required.

## Raw-Detail Or Server-Number-Only Execute Constructor

Rejected because either form could bypass authenticated provenance, freshness,
or capability checks. Execute construction consumes authenticated short-lived
detail state and verifies the selected advertised capability.

## Caller-Supplied Credential Scope As Provenance

Rejected because `PlanFingerprintScope` is caller policy, not proof of which
credential sent the preflight. Reset authority instead binds an opaque
transport-owned credential lineage into digest-only evidence and rechecks it
at dispatch.

## Free-Form Reset Type

Rejected because Robot documents a finite set. An enum prevents spelling
errors and future undocumented values from silently becoming destructive
requests.

## Mutation Permit For Read Requests

Rejected because list and detail requests do not require destructive
authority. The public reset plan constructor accepts only execute requests.

## Exact Sensitive Fingerprint

Rejected because the form body contains operationally sensitive destructive
intent. Reset execution uses the strong-digest plan builder.

## Automatic Retry

Rejected even when a reset type appears repeatable. Delivery uncertainty and
provider-side timing can make a second reset destructive in a different state.

## Require Action Server Number

Rejected because the official POST example omits `server_number` while the
output table lists it. The field is optional only for action acknowledgement;
when present it must match checked state. Both addresses always remain bound.

## Global Reset Error Admission

Rejected because a finite code union without operation association would
admit valid provider text in the wrong control-plane context. Each request
admits only its documented status and code combinations.
