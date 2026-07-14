# Hetzner API Source Lock

Status: source-locked for `v0.2.0`.

Retrieved: 2026-07-08
Reference page: <https://docs.hetzner.cloud/reference/cloud>
Changelog page: <https://docs.hetzner.cloud/changelog>

## Locked Specs

| API | URL | OpenAPI | Title | Spec Version | Paths | Operations | SHA-256 | Last-Modified | ETag | Content-Length |
| --- | --- | --- | --- | --- | ---: | ---: | --- | --- | --- | ---: |
| `cloud` | <https://docs.hetzner.cloud/cloud.spec.json> | `3.1.2` | `Hetzner Cloud API` | `1.0.0` | 151 | 189 | `9ca6b542a057b002804b9f4f45ccfdb8b9a28c92b7e5bf5ae1b7f46b54fe0093` | `Wed, 08 Jul 2026 11:25:09 GMT` | `W/"34b0fd-19f41797e95"` | 3453181 |
| `hetzner` | <https://docs.hetzner.cloud/hetzner.spec.json> | `3.1.2` | `Hetzner API` | `1.0.0` | 23 | 32 | `f70750016d81c927ddf877e103541c90d3e3372723cdf54e6fd7b2eba4a8108a` | `Wed, 08 Jul 2026 11:25:09 GMT` | `W/"7ecd4-19f41797e96"` | 519380 |

Total source-locked operations: 221 (`cloud`: 189, `hetzner`: 32).

The rendered documentation page configures these machine-readable specs for the
client-side API reference. `cloud.spec.json` covers the Cloud and DNS API
surface. `hetzner.spec.json` currently covers Storage Box operations.

## Drift Detection

Locked operation fingerprints live in `docs/API_FINGERPRINTS.tsv`. Locked
component schema fingerprints live in `docs/API_SCHEMA_FINGERPRINTS.tsv`.

Use the drift detector before endpoint-model work or release prep:

```bash
scripts/check_hetzner_api_drift.py --fetch
```

The detector reports added, removed, deprecated, and changed operations plus
schema-only and source-digest changes. It strips prose-only OpenAPI fields such
as descriptions and examples before hashing so documentation copy changes do
not create semantic release noise. The separately indexed deprecation flag is
excluded from the semantic fingerprint, so a deprecation-only transition and a
simultaneous contract change are classified independently. The complete
maintenance and decision flow is documented in
`docs/API_DRIFT_MAINTENANCE.md`.

Live fetches use Python's default certificate- and hostname-validating TLS
context, require the response to remain at the exact official HTTPS URL without
a redirect, and enforce connection, total-time, and 32 MiB limits. Fetched
documents must be valid UTF-8 JSON objects. A digest mismatch is parsed only to
produce the maintenance drift report and always fails the command; fetched
content is never accepted, compiled, or packaged automatically. Caller-supplied
local documents must match the pinned SHA-256 before JSON parsing.

When an upstream change is accepted after complete source review, update the
pinned spec hashes in this document and both drift scripts. Then refresh the
fingerprints intentionally:

```bash
scripts/check_hetzner_api_drift.py --fetch --write-lock --accept-lock-refresh
```

The write path still requires both explicit acceptance flags, requires fetched
bytes to match the reviewed pins, and does not update source pins. Update
`docs/API_MATRIX.md`, `docs/SPEC_LOCK.md`, release notes, and pentest/retest
evidence in the same reviewed source-lock pass. Use
`docs/API_DRIFT_RELEASE_NOTE_TEMPLATE.md` to record the decision and evidence.

## Changelog Items Considered

- 2025-11-12: Firewall `source_ips` and `destination_ips` stopped accepting
  CIDRs with host bits set on 2025-12-10. Canonical networks and individual
  `/32` or `/128` hosts remain valid.
- 2026-07-08: omitted `ttl` for `POST /zones/{id_or_name}/rrsets/{rr_name}/{rr_type}/actions/change_ttl` is deprecated. Future models must require explicit `ttl` or `null` once the API removal date is reached.
- 2026-07-08: omitted `dns_ptr` for DNS pointer change actions is deprecated for servers, primary IPs, floating IPs, and load balancers. Future models must require explicit `dns_ptr` or `null` once the API removal date is reached.
- 2026-07-01: `datacenter` was removed from Servers and Primary IPs create/update request and response shapes.
- 2026-06-05: Load Balancer Type `deprecated` is deprecated in favor of `deprecation`.
- 2026-06-02: `GET /datacenters` and `GET /datacenters/{id}` are deprecated, with removal announced after 2026-10-01.
- 2026-04-30: resource-local `GET .../actions/{action_id}` lookups are deprecated. Prefer global action lookup or non-deprecated resource action surfaces where available.
- 2026-01-15: Storage Box Subaccount includes a `name` property.

## v0.12.0 DNS TTL Policy

The source-locked Zone create schema permits its default `ttl` field to be
omitted, so `ZoneCreateRequest` retains optional explicit TTL intent. The Zone
`change_ttl` action requires `ttl`, so `ZoneTtlRequest` cannot represent
omission. The 2026-07-08 omitted-TTL deprecation applies to the separate RRSet
`change_ttl` action and remains assigned to `v0.13.0` with RRSet models.

## v0.12.0 DNS TSIG Policy

The source-locked Hetzner schema accepts `md5`, `sha1`, and `sha256` for
secondary-zone TSIG credentials. The SDK deliberately exposes only SHA-256.
RFC 8945 [prohibits HMAC-MD5 use, does not recommend HMAC-SHA1 use, and
recommends HMAC-SHA256](https://www.rfc-editor.org/rfc/rfc8945.html#section-6),
while its [local policy rules](https://www.rfc-editor.org/rfc/rfc8945.html#section-7)
permit stricter rejection. TSIG secrets must decode to at least 32 bytes to
match the SHA-256 output size; callers remain responsible for the RFC's
[CSPRNG generation and two-party scope requirements](https://www.rfc-editor.org/rfc/rfc8945.html#section-8),
protected storage, and rotation.

## v0.13.0 DNS RRSet Policy

The source-locked RRSet surface supports `A`, `AAAA`, `CAA`, `CNAME`, `DS`,
`HINFO`, `HTTPS`, `MX`, `NS`, `PTR`, `RP`, `SOA`, `SRV`, `SVCB`, `TLSA`, and
`TXT`. Mutation actions admit `1..=50` records and identify records by value;
the SDK rejects duplicate values before transport. The create schema requires
a nonempty distinct list but does not publish a numeric maximum; the SDK
deliberately applies the same 50-record request ceiling to create operations as
a conservative resource bound.

The RRSet `change_ttl` request requires its `ttl` property. The SDK therefore
represents only an explicit bounded TTL or explicit JSON `null` inheritance,
closing the 2026-07-08 omitted-field deprecation. Create and add-records retain
an outer optional TTL because omission remains source-valid for those distinct
operations.

Record values are bounded and safely writable as JSON strings, but the SDK
does not normalize every type-specific RDATA grammar. Hetzner remains the
authoritative validator for record semantics. Duplicate detection therefore
uses exact value bytes, matching the source schema's item uniqueness without
incorrectly case-folding case-sensitive RDATA such as `TXT`. Callers that need
semantic uniqueness for domain-name-valued records must canonicalize those
values before constructing `RecordValue` instances.

The per-record and per-request count bounds can still describe a large
aggregate body. The optional serialization and transport layers must enforce a
separate current provider request-body limit before allocation or transmission;
the request-domain bounds are not a transport-size guarantee.

Validated endpoint paths are bounded to 1024 bytes. This covers the complete
path assembled from independently maximum-sized validated Zone and RRSet names,
percent encoding, RR type, and the longest action suffix while retaining a
finite transport-facing path policy.

## v0.14.0 Serde Policy

Serde is optional, enables allocation but not `std`, and remains absent from
the default normal dependency graph. Complete RRSet request structs do not
implement `Serialize`; callers must construct `RrsetRequestBody`, which omits
endpoint selectors and checks a conservative 1 MiB JSON upper bound before the
wrapper becomes serializable. The estimate assumes a JSON serializer may escape
every control or non-ASCII scalar, including surrogate pairs. Control-byte
accounting remains conservative even though current record constructors reject
those bytes before estimation.

The boundary serializes create, labels update, protection, TTL, set-records,
add-records, remove-records, and update-record-comments bodies. Explicit
`RrsetTtl::InheritZoneDefault` serializes as JSON `null`; an absent optional TTL
is omitted only where the source schema permits omission.

Shared action and API error responses deserialize through private wire models.
Known duplicate fields, missing required fields, zero IDs, unknown action
statuses, progress above 100, and control bytes in interpreted response text
are rejected. Unknown response fields are ignored for additive provider
compatibility. `Cow` preserves borrowing for ordinary strings and owns strings
that require JSON unescaping. Required nullable action fields distinguish an
explicit JSON `null` from an omitted field.

Callers must construct `ResponseBytes` before invoking their selected Serde
format parser. It caps raw input at 8 MiB before parser allocation. Parsed
action responses additionally admit at most 256 related resources and bound
interpreted command, timestamp, resource-type, error-code, and error-message
text. Raw response bytes and API error messages are redacted from `Debug`.

No other request body or resource response is Serde-enabled in this release.
Adding one requires an explicit source-locked mapping and adversarial fixtures;
blanket derives over validated request or path types are prohibited.

## v0.18.0 Pagination, Action, And Rate-Limit Policy

Both pinned official specifications document one-based `page` values, a
default `per_page` of 25, and a maximum of 50 unless an operation explicitly
states otherwise. Paginated JSON object responses include
`meta.pagination`; `previous_page`, `next_page`, `last_page`, and
`total_entries` are required nullable fields. The SDK rejects omitted fields,
zero pages, page sizes outside `1..=50`, non-adjacent or repeated navigation, a
next page beyond the known last page, and empty pages that still advertise a
continuation. Advertised previous and next pages must equal `page - 1` and
`page + 1` respectively, with checked arithmetic. A known last page must agree
with terminal state. Decoded entries cannot exceed `per_page`; when
`total_entries` is present, the current page count and continuation state must
match it exactly. The cursor binds the caller's requested `per_page` value and
the first accepted response's nullable `total_entries` and `last_page` values
for the entire traversal. Any change fails before advancing and requires a new
traversal, preventing page-size changes or concurrent snapshot drift from
silently skipping resources. A caller-selected hard page limit remains
mandatory even when the provider supplies a last page.

Actions remain `running` until the provider reports `success` or `error`.
Polling frequency is intentionally caller-owned because the official source
warns against frequent requests. The SDK rejects zero-delay polling and
progress regression, propagates the optional validated provider error on a
terminal failure, and never owns a sleep, retry loop, clock, deadline, or
executor. Terminal success or failure takes precedence over non-authoritative
progress telemetry so the provider's final result is not discarded.

The official response metadata uses the complete `RateLimit-Limit`,
`RateLimit-Remaining`, and `RateLimit-Reset` header set. Reqwest adapters parse
only unsigned decimal values, require each of the three headers exactly once
when any is present, reject duplicates, zero limits, and remaining values above
the limit, and expose the result through `TransportResponse`. They do not infer
a retry delay or automatically replay a request.

## v0.19.0 Live Smoke Policy

The live harness covers only source-locked `GET` operations for locations,
server types, load balancer types, ISOs, public system images, and pricing.
List requests use the source-locked `per_page` parameter and strict shared
`meta.pagination` parser. Pricing validates its documented singleton envelope.
The harness does not infer API coverage from a successful smoke run and does
not replace operation/schema fingerprint drift checks.

The authenticated origin is fixed to the source-locked Cloud API v1 URL.
Response bodies are bounded, parsed only after HTTP success, cleared after each
probe, and never logged. Mutation operations and configurable origins are not
part of this harness.

## v0.20.0 Platform Evidence Policy

Platform claims distinguish portable crate compilation from native transport
support. The portable allowlist contains representative Linux, Windows,
FreeBSD, macOS, Android, iOS, WebAssembly, and bare-metal targets. Every target
checks default no_std crates and allocation-bearing core, testkit, and Hetzner
Serde combinations.

The optional reqwest/rustls graph is native evidence only on Linux, Windows,
macOS ARM64, and macOS x86-64. Cross-compilation never upgrades a platform to a
native transport claim. FreeBSD transport is best effort; Android, iOS, WASM,
and bare-metal users must supply a target-native implementation of the core
transport contract.

The default dependency boundary rejects activation of network, TLS, runtime,
socket, and operating-system abstraction crates. New targets or transport
claims require an explicit allowlist, CI, documentation, and release-evidence
change rather than automatic host inference.

## v0.21.0 Documentation Evidence Policy

The v0.21 examples exercise only already source-locked endpoint methods,
paths, queries, request models, pagination metadata, and action responses. They
do not expand the API coverage claim or change provider behavior.

Executable examples are compiled as Cargo example targets. Serde-dependent
pagination and action examples declare their required feature explicitly.
Publishable crate READMEs remain rustdoc inputs and run under the all-feature
workspace doctest gate. Repository-local Markdown and HTML link targets are
validated without fetching unauthenticated external content.

## v0.22.0 Fuzz Evidence Policy

The v0.22 fuzz harness exercises only source-locked request construction,
validation, pagination, action, and response-envelope behavior. A successful
campaign is evidence for explored inputs, not a new API coverage claim and not
proof that defects are absent.

Nightly Rust, cargo-fuzz, libfuzzer-sys, generated corpora, and crash artifacts
remain outside every published crate and supported stable compiler graph. The
excluded `fuzz/` package has a pinned toolchain, independent lockfile, Cargo
Deny and RustSec checks, and a separate SBOM. Committed seeds are synthetic and
named; they must not contain credentials, production responses, customer data,
or billable resource identifiers.

CI and the release gate build every target and replay bounded copies of the
reviewed seeds. Longer campaigns and crash minimization remain explicit local
operations. Every confirmed defect must become a deterministic regression test
in the owning published crate before release.

## Pending Pre-1.0 Scope

Robot Webservice is required for the full Hetzner 1.0 SDK. Its source
reference is:

- <https://robot.hetzner.com/doc/webservice/en.html>

`v0.31.0` must pin Robot separately before any Robot operation is implemented.
The lock must distinguish active operations from deprecated alternatives and
exclude the deprecated Robot Storage Box family, whose supported replacement
is already tracked by the Console Storage Box source.
