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

- 2026-07-08: omitted `ttl` for `POST /zones/{id_or_name}/rrsets/{rr_name}/{rr_type}/actions/change_ttl` is deprecated. Future models must require explicit `ttl` or `null` once the API removal date is reached.
- 2026-07-08: omitted `dns_ptr` for DNS pointer change actions is deprecated for servers, primary IPs, floating IPs, and load balancers. Future models must require explicit `dns_ptr` or `null` once the API removal date is reached.
- 2026-07-01: `datacenter` was removed from Servers and Primary IPs create/update request and response shapes.
- 2026-06-05: Load Balancer Type `deprecated` is deprecated in favor of `deprecation`.
- 2026-06-02: `GET /datacenters` and `GET /datacenters/{id}` are deprecated, with removal announced after 2026-10-01.
- 2026-04-30: resource-local `GET .../actions/{action_id}` lookups are deprecated. Prefer global action lookup or non-deprecated resource action surfaces where available.
- 2026-01-15: Storage Box Subaccount includes a `name` property.

## Deferred Scope

Robot Webservice is explicitly deferred until after the Cloud/DNS SDK reaches
1.0. Its source reference is:

- <https://robot.hetzner.com/doc/webservice/en.html>

`v1.1.0` must pin Robot separately before any Robot operation is implemented.
