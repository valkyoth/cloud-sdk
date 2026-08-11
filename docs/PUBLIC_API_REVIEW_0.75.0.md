# v0.75.0 Public API Review

Status: implementation stop reached; pentest required.

Scope: cumulative published changes from v0.70.0 and incremental changes from
signed v0.74.0 through the v0.75.0 implementation stop.

## Provider-Neutral Addition

`SnapshotEncoder::form_component` adds standard HTML-form component encoding
to the existing no_std transactional snapshot writer. It does not add a form
object model, allocation, transport behavior, dependency, or provider policy.

## Hetzner Robot Addition

`cloud_sdk_hetzner::robot` exposes:

- `RobotFormField` with public and sensitive constructors;
- `RobotForm` for ordered, duplicate-preserving immutable field snapshots;
- `EncodedRobotForm`, a non-cloneable cleanup-owning output guard;
- `RobotFormSensitivity` and payload-free `RobotFormError`; and
- public field, value, body, and count bounds.

The API intentionally exposes encoded bytes only while the guard owns the
mutable destination borrow. It never returns mutable body access or a source
value accessor. Field names are public provider metadata; value-bearing Debug
output is always redacted, including values marked public.

## Cumulative Provider APIs

The v0.75 provider package also publishes named client methods accumulated in
v0.71-v0.73 for every active DNS, Security, and Console Storage Box operation.
Those APIs retain the operation associations, permits, official endpoint
policies, bounded workspaces, and checked response behavior reviewed at each
signed milestone.

## Explicit Non-Claims

The form codec does not authenticate, select an endpoint, create a prepared
request, send bytes, retry, decode a response, or perform a Robot operation.
Robot credentials begin in v0.76; endpoint-family APIs begin in v0.78. The 16
deprecated legacy Robot Storage Box operations remain unavailable.
