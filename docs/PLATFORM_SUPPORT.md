# Platform Support

Status: `v0.20.0` compile matrix implemented; runtime support remains bounded
by the selected transport and operating environment.

## Scope Of The Claim

`cloud-sdk`, `cloud-sdk-hetzner`, `cloud-sdk-sanitization`, and
`cloud-sdk-testkit` are portable no_std-first crates. Their default features do
not require an allocator, network client, TLS implementation, async runtime,
filesystem, clock, socket API, or operating-system abstraction crate.

`cloud-sdk-reqwest` is different. Its default feature set is transport-free,
but `blocking-rustls`, `blocking-rustls-webpki-roots`,
`blocking-rustls-fips`, and `async-rustls` deliberately enable std, reqwest,
rustls, sockets, DNS, and runtime integration. A portable provider model
compiling for a target does not imply that this optional adapter is supported
on that target.

## Support Terms

- **Native CI**: the complete workspace and every feature compile on a native
  GitHub-hosted runner on every change.
- **Portable CI**: the portable crates compile for the named target with both
  default no_std features and their alloc/Serde feature combination.
- **Best effort**: the architecture is compatible with the crate design, but
  no native runtime or transport job is part of the release gate.
- **Unsupported transport**: no `cloud-sdk-reqwest` compatibility claim is
  made. Use a target-native implementation of the core transport traits.

Compile evidence is not a promise that a provider endpoint is reachable from a
specific device, application sandbox, network policy, or TLS trust store.

## Target Matrix

| Platform | Representative Rust target | Portable crates | Reqwest adapter |
| --- | --- | --- | --- |
| Linux x86-64 | `x86_64-unknown-linux-gnu` | Portable CI | Native CI |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | Portable CI | Best effort |
| Windows x86-64 | `x86_64-pc-windows-msvc` | Portable CI | Native CI |
| FreeBSD x86-64 | `x86_64-unknown-freebsd` | Portable CI | Best effort |
| macOS x86-64 | `x86_64-apple-darwin` | Portable CI | Native CI on `macos-15-intel` |
| macOS ARM64 | `aarch64-apple-darwin` | Portable CI | Native CI on `macos-15` |
| Android ARM64 | `aarch64-linux-android` | Portable CI | Unsupported transport |
| iOS ARM64 | `aarch64-apple-ios` | Portable CI | Unsupported transport |
| WebAssembly | `wasm32-unknown-unknown` | Portable CI | Unsupported transport |
| Cortex-M4F class | `thumbv7em-none-eabihf` | Portable CI | Unsupported transport |

Other BSDs, Linux architectures, Windows architectures, Apple targets, and
embedded triples are best effort until added to the explicit allowlist and CI
matrix. Aesynx is not yet a Rust target and cannot be compiled here today; its
future integration should implement the provider-neutral transport contracts
without changing default crate dependencies.

## Feature Evidence

For every portable target, CI runs these equivalent checks:

```sh
scripts/check_platform_matrix.sh --portable TARGET
```

The command checks:

- all four portable crates with `--no-default-features`;
- `cloud-sdk/alloc`;
- `cloud-sdk-hetzner/serde`, which includes its alloc boundary; and
- `cloud-sdk-testkit/alloc`.

Native Linux, Windows, macOS ARM64, and macOS x86-64 jobs run:

```sh
scripts/check_platform_matrix.sh --native
```

That command checks every portable crate with all features and the standard,
deterministic-root, and async reqwest/rustls adapters. Linux remains the
runtime test platform in the main check gate; Windows and both macOS
architectures provide native compile evidence without enabling the separately
scoped FIPS feature.

The FIPS feature has a narrower claim. A dedicated Linux job builds the
Cargo-authenticated bundled AWS-LC-FIPS source and verifies that the provider
and complete client configuration report FIPS operation. The client requires
deployment-managed roots and complete CRLs rather than Linux platform trust
without revocation. No other target and no NIST-validated operating
environment is claimed for this dependency line.

## Default Dependency Proof

The local and CI check gate runs:

```sh
scripts/check_platform_matrix.sh --default-boundary
```

It inspects the normal default-feature workspace graph for all target-specific
dependency branches. Only the five first-party workspace crates and the
admitted `sanitization` package are allowed; every other package fails closed.
Regression tests bind validation to the exact all-target Cargo command and
prove that unknown targets, missing target libraries, extra arguments, and
unlisted dependencies fail closed before a platform claim is accepted.

## Transport Selection

- Linux, Windows, and macOS applications may opt into
  `cloud-sdk-reqwest/blocking-rustls`, `blocking-rustls-webpki-roots`, or
  `async-rustls` subject to their own deployment and trust-store testing. The
  deterministic snapshot mode excludes host private and enterprise roots.
- `blocking-rustls-fips` has repository runtime evidence only on Linux x86-64;
  it requires caller-managed roots and CRLs and is not a compliance or
  cross-platform support claim.
- FreeBSD users may evaluate the reqwest adapter, but this repository does not
  provide a native FreeBSD transport job.
- Android and iOS applications should implement the core blocking or async
  transport contract using a platform-reviewed networking stack.
- Browser and non-browser WASM environments need a WASM-native transport; the
  native reqwest/rustls feature graph is intentionally not enabled.
- Bare-metal targets need an allocator-free or caller-buffer transport suited
  to their network stack. The SDK does not select one.

Target-specific transports must preserve the SDK's bounded response,
credential redaction, timeout, redirect, retry, and cleanup policies. Platform
support does not weaken those security boundaries.
