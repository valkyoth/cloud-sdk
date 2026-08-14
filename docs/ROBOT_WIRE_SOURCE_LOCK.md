# Robot Wire Source Lock

Status: narrow protocol fixture source-locked for `v0.42.0`; complete operation
inventory source-locked for `v0.74.0`; bounded form codec implemented in
`v0.75.0`; protected credentials and lockout-aware generations implemented in
`v0.76.0`; bounded error and quota protocol implemented in `v0.77.0`; server
list, get, and rename operations implemented in `v0.78.0`.
All nine cancellation operations are implemented in `v0.79.0`; all six
single-IP and separate-MAC operations are implemented in `v0.80.0`. Subnet and
subnet-MAC operations are implemented in `v0.81.0`; reset discovery and
execution are implemented in `v0.82.0`; failover discovery and route
transitions are implemented in `v0.83.0`; Wake-on-LAN is implemented in
`v0.84.0`; all active boot-configuration operations are implemented in
`v0.85.0`; all active reverse-DNS operations are implemented in `v0.86.0`.

Retrieved: 2026-07-30

Official source:
<https://robot.hetzner.com/doc/webservice/en.html>

SHA-256:
`4b396790acc449f47b2b3b893f8eff759c0c25196dc38b1e5e92a12c9704771a`

## Locked Protocol

The Robot Webservice uses the fixed HTTPS origin
`https://robot-ws.your-server.de`, HTTP Basic authentication,
`application/x-www-form-urlencoded` POST bodies, and JSON responses. Ordinary
success is `200`; resource creation uses `201`.

Authentication rejection is `401`. The official source warns that three
failed logins block the caller's source IP for ten minutes. Tests and fixtures
therefore never send credentials or intentionally exercise rejection against
the live service.

Quota exhaustion is `403` with code `RATE_LIMIT_EXCEEDED` and integer
`max_request` and `interval` fields. Maintenance is `503`. Some successful
mutations have no response body.

## Fixture Scope

[`v0.42.0.json`](../tests/fixtures/robot-wire/v0.42.0.json) contains:

- one non-executing `GET /server/321` read;
- one non-executing `POST /vswitch/4321/server` form body preserving two
  ordered `server[]` fields;
- read success, general error, invalid-input, authentication-rejection, quota,
  maintenance, and empty-success response shapes.

The fixture contains no `Authorization`, cookie, token, username, password, or
live account data. It lives outside every publishable crate. It proves only
the listed protocol distinctions and makes no Robot operation-coverage claim.

## Complete Operation Lock

[`v0.74.0.json`](../tests/fixtures/robot-api/v0.74.0.json) records all 105
operation headings from the same authenticated source document in exact source
order. It classifies 89 headings as active and all 16 legacy `/storagebox`
headings as deprecated and excluded. The supported replacement is the Hetzner
Console Storage Box API already implemented by `cloud-sdk-hetzner`.

The lock assigns every active operation to its implementation milestone from
`v0.78.0` through `v0.93.0`. Deprecated server-IP aliases and deprecated input
or output fields are not separate operation headings and remain excluded under
the repository's deprecated-endpoint policy. No Robot runtime module, request,
decoder, credential, or client is introduced by the source lock.

Local validation binds the complete canonical operation array to SHA-256
`896e23812d536999ad0deb1509fec9a23f92eae28ca0a404e11063b3644a5d76`.
Structural totals therefore cannot conceal swapped IDs or cross-family
ownership and milestone changes. The lock reader consumes at most 256 KiB
before rejecting an oversized file.

## Form Codec Contract

`v0.75.0` implements the first runtime primitive in
`cloud_sdk_hetzner::robot`. `RobotForm` accepts an ordered bounded slice of
validated fields, preserves repeated names, performs exact checked preflight,
and emits standard `application/x-www-form-urlencoded` bytes. Spaces become
`+`; literal separators, plus signs, brackets, controls, and non-ASCII UTF-8
bytes are percent encoded with uppercase hexadecimal digits.

Validation and capacity failures leave output unchanged. After exact capacity
admission, the complete destination is volatile-cleared before writing and is
owned by `EncodedRobotForm` until that guard clears the complete buffer on
drop. Borrowed source values and downstream transport copies remain caller and
operational cleanup boundaries. The codec does not add Robot credentials,
endpoint operations, response decoding, retries, or network execution.

## Credential And Lockout Contract

`v0.76.0` adds a Robot-only protected username/password owner fixed to the
source-locked HTTPS origin, Hetzner provider identity, and Robot service
identity. Mutable and guarded input clears on ingestion and rotation; secret
text is available only inside one still-open attempt bound to the exact
credential owner and cannot be borrowed out of the closure. Attempts from a
different owner fail before equal generation values are considered. Robot
attempts own an opaque shared lineage, are non-hashable, can move across task
boundaries, and do not block rotation; an older response after rotation is
classified as stale.

Authentication rejection atomically closes the attempted generation. No
automatic retry, pager, poller, or client can reopen it. Newly supplied
replacement credentials advance the generation, while reuse of unchanged
material requires an explicit caller reconfirmation after rejection.
Reconfirmation while open, stale transitions, and generation wrap fail closed.
The milestone sends no request and does not intentionally test invalid live
credentials.

## Error And Quota Contract

`v0.77.0` adds the first Robot response decoder. It consumes only an admitted
`TransportResponse`, limits an error body to 64 KiB, requires JSON for bodyful
errors, rejects duplicate or extra fields, and moves provider text into
cleanup-owning protected strings. Diagnostics expose finite categories and
counts but never provider message or input-name bytes.

The decoder admits only the source-locked protocol available at this
milestone: bodyless 401 authentication rejection, `INVALID_INPUT` at 400,
`RATE_LIMIT_EXCEEDED` at 403, `SERVER_NOT_FOUND` at 404, and bodyless 503
maintenance. The JSON `status` must equal the HTTP status. Unknown statuses,
unknown codes, malformed nullability, zero quota values, and future envelope
fields fail closed until the source lock and decoder are reviewed together.

Authentication rejection has retry disposition `Never`. Quota exposes the
provider maximum and interval and can produce one exhausted `robot-global`
provider-neutral quota bucket. Maintenance and explicitly constructed
transport failures require caller policy. Provider bytes cannot construct the
transport variant, so an unknown code can never become a transient fallback.

## Server Operation Contract

`v0.78.0` implements the three active `server` rows from the complete lock:
`GET /server`, `GET /server/{server-number}`, and
`POST /server/{server-number}`. The two deprecated IPv4 path aliases remain
unrepresentable. Default-feature request preparation binds only positive
server numbers, the official Robot origin and service, HTTP Basic scope,
explicit operation metadata, and form encoding for the sole update field
`server_name`.

With `serde`, checked success decoding admits the exact summary and detail
field sets, finite `ready` and `in process` states, positive resource IDs,
calendar-valid paid-through dates, canonical single addresses and subnets,
bounded duplicate-free lists, and the documented nullable subnet shape. Get
and update decoding reject a server number that differs from the request.
`linked_storagebox` is optional because the official update example omits it
while the corresponding output table lists it; a missing or zero value maps to
no linked Storage Box. This documented source inconsistency must be re-reviewed
if the upstream document changes.

The field contract is committed in
[`v0.78.0.json`](../tests/fixtures/robot-server/v0.78.0.json) and related back
to the complete operation lock by `scripts/check_robot_server_contract.py`.

## IP Operation Contract

`v0.80.0` implements all six active IP rows: list, detail, traffic-policy
update, and separate-MAC get, generate, and delete. The implementation uses
only canonical protected address values and canonical lowercase EUI-48 MACs.
It binds the optional list filter and every mutation response to the exact
request and enforces bounded duplicate-free lists, internally consistent
network fields, non-empty partial update forms, and exact nullable-MAC state.

Traffic update, MAC generation, and MAC deletion use request-bound mutation or
destructive permits. Sensitive traffic forms require digest fingerprints; MAC
generation and deletion remain automatic-retry denied. The field and policy
contract is committed in
[`v0.80.0.json`](../tests/fixtures/robot-ip/v0.80.0.json) and checked against
the complete operation lock by `scripts/check_robot_ips.sh`.

## Subnet Operation Contract

`v0.81.0` implements all six active subnet rows: list, detail, traffic-policy
update, and subnet-MAC get, explicit assignment, and default restoration. The
implementation admits canonical protected route identities, a nullable IPv4
server main address, positive server numbers, finite prefix lengths, and
same-family gateways within the addressed prefix.

The official list example permits `server_ip: null` although the output table
calls it a string. Subnet detail masks are JSON integers, while subnet-MAC
masks are canonical decimal strings. Official IPv4 examples also use route
identities with host bits set. These differences are explicit source-locked
contracts: the SDK computes the mathematical network and IPv4 broadcast but
does not replace or reject the provider's route identity.

The MAC response admits a nonempty bounded map from canonical IP addresses to
canonical lowercase EUI-48 values. The current MAC must occur in that map, and
an explicit assignment response must equal the requested MAC. Traffic update
and MAC assignment forms are sensitive and require digest fingerprints;
assignment and restoration deny automatic retry. The exact contract is
committed in
[`v0.81.0.json`](../tests/fixtures/robot-subnet/v0.81.0.json) and checked by
`scripts/check_robot_subnets.sh`.

Default restoration consumes checked subnet and MAC snapshots. The assigned
server main address selects the expected default MAC from `possible_mac`, and
both snapshots must fit a fixed 30-second freshness window. A protected
same-resource external-lock lease is required through dispatch. The complete
server/MAC/freshness/lock evidence is accepted only by the digest plan builder
and bounds permit validity. It is checked at permit entry and immediately
before transport using the generic check's clock sample; async checks occur on
first poll. DELETE success must return and continue to advertise that exact
mapping. The
fixture records every documented `(status, code)` pair; request-associated
decoders admit each pair only for its exact operation. The checker compares
the full normalized operation, field, inconsistency, and security contract and
runs the compiled subnet contract tests.

## Reset Operation Contract

`v0.82.0` implements all three active reset rows: list capabilities, get one
server's checked reset detail, and execute one advertised capability. The
finite types are `sw`, `hw`, `power`, `power_long`, and `man`. Lists and
capabilities are bounded and duplicate-free, address spelling is canonical,
and detail identity is bound to the requested positive server number.

Execution is constructible only from an exact authenticated detail execution
and an explicitly selected advertised type. Raw decoded details cannot grant
authority. The resulting state binds an opaque transport credential lineage
to a 30-second observation window. The sensitive `type` form is destructive,
non-idempotent, and never automatically retried. Read requests cannot create
the reset plan wrapper. Execute requests require strong-digest request-bound
direct or shared destructive permits across blocking, Send-async, and
local-async execution. The digest includes credential lineage, complete server
identity, capability, observation, and expiry; dispatch rechecks credential and
expiry before network access. Success limits are 2 MiB for list, 4 KiB for
detail, and 2 KiB for action responses.

Action success must match checked IPv4, IPv6 network, and requested type. The
official POST example omits `server_number` although the output table lists
it, so the field is narrowly optional and is identity-checked when present.
The exact field, quota, error, inconsistency, and security contract is
committed in
[`v0.82.0.json`](../tests/fixtures/robot-reset/v0.82.0.json) and checked by
`scripts/check_robot_resets.sh`.

## Failover Operation Contract

`v0.83.0` implements all four active failover rows: list and get exact routes,
reroute one failover address, and delete its active route. Failover, owner,
and destination addresses are canonical protected values with redacted
diagnostics. Route masks must be family-matched and contiguous, route host
bits are rejected, active destinations must use the route family, and lists
are bounded and duplicate-free.

Reroute uses the sensitive `active_server_ip` form, is non-idempotent, never
automatically retried, and requires a request-bound mutation permit. Route
deletion is also non-idempotent and never automatically retried, and requires
a destructive permit. Strong digests are mandatory whenever the prepared plan
contains the sensitive reroute body. Checked success remains associated with
the exact request and route; reroute must echo the exact destination.

The official DELETE example returns a JSON failover object with
`active_server_ip: null`, even though the field table calls the value a
string. The implementation therefore requires that exact nullable JSON
acknowledgement and does not admit `204` or an empty body. The complete field,
quota, error, inconsistency, and security contract is committed in
[`v0.83.0.json`](../tests/fixtures/robot-failover/v0.83.0.json) and checked by
`scripts/check_robot_failovers.sh`.

## Boot Configuration Contract

`v0.85.0` implements the complete 15-operation boot family: the four-family
overview, Rescue and Linux current/activate/deactivate/last operations, and
VNC and Windows current/activate/deactivate operations. All paths use only a
canonical positive server number. Deprecated server-IP aliases and request
`arch` fields are absent.

Selectors, keyboard layouts, languages, and repeated authorized-key
fingerprints are bounded and atomically form encoded. All mutations are
non-idempotent and deny automatic retry. Linux, VNC, and Windows activation is
destructive because rebooting into an installer can erase server data.
Generated passwords, authorized keys, and host keys use protected owned
storage with closure-scoped access and redacted diagnostics.

Strict decoding requires canonical IPv4/IPv6 identity families and the exact
requested server number. Unknown and duplicate fields, oversized or duplicate
options or keys, contradictory active/password/selection state, selector
mismatch, and cross-operation response use fail closed. Decoding is bound to
the typed operation's overview, current, last, activation, or deactivation
shape. An overview admits the documented inactive Windows `lang: null` shape
only in that context and rejects more than one active family. Deprecated
response `arch` and Windows `dist` fields are accepted only where
source-locked, validated, then discarded rather than exposed. The exact
operation, field, quota, error, deprecation, and security contract is committed in
[`v0.85.0.json`](../tests/fixtures/robot-boot/v0.85.0.json) and checked by
`scripts/check_robot_boot.sh`.

## Reverse-DNS Contract

`v0.86.0` implements all five active reverse-DNS operations: list, get, set,
update-or-create, and delete. Paths contain only canonical protected IPv4 or
IPv6 text. The optional list filter accepts only a canonical main-server IPv4
address and uses the exact `server_ip` query name.
PTR values are lowercase bounded DNS names without a trailing dot, controls,
empty labels, or oversized labels.

Set and update are non-idempotent mutations and require exact request-bound
mutation permits. Delete is destructive and requires its own request-bound
destructive permit. None is automatically retryable. Set and update responses
must echo the exact requested address and PTR. Delete requires status `200`
with an empty body; no-content and JSON acknowledgements are not admitted.
List decoding caps the collection at 4,096 entries and rejects duplicate IP
identities. Raw decoders are internal. A filtered response does not echo the
server association, so its checked wrapper requires independently checked IP
inventory and verifies every returned address against the exact filter through
a sorted bounded assignment index. Executable tests cap each maximum-size
lookup at 13 comparisons; the Python checker validates only the immutable
source contract and makes no semantic implementation claim. Empty filtered
responses fail closed. The distinct successful result proves non-empty
membership only, not completeness or authoritative absence. The provider can
still change assignment state
between the inventory and reverse-DNS reads. The exact operation, form, status,
quota, error, and response contract is committed in
[`v0.86.0.json`](../tests/fixtures/robot-rdns/v0.86.0.json) and checked by
`scripts/check_robot_rdns.sh`.

`v0.87.0` implements the one active traffic row. Robot receives repeated IP
and subnet form fields, exact source-compatible interval text, an aggregation
type, and optional individual-value selection. The operation is a read-only
query despite its POST wire method, remains explicitly policy-gated for retry,
and is limited by the documented 200 requests/hour source quota. Incremental
success decoding binds the echoed interval and every dynamic target to the
request, preserves exact non-negative numeric text, validates canonical subnet
CIDRs, and sorts bounded sparse period values. The normalized contract is
committed in
[`v0.87.0.json`](../tests/fixtures/robot-traffic/v0.87.0.json) and checked by
`scripts/check_robot_traffic.sh`.

`v0.88.0` implements all five active SSH-key rows. Create accepts the
source-documented `name` and OpenSSH/SSH2 `data` form fields and requires
`201`; rename accepts only `name`; list, get, rename, and delete require `200`,
with an empty delete body. Names, fingerprints, and returned key data use
protected redacted storage. Checked decoding parses normalized RFC 4253 key
wire, requires source algorithm and size coherence, verifies the provider MD5
path identity, and computes a separate SHA-256 identity. Create reconciles
OpenSSH or RFC 4716 input with the returned normalized key. Mutations require
request-bound authority and cannot be retried automatically. The normalized
contract is committed in
[`v0.88.0.json`](../tests/fixtures/robot-ssh-keys/v0.88.0.json) and checked by
`scripts/check_robot_ssh_keys.sh`.

`v0.89.0` implements all eight active firewall rows. Server firewall get,
complete replacement, and clear use `/firewall/{server-id}`; template
list/create and get/replace/delete use `/firewall/template` and its numeric ID
path. Ordered input/output rules retain indexed source semantics. Canonical
IPv4 selectors, ports, protocols, TCP flags, actions, names, and IDs are
validated before atomic sensitive form preparation. Inline rules and template
application are mutually exclusive typed intents. Checked responses bind exact
server/template identity and complete mutation outcomes, while automatic
mutation retry remains forbidden. Official rules with ports and no protocol
are accepted; incompatible explicit protocols remain rejected. The source's
output table lists a template name, but its detailed examples omit it, so
template mutation decoding exposes an explicit reconciliation-required result.
That result owns non-erasable pending state. Confirmation consumes it with the
same-ID, name-bearing list summary and compares the protected name, all summary
flags, detailed flags, and ordered rules against the request. Robot exposes no
revision binding list and detail observations, so callers must serialize
concurrent mutations or repeat reconciliation after a possible race.
All eight operations are locked to 500 requests per hour. The normalized
contract and digest-bound official examples are committed in
[`v0.89.0.json`](../tests/fixtures/robot-firewall/v0.89.0.json) and checked by
`scripts/check_robot_firewalls.sh`.

## Verification

Run the local structural check:

```bash
scripts/check_robot_wire_fixture.py
scripts/check_robot_api_lock.py
```

Release preparation also authenticates the current official document against
the reviewed digest. The fetch rejects redirects, stops after 8 MiB, and uses
a 90-second POSIX wall-clock deadline in addition to per-operation timeouts.
An existing process-global real-time timer is rejected and left armed:

```bash
scripts/check_robot_wire_fixture.py --fetch
scripts/check_robot_api_lock.py --fetch
```

Any digest change is a review stop. Review the complete new official document,
update this lock and fixture together, rerun the security review, and record
the decision in release notes. Fetched bytes are never compiled, executed, or
copied into a published crate.
