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
`RateLimit-Remaining`, and `RateLimit-Reset` header set. Transports retain only
headers admitted by the prepared provider operation and reject duplicates.
The Hetzner provider decoder requires the complete set when any member is
present, accepts only bounded unsigned decimal values, and rejects zero limits
or remaining values above the limit. Standard `Retry-After` is decoded beside
provider quota with caller-supplied wall time. Core returns pure bounded delay
decisions; no adapter infers a delay or replays a request.

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

## v0.42.0 Robot Wire Policy

Robot Webservice is required for the full Hetzner 1.0 SDK. Its source
reference and narrow protocol fixture are locked in
[`ROBOT_WIRE_SOURCE_LOCK.md`](ROBOT_WIRE_SOURCE_LOCK.md).

The lock covers Basic authentication, HTTPS, form POST bodies, repeated fields,
JSON/error/quota/maintenance distinctions, lockout policy, and empty success
bodies. It contains no credentials and does not implement or claim Robot
operations. `v0.74.0` still pins the complete operation inventory before any
Robot operation is implemented. That complete lock must distinguish active
operations from deprecated alternatives and exclude the deprecated Robot
Storage Box family, whose supported replacement is already tracked by the
Console Storage Box source.

## v0.74.0 Robot Operation Policy

The complete Robot inventory is pinned in
[`tests/fixtures/robot-api/v0.74.0.json`](../tests/fixtures/robot-api/v0.74.0.json).
It records all 105 official operation headings in source order, classifies 89
as active, and excludes all 16 deprecated legacy Storage Box operations. Each
active row owns one implementation milestone from `v0.78.0` through `v0.93.0`.
The canonical complete operation array is independently bound by SHA-256
`896e23812d536999ad0deb1509fec9a23f92eae28ca0a404e11063b3644a5d76`,
so swapping IDs, routes, groups, statuses, milestones, or source order cannot
preserve a passing local policy check.

`scripts/check_robot_api_lock.py --fetch` authenticates the bounded official
document by exact SHA-256, rejects redirects, extracts every HTTP operation
heading, compares exact order and route identity, and requires an upstream
deprecation marker for each excluded Storage Box heading. Lock reads stop at
256 KiB before parsing; live fetches stop at 8 MiB and have a 90-second hard
wall-clock deadline. An existing process-global real-time timer causes a
fail-closed result and remains armed. Any source, count, ID, route, status,
group, milestone, or order change is a review stop.

## v0.75.0 Robot Form Policy

The form codec is bound to the `v0.42.0` repeated-field wire fixture and the
official `application/x-www-form-urlencoded` protocol statement. Ordered
duplicate names are retained. ASCII spaces encode as `+`; ASCII alphanumerics
and `*`, `-`, `.`, `_` remain literal; every other UTF-8 byte is uppercase
percent encoded. The SDK does not normalize Unicode, line endings, field
order, or duplicate fields.

Field names use one nonempty ASCII identifier root followed only by complete
bracketed identifier components; an empty bracket component is admitted for
Robot array parameters such as `server[]`. Malformed, unbalanced, or trailing
bracket text is rejected before encoding.

The public codec caps field count, name bytes, value bytes, and total encoded
body bytes. It performs exact immutable preflight before mutation and owns
complete-buffer cleanup after admission. These are SDK security policy bounds,
not provider-advertised service maxima.

## v0.76.0 Robot Credential And Lockout Policy

Robot credentials are bound to provider `hetzner`, service `robot`, and the
exact endpoint identity `https://robot-ws.your-server.de:443/`. Username and
password validation follows the reviewed Basic interoperability profile:
visible non-colon ASCII for usernames and printable ASCII for passwords, with
independent finite bounds.

Allocation-free credential attempts borrow the exact issuing state. The
alloc-backed Robot attempt instead owns an opaque shared lineage so it can
cross task boundaries and remain usable after credential rotation. Secret
access and authentication rejection validate owner identity before generation
or status, so an equal generation from another owner fails closed and an old
response after rotation is stale. Attempts do not expose owner identity to
`Hash`.

The v0.42 source fact that three authentication failures block the caller's
source IP for 600 seconds is enforced structurally: a 401-classified attempt
closes its complete generation, stale attempts cannot affect replacement
material, and unchanged credentials require explicit post-rejection caller
reconfirmation. v0.76 performs no live authentication.

## v0.77.0 Robot Error And Quota Policy

Robot error decoding is bound to the narrow wire fixture in
`tests/fixtures/robot-wire/v0.42.0.json` and the complete operation lock in
`tests/fixtures/robot-api/v0.74.0.json`. The admitted protocol is bodyless 401
authentication rejection, `INVALID_INPUT` at 400, `RATE_LIMIT_EXCEEDED` at
403, `SERVER_NOT_FOUND` at 404, and bodyless 503 maintenance.

Bodyful errors require JSON, remain under 64 KiB, reject duplicate and unknown
fields, validate HTTP and envelope status equality, and protect provider text.
Unknown status and code values are decoder errors, not transient failures.
Authentication rejection is non-retryable; maintenance, quota delay, and an
explicitly supplied delivery-classified transport failure remain caller-policy
decisions. This milestone adds no Robot request execution or endpoint-family
models.

## v0.78.0 Robot Server Policy

Robot server list, get, and update are bound to the three active `server` rows
in `tests/fixtures/robot-api/v0.74.0.json` and the exact field contract in
`tests/fixtures/robot-server/v0.78.0.json`. Public path identity is only a
positive server number. Deprecated GET and POST aliases using the main IPv4
address remain excluded.

List responses require exact summary fields. Get and update require those
fields plus eight capability booleans; `linked_storagebox` alone is optional
because the official update example omits the field while its output table
lists it. Status is exactly `ready` or `in process`. Dates must be valid
`yyyy-MM-dd`; assigned addresses and subnets are bounded and duplicate-free;
subnet host bits must be clear. Detail identity must equal the request.

Request preparation binds the official Robot origin and service, Basic scope,
form content type for rename, explicit operation metadata, and checked 200
JSON success policy. The milestone does not add authorization encoding,
network execution, or a Robot client.

## v0.79.0 Robot Cancellation Policy

The nine active server, IP, and subnet cancellation rows are bound to
`tests/fixtures/robot-api/v0.74.0.json` and the exact per-operation contract in
`tests/fixtures/robot-cancellation/v0.79.0.json`. All paths use canonical
protected identities. POST admits only `now` or a calendar-valid date; server
POST additionally models optional reason and explicit location-reservation
intent. POST and DELETE are destructive, are never automatically retryable,
and require the core execution permit boundary when executed. The public
cancellation plan, fingerprint, direct/shared destructive permit, and attempt
wrappers retain the exact request association through blocking, Send-async,
and local-async execution; permit execution returns `CheckedCancellation`
directly rather than an unbound checked response. Sensitive POST forms require
the strong-digest fingerprint builder and reject exact retention; bodyless
DELETE permits exact canonical or strong-digest fingerprints.

GET, POST, and IP/subnet DELETE require exact `200` JSON cancellation
envelopes. Server DELETE alone requires exact `200` with no body or content
type. Typed checked responses retain the exact request association. Response
identities must match the request; POST acknowledgement must match active
schedule, reason, and reservation intent; IP/subnet DELETE must report inactive
state. Cancellation date presence must match cancellation state, a scheduled
date cannot precede the earliest date, and a reserved location requires both
reservation availability and active cancellation. Server reason shape is an
array before cancellation and string or null afterward. For IP and subnet,
the official tables name `cancellation_date` while examples use
`cancellation-date`; exactly one reviewed spelling is admitted. Canonical
subnet host bits are mandatory.

Reservation acknowledgement is exact: `Omit` requires reservation to be both
unavailable and inactive, `Reserve` requires available and active reservation,
and `DoNotReserve` requires inactive reservation while permitting either
availability state.

## v0.80.0 Robot IP Policy

The six active single-IP and separate-MAC rows are bound to
`tests/fixtures/robot-api/v0.74.0.json` and the exact contract in
`tests/fixtures/robot-ip/v0.80.0.json`. The implemented set is `GET /ip`,
`GET /ip/{ip}`, `POST /ip/{ip}`, and GET, PUT, DELETE for
`/ip/{ip}/mac`. The list's optional `server_ip` query, every path identity,
and every returned address must be canonical.

The update body is a non-empty bounded form containing only explicitly chosen
`traffic_warnings`, `traffic_hourly`, `traffic_daily`, or `traffic_monthly`
fields. Thresholds are unsigned decimal values; hourly and daily units are
megabytes and monthly units are gigabytes. Update is mutation/idempotent with
explicit-policy retry eligibility. MAC generation is mutation/non-idempotent
and MAC deletion is destructive/idempotent; both deny automatic retry.

Checked list, detail, update, and MAC responses require exact `200` JSON
envelopes and retain the exact request association. Lists are bounded to 4,096
entries and duplicate-free by address; an empty inventory remains valid. Detail network family,
prefix, gateway, and broadcast values must be internally consistent. Optional
list filters bind every result to the requested server address. Update results
must acknowledge every explicitly requested field. MAC get/generate requires
a canonical lowercase EUI-48 value; delete requires an exact null MAC.

The request-bound plan and direct/shared permit wrappers preserve the exact
request through blocking, Send-async, and local-async execution. Sensitive
traffic forms require the strong-digest plan builder. Preparation failure and
unpolled attempts clear caller-provided path/body storage and consume only the
authority defined by the core permit lifecycle.

## v0.81.0 Robot Subnet Policy

The six active subnet and subnet-MAC rows are bound to
`tests/fixtures/robot-api/v0.74.0.json` and the exact contract in
`tests/fixtures/robot-subnet/v0.81.0.json`. The implemented set is
`GET /subnet`, `GET /subnet/{net-ip}`, `POST /subnet/{net-ip}`, and GET, PUT,
DELETE for `/subnet/{net-ip}/mac`.

List filtering admits only canonical IPv4 server main addresses. Detail and
list responses require exact fields, bounded duplicate-free subnet identities,
valid family-specific prefixes, and same-family gateways in the addressed
network. The officially demonstrated nullable `server_ip`, integer detail
mask, decimal-string MAC mask, and host-bits-set subnet identity are explicit
reviewed exceptions. Computed network and IPv4 broadcast accessors never
rewrite the exact provider route identity.

MAC responses require a nonempty map of at most 256 canonical IP-to-EUI-48
choices and require the current MAC to occur in that map. PUT requires an
explicit selected MAC and verifies exact acknowledgement. Traffic updates and
MAC assignment are sensitive forms requiring digest plan fingerprints.
Mutation/destructive permits retain request provenance through blocking,
Send-async, and local-async execution; PUT and DELETE deny automatic retry.

DELETE requests are constructed only from consumed checked subnet and MAC
snapshots. The snapshots must agree on route identity and prefix, the subnet
must have an assigned server main address, and that address must map to one
advertised MAC. Both reads must fit the fixed 30-second observation window and
a protected caller-provided external-lock lease must cover the same subnet
through that window. The assigned server, MAC, timestamps, evidence expiry,
lock generation, and lease expiry are digest-only authorization evidence;
permit validity cannot outlive that evidence. Permit entry and immediate
pre-dispatch checks reject stale evidence using the same clock sample as the
generic permit check; async checks run on first poll. DELETE acknowledgement
must return that exact default MAC and preserve its server-address mapping. Subnet failures use request-associated
decoders for the complete documented `(status, code)` sets, including the
source-locked `500` failures; cross-operation codes fail closed.

## v0.82.0 Robot Reset Policy

The three active reset rows are bound to
`tests/fixtures/robot-api/v0.74.0.json` and the exact contract in
`tests/fixtures/robot-reset/v0.82.0.json`: list all reset capabilities, get one
server's checked capability detail, and execute one advertised reset type.
Deprecated server-IP route aliases remain excluded.

The finite capability set is `sw`, `hw`, `power`, `power_long`, and `man`.
Lists are bounded and duplicate-free by server number; each capability list is
nonempty, finite, and duplicate-free. Addresses are canonical, server numbers
are positive, and detail identity must equal the request. Operating status is
retained in bounded protected storage.

Execution has no raw-detail or server-number-only constructor. Only exact
authenticated detail execution mints a 30-second `AuthorizedRobotReset` bound
to the transport credential lineage. It rejects a selected capability absent
from that state. Its `type`
form is sensitive, destructive, non-idempotent, and never automatically
retryable. Only the execute request can construct the reset plan wrapper;
strong-digest request-bound direct/shared destructive permits retain exact
association through blocking, Send-async, and local-async execution. Digest
evidence includes credential binding, complete identity, capability,
observation, and expiry; credential and freshness are rechecked at dispatch.
Execute does not implement generic preparation and its prepared wrapper cannot
return a generic request. Core retains a mandatory-evidence marker and rejects
that marker from generic plan builders, preventing permit type erasure.
List, detail, and action success limits are 2 MiB, 4 KiB, and 2 KiB.

Action acknowledgement must match checked IPv4 and IPv6 identities and the
exact requested reset type. `server_number` is optional only because the
official POST example omits it while the output table requires it; when
present it must match. Every documented error is status-and-operation bound.

## v0.83.0 Robot Failover Policy

The four active failover rows are bound to
`tests/fixtures/robot-api/v0.74.0.json` and the exact normalized contract in
`tests/fixtures/robot-failover/v0.83.0.json`: list, get, reroute, and delete
the active route. Paths accept only canonical protected failover addresses.
Responses contain exactly `ip`, `netmask`, `server_ip`, `server_ipv6_net`,
`server_number`, and `active_server_ip`.

The route and mask must use one family, the mask must be contiguous, and the
route must have no host bits under that mask. The owner server addresses must
be IPv4 and IPv6 respectively. A non-null active destination must use the
route family. Lists are bounded to 4,096 distinct route identities; unknown,
missing, duplicate, malformed, cross-family, and noncanonical values fail
closed.

Reroute is a sensitive-form non-idempotent mutation and deletion is a
non-idempotent destructive operation. Neither is automatically retryable.
Their request-bound direct/shared permits remain associated through blocking,
Send-async, and local-async execution; reroute plans require strong digests.
Success must match the request route. Reroute additionally requires the exact
requested destination, while deletion requires the official JSON object with
`active_server_ip: null`. Empty/no-content deletion is not admitted. Every
provider error is status-and-operation bound.

## v0.85.0 Robot Boot Policy

The 15 active boot rows are bound to
`tests/fixtures/robot-api/v0.74.0.json` and the exact normalized contract in
`tests/fixtures/robot-boot/v0.85.0.json`. The set covers the four-family
overview, Rescue and Linux current/activate/deactivate/last operations, and VNC
and Windows current/activate/deactivate operations. Paths accept only canonical
positive server numbers; deprecated server-IP aliases and architecture inputs
remain excluded.

Selectors and keyboard layouts are nonempty bounded text without controls or
bidirectional format controls. Rescue and Linux admit at most 64 unique
authorized-key fingerprints. Form fields preserve exact source order, use the
sensitive body policy, and clear complete caller storage on preparation
failure. Every mutation is non-idempotent and never automatically retryable;
Linux, VNC, and Windows activation is destructive.

Responses are capped at 1 MiB and require exact family envelopes, canonical
IPv4/IPv6 identities, the requested server number, bounded duplicate-free
choices and keys, and coherent active/password/selection state. Typed request
provenance selects the overview, current, last, activation, or deactivation
decoder shape; no shape-free decoder is public. The overview alone admits an
inactive Windows null language and rejects more than one active family.
Activation must select the exact requested primary value and language;
deactivation must return an inactive password-free available state, while a
last-operation response retains exact selected values. Generated passwords
and keys use protected owned storage. Literal deprecated response fields are
validated and discarded. Every provider error is status-and-operation bound.

## v0.86.0 Robot Reverse-DNS Policy

The five active reverse-DNS rows are bound to
`tests/fixtures/robot-api/v0.74.0.json` and the normalized contract in
`tests/fixtures/robot-rdns/v0.86.0.json`. Requests use canonical protected
IPv4/IPv6 values. PTR names are lowercase DNS names capped at 253 bytes, with
labels capped at 63 bytes and no trailing root marker.

The optional list `server_ip` filter accepts only a canonical main-server IPv4
address. List and get are read-only. Set and update are non-idempotent
mutations and delete is non-idempotent and destructive. Every mutation requires
an exact request-bound permit and denies automatic retry. Preparation
atomically writes the exact path, optional `server_ip` query, or `ptr` form and
clears caller storage on failure.

Success decoding rejects unknown or duplicate fields, noncanonical addresses,
invalid names, duplicate list identities, oversized collections, and response
identity substitution. Set and update acknowledgements must match the exact
requested address and PTR. Delete admits only an empty `200` response. Every
provider error is narrowed by operation and status. Raw decoders remain
internal. Because filtered responses omit server association, filtered list
decoding requires independently checked IP inventory, rejects empty results,
and rejects any address not assigned to the exact filter. The distinct result
type proves non-empty membership only, never completeness or authoritative
absence. A sorted bounded assignment index prevents cross-product response
work. No client, implicit retry, or live mutation is introduced.

## v0.87.0 Robot Traffic Policy

The active `POST /traffic` row is bound to the complete Robot inventory and
the normalized `tests/fixtures/robot-traffic/v0.87.0.json` contract. Requests
contain at least one distinct canonical `ip[]` or subnet-base `subnet[]`, exact
`day`, `month`, or `year` bounds, and optional `single_values=true`. Component
ranges are checked without imposing Gregorian month lengths because Hetzner's
published month example uses `2010-09-31`.

The POST body is an explicitly admitted read-only query, sensitive and
replayable, but retry requires caller policy. Successful reports require the
exact echoed type and bounds. Dynamic targets must match the requested kind
and address; subnet keys must be canonical family-valid CIDRs. Incremental
decoding rejects unknown and duplicate keys, negative or overlong numbers,
invalid period ordinals, excessive structure, and aggregate/grouped shape
confusion. Exact number text is retained without floating-point conversion.
Robot may omit requested targets with no data. No mutation, permit, automatic
retry, high-level Robot client, or network transport is introduced.

## v0.88.0 Robot SSH-Key Policy

The five active `/key` rows are bound to the complete Robot inventory and the
normalized `tests/fixtures/robot-ssh-keys/v0.88.0.json` contract. List/get are
read-only; create/rename are non-idempotent mutations; delete is destructive.
Every mutation denies automatic retry and requires exact request-bound
authority.

Create accepts bounded conservative OpenSSH or RFC 4716 SSH2 text. Forms are
sensitive and atomically clear caller storage on failure. Provider responses
must contain exactly `name`, `fingerprint`, `type`, `size`, `data`, and
`created_at`. The normalized OpenSSH value is decoded as RFC 4253 key wire;
algorithm and size must agree, provider MD5 is verified over the wire bytes,
and SHA-256 is computed independently. Lists are bounded and reject duplicate
fingerprints. Get/rename bind the exact path fingerprint, while create also
binds the normalized request key and name. Delete admits only an empty `200`
acknowledgement. No raw decoder, automatic retry, high-level Robot client, or
network transport is introduced.

## v0.89.0 Robot Firewall Policy

The eight active firewall rows are bound to the complete Robot inventory and
the normalized `tests/fixtures/robot-firewall/v0.89.0.json` contract. Server
get and template list/get are read-only. Server replacement and template
create/update are non-idempotent mutations. Server clear and template delete
are destructive. Every mutation denies automatic retry and requires exact
request-bound authority.

Input and output rules are independently bounded, retain source order, and
reject exact duplicates. IPv4 hosts/CIDRs, ports/ranges, protocol constraints,
TCP flags, names, and template IDs are validated before atomic form encoding.
Inline rules and template application are separate intent variants, preventing
the source-forbidden combination of `template_id` with `whitelist_hos` or
inline `rules`. A port may omit protocol exactly as shown in the official
request and response examples; explicitly incompatible protocols remain
invalid.

Strict protected response models reject unknown fields, malformed values,
duplicate identities, and mismatched server/template or mutation outcomes.
The official detailed template examples omit `name` despite the output table;
the field is therefore optional and mutation decoding requires later
reconciliation instead of claiming full confirmation when it is absent. The
pending result cannot be extracted as a confirmed template. Confirmation
consumes it with the corresponding name-bearing list summary and verifies the
retained original request's template identity, protected name, all summary
flags, detailed flags, and ordered rules. The confirmation API accepts no
replacement request policy. Robot provides no revision binding those
list/detail reads; callers must exclude concurrent mutation or repeat the
reconciliation after a possible race.
Protected policy comparison covers ports and TCP flags without ordinary
short-circuit text comparison.
Server clear requires an `in process` response with no rules; template delete
requires the exact empty `200` acknowledgement. Raw decoders, automatic
mutation retry, high-level Robot client, and network transport remain absent.
