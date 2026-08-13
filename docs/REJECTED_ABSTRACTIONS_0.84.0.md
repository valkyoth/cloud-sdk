# v0.84.0 Rejected Abstractions

Status: implementation stop; pentest required.

## Server Address Path Identity

Rejected because the official server-IP route is deprecated and operationally
ambiguous. Only positive `RobotServerNumber` identity is accepted.

## Direct Send Constructor

Rejected because the server model's capability bit or a raw decoded WOL value
does not prove current authenticated availability. Sending requires a fresh
authenticated `GET /wol/{server-number}` under the same credential lineage.

## Generic Prepared Mutation

Rejected because erasing the typed request would detach capability evidence
and exact response association. WOL send preparation remains provider-bound
and requires authorization evidence in the plan digest.

## Automatic Retry

Rejected because a timeout after request transmission cannot establish
whether Robot already sent the packet. The send is non-idempotent and never
automatically retried.

## Destructive Permit

Rejected because sending WOL changes state but does not itself delete a
resource or request data destruction. Mutation authority is the narrowest
matching scope.

## Empty Success Or No-Content Policy

Rejected because both official operations return the complete three-field WOL
identity object. Requiring that object binds acknowledgement to the intended
server and prevents a status-only false success.
