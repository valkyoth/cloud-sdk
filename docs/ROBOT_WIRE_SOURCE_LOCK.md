# Robot Wire Source Lock

Status: narrow protocol fixture source-locked for `v0.42.0`.

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
The complete active/deprecated operation inventory remains assigned to
`v0.74.0`.

## Verification

Run the local structural check:

```bash
scripts/check_robot_wire_fixture.py
```

Release preparation also authenticates the current official document against
the reviewed digest:

```bash
scripts/check_robot_wire_fixture.py --fetch
```

Any digest change is a review stop. Review the complete new official document,
update this lock and fixture together, rerun the security review, and record
the decision in release notes. Fetched bytes are never compiled, executed, or
copied into a published crate.
