# v0.86.0 Rejected Abstractions

Status: implementation stop; pentest required.

## One Free-Form Reverse-DNS Request

Rejected because caller-selected method, path, body, and response shape could
confuse reads, mutations, and deletion. Five named request types retain exact
operation and decoder association.

## Generic Unvalidated PTR Text

Rejected because free text permits controls, empty or oversized labels,
trailing-root ambiguity, and inconsistent case. `RobotRdnsName` establishes one
bounded canonical representation.

## Resource-Wide Mutation Authorization

Rejected because authorization for one address or PTR must not authorize a
different mutation. Permits bind the complete exact request fingerprint and
separate destructive deletion from set/update.

## Automatic Set/Update Fallback Or Retry

Rejected because delivery ambiguity can duplicate or overwrite provider state.
Callers must select the official operation and reconcile current state after
uncertain delivery.

## Status-Only Mutation Success

Rejected because a successful status cannot prove the intended mapping was
applied. Set and update must echo the exact address and PTR; delete must match
the source-locked empty-`200` contract.

## Live DNS Resolution In The Provider Crate

Rejected because it would add network, resolver, clock, platform, and policy
dependencies to the transport-free `no_std` provider graph without proving
domain ownership or eventual DNS propagation.
