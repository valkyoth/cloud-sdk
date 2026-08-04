# Payload-Free Diagnostics

`cloud-sdk` provides opt-in structured lifecycle observation without logging,
allocating, or exposing request and response payloads from core.

## Event Boundary

`DiagnosticEvent` contains only finite categories and validated bounded values:

- provider, service, and optional operation identifiers;
- operation impact and retry eligibility;
- HTTP status code;
- request-ID disposition, never request-ID bytes;
- structural preparation, authorization, endpoint, transport, response, and
  decode failure categories.

Events never contain credentials, authorization values, complete request
targets, headers, bodies, provider messages, cursors, resource identifiers, or
generic transport and decoder errors. Provider and operation identifiers are
public taxonomy, not customer data, and retain their existing hard bounds.

## Opt-In Observation

Ordinary `execute_blocking`, `execute_async`, and `execute_local_async` calls
use `NoopDiagnosticObserver`. Core never writes logs, retains events, chooses a
telemetry backend, or starts a task.

Applications opt in with the corresponding `*_observed` method:

```rust,ignore
use cloud_sdk::diagnostics::{DiagnosticEvent, DiagnosticObserver};

struct Observer;

impl DiagnosticObserver for Observer {
    type Error = core::convert::Infallible;

    fn observe(&self, event: DiagnosticEvent) -> Result<(), Self::Error> {
        let _event = event;
        Ok(())
    }
}

let result = kernel.execute_blocking_observed(&operation, lease, &Observer);
```

The observer receives a shared reference. A cross-thread async observer must
be `Sync`; local and blocking use may intentionally use local interior state.
The SDK holds no observer lock, so callback reentrancy cannot deadlock SDK-owned
diagnostic state.

## Failure Isolation

Observer return errors are deliberately ignored and never replace preparation,
transport, or decode results. The associated error type needs no `Debug`,
`Display`, `Error`, `Send`, or payload-bearing conversion.

An observer panic still follows the application's configured Rust panic
behavior. During unwinding, the client workspace lease clears all owned
buffers. In `panic = "abort"` builds, process termination semantics apply.
Observers must remain bounded, non-panicking, and must not perform recursive
SDK execution unless the application has explicitly budgeted that behavior.

## Request Identifiers

Request-ID handling follows each operation's `RequestIdPolicy`:

| Policy | Diagnostic value |
| --- | --- |
| `Discard` | `Discarded`, regardless of whether a header existed |
| `Protected` without an ID | `Absent` |
| `Protected` with an ID | `Protected` |
| `Retain` without an ID | `Absent` |
| `Retain` with an ID | `Retainable` |

No event exposes request-ID bytes. Under `Discard`, it also does not reveal
whether the provider sent an identifier.

## Caller Responsibilities

- Treat provider, service, operation, status, and event timing as operational
  metadata that may still be sensitive in a particular deployment.
- Do not enrich events with raw errors or request/response values without a
  separate application security review.
- Bound observer work and storage; avoid network calls or blocking work in an
  async observer callback.
- Keep metrics labels finite. Do not attach account, tenant, resource, cursor,
  target, message, or request-ID values as dynamic labels.
- Test observer panic and retention policy according to the deployment's own
  availability and data-governance requirements.
