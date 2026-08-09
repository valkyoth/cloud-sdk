# v0.69.0 Rejected Abstractions

Status: release candidate; pentest and final retest passed.

## A Separate Hetzner Client Crate

Client construction and provider policy belong to `cloud-sdk-hetzner`.
Creating `cloud-sdk-hetzner-client` would violate the one-primary-crate-per-
provider rule and multiply release, feature, and supply-chain surface.

## Runtime Service Selection

A client containing a runtime Cloud/DNS/security/Storage enum could admit
cross-service operation mistakes. v0.69 represents service ownership in the
type and requires exact operation-service equality.

## Executable Custom Endpoints By Policy Substitution

Replacing an operation's fixed official endpoint policy at execution time
would weaken source-locked credential scope. v0.69 permits explicit custom
construction but gives that trust marker no execution methods. A future custom
execution design must bind the acknowledged endpoint during preparation and
decoding rather than override policy afterward.

## Hidden Storage Allocation

Allocating workspace per request would change failure, cleanup, and concurrency
behavior. Caller-owned fixed buffers remain the default. Optional owned storage
is explicit, fallible, profile-bounded, and available only with `alloc`.

## Internal Queue, Runtime, Or Retry Loop

The facade delegates one attempt to the existing kernel. Admission, executor,
clock, retry, and backpressure remain caller policies, so cancellation and
delivery ambiguity are not obscured by client-owned orchestration.

## Direct Mutation Convenience

Adding generic mutation methods before service-specific plan-confirm workflows
would make the unsafe route easier than the reviewed route. v0.69 exposes only
read-only execution; permit-authorized client methods remain later milestones.
