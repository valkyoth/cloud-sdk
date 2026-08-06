# v0.57.0 Public API Review

Date: 2026-08-06

Scope: unpublished OVHcloud API v2 source-lock probe.

## Rust API

v0.57.0 adds no Rust public API and changes no provider, transport, client,
sanitization, or testkit runtime behavior. The `cloud-sdk` source package moves
to 0.57.0 for the repository tag while crates.io remains at 0.55.0 until the
v0.60.0 checkpoint.

## Probe Contract

The new repository contract consists of one excluded source inventory, threat
model, exact candidate table, neutral drift lock, canonical observation, and a
hard-coded reviewed adapter. It covers eight production read-only IAM routes
and source-derived API/token authorities, OAuth expiry, schema validation,
cursor pagination, task, and event evidence.

The probe has no Cargo manifest, workspace membership, public Rust module,
release-plan package, publish-order entry, runtime transport, credential input,
or supported-provider claim. Its names and files are not a compatibility
commitment for a future full OVHcloud provider.

## Security Review

Official source retrieval uses one validated global DNS answer set for the
actual socket while preserving original-host SNI and certificate checks. It
remains exact-URL, no-proxy, no-redirect, bounded, and digest authenticated.
The two official GitHub guides are commit-pinned. The adapter rejects duplicate
JSON members and IAM paths, non-UTF-8 documents, non-finite constants,
unexpected authorities, absent OAuth or task evidence, non-production
operations, and non-GET candidates. DNS admission is deduplicated and capped at
eight destinations under one TLS-setup deadline; successful setup restores the
normal HTTP I/O timeout. Lock and observation are compared independently, and
CI rejects accidental package publication.
