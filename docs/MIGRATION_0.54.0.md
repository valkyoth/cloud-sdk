# Migrating Source Users To v0.54.0

v0.54.0 is an internal source milestone and is not published separately to
crates.io. Existing client execution methods remain source-compatible.

## Opt-In Diagnostics

Use `execute_blocking_observed`, `execute_async_observed`, or
`execute_local_async_observed` when an application needs lifecycle events.
Implement `DiagnosticObserver` with a caller-owned sink. Ordinary execution
methods keep observation disabled through `NoopDiagnosticObserver`.

Do not convert raw transport or provider errors into diagnostic fields. The new
`DiagnosticErrorCategory`, `DiagnosticContext`, and `DiagnosticResponse` types
provide the complete admitted diagnostic surface. Request-ID diagnostics expose
only policy disposition, never identifier bytes.

Observer return errors are ignored by design. Observer panics follow normal
Rust panic behavior; keep callbacks bounded and non-panicking.

## Rust Version

The workspace MSRV remains Rust 1.92.0. v0.54 adds no dependency or feature.
