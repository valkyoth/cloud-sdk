# Threat Model

## Assets

- Hetzner API tokens supplied by callers.
- Cloud infrastructure state.
- DNS zone and RRSet state.
- DNS TSIG shared secrets and zonefile contents.
- Storage Box passwords, certificate private keys, and SSH key metadata.
- Local CI and release credentials.

## Primary Risks

- token exposure through logs, debug output, examples, test fixtures, or panic
  messages;
- accidental hidden network or secret-storage dependency in default no_std
  crates;
- incorrect pagination causing missing or repeated infrastructure operations;
- incorrect action polling causing premature success reports;
- rate-limit mishandling that triggers denial of service or retry storms;
- DNS record mutation mistakes;
- RRSet widening, duplicate values, ambiguous TTL inheritance, or unsafe RDATA
  interpolation;
- weak, downgraded, exposed, or variable-time-compared TSIG secrets;
- password, certificate, API error, or SSH key redaction failures;
- secret remnants in caller-owned request buffers or variable-time secret
  comparisons;
- unsafe JSON interpolation, oversized request bodies, duplicate response
  fields, or deserialization around validated constructors;
- API drift from Hetzner documentation;
- malicious or compromised third-party dependency.
- test fixtures accidentally performing network or filesystem operations;
- mock mismatch diagnostics disclosing request targets or bodies.
- authority replacement or path-normalization confusion when a future adapter
  combines untrusted request targets with an authenticated provider base URL;
- out-of-bounds response lengths from buggy or malicious safe transports;
- credential forwarding through redirects, proxies, normalized authorities,
  retries, referers, or environment-derived routing;
- decompression bombs, unbounded response reads, and timeout-free blocking;
- secret copies retained in adapter-owned allocation after request completion;
- compromised or attacker-extended host trust stores silently validating a
  hostile TLS endpoint;
- absent, incomplete, stale, or unauthenticated CRLs allowing a revoked cloud
  endpoint certificate to remain usable;
- downstream Cargo feature unification silently enabling a different DNS
  resolver or broader HTTP protocol parser;
- async cancellation exposing partially initialized response data or leaving
  adapter-owned secret response copies in memory;
- an async adapter silently owning a runtime or introducing one into default,
  provider, or testkit graphs;
- duplicate, malformed, or partial rate-limit headers causing incorrect request
  budgets;
- repeated, contradictory, or empty non-terminal pages causing loops or
  incomplete resource traversal;
- page-size, total-entry, or last-page changes combining different pagination
  snapshots and silently skipping resources;
- zero-delay action policies causing busy polling, or terminal provider errors
  being discarded;
- live-test credentials leaking through shell history, environment dumps,
  Cargo build-time processes, symlinked or permissive files, response logging,
  or configurable origins;
- live-smoke artifact or adjacent digest substitution from a writable directory,
  caller-CWD path confusion, or replacement between hashing and execution;
- an accidentally enabled live or destructive test creating billable resources
  in CI or a production project;
- fuzz seeds, generated corpora, or crash artifacts capturing credentials,
  production responses, or private infrastructure data;
- fuzz-only nightly, native build, or sanitizer dependencies leaking into a
  published or default SDK graph;

## Controls

- no_std default SDK crate with no transport or token storage;
- internal endpoint module boundaries plus optional adapter crates;
- dependency review before admission;
- cargo-deny and cargo-audit;
- explicit API source lock before endpoint implementation;
- mock and adversarial testkit before transport helpers are stabilized;
- SHA256-only TSIG policy, minimum secret size, redacted output, and no ordinary
  equality on secret-bearing types;
- provider-neutral volatile caller-buffer guards and no ordinary equality on
  Storage Box passwords, private keys, or containing request types;
- structural RRSet names/types, explicit TTL intent, bounded unique record
  mutations, and atomic JSON-string output;
- checked Serde request wrappers, aggregate body limits, private response wire
  models, post-parse validation, and default dependency-graph isolation;
- no_std mock transport with borrowed expectations, atomic bounded fixture
  writes, payload-free errors, and redacted request/response diagnostics;
- origin-form targets reject scheme-relative prefixes, backslashes, fragments,
  controls, spaces, and non-ASCII before an adapter can attach credentials;
- transport responses borrow only the initialized slice of the caller-owned
  buffer instead of trusting an independently reported numeric length;
- optional production blocking and async transports require exact HTTPS
  authority, rustls with TLS 1.2 minimum, explicit bounded timeouts, no
  redirects, retries, proxies, referers, or decompression, and caller-bounded
  responses;
- standard transports use platform trust stores explicitly; FIPS transport
  requires deployment-managed roots and complete CRLs, checks the full chain,
  denies unknown revocation status, and enforces CRL expiration;
- both clients force HTTP/1 and disable Hickory DNS; a locked external fixture
  tests both adapters with downstream reqwest HTTP/2 and Hickory features unified;
- the core async trait and testkit are executor-neutral; only the optional
  reqwest adapter requires caller-provided Tokio execution;
- async response data stays in caller-bounded sanitized temporary storage and
  reaches the cleared caller buffer only after complete success;
- adapter-owned bearer, request-body, and async response allocations are
  redacted or cleared through the provider-neutral sanitization boundary;
- rate-limit headers are parsed as a strict all-or-none decimal set, each field
  must occur exactly once, and values are validated before metadata is exposed;
- pagination requires a caller-selected hard page limit, exact expected-page
  transitions, adjacent advertised navigation, known-last terminal coherence,
  caller-bound page size, first-response total/last snapshot, bounded entry
  counts, total-entry reconciliation, nonempty continuation pages, and
  provider-validated navigation; snapshot changes require a new traversal;
- release drift fetches require exact non-redirecting HTTPS URLs under the
  default validating TLS context, bounded downloads, and pinned digest
  verification before parsing;
- action polling rejects running progress regression and zero-delay loops,
  gives terminal status precedence over progress telemetry, preserves provider
  failures, and delegates delay, timeout, and cancellation to caller policy
  without owning a clock or retry loop;
- the live harness is ignored by default; its repository-anchored clean-commit
  build phase rejects credential variables and produces only untrusted staging;
  an administrator installs the executable and runtime into root-owned
  non-writable paths; the authenticated runtime validates ownership, modes,
  parent directories, link count, manifest, and SHA-256 through open file
  descriptors and executes the verified descriptor; it invokes no Cargo or
  build tooling, requires an exact read-only marker and private token-file path,
  fixes the authenticated origin, rejects destructive opt-in, bounds and clears
  source/response buffers, issues only typed catalog GET requests, and emits no
  response bodies or resource IDs;
- destructive live execution is absent; its documented future plan requires a
  separate command, dedicated project and token, unique prefix, explicit cost
  review, cleanup on every path, and post-run inventory verification;
- the fuzz harness is excluded and non-published, uses a separate lockfile,
  calls only pure SDK boundaries, admits synthetic seeds, rejects tracked
  generated corpora/artifacts, bounds smoke inputs and time, and compiles under
  a separately pinned nightly without changing stable crate support;
- pentest report before every tag.
