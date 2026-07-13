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
- optional production blocking transport requires exact HTTPS authority,
  rustls with TLS 1.2 minimum, explicit bounded timeouts, no redirects,
  retries, proxies, referers, or decompression, and caller-bounded responses;
- platform trust-store use is explicit; v0.16 does not claim root, certificate,
  or public-key pinning;
- adapter-owned bearer and request-body allocations are redacted and cleared
  through the provider-neutral sanitization boundary;
- pentest report before every tag.
