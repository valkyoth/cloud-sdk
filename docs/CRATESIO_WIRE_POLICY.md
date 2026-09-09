# crates.io Wire Contract

Status: unreleased `1.1.0`, Commit 6 incremental pentest and remediation retest
passed. The [permanent report](../security/pentest/cratesio-commit-6.md) records
the reviewed range. GitHub CI and CodeQL passed on evidence checkpoint
`f079a65c`. No tag or publication is authorized.

## Source Contract

The [source lock](CRATESIO_SOURCE_LOCK.md) binds the public OpenAPI, Cargo
Registry Web API, and crates.io data-access policy. Cargo explicitly describes
`{"errors":[{"detail":"..."}]}` as an error envelope even on HTTP 200:
[Cargo protocol](https://doc.rust-lang.org/cargo/reference/registry-web-api.html).
The locked access policy requires identifying user agents and at most one API
request per second. Prefer the index, static downloads, feeds or dumps for bulk
data; see the [access policy](https://crates.io/data-access).

These requirements are provider-owned. Parser and HTTP-date mechanics are
provider-neutral. No automatic retry is authorized by a status or Retry-After.

## Response Admission

`wire::JsonResponsePolicy` binds an exact body-bearing 2xx status and a nonzero
caller limit up to 8 MiB. Apply `maximum_bytes()` to `ResponseBuffer` before
transport starts; admission independently rechecks the completed length.
The policy consumes committed storage, so success, rejection, drop and unwind
retain the core's full response/header cleanup. No external slice can masquerade
as a committed response.

The media policy requires `application/json` (case-insensitive), optionally
one `charset=utf-8` parameter (quoted UTF-8 is allowed). Missing/invalid media,
other charsets, extra or duplicate parameters, and non-identity content encoding
fail closed. Transport header collection rejects duplicate header names.
Empty-body/204/205 and binary-download contracts are separate future operation
policies, not silently admitted by the JSON path.

The complete JSON document is checked before a success wrapper is exposed.
The root must be an object. Unknown fields are accepted but fully checked,
including decoded duplicate keys, UTF-8, escapes, trailing data and resource
limits. A top-level `errors` member must contain 1 through 256 objects, each
with a string `detail`; malformed or empty error collections fail closed.
An error envelope overrides every successful HTTP status, including 200.
HTTP 4xx/5xx cannot become success even without the Cargo envelope; a valid
object in that case yields a provider error with zero validated detail entries.
Malformed error responses remain wire-policy errors, not success.

`ProviderError` exposes only HTTP status, error count and parsed Retry-After.
Provider detail text is discarded, not stored in public diagnostics or an
ordinary owned string. Public error chains do not expose lower-level payloads.
429 and 503 are classified explicitly, including missing Retry-After; an invalid
present delay is a wire error. Relative seconds and standard HTTP dates reuse
the neutral parser. Caller-supplied wall time resolves obsolete date years;
actual delays must be bounded by the caller before scheduling.

`JsonSuccess::visit` replays already-admitted events to a trusted resource
decoder and clears the wire buffer afterwards. Visitors must protect retained
copies. A visitor stop remains distinct from completion. This foundation does
not yet implement operation-specific success models or token-response models.

## Parser Continuity

The existing incremental decoder moved from Hetzner into
`cloud-sdk::incremental_json` under `alloc`; Hetzner's public serde exports
remain compatible. The same tests and fuzz entry point still exercise the
same implementation. Core's allocation feature now activates protected string
storage; `serde_json` is only a dev dependency for the independent grammar
oracle. No new external package, runtime parser, or default allocation enters
the provider graph.

The shared hard ceilings remain 8 MiB input, depth 64, 65,536 tokens and total
fields, 4,096 fields per object, 1 MiB per decoded string, 128 number bytes and
six exponent digits. Callers can lower these limits. Parser staging uses
fallible protected allocation; callback copies, deliberate leaks and process
abort remain outside cleanup guarantees.

## Scheduling And Identity

`IdentifyingUserAgent` requires explicit `product/version (contact)` text up to
256 printable ASCII bytes. Contact is an operator-supplied email or HTTPS URL;
syntax does not prove its ownership or reachability. Controls, CRLF, Unicode,
missing identification and oversized values fail. Debug redacts the value.
This public identity must never contain a secret.

Email syntax uses a restricted ASCII dot-atom profile: nonempty atoms separated
by single dots and at most 64 local-part bytes. Quoted local parts, address
literals and internationalized text are intentionally unsupported. Email
domains and HTTPS hosts require dotted DNS names with nonempty labels of at
most 63 bytes, alphanumeric ends and only interior alphanumeric/hyphen bytes.
The full 256-byte user-agent ceiling still applies. This profile follows
[RFC 5322 section 3.2.3](https://www.rfc-editor.org/rfc/rfc5322.html#section-3.2.3)
and the domain and local-part constraints in
[RFC 5321](https://www.rfc-editor.org/rfc/rfc5321.html#section-4.1.2);
it does not attempt to accept every legal mailbox representation.

`ApiSchedule` is a non-cloneable, clock-free state machine. All workers must
share one synchronized instance and one trusted monotonic clock. It admits
only immediate starts, never redeemable future permits. Failed attempts consume
their interval; rollback and overflow reject; deferral cannot shorten a wait.

Under `std`, `OfficialApiGate` shares a process-wide mutex and monotonic clock
across instances, origins and credentials. It conservatively serializes whole
blocking attempts and enforces one second of quiet time after completion.
Busy, early or poisoned calls fail rather than sleep. Reentrant use fails;
panics poison the gate. No global reset API exists.

The callback is a trusted blocking adapter extension: perform exactly one
immediate exchange, apply the supplied user agent, disable redirects/retries,
and do not return a future/deferred task. It does not verify endpoint or
credential binding on its own; combine those separately reviewed boundaries.
It is not a finished official client, and cannot constrain deliberately
bypassing adapters, other processes, or other applications sharing an egress IP.
Those require external coordination. Async execution and operation clients
must integrate admission at their own later checkpoints without holding an
ordinary mutex across await. Static downloads are outside this API gate.

## Verification

Regression tests cover exact success/status/media matrices, unknown nested
fields, malformed and duplicate envelopes, 200-with-errors, error-detail shape,
byte limits, cleanup, invalid/missing/overflowing Retry-After and HTTP dates,
user-agent injection, monotonic rollback/overflow, failed attempt accounting,
malformed contact atoms and DNS labels, exact local/label/header length bounds,
reentrant process-wide gate use, concurrent scheduler contention and redacted
diagnostics. Moved parser tests retain split-boundary, grammar-oracle, resource
bound, panic-poison and staging-cleanup checks. The existing incremental fuzz
target retains its Hetzner compatibility import.

Local qualification on 2026-09-08 passed `scripts/checks.sh`, the full Rust
1.92.0 through 1.98.1 matrix, all configured portable targets plus native
checks, all six package graphs, SBOM freshness, fresh RustSec scans of four
lockfiles, cargo-deny, direct-pin/tool freshness and live crates.io drift.
These are local qualification results. The separate pentest retest passed;
GitHub CI and CodeQL also passed on the Commit 6 evidence checkpoint.
