# Migrating To v0.55.0

v0.55.0 is the cumulative public checkpoint for tagged source milestones
v0.51.0 through v0.55.0. It follows the published v0.50.0 baseline.

## Package Versions

```toml
cloud-sdk = "0.55.0"
cloud-sdk-hetzner = "0.39.0"
cloud-sdk-reqwest = "0.32.4"
cloud-sdk-sanitization = "0.18.0"
cloud-sdk-testkit = "0.29.0"
```

`cloud-sdk-sanitization` moves to `0.18.0` for workspace-wide fail-closed test
assurance. Its public API and runtime behavior are unchanged.

## v0.51: Execution Permits

State-changing and cost-bearing prepared operations require explicit
plan-confirm execution permits. Migrate direct mutation execution to
`DirectExecutionPermit` or a caller-owned `SharedExecutionPermitState`.
Fingerprints bind the exact operation, endpoint, request, and validity policy;
do not cache permits independently of their planned request.

Read-only operations retain direct execution. Hetzner typed associations now
carry the provider-neutral execution policy used by prepared operations.

## v0.52: Client Kernel

`ClientKernel` joins preparation, endpoint and authentication checks, one
transport attempt, checked response classification, and typed decoding.
Callers provide a fixed `ClientWorkspace` and acquire one bounded lease per
in-flight request. Lease exhaustion is immediate; no hidden queue or allocator
was added.

The workspace and client execution futures require Rust 1.92.0. Projects still
on Rust 1.90 or 1.91 must remain on the published v0.50 line until they can
raise their toolchain.

## v0.53: Workflow Drivers

`PagerDriver` and `ActionDriver` own transactional workflow state and admit one
response at a time. Action polling separates cancellation, backoff, progress,
and clock policy. Configure request, item, state, observation, delay, and
elapsed-time limits explicitly; provider telemetry cannot extend local bounds.

## v0.54: Diagnostics

Use `execute_blocking_observed`, `execute_async_observed`, or
`execute_local_async_observed` for finite payload-free lifecycle events.
Ordinary methods remain unobserved. Observer errors are ignored and cannot
replace execution results. Never place raw transport or provider payloads in a
custom observer's external state.

## v0.55: Dynamic Testkit

Fixed scenarios can continue using `MockTransport`. For request-dependent or
multi-step scenarios, use `DynamicMockTransport` with caller-owned
`RequestRecordSlot` storage and either `DynamicResponder`, `PaginationScript`,
`ActionScript`, or a provider-owned `ProviderFixtureBuilder`.

Dynamic recording is deliberately not a request capture facility. It exposes
only finite method classification, lengths, counts, status, and successful
sequence. Use a fresh slot array for each scenario.

Stream fixtures add exact one-based source/sink faults and non-terminating
empty or alternating sources. Drive pattern sources only through a bounded
`StreamPolicy`.

See [`DYNAMIC_TESTKIT.md`](DYNAMIC_TESTKIT.md) for the complete transaction,
recording, cancellation, scripting, and fault boundary.

## Compatibility

Default core, Hetzner, and testkit graphs remain `no_std`. No default runtime,
network client, TLS stack, clock, filesystem, allocator, logger, or retry loop
was added. The workspace MSRV is Rust 1.92.0 and development remains pinned to
Rust 1.97.1.
