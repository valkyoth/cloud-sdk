# Rejected Abstractions 0.95.0

Status: release candidate; pentest and final retest passed.

## Robot Live Execution In CI

GitHub secrets and automated Robot requests were rejected. Build tooling and CI
receive no credential path. The authenticated phase is an explicit local
operator action against a root-sealed reviewed artifact.

## One Combined Credential File

A username/password document, URL, JSON object, or `user:password` file was
rejected. Separate files preserve type boundaries, avoid delimiter parsing,
allow independent rotation, and make same-file mistakes detectable.

## Raw Credential Environment Variables Or Arguments

Raw secrets in command arguments or environment variables were rejected
because process listings, shell history, inherited build state, and diagnostic
tooling can retain them. Only private file paths reach the isolated runner.

## Generic Live Operation Selector

A family name, method/path, operation ID, or arbitrary test selector was
rejected. The Robot launcher selects one exact ignored test, and that test
constructs only `RobotServerListRequest`.

## Live Mutation, Ordering, Or Invalid-Login Tests

Mutations, reset, Wake-on-LAN, traffic POST queries, transactions, billable
orders, destructive calls, automatic retry, and intentional `401` testing were
rejected. They add state, cost, delivery ambiguity, or source-IP lockout risk
without being necessary to prove the read-only client path.

## Custom Robot Endpoint

Custom endpoint configuration was rejected because it could exfiltrate Basic
credentials. The provider-owned endpoint policy and `RobotClient::official`
both bind the request to `https://robot-ws.your-server.de/`.
