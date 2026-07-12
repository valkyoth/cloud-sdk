# Hetzner API Matrix

Status: source-locked for `v0.2.0`.

Sources:

- Cloud and DNS: <https://docs.hetzner.cloud/cloud.spec.json>
- Storage Boxes: <https://docs.hetzner.cloud/hetzner.spec.json>

Retrieved: 2026-07-08
Total source-locked operations: 221 (`cloud`: 189, `hetzner`: 32).

## Matrix Rules

- Pagination is `yes` when an operation exposes both `page` and `per_page` query parameters.
- Sorting is `yes` when an operation exposes a `sort` query parameter.
- Action behavior is `action-list`, `action-get`, `resource-action-get`, `starts-action`, or `none`.
- Deprecated operations are kept in the matrix for drift tracking, but implementation status is `deferred-deprecated` until the SDK has an explicit compatibility policy.
- All non-deprecated operations are `planned` until endpoint models are implemented in later versions.

## Owner Modules

| Group | Owner module |
| --- | --- |
| Actions | `cloud_sdk_hetzner::actions` |
| Certificate Actions | `cloud_sdk_hetzner::security::certificates` |
| Certificates | `cloud_sdk_hetzner::security::certificates` |
| Data Centers | `cloud_sdk_hetzner::cloud::pricing` |
| Firewall Actions | `cloud_sdk_hetzner::cloud::firewalls` |
| Firewalls | `cloud_sdk_hetzner::cloud::firewalls` |
| Floating IP Actions | `cloud_sdk_hetzner::cloud::networks` |
| Floating IPs | `cloud_sdk_hetzner::cloud::networks` |
| ISOs | `cloud_sdk_hetzner::cloud::images` |
| Image Actions | `cloud_sdk_hetzner::cloud::images` |
| Images | `cloud_sdk_hetzner::cloud::images` |
| Load Balancer Actions | `cloud_sdk_hetzner::cloud::load_balancers` |
| Load Balancer Types | `cloud_sdk_hetzner::cloud::load_balancers` |
| Load Balancers | `cloud_sdk_hetzner::cloud::load_balancers` |
| Locations | `cloud_sdk_hetzner::cloud::pricing` |
| Network Actions | `cloud_sdk_hetzner::cloud::networks` |
| Networks | `cloud_sdk_hetzner::cloud::networks` |
| Placement Groups | `cloud_sdk_hetzner::cloud::servers` |
| Pricing | `cloud_sdk_hetzner::cloud::pricing` |
| Primary IP Actions | `cloud_sdk_hetzner::cloud::networks` |
| Primary IPs | `cloud_sdk_hetzner::cloud::networks` |
| SSH Keys | `cloud_sdk_hetzner::security::ssh_keys` |
| Server Actions | `cloud_sdk_hetzner::cloud::servers` |
| Server Types | `cloud_sdk_hetzner::cloud::pricing` |
| Servers | `cloud_sdk_hetzner::cloud::servers` |
| Storage Box Actions | `cloud_sdk_hetzner::storage::storage_boxes` |
| Storage Box Snapshots | `cloud_sdk_hetzner::storage::storage_boxes` |
| Storage Box Subaccount Actions | `cloud_sdk_hetzner::storage::storage_boxes` |
| Storage Box Subaccounts | `cloud_sdk_hetzner::storage::storage_boxes` |
| Storage Box Types | `cloud_sdk_hetzner::storage::storage_boxes` |
| Storage Boxes | `cloud_sdk_hetzner::storage::storage_boxes` |
| Volume Actions | `cloud_sdk_hetzner::cloud::volumes` |
| Volumes | `cloud_sdk_hetzner::cloud::volumes` |
| Zone Actions | `cloud_sdk_hetzner::dns::zones` |
| Zone RRSet Actions | `cloud_sdk_hetzner::dns::rrsets` |
| Zone RRSets | `cloud_sdk_hetzner::dns::rrsets` |
| Zones | `cloud_sdk_hetzner::dns::zones` |

## Operations

| API | Group | Method | Path | Operation | Owner | Pagination | Sorting | Action | Deprecated | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cloud | Actions | GET | `/actions` | `get_actions` | `cloud_sdk_hetzner::actions` | no | no | action-list | no | planned |
| cloud | Actions | GET | `/actions/{id}` | `get_action` | `cloud_sdk_hetzner::actions` | no | no | action-get | no | planned |
| cloud | Certificate Actions | GET | `/certificates/actions` | `list_certificates_actions` | `cloud_sdk_hetzner::security::certificates` | yes | yes | action-list | no | planned |
| cloud | Certificate Actions | GET | `/certificates/actions/{id}` | `get_certificates_action` | `cloud_sdk_hetzner::security::certificates` | no | no | action-get | no | planned |
| cloud | Certificate Actions | GET | `/certificates/{id}/actions` | `list_certificate_actions` | `cloud_sdk_hetzner::security::certificates` | yes | yes | action-list | no | planned |
| cloud | Certificate Actions | POST | `/certificates/{id}/actions/retry` | `retry_certificate` | `cloud_sdk_hetzner::security::certificates` | no | no | starts-action | no | implemented |
| cloud | Certificate Actions | GET | `/certificates/{id}/actions/{action_id}` | `get_certificate_action` | `cloud_sdk_hetzner::security::certificates` | no | no | resource-action-get | yes | deferred-deprecated |
| cloud | Certificates | GET | `/certificates` | `list_certificates` | `cloud_sdk_hetzner::security::certificates` | yes | yes | none | no | implemented |
| cloud | Certificates | POST | `/certificates` | `create_certificate` | `cloud_sdk_hetzner::security::certificates` | no | no | none | no | implemented |
| cloud | Certificates | DELETE | `/certificates/{id}` | `delete_certificate` | `cloud_sdk_hetzner::security::certificates` | no | no | none | no | implemented |
| cloud | Certificates | GET | `/certificates/{id}` | `get_certificate` | `cloud_sdk_hetzner::security::certificates` | no | no | none | no | implemented |
| cloud | Certificates | PUT | `/certificates/{id}` | `update_certificate` | `cloud_sdk_hetzner::security::certificates` | no | no | none | no | implemented |
| cloud | Data Centers | GET | `/datacenters` | `list_datacenters` | `cloud_sdk_hetzner::cloud::pricing` | yes | yes | none | yes | deferred-deprecated |
| cloud | Data Centers | GET | `/datacenters/{id}` | `get_datacenter` | `cloud_sdk_hetzner::cloud::pricing` | no | no | none | yes | deferred-deprecated |
| cloud | Firewall Actions | GET | `/firewalls/actions` | `list_firewalls_actions` | `cloud_sdk_hetzner::cloud::firewalls` | yes | yes | action-list | no | implemented-v0.10 |
| cloud | Firewall Actions | GET | `/firewalls/actions/{id}` | `get_firewalls_action` | `cloud_sdk_hetzner::cloud::firewalls` | no | no | action-get | no | implemented-v0.10 |
| cloud | Firewall Actions | GET | `/firewalls/{id}/actions` | `list_firewall_actions` | `cloud_sdk_hetzner::cloud::firewalls` | yes | yes | action-list | no | implemented-v0.10 |
| cloud | Firewall Actions | POST | `/firewalls/{id}/actions/apply_to_resources` | `apply_firewall_to_resources` | `cloud_sdk_hetzner::cloud::firewalls` | no | no | starts-action | no | implemented-v0.10 |
| cloud | Firewall Actions | POST | `/firewalls/{id}/actions/remove_from_resources` | `remove_firewall_from_resources` | `cloud_sdk_hetzner::cloud::firewalls` | no | no | starts-action | no | implemented-v0.10 |
| cloud | Firewall Actions | POST | `/firewalls/{id}/actions/set_rules` | `set_firewall_rules` | `cloud_sdk_hetzner::cloud::firewalls` | no | no | starts-action | no | implemented-v0.10 |
| cloud | Firewall Actions | GET | `/firewalls/{id}/actions/{action_id}` | `get_firewall_action` | `cloud_sdk_hetzner::cloud::firewalls` | no | no | resource-action-get | yes | deferred-deprecated |
| cloud | Firewalls | GET | `/firewalls` | `list_firewalls` | `cloud_sdk_hetzner::cloud::firewalls` | yes | yes | none | no | implemented-v0.10 |
| cloud | Firewalls | POST | `/firewalls` | `create_firewall` | `cloud_sdk_hetzner::cloud::firewalls` | no | no | none | no | implemented-v0.10 |
| cloud | Firewalls | DELETE | `/firewalls/{id}` | `delete_firewall` | `cloud_sdk_hetzner::cloud::firewalls` | no | no | none | no | implemented-v0.10 |
| cloud | Firewalls | GET | `/firewalls/{id}` | `get_firewall` | `cloud_sdk_hetzner::cloud::firewalls` | no | no | none | no | implemented-v0.10 |
| cloud | Firewalls | PUT | `/firewalls/{id}` | `update_firewall` | `cloud_sdk_hetzner::cloud::firewalls` | no | no | none | no | implemented-v0.10 |
| cloud | Floating IP Actions | GET | `/floating_ips/actions` | `list_floating_ips_actions` | `cloud_sdk_hetzner::cloud::networks` | yes | yes | action-list | no | implemented-v0.8 |
| cloud | Floating IP Actions | GET | `/floating_ips/actions/{id}` | `get_floating_ips_action` | `cloud_sdk_hetzner::cloud::networks` | no | no | action-get | no | implemented-v0.8 |
| cloud | Floating IP Actions | GET | `/floating_ips/{id}/actions` | `list_floating_ip_actions` | `cloud_sdk_hetzner::cloud::networks` | yes | yes | action-list | no | implemented-v0.8 |
| cloud | Floating IP Actions | POST | `/floating_ips/{id}/actions/assign` | `assign_floating_ip` | `cloud_sdk_hetzner::cloud::networks` | no | no | starts-action | no | implemented-v0.8 |
| cloud | Floating IP Actions | POST | `/floating_ips/{id}/actions/change_dns_ptr` | `change_floating_ip_dns_ptr` | `cloud_sdk_hetzner::cloud::networks` | no | no | starts-action | no | implemented-v0.8 |
| cloud | Floating IP Actions | POST | `/floating_ips/{id}/actions/change_protection` | `change_floating_ip_protection` | `cloud_sdk_hetzner::cloud::networks` | no | no | starts-action | no | implemented-v0.8 |
| cloud | Floating IP Actions | POST | `/floating_ips/{id}/actions/unassign` | `unassign_floating_ip` | `cloud_sdk_hetzner::cloud::networks` | no | no | starts-action | no | implemented-v0.8 |
| cloud | Floating IP Actions | GET | `/floating_ips/{id}/actions/{action_id}` | `get_floating_ip_action` | `cloud_sdk_hetzner::cloud::networks` | no | no | resource-action-get | yes | deferred-deprecated |
| cloud | Floating IPs | GET | `/floating_ips` | `list_floating_ips` | `cloud_sdk_hetzner::cloud::networks` | yes | yes | none | no | implemented-v0.8 |
| cloud | Floating IPs | POST | `/floating_ips` | `create_floating_ip` | `cloud_sdk_hetzner::cloud::networks` | no | no | none | no | implemented-v0.8 |
| cloud | Floating IPs | DELETE | `/floating_ips/{id}` | `delete_floating_ip` | `cloud_sdk_hetzner::cloud::networks` | no | no | none | no | implemented-v0.8 |
| cloud | Floating IPs | GET | `/floating_ips/{id}` | `get_floating_ip` | `cloud_sdk_hetzner::cloud::networks` | no | no | none | no | implemented-v0.8 |
| cloud | Floating IPs | PUT | `/floating_ips/{id}` | `update_floating_ip` | `cloud_sdk_hetzner::cloud::networks` | no | no | none | no | implemented-v0.8 |
| cloud | ISOs | GET | `/isos` | `list_isos` | `cloud_sdk_hetzner::cloud::images` | yes | no | none | no | implemented-v0.4 |
| cloud | ISOs | GET | `/isos/{id}` | `get_iso` | `cloud_sdk_hetzner::cloud::images` | no | no | none | no | implemented-v0.4 |
| cloud | Image Actions | GET | `/images/actions` | `list_images_actions` | `cloud_sdk_hetzner::cloud::images` | yes | yes | action-list | no | implemented-v0.7 |
| cloud | Image Actions | GET | `/images/actions/{id}` | `get_images_action` | `cloud_sdk_hetzner::cloud::images` | no | no | action-get | no | implemented-v0.7 |
| cloud | Image Actions | GET | `/images/{id}/actions` | `list_image_actions` | `cloud_sdk_hetzner::cloud::images` | yes | yes | action-list | no | implemented-v0.7 |
| cloud | Image Actions | POST | `/images/{id}/actions/change_protection` | `change_image_protection` | `cloud_sdk_hetzner::cloud::images` | no | no | starts-action | no | implemented-v0.7 |
| cloud | Image Actions | GET | `/images/{id}/actions/{action_id}` | `get_image_action` | `cloud_sdk_hetzner::cloud::images` | no | no | resource-action-get | yes | deferred-deprecated |
| cloud | Images | GET | `/images` | `list_images` | `cloud_sdk_hetzner::cloud::images` | yes | yes | none | no | implemented-v0.7 |
| cloud | Images | DELETE | `/images/{id}` | `delete_image` | `cloud_sdk_hetzner::cloud::images` | no | no | none | no | implemented-v0.7 |
| cloud | Images | GET | `/images/{id}` | `get_image` | `cloud_sdk_hetzner::cloud::images` | no | no | none | no | implemented-v0.7 |
| cloud | Images | PUT | `/images/{id}` | `update_image` | `cloud_sdk_hetzner::cloud::images` | no | no | none | no | implemented-v0.7 |
| cloud | Load Balancer Actions | GET | `/load_balancers/actions` | `list_load_balancers_actions` | `cloud_sdk_hetzner::cloud::load_balancers` | yes | yes | action-list | no | implemented-v0.11 |
| cloud | Load Balancer Actions | GET | `/load_balancers/actions/{id}` | `get_load_balancers_action` | `cloud_sdk_hetzner::cloud::load_balancers` | no | no | action-get | no | implemented-v0.11 |
| cloud | Load Balancer Actions | GET | `/load_balancers/{id}/actions` | `list_load_balancer_actions` | `cloud_sdk_hetzner::cloud::load_balancers` | yes | yes | action-list | no | implemented-v0.11 |
| cloud | Load Balancer Actions | POST | `/load_balancers/{id}/actions/add_service` | `add_load_balancer_service` | `cloud_sdk_hetzner::cloud::load_balancers` | no | no | starts-action | no | implemented-v0.11 |
| cloud | Load Balancer Actions | POST | `/load_balancers/{id}/actions/add_target` | `add_load_balancer_target` | `cloud_sdk_hetzner::cloud::load_balancers` | no | no | starts-action | no | implemented-v0.11 |
| cloud | Load Balancer Actions | POST | `/load_balancers/{id}/actions/attach_to_network` | `attach_load_balancer_to_network` | `cloud_sdk_hetzner::cloud::load_balancers` | no | no | starts-action | no | implemented-v0.11 |
| cloud | Load Balancer Actions | POST | `/load_balancers/{id}/actions/change_algorithm` | `change_load_balancer_algorithm` | `cloud_sdk_hetzner::cloud::load_balancers` | no | no | starts-action | no | implemented-v0.11 |
| cloud | Load Balancer Actions | POST | `/load_balancers/{id}/actions/change_dns_ptr` | `change_load_balancer_dns_ptr` | `cloud_sdk_hetzner::cloud::load_balancers` | no | no | starts-action | no | implemented-v0.11 |
| cloud | Load Balancer Actions | POST | `/load_balancers/{id}/actions/change_protection` | `change_load_balancer_protection` | `cloud_sdk_hetzner::cloud::load_balancers` | no | no | starts-action | no | implemented-v0.11 |
| cloud | Load Balancer Actions | POST | `/load_balancers/{id}/actions/change_type` | `change_load_balancer_type` | `cloud_sdk_hetzner::cloud::load_balancers` | no | no | starts-action | no | implemented-v0.11 |
| cloud | Load Balancer Actions | POST | `/load_balancers/{id}/actions/delete_service` | `delete_load_balancer_service` | `cloud_sdk_hetzner::cloud::load_balancers` | no | no | starts-action | no | implemented-v0.11 |
| cloud | Load Balancer Actions | POST | `/load_balancers/{id}/actions/detach_from_network` | `detach_load_balancer_from_network` | `cloud_sdk_hetzner::cloud::load_balancers` | no | no | starts-action | no | implemented-v0.11 |
| cloud | Load Balancer Actions | POST | `/load_balancers/{id}/actions/disable_public_interface` | `disable_load_balancer_public_interface` | `cloud_sdk_hetzner::cloud::load_balancers` | no | no | starts-action | no | implemented-v0.11 |
| cloud | Load Balancer Actions | POST | `/load_balancers/{id}/actions/enable_public_interface` | `enable_load_balancer_public_interface` | `cloud_sdk_hetzner::cloud::load_balancers` | no | no | starts-action | no | implemented-v0.11 |
| cloud | Load Balancer Actions | POST | `/load_balancers/{id}/actions/remove_target` | `remove_load_balancer_target` | `cloud_sdk_hetzner::cloud::load_balancers` | no | no | starts-action | no | implemented-v0.11 |
| cloud | Load Balancer Actions | POST | `/load_balancers/{id}/actions/update_service` | `update_load_balancer_service` | `cloud_sdk_hetzner::cloud::load_balancers` | no | no | starts-action | no | implemented-v0.11 |
| cloud | Load Balancer Actions | GET | `/load_balancers/{id}/actions/{action_id}` | `get_load_balancer_action` | `cloud_sdk_hetzner::cloud::load_balancers` | no | no | resource-action-get | yes | deferred-deprecated |
| cloud | Load Balancer Types | GET | `/load_balancer_types` | `list_load_balancer_types` | `cloud_sdk_hetzner::cloud::load_balancers` | yes | no | none | no | implemented-v0.4 |
| cloud | Load Balancer Types | GET | `/load_balancer_types/{id}` | `get_load_balancer_type` | `cloud_sdk_hetzner::cloud::load_balancers` | no | no | none | no | implemented-v0.4 |
| cloud | Load Balancers | GET | `/load_balancers` | `list_load_balancers` | `cloud_sdk_hetzner::cloud::load_balancers` | yes | yes | none | no | implemented-v0.11 |
| cloud | Load Balancers | POST | `/load_balancers` | `create_load_balancer` | `cloud_sdk_hetzner::cloud::load_balancers` | no | no | none | no | implemented-v0.11 |
| cloud | Load Balancers | DELETE | `/load_balancers/{id}` | `delete_load_balancer` | `cloud_sdk_hetzner::cloud::load_balancers` | no | no | none | no | implemented-v0.11 |
| cloud | Load Balancers | GET | `/load_balancers/{id}` | `get_load_balancer` | `cloud_sdk_hetzner::cloud::load_balancers` | no | no | none | no | implemented-v0.11 |
| cloud | Load Balancers | PUT | `/load_balancers/{id}` | `update_load_balancer` | `cloud_sdk_hetzner::cloud::load_balancers` | no | no | none | no | implemented-v0.11 |
| cloud | Load Balancers | GET | `/load_balancers/{id}/metrics` | `get_load_balancer_metrics` | `cloud_sdk_hetzner::cloud::load_balancers` | no | no | none | no | implemented-v0.11 |
| cloud | Locations | GET | `/locations` | `list_locations` | `cloud_sdk_hetzner::cloud::pricing` | yes | yes | none | no | implemented-v0.4 |
| cloud | Locations | GET | `/locations/{id}` | `get_location` | `cloud_sdk_hetzner::cloud::pricing` | no | no | none | no | implemented-v0.4 |
| cloud | Network Actions | GET | `/networks/actions` | `list_networks_actions` | `cloud_sdk_hetzner::cloud::networks` | yes | yes | action-list | no | implemented-v0.10 |
| cloud | Network Actions | GET | `/networks/actions/{id}` | `get_networks_action` | `cloud_sdk_hetzner::cloud::networks` | no | no | action-get | no | implemented-v0.10 |
| cloud | Network Actions | GET | `/networks/{id}/actions` | `list_network_actions` | `cloud_sdk_hetzner::cloud::networks` | yes | yes | action-list | no | implemented-v0.10 |
| cloud | Network Actions | POST | `/networks/{id}/actions/add_route` | `add_network_route` | `cloud_sdk_hetzner::cloud::networks` | no | no | starts-action | no | implemented-v0.10 |
| cloud | Network Actions | POST | `/networks/{id}/actions/add_subnet` | `add_network_subnet` | `cloud_sdk_hetzner::cloud::networks` | no | no | starts-action | no | implemented-v0.10 |
| cloud | Network Actions | POST | `/networks/{id}/actions/change_ip_range` | `change_network_ip_range` | `cloud_sdk_hetzner::cloud::networks` | no | no | starts-action | no | implemented-v0.10 |
| cloud | Network Actions | POST | `/networks/{id}/actions/change_protection` | `change_network_protection` | `cloud_sdk_hetzner::cloud::networks` | no | no | starts-action | no | implemented-v0.10 |
| cloud | Network Actions | POST | `/networks/{id}/actions/delete_route` | `delete_network_route` | `cloud_sdk_hetzner::cloud::networks` | no | no | starts-action | no | implemented-v0.10 |
| cloud | Network Actions | POST | `/networks/{id}/actions/delete_subnet` | `delete_network_subnet` | `cloud_sdk_hetzner::cloud::networks` | no | no | starts-action | no | implemented-v0.10 |
| cloud | Network Actions | GET | `/networks/{id}/actions/{action_id}` | `get_network_action` | `cloud_sdk_hetzner::cloud::networks` | no | no | resource-action-get | yes | deferred-deprecated |
| cloud | Networks | GET | `/networks` | `list_networks` | `cloud_sdk_hetzner::cloud::networks` | yes | yes | none | no | implemented-v0.10 |
| cloud | Networks | POST | `/networks` | `create_network` | `cloud_sdk_hetzner::cloud::networks` | no | no | none | no | implemented-v0.10 |
| cloud | Networks | DELETE | `/networks/{id}` | `delete_network` | `cloud_sdk_hetzner::cloud::networks` | no | no | none | no | implemented-v0.10 |
| cloud | Networks | GET | `/networks/{id}` | `get_network` | `cloud_sdk_hetzner::cloud::networks` | no | no | none | no | implemented-v0.10 |
| cloud | Networks | PUT | `/networks/{id}` | `update_network` | `cloud_sdk_hetzner::cloud::networks` | no | no | none | no | implemented-v0.10 |
| cloud | Placement Groups | GET | `/placement_groups` | `list_placement_groups` | `cloud_sdk_hetzner::cloud::servers` | yes | yes | none | no | implemented-v0.7 |
| cloud | Placement Groups | POST | `/placement_groups` | `create_placement_group` | `cloud_sdk_hetzner::cloud::servers` | no | no | none | no | implemented-v0.7 |
| cloud | Placement Groups | DELETE | `/placement_groups/{id}` | `delete_placement_group` | `cloud_sdk_hetzner::cloud::servers` | no | no | none | no | implemented-v0.7 |
| cloud | Placement Groups | GET | `/placement_groups/{id}` | `get_placement_group` | `cloud_sdk_hetzner::cloud::servers` | no | no | none | no | implemented-v0.7 |
| cloud | Placement Groups | PUT | `/placement_groups/{id}` | `update_placement_group` | `cloud_sdk_hetzner::cloud::servers` | no | no | none | no | implemented-v0.7 |
| cloud | Pricing | GET | `/pricing` | `get_pricing` | `cloud_sdk_hetzner::cloud::pricing` | no | no | none | no | implemented-v0.4 |
| cloud | Primary IP Actions | GET | `/primary_ips/actions` | `list_primary_ips_actions` | `cloud_sdk_hetzner::cloud::networks` | yes | yes | action-list | no | implemented-v0.7 |
| cloud | Primary IP Actions | GET | `/primary_ips/actions/{id}` | `get_primary_ips_action` | `cloud_sdk_hetzner::cloud::networks` | no | no | action-get | no | implemented-v0.7 |
| cloud | Primary IP Actions | GET | `/primary_ips/{id}/actions` | `list_primary_ip_actions` | `cloud_sdk_hetzner::cloud::networks` | yes | yes | action-list | no | implemented-v0.7 |
| cloud | Primary IP Actions | POST | `/primary_ips/{id}/actions/assign` | `assign_primary_ip` | `cloud_sdk_hetzner::cloud::networks` | no | no | starts-action | no | implemented-v0.7 |
| cloud | Primary IP Actions | POST | `/primary_ips/{id}/actions/change_dns_ptr` | `change_primary_ip_dns_ptr` | `cloud_sdk_hetzner::cloud::networks` | no | no | starts-action | no | implemented-v0.7 |
| cloud | Primary IP Actions | POST | `/primary_ips/{id}/actions/change_protection` | `change_primary_ip_protection` | `cloud_sdk_hetzner::cloud::networks` | no | no | starts-action | no | implemented-v0.7 |
| cloud | Primary IP Actions | POST | `/primary_ips/{id}/actions/unassign` | `unassign_primary_ip` | `cloud_sdk_hetzner::cloud::networks` | no | no | starts-action | no | implemented-v0.7 |
| cloud | Primary IP Actions | GET | `/primary_ips/{id}/actions/{action_id}` | `get_primary_ip_action` | `cloud_sdk_hetzner::cloud::networks` | no | no | resource-action-get | yes | deferred-deprecated |
| cloud | Primary IPs | GET | `/primary_ips` | `list_primary_ips` | `cloud_sdk_hetzner::cloud::networks` | yes | yes | none | no | implemented-v0.7 |
| cloud | Primary IPs | POST | `/primary_ips` | `create_primary_ip` | `cloud_sdk_hetzner::cloud::networks` | no | no | none | no | implemented-v0.7 |
| cloud | Primary IPs | DELETE | `/primary_ips/{id}` | `delete_primary_ip` | `cloud_sdk_hetzner::cloud::networks` | no | no | none | no | implemented-v0.7 |
| cloud | Primary IPs | GET | `/primary_ips/{id}` | `get_primary_ip` | `cloud_sdk_hetzner::cloud::networks` | no | no | none | no | implemented-v0.7 |
| cloud | Primary IPs | PUT | `/primary_ips/{id}` | `update_primary_ip` | `cloud_sdk_hetzner::cloud::networks` | no | no | none | no | implemented-v0.7 |
| cloud | SSH Keys | GET | `/ssh_keys` | `list_ssh_keys` | `cloud_sdk_hetzner::security::ssh_keys` | yes | yes | none | no | implemented |
| cloud | SSH Keys | POST | `/ssh_keys` | `create_ssh_key` | `cloud_sdk_hetzner::security::ssh_keys` | no | no | none | no | implemented |
| cloud | SSH Keys | DELETE | `/ssh_keys/{id}` | `delete_ssh_key` | `cloud_sdk_hetzner::security::ssh_keys` | no | no | none | no | implemented |
| cloud | SSH Keys | GET | `/ssh_keys/{id}` | `get_ssh_key` | `cloud_sdk_hetzner::security::ssh_keys` | no | no | none | no | implemented |
| cloud | SSH Keys | PUT | `/ssh_keys/{id}` | `update_ssh_key` | `cloud_sdk_hetzner::security::ssh_keys` | no | no | none | no | implemented |
| cloud | Server Actions | GET | `/servers/actions` | `list_servers_actions` | `cloud_sdk_hetzner::cloud::servers` | yes | yes | action-list | no | implemented |
| cloud | Server Actions | GET | `/servers/actions/{id}` | `get_servers_action` | `cloud_sdk_hetzner::cloud::servers` | no | no | action-get | no | implemented |
| cloud | Server Actions | GET | `/servers/{id}/actions` | `list_server_actions` | `cloud_sdk_hetzner::cloud::servers` | yes | yes | action-list | no | implemented |
| cloud | Server Actions | POST | `/servers/{id}/actions/add_to_placement_group` | `add_server_to_placement_group` | `cloud_sdk_hetzner::cloud::servers` | no | no | starts-action | no | implemented |
| cloud | Server Actions | POST | `/servers/{id}/actions/attach_iso` | `attach_server_iso` | `cloud_sdk_hetzner::cloud::servers` | no | no | starts-action | no | implemented |
| cloud | Server Actions | POST | `/servers/{id}/actions/attach_to_network` | `attach_server_to_network` | `cloud_sdk_hetzner::cloud::servers` | no | no | starts-action | no | implemented |
| cloud | Server Actions | POST | `/servers/{id}/actions/change_alias_ips` | `change_server_alias_ips` | `cloud_sdk_hetzner::cloud::servers` | no | no | starts-action | no | implemented |
| cloud | Server Actions | POST | `/servers/{id}/actions/change_dns_ptr` | `change_server_dns_ptr` | `cloud_sdk_hetzner::cloud::servers` | no | no | starts-action | no | implemented |
| cloud | Server Actions | POST | `/servers/{id}/actions/change_protection` | `change_server_protection` | `cloud_sdk_hetzner::cloud::servers` | no | no | starts-action | no | implemented |
| cloud | Server Actions | POST | `/servers/{id}/actions/change_type` | `change_server_type` | `cloud_sdk_hetzner::cloud::servers` | no | no | starts-action | no | implemented |
| cloud | Server Actions | POST | `/servers/{id}/actions/create_image` | `create_server_image` | `cloud_sdk_hetzner::cloud::servers` | no | no | starts-action | no | implemented |
| cloud | Server Actions | POST | `/servers/{id}/actions/detach_from_network` | `detach_server_from_network` | `cloud_sdk_hetzner::cloud::servers` | no | no | starts-action | no | implemented |
| cloud | Server Actions | POST | `/servers/{id}/actions/detach_iso` | `detach_server_iso` | `cloud_sdk_hetzner::cloud::servers` | no | no | starts-action | no | implemented |
| cloud | Server Actions | POST | `/servers/{id}/actions/disable_backup` | `disable_server_backup` | `cloud_sdk_hetzner::cloud::servers` | no | no | starts-action | no | implemented |
| cloud | Server Actions | POST | `/servers/{id}/actions/disable_rescue` | `disable_server_rescue` | `cloud_sdk_hetzner::cloud::servers` | no | no | starts-action | no | implemented |
| cloud | Server Actions | POST | `/servers/{id}/actions/enable_backup` | `enable_server_backup` | `cloud_sdk_hetzner::cloud::servers` | no | no | starts-action | no | implemented |
| cloud | Server Actions | POST | `/servers/{id}/actions/enable_rescue` | `enable_server_rescue` | `cloud_sdk_hetzner::cloud::servers` | no | no | starts-action | no | implemented |
| cloud | Server Actions | POST | `/servers/{id}/actions/poweroff` | `poweroff_server` | `cloud_sdk_hetzner::cloud::servers` | no | no | starts-action | no | implemented |
| cloud | Server Actions | POST | `/servers/{id}/actions/poweron` | `poweron_server` | `cloud_sdk_hetzner::cloud::servers` | no | no | starts-action | no | implemented |
| cloud | Server Actions | POST | `/servers/{id}/actions/reboot` | `reboot_server` | `cloud_sdk_hetzner::cloud::servers` | no | no | starts-action | no | implemented |
| cloud | Server Actions | POST | `/servers/{id}/actions/rebuild` | `rebuild_server` | `cloud_sdk_hetzner::cloud::servers` | no | no | starts-action | no | implemented |
| cloud | Server Actions | POST | `/servers/{id}/actions/remove_from_placement_group` | `remove_server_from_placement_group` | `cloud_sdk_hetzner::cloud::servers` | no | no | starts-action | no | implemented |
| cloud | Server Actions | POST | `/servers/{id}/actions/request_console` | `request_server_console` | `cloud_sdk_hetzner::cloud::servers` | no | no | starts-action | no | implemented |
| cloud | Server Actions | POST | `/servers/{id}/actions/reset` | `reset_server` | `cloud_sdk_hetzner::cloud::servers` | no | no | starts-action | no | implemented |
| cloud | Server Actions | POST | `/servers/{id}/actions/reset_password` | `reset_server_password` | `cloud_sdk_hetzner::cloud::servers` | no | no | starts-action | no | implemented |
| cloud | Server Actions | POST | `/servers/{id}/actions/shutdown` | `shutdown_server` | `cloud_sdk_hetzner::cloud::servers` | no | no | starts-action | no | implemented |
| cloud | Server Actions | GET | `/servers/{id}/actions/{action_id}` | `get_server_action` | `cloud_sdk_hetzner::cloud::servers` | no | no | resource-action-get | yes | deferred-deprecated |
| cloud | Server Types | GET | `/server_types` | `list_server_types` | `cloud_sdk_hetzner::cloud::pricing` | yes | no | none | no | implemented-v0.4 |
| cloud | Server Types | GET | `/server_types/{id}` | `get_server_type` | `cloud_sdk_hetzner::cloud::pricing` | no | no | none | no | implemented-v0.4 |
| cloud | Servers | GET | `/servers` | `list_servers` | `cloud_sdk_hetzner::cloud::servers` | yes | yes | none | no | implemented |
| cloud | Servers | POST | `/servers` | `create_server` | `cloud_sdk_hetzner::cloud::servers` | no | no | none | no | implemented |
| cloud | Servers | DELETE | `/servers/{id}` | `delete_server` | `cloud_sdk_hetzner::cloud::servers` | no | no | none | no | implemented |
| cloud | Servers | GET | `/servers/{id}` | `get_server` | `cloud_sdk_hetzner::cloud::servers` | no | no | none | no | implemented |
| cloud | Servers | PUT | `/servers/{id}` | `update_server` | `cloud_sdk_hetzner::cloud::servers` | no | no | none | no | implemented |
| cloud | Servers | GET | `/servers/{id}/metrics` | `get_server_metrics` | `cloud_sdk_hetzner::cloud::servers` | no | no | none | no | implemented |
| cloud | Volume Actions | GET | `/volumes/actions` | `list_volumes_actions` | `cloud_sdk_hetzner::cloud::volumes` | yes | yes | action-list | no | implemented-v0.8 |
| cloud | Volume Actions | GET | `/volumes/actions/{id}` | `get_volumes_action` | `cloud_sdk_hetzner::cloud::volumes` | no | no | action-get | no | implemented-v0.8 |
| cloud | Volume Actions | GET | `/volumes/{id}/actions` | `list_volume_actions` | `cloud_sdk_hetzner::cloud::volumes` | yes | yes | action-list | no | implemented-v0.8 |
| cloud | Volume Actions | POST | `/volumes/{id}/actions/attach` | `attach_volume` | `cloud_sdk_hetzner::cloud::volumes` | no | no | starts-action | no | implemented-v0.8 |
| cloud | Volume Actions | POST | `/volumes/{id}/actions/change_protection` | `change_volume_protection` | `cloud_sdk_hetzner::cloud::volumes` | no | no | starts-action | no | implemented-v0.8 |
| cloud | Volume Actions | POST | `/volumes/{id}/actions/detach` | `detach_volume` | `cloud_sdk_hetzner::cloud::volumes` | no | no | starts-action | no | implemented-v0.8 |
| cloud | Volume Actions | POST | `/volumes/{id}/actions/resize` | `resize_volume` | `cloud_sdk_hetzner::cloud::volumes` | no | no | starts-action | no | implemented-v0.8 |
| cloud | Volume Actions | GET | `/volumes/{id}/actions/{action_id}` | `get_volume_action` | `cloud_sdk_hetzner::cloud::volumes` | no | no | resource-action-get | yes | deferred-deprecated |
| cloud | Volumes | GET | `/volumes` | `list_volumes` | `cloud_sdk_hetzner::cloud::volumes` | yes | yes | none | no | implemented-v0.8 |
| cloud | Volumes | POST | `/volumes` | `create_volume` | `cloud_sdk_hetzner::cloud::volumes` | no | no | none | no | implemented-v0.8 |
| cloud | Volumes | DELETE | `/volumes/{id}` | `delete_volume` | `cloud_sdk_hetzner::cloud::volumes` | no | no | none | no | implemented-v0.8 |
| cloud | Volumes | GET | `/volumes/{id}` | `get_volume` | `cloud_sdk_hetzner::cloud::volumes` | no | no | none | no | implemented-v0.8 |
| cloud | Volumes | PUT | `/volumes/{id}` | `update_volume` | `cloud_sdk_hetzner::cloud::volumes` | no | no | none | no | implemented-v0.8 |
| cloud | Zone Actions | GET | `/zones/actions` | `list_zones_actions` | `cloud_sdk_hetzner::dns::zones` | yes | yes | action-list | no | implemented-v0.12 |
| cloud | Zone Actions | GET | `/zones/actions/{id}` | `get_zones_action` | `cloud_sdk_hetzner::dns::zones` | no | no | action-get | no | implemented-v0.12 |
| cloud | Zone Actions | GET | `/zones/{id_or_name}/actions` | `list_zone_actions` | `cloud_sdk_hetzner::dns::zones` | yes | yes | action-list | no | implemented-v0.12 |
| cloud | Zone Actions | POST | `/zones/{id_or_name}/actions/change_primary_nameservers` | `change_zone_primary_nameservers` | `cloud_sdk_hetzner::dns::zones` | no | no | starts-action | no | implemented-v0.12 |
| cloud | Zone Actions | POST | `/zones/{id_or_name}/actions/change_protection` | `change_zone_protection` | `cloud_sdk_hetzner::dns::zones` | no | no | starts-action | no | implemented-v0.12 |
| cloud | Zone Actions | POST | `/zones/{id_or_name}/actions/change_ttl` | `change_zone_ttl` | `cloud_sdk_hetzner::dns::zones` | no | no | starts-action | no | implemented-v0.12 |
| cloud | Zone Actions | POST | `/zones/{id_or_name}/actions/import_zonefile` | `import_zone_zonefile` | `cloud_sdk_hetzner::dns::zones` | no | no | starts-action | no | implemented-v0.12 |
| cloud | Zone Actions | GET | `/zones/{id_or_name}/actions/{action_id}` | `get_zone_action` | `cloud_sdk_hetzner::dns::zones` | no | no | resource-action-get | yes | deferred-deprecated |
| cloud | Zone RRSet Actions | POST | `/zones/{id_or_name}/rrsets/{rr_name}/{rr_type}/actions/add_records` | `add_zone_rrset_records` | `cloud_sdk_hetzner::dns::rrsets` | no | no | starts-action | no | implemented-v0.13 |
| cloud | Zone RRSet Actions | POST | `/zones/{id_or_name}/rrsets/{rr_name}/{rr_type}/actions/change_protection` | `change_zone_rrset_protection` | `cloud_sdk_hetzner::dns::rrsets` | no | no | starts-action | no | implemented-v0.13 |
| cloud | Zone RRSet Actions | POST | `/zones/{id_or_name}/rrsets/{rr_name}/{rr_type}/actions/change_ttl` | `change_zone_rrset_ttl` | `cloud_sdk_hetzner::dns::rrsets` | no | no | starts-action | no | implemented-v0.13 |
| cloud | Zone RRSet Actions | POST | `/zones/{id_or_name}/rrsets/{rr_name}/{rr_type}/actions/remove_records` | `remove_zone_rrset_records` | `cloud_sdk_hetzner::dns::rrsets` | no | no | starts-action | no | implemented-v0.13 |
| cloud | Zone RRSet Actions | POST | `/zones/{id_or_name}/rrsets/{rr_name}/{rr_type}/actions/set_records` | `set_zone_rrset_records` | `cloud_sdk_hetzner::dns::rrsets` | no | no | starts-action | no | implemented-v0.13 |
| cloud | Zone RRSet Actions | POST | `/zones/{id_or_name}/rrsets/{rr_name}/{rr_type}/actions/update_records` | `update_zone_rrset_records` | `cloud_sdk_hetzner::dns::rrsets` | no | no | starts-action | no | implemented-v0.13 |
| cloud | Zone RRSets | GET | `/zones/{id_or_name}/rrsets` | `list_zone_rrsets` | `cloud_sdk_hetzner::dns::rrsets` | yes | yes | none | no | implemented-v0.13 |
| cloud | Zone RRSets | POST | `/zones/{id_or_name}/rrsets` | `create_zone_rrset` | `cloud_sdk_hetzner::dns::rrsets` | no | no | none | no | implemented-v0.13 |
| cloud | Zone RRSets | DELETE | `/zones/{id_or_name}/rrsets/{rr_name}/{rr_type}` | `delete_zone_rrset` | `cloud_sdk_hetzner::dns::rrsets` | no | no | none | no | implemented-v0.13 |
| cloud | Zone RRSets | GET | `/zones/{id_or_name}/rrsets/{rr_name}/{rr_type}` | `get_zone_rrset` | `cloud_sdk_hetzner::dns::rrsets` | no | no | none | no | implemented-v0.13 |
| cloud | Zone RRSets | PUT | `/zones/{id_or_name}/rrsets/{rr_name}/{rr_type}` | `update_zone_rrset` | `cloud_sdk_hetzner::dns::rrsets` | no | no | none | no | implemented-v0.13 |
| cloud | Zones | GET | `/zones` | `list_zones` | `cloud_sdk_hetzner::dns::zones` | yes | yes | none | no | implemented-v0.12 |
| cloud | Zones | POST | `/zones` | `create_zone` | `cloud_sdk_hetzner::dns::zones` | no | no | none | no | implemented-v0.12 |
| cloud | Zones | DELETE | `/zones/{id_or_name}` | `delete_zone` | `cloud_sdk_hetzner::dns::zones` | no | no | none | no | implemented-v0.12 |
| cloud | Zones | GET | `/zones/{id_or_name}` | `get_zone` | `cloud_sdk_hetzner::dns::zones` | no | no | none | no | implemented-v0.12 |
| cloud | Zones | PUT | `/zones/{id_or_name}` | `update_zone` | `cloud_sdk_hetzner::dns::zones` | no | no | none | no | implemented-v0.12 |
| cloud | Zones | GET | `/zones/{id_or_name}/zonefile` | `get_zone_zonefile` | `cloud_sdk_hetzner::dns::zones` | no | no | none | no | implemented-v0.12 |
| hetzner | Storage Box Actions | GET | `/storage_boxes/actions` | `list_storage_boxes_actions` | `cloud_sdk_hetzner::storage::storage_boxes` | yes | yes | action-list | no | implemented |
| hetzner | Storage Box Actions | GET | `/storage_boxes/actions/{id}` | `get_storage_boxes_action` | `cloud_sdk_hetzner::storage::storage_boxes` | no | no | action-get | no | implemented |
| hetzner | Storage Box Actions | GET | `/storage_boxes/{id}/actions` | `list_storage_box_actions` | `cloud_sdk_hetzner::storage::storage_boxes` | yes | yes | action-list | no | implemented |
| hetzner | Storage Box Actions | POST | `/storage_boxes/{id}/actions/change_protection` | `change_storage_box_protection` | `cloud_sdk_hetzner::storage::storage_boxes` | no | no | starts-action | no | implemented |
| hetzner | Storage Box Actions | POST | `/storage_boxes/{id}/actions/change_type` | `change_storage_box_type` | `cloud_sdk_hetzner::storage::storage_boxes` | no | no | starts-action | no | implemented |
| hetzner | Storage Box Actions | POST | `/storage_boxes/{id}/actions/disable_snapshot_plan` | `disable_storage_box_snapshot_plan` | `cloud_sdk_hetzner::storage::storage_boxes` | no | no | starts-action | no | implemented |
| hetzner | Storage Box Actions | POST | `/storage_boxes/{id}/actions/enable_snapshot_plan` | `enable_storage_box_snapshot_plan` | `cloud_sdk_hetzner::storage::storage_boxes` | no | no | starts-action | no | implemented |
| hetzner | Storage Box Actions | POST | `/storage_boxes/{id}/actions/reset_password` | `reset_storage_box_password` | `cloud_sdk_hetzner::storage::storage_boxes` | no | no | starts-action | no | implemented |
| hetzner | Storage Box Actions | POST | `/storage_boxes/{id}/actions/rollback_snapshot` | `rollback_storage_box_snapshot` | `cloud_sdk_hetzner::storage::storage_boxes` | no | no | starts-action | no | implemented |
| hetzner | Storage Box Actions | POST | `/storage_boxes/{id}/actions/update_access_settings` | `update_storage_box_access_settings` | `cloud_sdk_hetzner::storage::storage_boxes` | no | no | starts-action | no | implemented |
| hetzner | Storage Box Actions | GET | `/storage_boxes/{id}/actions/{action_id}` | `get_storage_box_action` | `cloud_sdk_hetzner::storage::storage_boxes` | no | no | resource-action-get | yes | deferred-deprecated |
| hetzner | Storage Box Snapshots | GET | `/storage_boxes/{id}/snapshots` | `list_storage_box_snapshots` | `cloud_sdk_hetzner::storage::storage_boxes` | no | yes | none | no | implemented |
| hetzner | Storage Box Snapshots | POST | `/storage_boxes/{id}/snapshots` | `create_storage_box_snapshot` | `cloud_sdk_hetzner::storage::storage_boxes` | no | no | none | no | implemented |
| hetzner | Storage Box Snapshots | DELETE | `/storage_boxes/{id}/snapshots/{snapshot_id}` | `delete_storage_box_snapshot` | `cloud_sdk_hetzner::storage::storage_boxes` | no | no | none | no | implemented |
| hetzner | Storage Box Snapshots | GET | `/storage_boxes/{id}/snapshots/{snapshot_id}` | `get_storage_box_snapshot` | `cloud_sdk_hetzner::storage::storage_boxes` | no | no | none | no | implemented |
| hetzner | Storage Box Snapshots | PUT | `/storage_boxes/{id}/snapshots/{snapshot_id}` | `update_storage_box_snapshot` | `cloud_sdk_hetzner::storage::storage_boxes` | no | no | none | no | implemented |
| hetzner | Storage Box Subaccount Actions | POST | `/storage_boxes/{id}/subaccounts/{subaccount_id}/actions/change_home_directory` | `change_storage_box_subaccount_home_directory` | `cloud_sdk_hetzner::storage::storage_boxes` | no | no | starts-action | no | implemented |
| hetzner | Storage Box Subaccount Actions | POST | `/storage_boxes/{id}/subaccounts/{subaccount_id}/actions/reset_subaccount_password` | `reset_storage_box_subaccount_password` | `cloud_sdk_hetzner::storage::storage_boxes` | no | no | starts-action | no | implemented |
| hetzner | Storage Box Subaccount Actions | POST | `/storage_boxes/{id}/subaccounts/{subaccount_id}/actions/update_access_settings` | `update_storage_box_subaccount_access_settings` | `cloud_sdk_hetzner::storage::storage_boxes` | no | no | starts-action | no | implemented |
| hetzner | Storage Box Subaccounts | GET | `/storage_boxes/{id}/subaccounts` | `list_storage_box_subaccounts` | `cloud_sdk_hetzner::storage::storage_boxes` | no | yes | none | no | implemented |
| hetzner | Storage Box Subaccounts | POST | `/storage_boxes/{id}/subaccounts` | `create_storage_box_subaccount` | `cloud_sdk_hetzner::storage::storage_boxes` | no | no | none | no | implemented |
| hetzner | Storage Box Subaccounts | DELETE | `/storage_boxes/{id}/subaccounts/{subaccount_id}` | `delete_storage_box_subaccount` | `cloud_sdk_hetzner::storage::storage_boxes` | no | no | none | no | implemented |
| hetzner | Storage Box Subaccounts | GET | `/storage_boxes/{id}/subaccounts/{subaccount_id}` | `get_storage_box_subaccount` | `cloud_sdk_hetzner::storage::storage_boxes` | no | no | none | no | implemented |
| hetzner | Storage Box Subaccounts | PUT | `/storage_boxes/{id}/subaccounts/{subaccount_id}` | `update_storage_box_subaccount` | `cloud_sdk_hetzner::storage::storage_boxes` | no | no | none | no | implemented |
| hetzner | Storage Box Types | GET | `/storage_box_types` | `list_storage_box_types` | `cloud_sdk_hetzner::storage::storage_boxes` | yes | no | none | no | implemented |
| hetzner | Storage Box Types | GET | `/storage_box_types/{id}` | `get_storage_box_type` | `cloud_sdk_hetzner::storage::storage_boxes` | no | no | none | no | implemented |
| hetzner | Storage Boxes | GET | `/storage_boxes` | `list_storage_boxes` | `cloud_sdk_hetzner::storage::storage_boxes` | yes | yes | none | no | implemented |
| hetzner | Storage Boxes | POST | `/storage_boxes` | `create_storage_box` | `cloud_sdk_hetzner::storage::storage_boxes` | no | no | none | no | implemented |
| hetzner | Storage Boxes | DELETE | `/storage_boxes/{id}` | `delete_storage_box` | `cloud_sdk_hetzner::storage::storage_boxes` | no | no | none | no | implemented |
| hetzner | Storage Boxes | GET | `/storage_boxes/{id}` | `get_storage_box` | `cloud_sdk_hetzner::storage::storage_boxes` | no | no | none | no | implemented |
| hetzner | Storage Boxes | PUT | `/storage_boxes/{id}` | `update_storage_box` | `cloud_sdk_hetzner::storage::storage_boxes` | no | no | none | no | implemented |
| hetzner | Storage Boxes | GET | `/storage_boxes/{id}/folders` | `list_storage_box_folders` | `cloud_sdk_hetzner::storage::storage_boxes` | no | no | none | no | implemented |

## Post-1.0 Robot Webservice

Robot Webservice is not part of the Cloud/DNS 1.0 endpoint matrix. It is
planned for `v1.1.0+` and must be tracked in a separate Robot matrix because it
uses a different base URL, authentication model, request encoding, and resource
set.

Initial Robot groups to source-lock later:

| Group | Planned Module | Status |
| --- | --- | --- |
| server | `cloud_sdk_hetzner::robot::server` | post-1.0 |
| IP | `cloud_sdk_hetzner::robot::ip` | post-1.0 |
| subnet | `cloud_sdk_hetzner::robot::subnet` | post-1.0 |
| reset | `cloud_sdk_hetzner::robot::reset` | post-1.0 |
| failover | `cloud_sdk_hetzner::robot::failover` | post-1.0 |
| wake on LAN | `cloud_sdk_hetzner::robot::wol` | post-1.0 |
| boot configuration | `cloud_sdk_hetzner::robot::boot` | post-1.0 |
| reverse DNS | `cloud_sdk_hetzner::robot::rdns` | post-1.0 |
| traffic | `cloud_sdk_hetzner::robot::traffic` | post-1.0 |
| SSH keys | `cloud_sdk_hetzner::robot::ssh_keys` | post-1.0 |
| server ordering | `cloud_sdk_hetzner::robot::ordering` | post-1.0 |
| storage box | `cloud_sdk_hetzner::robot::storage_box` | post-1.0 |
| firewall | `cloud_sdk_hetzner::robot::firewall` | post-1.0 |
| vSwitch | `cloud_sdk_hetzner::robot::vswitch` | post-1.0 |
