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
| IAM console schema | `https://api.eu.ovhcloud.com/v2/iam.json` | `27a1c172c055615e25673d46f583ed0f0d4ba2d77df5a70324b7a446eeacc585` |
| API v2 principles | official `ovh/ovhcloud-docs` guide at `eb5d926b9030000cfb03386c4cbe6d60491ab63a` | `afd16253ec3d126c8ca1c66b2298be87976b35af384ffc38e6f2d7f029098ae7` |
| OAuth2 service accounts | official `ovh/ovhcloud-docs` guide at `eb5d926b9030000cfb03386c4cbe6d60491ab63a` | `84096963fe74b598a2a35adfa4f7f0a65efa878ecf1a9e8f4110371da92c30b7` |

The complete URLs, byte limits, contracts, owners, and compatibility policy
are in
[`ovhcloud-v2-probe.lock.json`](../../provider-drift/providers/ovhcloud-v2-probe.lock.json).
The live release gate fetches all four exact URLs without ambient proxies or
redirects and requires the independently generated observation to match
[`ovhcloud-v2-probe.observed.json`](../../provider-drift/providers/ovhcloud-v2-probe.observed.json).
The IAM digest covers the complete strict UTF-8 JSON object after sorting only
its unique top-level API path entries; OVH emits those entries in unstable
order. Object member order and insignificant JSON formatting are canonicalized;
every value and remaining list order stays authenticated. Non-finite constants
are rejected. The other three digests cover exact raw bytes.

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

## v0.58 Authority And OAuth Conformance

| Region | API identity | Token identity |
| --- | --- | --- |
| `eu` | `https://eu.api.ovh.com:443/v2` | `https://www.ovh.com:443/auth/oauth2/token` |
| `ca` | `https://ca.api.ovh.com:443/v2` | `https://ca.ovh.com:443/auth/oauth2/token` |

The source-bound fixture admits only these exact pairs. It rejects aliases,
cross-region combinations, unknown regions, duplicate identities, HTTP, and
credentialed redirects. Console and historical API hostnames are never token
or bearer destinations.

The neutral core converts the documented `expires_in` through explicit
caller-owned monotonic time and a caller-selected refresh margin. The reqwest
adapter binds refresh to the exact credential lineage and generation, permits
time-qualified handoff only inside the refresh window, rejects expiry and
clock rollback, and atomically installs a replacement token and lifetime.
Neither core nor the probe owns a clock, acquisition task, retry policy, or
secret store.

Run `scripts/check_ovhcloud_authority_conformance.sh` to bind the authority
fixture and OAuth response shape to the reviewed source lock and execute the
core plus blocking/async credential lifecycle tests.

## v0.59 Cursor And Header Conformance

Four locked IAM collection operations use the exact request headers
`X-Pagination-Size` and `X-Pagination-Cursor` and the response header
`X-Pagination-Cursor-Next`. The response header is retained as sensitive raw
metadata, moved into bounded cleanup-owning cursor storage, checked through
exact cycle history, and sent back only by the same retained prepared request
context and exact initially observed endpoint. Raw cursor headers are not
public. Its absence is the source-defined terminal-page signal; no body-length
heuristic overrides that signal. Binding rejects a response policy that would
discard this header, and pre-dispatch failures clear every caller buffer.

The locked IAM console schema and validation example both select schema
version `1.0`. `X-Schemas-Version` remains an explicit validation-only header;
normal production calls omit it and use the account-selected major. A schema
major change requires a new authenticated source lock and review rather than
automatic selection.

Run `scripts/check_ovhcloud_header_conformance.sh` to bind these names, the
four-operation pagination surface, terminal behavior, reviewed schema major,
prepared-response decoding, context rebinding rejection, and adversarial core
tests to the immutable probe.
