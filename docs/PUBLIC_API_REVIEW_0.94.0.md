# Public API Review 0.94.0

Status: implementation stop; incremental pentest required.

## Added Surface

`cloud_sdk_hetzner::client` adds `RobotClient<T>`, lifecycle and execution
errors, sealed Robot operation traits, checked success/failure output,
source-locked method descriptors, and a generic permit family for the nine
server and boot mutations that lacked specialized permit wrappers.

`cloud_sdk::client::ClientKernel` adds provider-decoder execution callbacks for
blocking, `Send` async, and local-async transports. `ClientResponse` adds a
guarded decoder callback. `PreparedExecutionError` now preserves an exact
unexpected status code.

## Contract Review

- `RobotClient::official` accepts only the exact official Robot endpoint and
  captures its Basic-auth credential binding.
- Every one of the 89 active operation request types implements the sealed
  `RobotClientOperation` contract. The 45 read-only operations additionally
  implement the sealed direct-execution contract.
- Every state-changing route requires a non-forgeable permit. Existing
  cancellation, IP, subnet, reset, failover, Wake-on-LAN, reverse-DNS, SSH-key,
  firewall, vSwitch, and billable-order evidence remains mandatory.
- The generic mutation permit is sealed to server update plus rescue, Linux,
  VNC, and Windows activation/deactivation. It cannot erase stronger evidence
  required by other mutation families.
- Checked decoders retain the concrete request and cleanup-owned response
  storage. Provider errors remain typed and bounded.
- Credential rejection closes one shared generation. Explicit replacement
  requires a different credential binding; reconfirmation requires the
  existing acknowledgement type.
- `CredentialDispatchGuard` admits one in-flight request per generation and
  rejects on unclassified drop. `TransportFailure::observed_status` retains a
  final status across later response-processing failure.
- Replacement endpoint failures retain `OfficialEndpointError` through
  `RobotClientLifecycleError::ReplacementEndpoint` and `Error::source()`.

## Compatibility

The client surface is additive pre-1.0 API. Adding
`PreparedExecutionError::UnexpectedStatus`,
`CredentialAttemptError::DispatchBusy`, and
`RobotClientLifecycleError::ReplacementEndpoint` can require extra match arms
in source code that exhaustively matches this pre-1.0 API. The previous generic
response-policy classification is retained for high-level client diagnostics.

`cloud-sdk` advances to `0.94.0` for the internal tag.
`cloud-sdk-hetzner` remains at published package version `0.45.0` until the
v0.95.0 cumulative checkpoint.
