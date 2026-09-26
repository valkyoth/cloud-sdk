# crates.io Live Read Qualification

The unpublished `live_read` integration target is operator-only. It is ignored
by normal workspace tests, requires `blocking-rustls`, and refuses execution if
any supported CI marker is present (even if its value is `false`). CI runs only
offline regressions and deliberate rejection probes. No token is configured
in a workflow, repository file, command-line argument or credential environment
variable by this harness.

## Anonymous Run

From the repository root, set a truthful identifying user agent with contact:

```sh
CLOUD_SDK_CRATESIO_LIVE=anonymous \
CLOUD_SDK_CRATESIO_USER_AGENT='your-tool/1 (your-contact)' \
cargo test --locked -p cloud-sdk-cratesio --no-default-features \
  --features blocking-rustls --test live_read -- --ignored --exact live_read
```

The run makes at most two production GET requests: site metadata, followed by
one keyword page with `per_page=1`. It does not paginate or fetch returned URLs.
There is a one-second quiet interval after the first exchange; the SDK's shared
gate additionally enforces admission and provider delays. A failure stops the
run. There are no retries, redirects, cookies, publication or mutation calls.
Separate processes sharing an egress address must coordinate externally.

## Isolated-Account Token Run

Use a dedicated test account with no organization memberships or ownership of
valuable crates and a short-lived token. Prefer the narrowest available scope.
The token may have write permissions: the read-only boundary here is the
harness's dispatch allowlist, not a claim about registry token permissions.
Never substitute a production-account credential. Provide the token from a
protected file or secret-manager pipe through redirected stdin:

```sh
CLOUD_SDK_CRATESIO_LIVE=token \
CLOUD_SDK_CRATESIO_TOKEN_ACK=isolated-account-read-requests \
CLOUD_SDK_CRATESIO_USER_AGENT='your-tool/1 (your-contact)' \
cargo test --locked -p cloud-sdk-cratesio --no-default-features \
  --features blocking-rustls --test live_read -- --ignored --exact live_read \
  < /absolute/path/to/private-test-account-token
```

The acknowledgement confirms operator intent, not account isolation or token
permissions; neither can be proved from an opaque token. The token is read
into fixed guarded mutable storage, at most 1,027 bytes including overflow
detection and an optional terminal LF/CRLF. Extra lines, whitespace and oversized
credentials fail. The input buffer clears on all exits. The owned API token and
request/response scratch follow the existing sanitization policy. Stdin must
reach EOF; local secret-provider stalls are not covered by HTTP deadlines.
Shell history, file permissions, producer buffers and operating-system copies
remain operator responsibilities. Never place a real token in an example or log.

The first request is anonymous site metadata. The second is exactly
`GET /api/v1/crates?following=yes&per_page=1`, with the production-bound raw API
token. A dispatch wrapper independently rejects other methods, routes, bodies,
inline authorization/cookies and non-production origins. It has no upload
executor implementation. Results and credentials are not printed; errors are
static, payload-free messages. No account or registry state is modified.

## Evidence And Limits

`python3 scripts/check_cratesio_live_read.py` builds the target, runs its offline
tests and launches the real ignored-test entry point under ten combinations of
CI marker and mode. Each must reject before credential input or networking.
`scripts/checks.sh` includes this proof. Runtime fixture tests cover request
allowlisting, zero dispatch on rejection, one dispatch for an admitted read,
origin binding, credential lengths/line endings and reader failures.

Anonymous live reads passed on 2026-09-26 against production using the bundled
adapter and identifying repository user agent. Token-backed evidence is recorded
separately in the qualification ledger; CI never receives the test token.
These two reads do not qualify all provider endpoints, live mutations, all
platforms, or arbitrary custom adapters.

## Publication Cleanup Boundary

Do not publish a disposable crate through this harness. The upstream
[crate deletion handler](https://github.com/rust-lang/crates.io/blob/main/src/controllers/krate/delete.rs)
requires browser-cookie authentication, which this SDK deliberately excludes.
A full-permission API token cannot perform that cleanup. Yanking does not delete
the archive. Normal CI publication and ownership tests remain mock-only. A
separate operator-authorized disposable publication may use manual website
cleanup; it must not expand this read harness's dispatch allowlist. Results and
any outstanding cleanup are recorded in [qualification](CRATESIO_QUALIFICATION.md).
