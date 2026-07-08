# cloud-sdk 0.1.0 Release Notes

Status: draft

## Added

- Initial Rust workspace.
- Provider-neutral `cloud-sdk` crate.
- Hetzner provider crate with internal Cloud, DNS, security, and Storage Box
  modules.
- Placeholder crates for future reqwest transport, testkit, and sanitization
  boundaries.
- MIT OR Apache-2.0 license metadata.
- Security, implementation, release, modularity, and supply-chain docs.
- Local check and release gate scripts.

## Known Limitations

- No endpoint request/response models yet.
- No transport adapter.
- No serde boundary.
- No live Hetzner API tests.
