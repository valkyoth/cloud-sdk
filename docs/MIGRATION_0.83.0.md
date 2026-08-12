# Migrating Source Users To v0.83.0

v0.83.0 is an internal source milestone after the published v0.80.0
checkpoint. No crate is published for this tag; crates.io publication remains
deferred to v0.85.0.

## Workspace Version

Source users should update exact workspace references from `v0.82.0` to
`v0.83.0`. The facade source version advances to `0.83.0` while the unpublished
provider source still declares `cloud-sdk-hetzner` `0.43.0`.

## New Failover API

- Use `RobotFailoverListRequest::new()` for `GET /failover`.
- Use `RobotFailoverGetRequest::new(route)` for one canonical route.
- Use `RobotFailoverRerouteRequest::new(route, destination)` to express a
  family-checked reroute.
- Use `RobotFailoverDeleteRouteRequest::new(route)` to express route removal.
- Enable `serde` for strict `RobotFailover`/`RobotFailoverList` decoding and
  request-bound execution permits.
- Use `MAX_ROBOT_FAILOVER_LIST_RESPONSE_BYTES` and
  `MAX_ROBOT_FAILOVER_ITEM_RESPONSE_BYTES` when sizing caller response
  storage. Free decoders independently reject bodies above those limits.

Read requests can use `prepare_bound`, validate a checked JSON response, and
call `decode_response`. Reroute and deletion must go through
`RobotFailoverPlanConfirmation` and the matching mutation or destructive
permit. Reroute bodies are sensitive and therefore require
`build_robot_failover_plan_digest`; exact fingerprints are rejected.

## Behavioral Requirements

- Route addresses and destinations must use canonical IP spelling.
- Reroute route and destination families must match.
- Success models reject noncontiguous masks, route host bits, duplicate routes,
  cross-family values, unknown fields, and response identity substitutions.
- Reroute success must return the requested destination.
- Delete success must return JSON with `active_server_ip: null`; do not expect
  `204 No Content`.
- Neither transition is automatically retryable. Reconcile uncertain delivery
  before issuing another request.

No existing v0.82 public source API is removed or renamed.
