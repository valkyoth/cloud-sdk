# v0.70.0 Rejected Abstractions

Status: implementation complete; pentest required.

## Hand-Written Per-Operation Methods

Maintaining 139 methods separately from the operation manifest would allow
classification and naming drift. v0.70 generates the complete surface and
checks the committed output for freshness.

## Direct State-Changing Convenience Calls

A method that accepted only a request and transport would make mutation,
destruction, or billing possible without plan review. v0.70 keeps named
preparation and permit-authorized execution as separate phases.

## Client-Owned Buffers

Implicit allocation would hide capacity, cleanup, and concurrency behavior.
Read methods require a caller-owned workspace lease; state-changing preparation
requires caller-owned cleanup-guarded buffers.

## Implicit Retry Or Runtime Selection

The named surface performs one transport attempt. It does not choose an async
runtime, spawn work, sleep, retry, reconcile, or infer idempotency.

## Executable Custom Endpoints

Allowing official operations to execute on a custom client would weaken the
source-locked credential destination. Custom construction stays visible and
explicit, but custom-trust clients expose no Cloud execution methods.

## A Separate Hetzner Client Crate

The client belongs to the provider crate. A second provider-specific client
package would violate the one-primary-crate-per-provider rule and multiply the
release and supply-chain surface.
