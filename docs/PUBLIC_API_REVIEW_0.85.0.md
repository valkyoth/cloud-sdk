# v0.85.0 Public API Review

Status: implementation stop; pentest required.

Scope: cumulative public changes after signed v0.80.0 through the v0.85.0
implementation stop, with detailed incremental review after signed v0.84.0.

## Robot Boot Requests

`cloud_sdk_hetzner::robot` adds 15 named request types for the complete active
Robot boot family. Every path accepts only `RobotServerNumber`, fixes the
official Robot endpoint and Basic scope, exposes the 500-request/hour quota,
and binds exact method, operation ID, response policy, and raw-wire policy.

Activation values are bounded by `RobotBootValue`, `RobotBootKey`, and
`RobotKeyboardLayout`. Rescue and Linux reject repeated key fingerprints and
more than 64 keys. Forms are sensitive and preparation clears target and body
storage on every failure. All mutations are non-idempotent and retry-denied;
Linux, VNC, and Windows activation is destructive.

## Strict Results And Failures

With `serde`, `PreparedRobotBoot<R>` and `CheckedRobotBoot<R>` preserve exact
request provenance. `RobotBoot`, `RobotBootEntry`, `RobotBootChoice`, and
`RobotBootFamily` expose bounded typed state. Passwords, selectors, authorized
keys, and host keys are represented by redacted `RobotBootSecret` values with
closure-scoped access.

Decoding requires canonical address families, the exact server number, exact
family envelopes, bounded duplicate-free collections, and coherent
active/password/selection state. Activation acknowledgements must match the
requested selector and language; deactivation must return inactive,
password-free state. Each checked request supplies its internal overview,
current, last, activation, or deactivation response shape; shape-free decoder
functions are not exposed. The documented inactive Windows overview null
language is admitted only there, and overviews reject multiple active
families. Operation-specific failures admit only the source-locked boot and
Windows codes.

## Cumulative Publication

The checkpoint also publishes the reviewed v0.81 subnet, v0.82 reset, v0.83
failover, and v0.84 Wake-on-LAN APIs. `cloud-sdk` becomes `0.85.0` and
`cloud-sdk-hetzner` becomes `0.44.0`. Reqwest and testkit receive
dependency-only patch releases; sanitization remains unchanged and
unselected.

## Explicit Non-Claims

The SDK does not reboot a server, prove that a later reboot used the selected
configuration, infer installation completion, retry uncertain mutations, or
provide a high-level Robot client. Provider-specific boot permit wrappers are
not introduced in this milestone; operation impact, non-idempotency, retry
denial, authenticated transport, and caller reconciliation remain explicit
execution boundaries.
