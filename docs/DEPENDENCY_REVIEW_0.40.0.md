# v0.40.0 Dependency Review

Date: 2026-07-29

Scope: bounded raw HTTP/1 execution for opt-in reqwest adapter features.

## Result

No dependency enters a default feature graph. `http`, `http-body-util`,
`hyper`, `hyper-rustls`, `hyper-util`, `rustls`,
`rustls-platform-verifier`, and Tokio become direct optional dependencies of
`cloud-sdk-reqwest` because the raw executor must configure parser limits,
observe informational heads and trailers, disable canceled-request retries,
and stream frames directly into caller storage. These packages were already
present in the admitted reqwest/rustls graph.

| Package | Version | Role |
| --- | --- | --- |
| `http` | `1.4.2` | request and bounded header representation |
| `http-body-util` | `0.1.4` | owned request body and response-frame access |
| `hyper` | `1.11.0` | HTTP/1 protocol implementation |
| `hyper-rustls` | `0.27.9` | rustls HTTPS connector |
| `hyper-util` | `0.1.20` | client, connector, and Tokio runtime adapters |
| `tokio` | `1.53.1` | opt-in async execution, notification, and private blocking raw runtime |

Exact reqwest, rustls, AWS-LC, roots, cleanup, and tooling versions remain
recorded in the lockfile, SBOM, and their dedicated admission documents.
Live crates.io metadata checks on 2026-07-29 confirmed the selected stable
`http`, `http-body-util`, `hyper`, `hyper-rustls`, `hyper-util`, Tokio,
reqwest, and rustls releases are current; rustls `0.24.0-dev.1` remains a
prerelease and is not selected.

## Boundary

- Default, `std`-only, provider, and testkit graphs remain network-free.
- HTTP/2, compression, proxy, cookie, JSON, multipart, SOCKS, and Hickory DNS
  features remain absent from production adapter graphs.
- Raw HTTP uses HTTP/1 only, no idle connection pool, no automatic canceled
  request retry, explicit total/connect timeouts, and the selected TLS trust
  policy.
- FIPS raw execution receives the same explicitly validated roots, complete
  CRLs, provider, and client configuration as the existing FIPS adapter.
- The `fuzzing` feature is isolated to the fuzz workspace and reuses the
  blocking graph only to expose the exact production raw parser to libFuzzer.

## Required Verification

- `scripts/check_raw_http_executor.sh`
- `scripts/check_reqwest_boundary.sh`
- deterministic-root and FIPS boundary checks
- default/no_std/platform/MSRV/package/deny/audit/SBOM checks
- `scripts/release_0_40_gate.sh` after pentest evidence is committed
