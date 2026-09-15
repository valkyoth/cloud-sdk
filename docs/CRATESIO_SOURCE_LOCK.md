# crates.io Source Lock

Status: Commits 1 through 10 are accepted in the unreleased `1.1.0` train.
Commit 11 adds [downloads/statistics](CRATESIO_DOWNLOAD_POLICY.md) and passed
incremental pentest for `51a7d946..03301ac5`; GitHub approval remains pending.
No tag or publication is authorized.
See the [catalog contract](CRATESIO_CATALOG_POLICY.md),
[discovery contract](CRATESIO_DISCOVERY_POLICY.md), [request policy](CRATESIO_REQUEST_POLICY.md),
[credential contract](CRATESIO_CREDENTIAL_POLICY.md) and
[wire policy](CRATESIO_WIRE_POLICY.md).

This lock establishes the finite reviewed scope for provider implementation. The
machine-readable source manifest is
[`provider-drift/providers/cratesio-source.lock.json`](../provider-drift/providers/cratesio-source.lock.json).
The separately bounded production, staging, and static-download authority
evidence is in
[`provider-drift/providers/cratesio-endpoints.lock.json`](../provider-drift/providers/cratesio-endpoints.lock.json).
The complete public operation inventory is
[`CRATESIO_API_SCOPE.tsv`](CRATESIO_API_SCOPE.tsv), and the independently
reviewed Cargo overlap is
[`CRATESIO_CARGO_COMPATIBILITY.tsv`](CRATESIO_CARGO_COMPATIBILITY.tsv).

## Reviewed Sources

The 2026-09-04 observation binds six official HTTPS representations:

| Evidence | Exact source | Bound |
| --- | --- | --- |
| Public API | `https://crates.io/api/openapi.json` | 1 MiB |
| Stable Cargo contract | `https://doc.rust-lang.org/cargo/reference/registry-web-api.html` | 512 KiB |
| Deployed access policy | `https://crates.io/data-access` | 128 KiB |
| OpenAPI implementation | `rust-lang/crates.io` `src/openapi.rs` at `9ae7f769cea32f38ebc2ea9ec2ce455b47641511` | 128 KiB |
| Policy implementation | `rust-lang/crates.io` `svelte/src/routes/data-access/+page.svelte` at the same commit | 128 KiB |
| Current policy observation | the same file on the official `rust-lang/crates.io` `main` branch | 128 KiB |

The manifest records each requested URL, final URL, redirect chain, media type,
exact byte length, and SHA-256 digest. Retrieval requires HTTPS, a bounded
response, a 60-second total read deadline, and same-origin redirects only. The
accepted observation has no redirects. Validation additionally requires each
requested and final URL to equal its approved official authority and path;
changing a URL, query, fragment, credential component, retrieval media type, or
bound is not an accepted lock refresh.

The deployed policy uses HTTP content negotiation. A request without an HTML
`Accept` header currently returns `404`; the authoritative
`Accept: text/html` representation returns the complete policy. The source-lock
fetcher records and enforces that request representation.

The commit-pinned policy source preserves reviewed provenance. The separately
bound `main` representation detects later changes to rate, identifying
`User-Agent`, contact, API-fallback, and preferred-data-source rules. It is
never treated as accepted evidence merely because it was fetched successfully.

## Finite Scope

The locked OpenAPI 3.1 document contains exactly 40 paths and 51 public
operations. Every operation is classified `included`; no private,
undocumented, or browser-only route is inferred. The two operations marked
deprecated by the public document remain explicit included rows so their
status cannot disappear silently.

The OpenAPI exposes `api_token`, `trustpub_token`, and `cookie`. Cookie is
recorded as upstream evidence but excluded from admitted SDK authentication.
Every operation has an anonymous, token, trusted-publishing, OIDC-body, or
one-time path-token route that does not require browser-session replay.

Seven stable Cargo Registry Web API operations overlap public OpenAPI rows:
publish, yank, unyank, owner list, owner add, owner remove, and search. Their
Cargo rows are classified `superseded` by the named OpenAPI operation rather
than implemented twice. Cargo's `/me` login URL is classified `excluded`
because it is an instruction target, not an API operation.

## Access Policy

The deployed policy requires API clients to use at most one request per second
and send an identifying `User-Agent`; contact information is strongly
recommended. API access is a fallback after the sparse index, static crate
downloads, RSS feeds, and daily database dumps. These are source-locked
provider requirements, not optional SDK defaults.

## Verification

Validate committed evidence without network access:

```console
scripts/check_cratesio_source_lock.py
```

Re-fetch every official source and reconstruct both inventories in memory:

```console
scripts/check_cratesio_source_lock.py --fetch
```

Emit a canonical payload-free semantic drift report from current official
bytes:

```console
scripts/check_cratesio_drift.py --fetch
```

The live command never rewrites accepted evidence. Digest, size, redirect,
media-type, OpenAPI version, reference, operation identity, auth, Cargo
contract, policy, or classification changes fail closed and require explicit
review. The manifest also binds both committed TSV artifacts by SHA-256, and the
final `scripts/release_1_1_gate.sh` reconstructs the observation from the
approved authorities. The semantic adapter fingerprints every operation,
parameter/request structure, component schema, authentication route, content
type, response status, stability classification, stable Cargo overlap, and
access-policy rule. Additions, removals, renames, and changed fields are
classified through the provider-neutral drift report.

Stable Cargo route comparison preserves path-parameter identity and position.
Only the operation-scoped, reviewed OpenAPI `name` to Cargo `crate_name` alias
is admitted for yank, unyank, and owner operations. Swapped, duplicated,
missing, malformed, or unreviewed parameter names fail the compatibility gate.
Every OpenAPI path placeholder must also have exactly one direct `in: path`,
`required: true` declaration across its path item and operation. Extra,
duplicate, misplaced, optional, malformed, or referenced declarations fail
closed; parameter references require a separately reviewed resolver.
Path parameters belonging to the seven stable Cargo overlaps additionally
require their exact reviewed `{"type":"string"}` schemas and the exact
OpenAPI default wire profile: `simple` style, no explosion, no
reserved-character allowance, and no content-based encoding. Restrictive JSON
Schema assertions and serialization drift therefore cannot remain
Cargo-compatible. The document default and every `$schema` declaration inside
an actual OpenAPI Schema Object must select the OpenAPI 3.1 base dialect;
custom or malformed dialect selectors fail closed before compatibility
classification. Instance-valued examples, defaults, constants, enums, and
property names remain payload data rather than dialect declarations. Schema
Object `$dynamicRef` is rejected until every target resource, dynamic anchor,
and resolution scope can be bundled and digest-bound without network access.
Ordinary `$ref` controls are interpreted only inside actual Schema Objects and
typed OpenAPI Reference Object positions. They must be strings, must use local
`#/` JSON Pointers, and must resolve inside the digest-bound document. Resolved
targets are followed and validated using the schema, parameter, header,
request-body, response, callback, path-item, example, link, or security-scheme
context that admitted the reference. Cycle guards bound recursive references.
One exception-safe traversal budget admits at most 128 simultaneously followed
typed references and inline structural descents. Long acyclic reference chains,
nested callbacks, and recursive content/encoding/header structures therefore
fail with a controlled source-lock error before reaching the Python recursion
limit. Reference and inline nesting share the same counter, so mixed structures
cannot bypass the limit.
Pointer evaluation strictly decodes URI fragments and RFC 6901 escapes, admits
canonical array indices, and rejects malformed escapes, invalid UTF-8,
noncanonical indices, and nonexistent values. Payload examples, Example Object
values, defaults, constants, enums, and other instance data may contain a
`$ref` property without being mistaken for an OpenAPI control.

## Controlled Refresh

After reviewing an upstream crates.io repository commit, stage one complete
candidate bundle without modifying accepted evidence:

```console
scripts/stage_cratesio_lock_refresh.py stage \
  --source-commit <40-character-commit> \
  --reviewed-at YYYY-MM-DD \
  --output /tmp/cratesio-refresh.json
scripts/stage_cratesio_lock_refresh.py verify /tmp/cratesio-refresh.json
```

The candidate contains the bounded source payloads, rich source manifest, both
TSV inventories, neutral provider lock, and its matching observation.
Verification checks every embedded payload against its manifest digest and
size, then reconstructs every derivative before accepting the bundle.
The typed data-access rules are emitted only when the current policy bytes are
identical to the policy at the reviewed source commit; any prose change
requires an explicit source-commit and policy review rather than heuristic
natural-language interpretation.
Publication is one non-overwriting atomic link after every source, parser,
model, digest, stable Cargo compatibility, and clean-report check succeeds. The
command never promotes or rewrites repository evidence. A reviewer must inspect
the exact payloads, report, and candidate, apply all accepted artifacts
together, run the full checks and pentest, and commit the result.
Failed, timed-out, oversized, redirected, malformed, incomplete, or internally
inconsistent observations leave accepted files untouched.

## Reviewed Drift 2026-09-15

The live OpenAPI changed the `Owner` component and the response fingerprints
for `add_owners` and `remove_owners`. The reviewed schema distinguishes user and
team owners, including user `github_username_matches` and the tagged kind.
No operation, request parameter or authentication inventory changed. Ownership
is not implemented by Commit 10; later ownership checkpoints use this refreshed
schema. The generated Version projection and existing catalog/discovery
fixtures are unchanged. The semantic drift tool correctly reported the three
schema entries plus source digests rather than declaring the API clean.

Deployed policy-page bytes also changed. The typed data-access policy and
current/pinned Svelte source remain identical: identifying user agent, one
request per second, and preferred sparse index/static/RSS/dump sources are
unchanged. All five related lock/observation/inventory artifacts were rebuilt
in a staged candidate and reviewed together; no admission rule was loosened.
The reviewed implementation source commit remains `9ae7f769cea32f38ebc2ea9ec2ce455b47641511`.

Version-list, README, author and dependency controller byte digests are now
also checked by the request-policy fetch gate. This captures seek-only
pagination, unpaginated omission of `per_page`, and the missing authors success
schema, which the OpenAPI parameter/response surface alone does not express.
Hetzner's live Cloud/Storage API specifications showed no drift in this check.

### Commit 11 Follow-up

A later observation on 2026-09-15 changed only the OpenAPI `add_owners`
description (supported owner-name prefixes) and deployed policy-page bytes.
Normalized operation/schema/parameter/auth contracts and typed policy remain
unchanged. The staged source-only refresh was reviewed and accepted. Three
additional download/reverse-dependency controller byte locks now preserve
inclusive count-window and numbered-page behavior not fully described by OpenAPI.
The subsequent Hetzner check detected a newly added network-members endpoint;
unlike documentation-only crates.io drift, it remains unaccepted until SDK
coverage is added. Details are in the [download checkpoint drift record](CRATESIO_DOWNLOAD_POLICY.md#source-and-drift-review).
