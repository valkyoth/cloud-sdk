# Public API Review 0.95.0

Status: implementation stop; pentest required.

## Published Checkpoint

`cloud-sdk 0.95.0` publishes the accumulated core changes from v0.91-v0.94,
including exact observed-status transport failures, dispatch guards, guarded
provider decoding, and complete Robot client support.

`cloud-sdk-hetzner 0.46.0` publishes read-only Robot ordering catalogs,
transaction snapshots, cost-authorized orders, and the complete official
`RobotClient` surface for all 89 active operations. All 45 read-only operations
have direct execution; every state-changing operation remains permit-gated.

`cloud-sdk-reqwest 0.36.0` publishes accumulated exact-status behavior and the
reviewed optional dependency updates. `cloud-sdk-testkit 0.31.0` publishes the
corresponding exact-status and client regression fixtures.

## v0.95 Surface

v0.95 adds no public library API. Its Robot live evidence is an ignored
integration test and root-owned repository operator harness. The harness
reuses existing public `BlockingBasicClientBuilder`, `RobotClient::official`,
`RobotServerListRequest`, bounded workspace, and checked response contracts.

## Contract Review

- The Robot launcher selects one exact test and accepts no operation selector.
- The test constructs only the exact official Robot endpoint and a
  provider/service/endpoint-scoped Basic credential.
- Username and password originate in separate bounded private regular files
  and are cleared after construction; diagnostics expose no credential,
  filename, endpoint, response body, or resource identity.
- The sole request is bodyless `GET /server`, with one attempt, no retry, an
  8 MiB response cap, strict typed decoding, and cleanup-owned storage.
- CI and Cargo receive no Robot credential variable, and the static release
  contract rejects mutation, order, transaction, custom-endpoint, or GitHub
  workflow execution paths.

## Compatibility

The library surface remains additive within the pre-1.0 contract. Consumers
updating from the v0.90 public checkpoint must account for the cumulative
pre-1.0 additions and exhaustive-enum changes documented by the four
intervening migration guides. Default features, MSRV, `no_std` behavior, and
provider transport separation are unchanged.
