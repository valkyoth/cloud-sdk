# Robot Wire Source Lock

Status: narrow protocol fixture source-locked for `v0.42.0`; complete operation
inventory source-locked for `v0.74.0`; bounded form codec implemented in
`v0.75.0`; protected credentials and lockout-aware generations implemented in
`v0.76.0`; bounded error and quota protocol implemented in `v0.77.0`; server
list, get, and rename operations implemented in `v0.78.0`.
All nine cancellation operations are implemented in `v0.79.0`; all six
single-IP and separate-MAC operations are implemented in `v0.80.0`. Subnet and
subnet-MAC operations are implemented in `v0.81.0`.

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
DELETE success must return and continue to advertise that exact mapping. The
fixture records every documented `(status, code)` pair; request-associated
decoders admit each pair only for its exact operation. The checker compares
the full normalized operation, field, inconsistency, and security contract and
runs the compiled subnet contract tests.

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
