# crates.io Endpoint Policy

Status: Commit 4 implementation boundary for the unreleased `1.1.0` candidate;
incremental pentest and GitHub checks are required before acceptance.

## Official Authorities

| Purpose | Exact origin | Base path | Authorization |
| --- | --- | --- | --- |
| Production API | `https://crates.io` | `/` | operation policy decides in Commit 5 and later |
| Staging API | `https://staging.crates.io` | `/` | separate staging context; production credentials are not implied |
| Static package downloads | `https://static.crates.io` | `/` | always omitted |

The production API and static download origins match the official
[`crates.io-index` configuration](https://github.com/rust-lang/crates.io-index/blob/master/config.json).
The official crates.io repository documents
[`staging.crates.io`](https://github.com/rust-lang/crates.io/blob/main/docs/CONTRIBUTING.md)
as its staging backend. Commit 4 freezes those exact lowercase ASCII hosts,
HTTPS, effective port 443, and root base path in provider code and tests.
The exact index configuration and commit-pinned staging documentation are
recorded in `provider-drift/providers/cratesio-endpoints.lock.json`. Offline
validation runs in the complete repository gate; live reconstruction is:

```console
scripts/check_cratesio_endpoints.py --fetch
```

No external GitHub or GitLab authority is an SDK-owned trusted-publishing token
destination. The public crates.io exchange operation accepts a caller-supplied
OIDC assertion body at the production crates.io authority. OIDC acquisition is
outside this provider's endpoint set and remains caller-owned.

## Request Targets

`ApiRequestTarget` accepts only bounded canonical origin-form targets below
`/api/v1/`. It reuses the provider-neutral request-target grammar, which rejects
absolute URLs, network-path references, fragments, controls, Unicode, dot
segments, ambiguous separators, lowercase or malformed percent triplets,
encoded separators and controls, and needless encoding of unreserved bytes.

`StaticDownloadTarget` additionally requires a query-free
`/crates/{name}/{archive}.crate` path. Operation-specific crate-name and version
construction remains assigned to later checkpoints.

## Redirects

The transport adapters keep automatic redirects disabled. A caller may create
a `DownloadRedirect` only when all of these conditions hold:

1. The source endpoint is the exact production API origin.
2. The source target is exactly
   `/api/v1/crates/{name}/{version}/download` without a query.
3. The absolute `Location` starts with the exact
   `https://static.crates.io` origin and contains a canonical static target.
4. The destination directory and `{name}-{version}.crate` archive match the
   source target exactly.
5. The follow-up request omits API authorization.

Downgrades, ports, user information, Unicode authorities, trailing-dot aliases,
host suffixes, fragments, queries, traversal, encoded separators, mismatched
archives, staging-to-production redirects, and oversized locations fail
closed. Redirect execution itself is introduced only with the later checked
client workflow; Commit 4 defines the authority and target proof required by
that execution.

## Custom Endpoints

`AcknowledgedCustomApiEndpoint::new` requires both a previously validated HTTPS
`EndpointIdentity` and an explicit
`CustomEndpointAcknowledgement::trusted_operator_configuration()` value. This
makes operator trust visible but cannot determine whether configuration is
operationally trustworthy. Custom endpoint values must never be selected by a
tenant, request payload, webhook, or other untrusted source.

Custom endpoints are API destinations only. They are not admitted as static
download authorities and cannot participate in the official cross-authority
download redirect proof.
