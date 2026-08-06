# OVHcloud API v2 Architecture Probe

Status: unpublished conformance evidence; not a supported provider.

This probe challenges provider-neutral `cloud-sdk` contracts before their
v0.62.0 freeze. It is deliberately limited to eight stable, authenticated,
read-only IAM operations. It does not provide an OVHcloud SDK, create a Cargo
package, make an API-completeness claim, or approve any mutation or billable
operation.

## Locked Sources

| Evidence | Official source | SHA-256 |
| --- | --- | --- |
| API v2 section index | `https://api.eu.ovhcloud.com/v2` | `88e2566259826d7a224ce88c7fc141ed742cad0365457f3132c9d774bf19db6c` |
| IAM console schema | `https://api.eu.ovhcloud.com/v2/iam.json` | `f354014402442e9369c42873feb194151b30b3cea0e306bfdaada8b791870d44` |
| API v2 principles | official `ovh/ovhcloud-docs` English guide | `afd16253ec3d126c8ca1c66b2298be87976b35af384ffc38e6f2d7f029098ae7` |
| OAuth2 service accounts | official `ovh/ovhcloud-docs` English guide | `84096963fe74b598a2a35adfa4f7f0a65efa878ecf1a9e8f4110371da92c30b7` |

The complete URLs, byte limits, contracts, owners, and compatibility policy
are in
[`ovhcloud-v2-probe.lock.json`](../../provider-drift/providers/ovhcloud-v2-probe.lock.json).
The live release gate fetches all four exact URLs without ambient proxies or
redirects and requires the independently generated observation to match
[`ovhcloud-v2-probe.observed.json`](../../provider-drift/providers/ovhcloud-v2-probe.observed.json).

## Candidate Surface

[`CANDIDATES.tsv`](CANDIDATES.tsv) records the eight selected operations. Every
row is `GET`, stable production, bearer-authenticated, and either unpaginated
or uses the documented cursor headers. The source adapter independently
reconstructs method, path, response type, IAM actions, authentication, and
pagination from the console schema.

This surface was selected to exercise collection and resource responses,
encoded URN and UUID path identities, operations with and without declared IAM
actions, and header cursor pagination without granting mutation authority.

## Recorded Differences

The source lock captures architecture facts that differ from Hetzner:

- EU and CA API authorities pair with distinct OAuth2 token authorities.
- OAuth2 client credentials return expiring bearer tokens.
- `X-Schemas-Version` is a validation-only schema override.
- Cursor and page size are request headers; the next cursor is a response
  header whose absence terminates pagination.
- Long-running provider changes may expose `/task` and `/event` resources.

v0.58.0 through v0.61.0 implement conformance fixtures for these facts. A
future supported `cloud-sdk-ovhcloud` requires a separate full source lock,
scope, threat model, release plan, crate version history, and pentest.
