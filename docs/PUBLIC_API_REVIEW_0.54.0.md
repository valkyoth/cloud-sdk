# v0.54.0 Public API Review

Date: 2026-08-04

Scope: provider-neutral payload-free client lifecycle diagnostics.

## Added API

The new `diagnostics` module exports finite error, retry, and request-ID
categories; bounded operation and response context; copy-only lifecycle events;
the `DiagnosticObserver` contract; and `NoopDiagnosticObserver`.

`ClientKernel` adds explicit observed variants for blocking, cross-thread async,
and local async execution. Existing methods retain their signatures and use the
no-op observer internally.

## Security Review

Events contain no generic parameters, dynamic strings, slices, request targets,
headers, credentials, bodies, cursors, provider messages, or request-ID bytes.
Allowed identifiers reuse bounded validated provider-neutral taxonomy types.
Request-ID presence is hidden when policy is `Discard`.

Observer failures cannot replace SDK results and place no formatting trait bound
on downstream errors. Shared callbacks support reentrancy without SDK-owned
locks. Cross-thread async requires `Sync`, preserving the existing `Send` future
guarantee. Panic behavior remains caller-owned and workspace RAII cleanup stays
active during unwinding.

The API remains allocation-free and `no_std`, adds no automatic log, runtime,
clock, queue, retry, or retention behavior, and supports Rust 1.92.0 through the
pinned stable toolchain.
