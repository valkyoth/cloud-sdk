# v0.74.0 Public API Review

Status: release candidate; pentest and final retest passed.

Scope: changes from signed v0.73.0 through the v0.74.0 implementation stop.

## Published API

No public type, trait, function, module, feature, dependency, request model,
response model, decoder, operation, or client behavior changes. `cloud-sdk`
advances to the workspace milestone version only.

The Robot API lock, checker, tests, and documentation are repository release
evidence outside every publishable package. `cloud-sdk-hetzner::robot` is not
introduced in this milestone.

## Future Surface Ownership

The lock assigns each of the 89 active Robot operations to one milestone from
v0.78 through v0.93. Those milestones must still define and review their own
request, response, error, secret, permit, retry, and client contracts. The
source lock is not a public API promise for an unimplemented shape.

All 16 legacy Robot Storage Box operations and deprecated route aliases remain
unimplemented. Existing Console Storage Box APIs remain the supported path.
