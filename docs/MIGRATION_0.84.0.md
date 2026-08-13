# Migrating Source Users To v0.84.0

v0.84.0 is an internal source milestone after the published v0.80.0
checkpoint. No crate is published for this tag; crates.io publication remains
deferred to v0.85.0.

## Workspace Version

Source users should update exact workspace references from `v0.83.0` to
`v0.84.0`. The facade source version advances to `0.84.0` while the unpublished
provider source still declares `cloud-sdk-hetzner` `0.43.0`.

## New Wake-on-LAN API

- Use `RobotWolGetRequest::new(number)` for authenticated availability
  discovery at `GET /wol/{server-number}`.
- Execute the typed prepared read through `execute_authorizing_blocking`,
  `execute_authorizing_async`, or `execute_authorizing_local_async` to obtain
  30-second `AuthorizedRobotWol` evidence.
- Construct `RobotWolSendRequest::from_checked(&evidence,
  RobotWolIntent::Send)` and prepare it through `prepare_bound`.
- Build a strong digest with `build_robot_wol_plan_digest`, then use
  `RobotWolMutationPermit` or `RobotWolSharedMutationPermit` for execution.
- Size success storage using `MAX_ROBOT_WOL_RESPONSE_BYTES` (16 KiB).

## Behavioral Requirements

- Server IPv4 aliases are deprecated upstream and intentionally unsupported;
  use `RobotServerNumber`.
- Raw `decode_robot_wol` output cannot authorize a packet send.
- Discovery and send responses must identify the exact requested number and
  contain only canonical IPv4, IPv6 network, and server-number fields.
- Credential rotation or evidence expiry invalidates execution authority.
- A send is non-idempotent and never automatically retried. Reconcile
  uncertain delivery before another attempt.

No existing v0.83 public source API is removed or renamed.
