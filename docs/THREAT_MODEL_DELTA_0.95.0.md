# Threat Model Delta 0.95.0

Status: implementation stop; pentest required.

## New Assets And Threats

v0.95 introduces operator-held Robot username and password files, a staged
live-test executable, two installed launchers, a root-owned manifest, and one
authenticated read-only network request. New threats include credential
exposure to build tooling or CI, launcher or artifact substitution, mixed
Cloud/Robot credentials, same-file aliases, filesystem races, secret retention,
custom endpoint exfiltration, invalid-login lockout, accidental mutation or
ordering, response/resource disclosure, and hidden retries.

## Controls

- Cargo staging rejects all Cloud and Robot credential variables and requires
  a clean reviewed commit. Credentials are provisioned only after the build
  environment exits and the bundle is installed in root-owned non-writable
  paths.
- Manifest format 3 binds the executable, isolated runner, Cloud launcher,
  Robot launcher, and reviewed commit. The runner validates ownership, modes,
  link count, directory trust, and already-open descriptor hashes before
  descriptor execution.
- The Robot launcher has one fixed mode. The runner clears inherited state,
  rejects additional arguments, destructive opt-in, bearer tokens, incomplete
  or mixed credentials, and selects exactly `read_only_robot_server_smoke`.
- Credential files must be separate private regular files in owner-only parent
  directories. Unix opens are descriptor-based with no-follow semantics and
  require effective-user ownership, one link, and owner-only permissions;
  non-Unix live loading fails closed. Oversized or empty values are rejected,
  and both complete source allocations clear on every return.
- Basic authorization is scoped to Hetzner, Robot, and the exact official HTTPS
  endpoint. `RobotClient::official` verifies that destination again.
- The only live request is bodyless `GET /server`. A compiled exact-match
  transport test exercises the shared live execution function and rejects any
  method, target, body, header, endpoint, or dispatch-count change. Static
  source checks remain secondary tripwires for mutation, permits, orders,
  transactions, custom endpoints, and workflow execution. No invalid
  credential or automatic retry is intentionally sent.
- Request, response, header, authorization, and credential storage stays
  bounded and cleanup-owned. Output and errors remain static and payload-free.

## Residual Boundaries

The SDK cannot prove provider-side Robot permissions or credential validity.
Even a read-only executable may hold credentials capable of mutations in other
software. Operators must use the narrowest separate Webservice account
available, verify credentials before the one run without intentionally causing
failed logins, monitor Robot security state, and revoke or rotate both values
afterward.

Root ownership is a local operational trust anchor, not reproducible signed
binary provenance. Filesystem caches, shell input, OS and TLS copies, crash
tooling, swap, process abort, allocator exhaustion, remote logging, provider
availability, and account billing remain operational boundaries. Live success
is point-in-time evidence and does not replace source drift, mocks, fuzzing,
platform qualification, or the controlled-mutation plan.
