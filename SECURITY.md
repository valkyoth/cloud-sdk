# Security Policy

`cloud-sdk` is security-sensitive infrastructure software. Treat
authentication, bearer tokens, request signing if ever added, endpoint parsing,
pagination, action polling, retry logic, rate-limit handling, TLS transport,
DNS operations, certificate operations, CI, release scripts, and dependency
updates as high-risk until reviewed and tested.

## Routine Checks

Run these regularly and before releases:

```bash
scripts/checks.sh
scripts/check_latest_tools.sh
scripts/release_0_1_gate.sh
cargo deny check
cargo audit
scripts/generate-sbom.sh
```

GitHub Actions run CI. GitHub CodeQL default setup should be enabled in the
repository security settings. Do not add an advanced CodeQL workflow while
default setup is active.

## Dependency Policy

The dependency policy lives in `deny.toml`. Unknown registries and git sources
are denied by default. Git dependencies require exact `rev` pinning and a
documented exception before use.

New third-party crates require:

- current version check before admission;
- license and maintenance review;
- feature impact review;
- no hidden `std`, network, TLS, native-code, async-runtime, filesystem, or
  secret-storage expansion in core crates;
- tests for the behavior being admitted;
- `cargo deny check` and `cargo audit` evidence.

## Secret Policy

The default SDK must not store API tokens. Callers provide authorization at the
transport boundary. Optional transport adapters must redact and sanitize their
owned secret storage, while callers remain responsible for original token and
response buffers. Any future secret-bearing helper must be optional, reviewed,
tested, and isolated from the default no_std graph.

## Reporting

Do not publish exploitable security details before a fix is available. Open a
private security advisory or contact the maintainers directly once the public
repository security channels are configured.
