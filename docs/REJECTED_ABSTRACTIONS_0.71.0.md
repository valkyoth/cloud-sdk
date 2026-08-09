# v0.71.0 Rejected Abstractions

Status: implementation stop reached; pentest required before tagging.

## Hand-Written DNS Method Classification

Maintaining 24 methods separately from the operation manifest would permit
method, service, pagination, and permit drift. v0.71 generates and checks the
complete surface.

## One Untyped Provider Client

A client accepting Cloud and DNS operations interchangeably would weaken
credential scope and endpoint-policy evidence. DNS methods remain available
only on the `DnsService` client type.

## Direct DNS Mutation Convenience Calls

Zone, RRSet, record, protection, TTL, and zonefile changes can cause outage or
data exposure. Named mutation and destructive execution therefore requires an
exact plan-confirm permit attempt.

## Owned Hidden Request Storage

Implicit client-owned buffers would obscure capacity and secret cleanup. DNS
reads use caller-owned workspace leases, while sensitive state-changing
preparation uses `PreparationStorageGuard`.

## Executable Custom Endpoints

Official operation credentials are not sent to configurable destinations.
Custom clients remain constructible only with explicit operator trust and have
no generated execution methods.

## Retaining Experimental FIPS

A feature flag cannot establish module, certificate, operating-environment,
build, or organizational compliance. The experimental adapter is removed
instead of carrying a misleading partial FIPS boundary into 1.0; future work
waits for Brynja and the explicit admission policy.
