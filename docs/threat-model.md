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
- response identifiers, headers, cursor/link staging, or decoder scratch
  surviving response-policy and decode lifetimes;
- unsafe JSON interpolation, oversized request bodies, duplicate response
  fields, or deserialization around validated constructors;
- API drift from Hetzner documentation;
- malicious or compromised third-party dependency.
- test fixtures accidentally performing network or filesystem operations;
- mock mismatch diagnostics disclosing request targets or bodies.
- lifecycle telemetry exposing generic errors, request-ID bytes, provider
  messages, resource identifiers, cursors, targets, headers, or bodies;
- observer failures changing request results, hidden observer locks causing
  reentrancy deadlocks, or unbounded callbacks harming availability;
- authority replacement or path-normalization confusion when a future adapter
  combines untrusted request targets with an authenticated provider base URL;
- out-of-bounds response lengths from buggy or malicious safe transports;
- credential forwarding through redirects, proxies, normalized authorities,
  retries, referers, or environment-derived routing;
- decompression bombs, unbounded response reads, and timeout-free blocking;
- secret copies retained in adapter-owned allocation after request completion;
- credential rotation races applying a partial token, changing an in-flight
  request, holding a secret lock across I/O, or retaining retired token storage;
- downgraded or incomplete provider policy allowing a token to cross provider,
  service, endpoint, audience, account, or tenant boundaries;
- slow or foreign-store credential refresh overwriting another lifecycle, a
  newer rotation, or a successful refresh;
- custom endpoint drift sending valid credentials to a changed host, subdomain,
  port, scheme, or base path;
- compromised or attacker-extended host trust stores silently validating a
  hostile TLS endpoint;
- absent, incomplete, stale, or unauthenticated CRLs allowing a revoked cloud
  endpoint certificate to remain usable;
- downstream Cargo feature unification silently enabling a different DNS
  resolver or broader HTTP protocol parser;
- async cancellation exposing partially initialized response data or leaving
  adapter-owned secret response copies in memory;
- a prepared operation admitting a smaller response window while residual
  bytes from an earlier larger response remain in the caller buffer tail;
- an async adapter silently owning a runtime or introducing one into default,
  provider, or testkit graphs;
- duplicate, malformed, or partial rate-limit headers causing incorrect request
  budgets;
- repeated, contradictory, or empty non-terminal pages causing loops or
  incomplete resource traversal;
- page-size, total-entry, or last-page changes combining different pagination
  snapshots and silently skipping resources;
- opaque cursor cycles or digest collisions bypassing traversal limits;
- provider next links changing scheme, authority, path, method, operation, or
  raw query semantics and sending a request outside the original boundary;
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
- copied, replayed, stale, expired, price-mismatched, or request-mismatched
  authority causing an unintended state change or charge;
- treating uncertain delivery as not sent and repeating an operation without
  provider-specific reconciliation;

## Controls

- no_std default SDK crate with no transport or token storage;
- internal endpoint module boundaries plus optional adapter crates;
- dependency review before admission;
- cargo-deny and cargo-audit;
- explicit API source lock before endpoint implementation;
- mock and adversarial testkit before transport helpers are stabilized;
- dynamic testkit scenarios serialize request-dependent selection, bound
  successful observations in caller-owned atomic slots, retain lengths and
  finite classifications only, validate complete pagination/action scripts,
  and exercise endless zero-progress, partial-I/O, and exact injected faults;
- SHA256-only TSIG policy, minimum secret size, redacted output, and no ordinary
  equality on secret-bearing types;
- provider-neutral volatile caller-buffer guards and no ordinary equality on
  Storage Box passwords, private keys, or containing request types;
- structural RRSet names/types, explicit TTL intent, bounded unique record
  mutations, and atomic JSON-string output;
- request paths, queries, and JSON bodies use immutable snapshot
  measure/write/verify encoding with checked arithmetic and aggregate caps;
  undersized output is unchanged and observed pass drift clears the exact
  admitted destination;
- guarded preparation storage keeps target and body cleanup ownership alive
  through transport use and clears both complete caller regions on drop;
- checked Serde request wrappers, aggregate body limits, private response wire
  models, post-parse validation, and default dependency-graph isolation;
- no_std mock transport with borrowed expectations, atomic bounded fixture
  writes, payload-free errors, and redacted request/response diagnostics;
- opt-in copy-only lifecycle events admit finite categories plus bounded public
  provider taxonomy, status, and request-ID disposition only; core never logs,
  retains observer state, or exposes discarded-ID presence; observer return
  errors are isolated from execution and no SDK-owned observer lock exists;
- origin-form targets reject scheme-relative prefixes, backslashes, fragments,
  controls, spaces, and non-ASCII before an adapter can attach credentials;
- transports receive only a sealed writer over the admitted caller-owned
  prefix; they commit status, bounded metadata, and a checked initialized
  length but cannot substitute external or static response bytes;
- core distinguishes absent response content type from malformed syntax or
  invalid UTF-8 and rejects every present parse failure before provider
  decoding, including under optional and forbidden policies;
- a cleanup-owning response guard uses one audited volatile primitive to clear
  the complete caller body and header buffers before admission and on every
  ordinary exit; temporary headers, request identifiers, cursor/link staging,
  and decoder scratch have the same cleanup owner; borrowed decoding cannot
  escape;
- provider operation metadata explicitly retains, protects, or discards
  request identifiers; sensitive bytes remain in stable caller-owned storage,
  explicit retention copies directly into another stable caller destination,
  and failed transfer clears source and destination;
- optional platform sanitizers are additive between mandatory core clears, so
  a no-op, recontaminating, or panicking hook cannot weaken the final clear;
- optional production blocking and async transports require exact HTTPS
  authority, rustls with TLS 1.2 minimum, explicit bounded timeouts, no
  redirects, retries, proxies, referers, or decompression, and caller-bounded
  responses;
- Send and local async contracts expose only non-committing response staging;
  SDK drivers commit after `Ready(Ok)`, cancellation leaves no committed
  response, remains conservatively possibly sent, and never proves a mutation
  can be repeated;
- transport sends use shared references; cloneable reqwest clients take an
  atomic token snapshot under a short-lived lock, release it before I/O or
  `.await`, retain old snapshots only for in-flight requests, and sanitize
  retired adapter-owned storage after the last snapshot;
- mutable and guarded token ingestion clears the complete source on success or
  rejection, while rejected rotation leaves the active token unchanged;
- bearer and Basic credentials bind to immutable provider, service, normalized HTTPS
  endpoint, audience, account, and tenant scope; provider or operation policy
  explicitly requires, permits, or forbids every field before authorization
  header construction, and authenticated clients expose no policy-free
  transport implementation;
- Basic credentials reject ambiguous username delimiters, controls,
  non-ASCII input, and individual or aggregate overflow; mutable sources,
  intermediate `user:password` bytes, encoded storage, and header copies have
  cleanup ownership;
- v2 canonical signing input length-frames provider, service, normalized
  endpoint with tagged canonical host identity, optional scope, key ID,
  distinct digest and signature algorithms, exact method/target, and selected
  headers; equivalent IPv6 spellings encode identically; construction rejects
  a hasher/context digest-algorithm mismatch, then hashes and retains the exact
  request body, while validated signed output retains the same request and
  clears on failure, unwind, or drop; providers and callers retain algorithm
  implementations, keys, clocks, nonces, replay state, and verifier policy;
- credential generations advance without wrapping; external refresh receives
  an opaque compare-and-swap handoff so stale completion cannot replace newer
  state, while acquisition, expiry, clocks, tasks, and secret stores remain
  caller-owned;
- credential-bound transports report immutable normalized endpoint identity so
  the Hetzner provider can verify exact scheme, host, effective port, and base
  path for both official v1 API families before execution;
- provider and service routing metadata uses distinct bounded canonical IDs;
  provider crates own open markers, services declare their provider, and no
  central enum or catch-all identity can silently absorb a new API surface;
- poisoned credential locks recover while holding a guard over one complete
  token `Arc`, preventing permanent failure across every client clone;
- standard transports use platform trust stores explicitly; FIPS transport
  requires deployment-managed roots and complete CRLs, checks the full chain,
  denies unknown revocation status, and enforces CRL expiration;
- both clients force HTTP/1 and disable Hickory DNS; a locked external fixture
  tests both adapters with downstream reqwest HTTP/2 and Hickory features unified;
- the core async trait and testkit are executor-neutral; only the optional
  reqwest adapter requires caller-provided Tokio execution;
- async response data stays in caller-bounded sanitized temporary storage and
  reaches the cleared caller buffer only after complete success;
- adapter-owned bearer, Basic, authorization-header, request-body, and async response
  allocations are redacted or cleared through the provider-neutral
  sanitization boundary;
- rate-limit headers are parsed as a strict all-or-none decimal set, each field
  must occur exactly once, and values are validated before metadata is exposed;
- pagination separates numbered, offset, cursor, marker, and provider-link
  state; requires hard request, item, state, and history budgets; keeps
  snapshot policy explicit, retains exact bounded snapshot identities, and
  keeps progression transactional; clears non-Copy opaque state on drop;
  fails closed on exact cursor cycles and digest collisions; and binds raw
  provider links to the endpoint, method, operation, and exact path, coupling
  endpoint verification and authenticated dispatch through one transport
  object without a redirectable callback or structured-query recomposition;
  validation and transport failures share one flattened error result, with
  transport details redacted from diagnostics;
- release drift fetches require exact non-redirecting HTTPS URLs under the
  default validating TLS context, bounded downloads, and pinned digest
  verification before parsing;
- pager and action drivers admit exactly one response per request and remain
  non-cloneable; pagination retains transactional hard budgets; action polling
  rejects zero-delay loops, bounds observations and cumulative delay, separates
  cancellation from backoff, admits progress reset only under bounded explicit
  policy, and uses monotonic observations exclusively for local elapsed state;
  provider wall-clock rollback cannot extend execution and generic policy
  failures are redacted;
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
- complete release and security-delta review before every tag, with cumulative
  pentest reports every fifth pre-1.0 minor and exceptional assessments for
  material trust-boundary changes.
- direct state-changing prepared execution fails closed; non-`Copy` direct
  permits and explicitly shared atomic permits bind one exact versioned plan,
  endpoint, scope, validity interval, attempt budget, replay policy,
  idempotency identity, and cost ceiling; stale recovery generations,
  backward clocks, no-op plans, mismatched fingerprints, concurrent spend,
  and uncertain delivery fail closed pending operation-specific reconciliation;
  authenticated request construction/extraction is internal, permit attempts
  expose no reusable prepared request, exact confirmed endpoints are rechecked
  at dispatch, read-only direct execution requires both read-only metadata and
  a `GET` or `HEAD` wire method, and caller-owned clocks enforce exclusive
  expiry immediately before blocking transport access or on first async poll.
- provider-generic client execution owns no queue, executor, clock, retry loop,
  or storage; each in-flight request consumes one bounded atomic slot and four
  caller-owned uniquely borrowed buffers; cancellation clears complete target,
  request, response, and header regions before releasing admission; checked
  decoders cannot recover the already-sent prepared request, and the direct
  kernel path cannot bypass state-change or cost permits.

Cleanup does not cover process abort, `mem::forget` or deliberately leaked
guards, immutable/external copies, TLS and allocator internals, kernel/device
buffers, swap, crash dumps, or remote systems.
