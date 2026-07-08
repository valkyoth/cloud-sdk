# Hetzner API Matrix

Status: initial source-derived plan. `v0.2.0` must verify this against the
current official API source before implementation claims are made.

Official reference: <https://docs.hetzner.cloud/reference/cloud>

| Group | Owner Crate | Planned Status |
| --- | --- | --- |
| actions | `sdk::actions` | planned |
| servers | `sdk::cloud::servers` | planned |
| server actions | `sdk::cloud::servers` | planned |
| server types | `sdk::cloud::pricing` | planned |
| images | `sdk::cloud::images` | planned |
| image actions | `sdk::cloud::images` | planned |
| isos | `sdk::cloud::images` | planned |
| placement groups | `sdk::cloud::servers` | planned |
| primary IPs | `sdk::cloud::networks` | planned |
| primary IP actions | `sdk::cloud::networks` | planned |
| volumes | `sdk::cloud::volumes` | planned |
| volume actions | `sdk::cloud::volumes` | planned |
| floating IPs | `sdk::cloud::networks` | planned |
| floating IP actions | `sdk::cloud::networks` | planned |
| firewalls | `sdk::cloud::firewalls` | planned |
| firewall actions | `sdk::cloud::firewalls` | planned |
| load balancers | `sdk::cloud::load_balancers` | planned |
| load balancer actions | `sdk::cloud::load_balancers` | planned |
| load balancer types | `sdk::cloud::load_balancers` | planned |
| networks | `sdk::cloud::networks` | planned |
| network actions | `sdk::cloud::networks` | planned |
| zones | `sdk::dns::zones` | planned |
| zone actions | `sdk::dns::zones` | planned |
| zone RRSets | `sdk::dns::rrsets` | planned |
| zone RRSet actions | `sdk::dns::rrsets` | planned |
| certificates | `sdk::security::certificates` | planned |
| certificate actions | `sdk::security::certificates` | planned |
| SSH keys | `sdk::security::ssh_keys` | planned |
| storage boxes | `sdk::storage::storage_boxes` | planned; discovered from current changelog and must be source-locked in `v0.2.0` |
| storage box actions | `sdk::storage::storage_boxes` | planned; verify exact operations in `v0.2.0` |
| storage box subaccounts | `sdk::storage::storage_boxes` | planned; verify exact operations in `v0.2.0` |
| locations | `sdk::cloud::pricing` | planned |
| pricing | `sdk::cloud::pricing` | planned |

## Cross-Cutting Semantics To Model

- authentication;
- query parameters;
- errors;
- actions;
- labels and label selectors;
- pagination;
- rate limiting;
- server metadata;
- sorting;
- deprecation notices.

## Post-1.0 Robot Webservice

Robot Webservice is not part of the Cloud/DNS 1.0 endpoint matrix. It is
planned for `v1.1.0+` and must be tracked in a separate Robot matrix because it
uses a different base URL, authentication model, request encoding, and resource
set.

Initial Robot groups to source-lock later:

| Group | Planned Module | Status |
| --- | --- | --- |
| server | `sdk::robot::server` | post-1.0 |
| IP | `sdk::robot::ip` | post-1.0 |
| subnet | `sdk::robot::subnet` | post-1.0 |
| reset | `sdk::robot::reset` | post-1.0 |
| failover | `sdk::robot::failover` | post-1.0 |
| wake on LAN | `sdk::robot::wol` | post-1.0 |
| boot configuration | `sdk::robot::boot` | post-1.0 |
| reverse DNS | `sdk::robot::rdns` | post-1.0 |
| traffic | `sdk::robot::traffic` | post-1.0 |
| SSH keys | `sdk::robot::ssh_keys` | post-1.0 |
| server ordering | `sdk::robot::ordering` | post-1.0 |
| storage box | `sdk::robot::storage_box` | post-1.0 |
| firewall | `sdk::robot::firewall` | post-1.0 |
| vSwitch | `sdk::robot::vswitch` | post-1.0 |
