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

The detector reports added operations, removed operations, changed operation
fingerprints, and changed component schemas. It strips prose-only OpenAPI fields
such as descriptions and examples before hashing so documentation copy changes
do not create release noise.

When an upstream change is accepted, first update the pinned spec hashes in this
document and in the drift checker. Then refresh the fingerprints intentionally:

```bash
scripts/check_hetzner_api_drift.py --fetch --write-lock --accept-lock-refresh
```

The write path verifies fetched spec bytes against the pinned SHA-256 values
before overwriting the fingerprint files. Then update `docs/API_MATRIX.md`,
`docs/SPEC_LOCK.md`, release notes, and the pentest/retest evidence in the same
reviewed source-lock pass.

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
every non-ASCII scalar, including surrogate pairs.

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

## Deferred Scope

Robot Webservice is explicitly deferred until after the Cloud/DNS SDK reaches
1.0. Its source reference is:

- <https://robot.hetzner.com/doc/webservice/en.html>

`v1.1.0` must pin Robot separately before any Robot operation is implemented.
