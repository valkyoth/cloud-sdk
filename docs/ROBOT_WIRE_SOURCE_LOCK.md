# Robot Wire Source Lock

Status: narrow protocol fixture source-locked for `v0.42.0`; complete operation
inventory source-locked for `v0.74.0`; bounded form codec implemented in
`v0.75.0`; protected credentials and lockout-aware generations implemented in
`v0.76.0`; bounded error and quota protocol implemented in `v0.77.0`.

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
