# crates.io Credential Contract

Status: Commit 5 incremental pentest and remediation retest passed; GitHub
approval is pending. Candidate `1.1.0` is not publishable. No authenticated API
workflow is claimed complete.

## Source And Context

The admitted modes come from the reviewed
[API inventory](CRATESIO_API_SCOPE.tsv) and
[source lock](CRATESIO_SOURCE_LOCK.md). The public
[OpenAPI](https://crates.io/api/openapi.json) defines raw API-key Authorization
and temporary Bearer authentication; the
[Cargo Registry Web API](https://doc.rust-lang.org/cargo/reference/registry-web-api.html)
also specifies the raw token header. The OIDC exchange carries a required JSON
`jwt`; email and invitation operations carry path tokens. Browser cookies and
private operations remain excluded. Source commit:
`9ae7f769cea32f38ebc2ea9ec2ce455b47641511`.

The literal API-token route table contains all 23 admitted method/template
pairs, including the explicitly tracked deprecated email notification route.
CI compares that table with the complete source inventory; missing, duplicate,
added and modified authentication routes fail. Temporary tokens have only
publish and self-revoke contexts. OIDC, email and invitation contexts have one
fixed operation each. Compile-fail tests cover mismatched credential kinds.

API route admission is not full request-field validation: dynamic segments use
a conservative unescaped ASCII profile, numeric IDs admit decimal digits, and
queries are disallowed on non-GET contexts. Later domain builders own complete
field, query, body, permission and mutation policy.

## Dependency Admission

`cloud-sdk-cratesio/alloc` opts into the existing provider-neutral
`cloud-sdk-sanitization/alloc`. This reuses the already admitted exact
`sanitization = 2.0.4` graph. No new third-party package, network, TLS, runtime,
clock, filesystem, random source or default feature is introduced.

Owned secret bytes go directly into fallibly allocated `SecretString` storage;
already protected strings transfer ownership without copying. The SDK does not
stage them in ordinary owned strings. Full mutable input slices are guarded
before validation and cleared on success/error/unwind. Rotation allocates and
validates before replacing the old credential, preserving it on failure.
Exclusive borrowing prevents simultaneous use and rotation. Credentials are
non-Clone, non-Copy and have no Serde implementation or public raw getter.

The [crate README](../crates/cloud-sdk-cratesio/README.md) records exact local
byte limits and lexical profiles. They are deliberately conservative SDK
bounds, not assertions about all possible upstream credential formats. OIDC
syntax checking does not verify signatures, issuer, audience, expiry or claims;
the caller obtains assertions and crates.io authenticates them remotely.

## Adapter Boundary

Production and staging are distinct immutable credential origins. Preparation
checks the actual supplied `BoundTransport` against the exact HTTPS host,
port 443 and root base path before calling the adapter. No custom or static
origin can be selected. Typed contexts prevent cross-kind application.

Preparation writes raw API authorization, complete Bearer authorization, OIDC
JSON, or the complete secret-bearing target into caller storage. It returns no
owning wire buffer. A higher-ranked callback receives redacted material and
the same transport reference. The entire buffer is wiped before preparation
and on drop, including rejection, short output, callback errors and unwind.
Path secrets cannot escape through SDK diagnostics.

This is a trusted extension point, not atomic authenticated dispatch. The
callback must honor method, target, destination and material; emit exactly one
sensitive Authorization header; set application/json for OIDC; omit cookies;
disable redirects; and clear any additional copies. The core forbids competing
ordinary Authorization/Proxy-Authorization headers. Arbitrary callback code
can copy raw bytes or choose another destination; this is not prevented by a
closure lifetime. Later client checkpoints enforce execution policy.

Drop clears storage through sanitization, but process abort, intentional leaks,
caller/adapter/OS copies and remote storage remain outside this guarantee.
Rotation is local replacement, not remote revocation; callers manage token
expiry, rotation schedules and remote invalidation. No hidden retries occur.

## Verification

- All-kind empty, limit, whitespace, CRLF, controls, Unicode and invalid-format
  rejection with source cleanup; maximum material and exact/short output tests.
- Header scheme formatting, JSON body and secret path routing; redacted errors,
  credentials and material; clone/type-mismatch/escaping-borrow compile failures.
- Independent literal method and target assertions for every fixed constructor
  in both origins, and for emitted material including temporary-token revocation.
  These do not derive their expected values from the context under test; the
  Python inventory comparison alone does not verify constructor behavior.
- Wrong host, static host, staging mismatch, scheme, port and base-path rejection
  before callback; source-locked operation/method and reserved-header rejection.
- Protected ownership transfer, successful/failed rotation, explicit clear,
  guard drop, callback error and unwind cleanup. Full allocator-capacity wiping
  relies on sanitization's existing tests, not reads of freed memory.
- Source-inventory checker regression tests, default/alloc/all-feature builds,
  MSRV, Clippy, doctests, package verification, SBOM and repository gates.

The user confirmed a green remediation retest. The
[permanent report](../security/pentest/cratesio-commit-5.md) records the exact
reviewed range. GitHub CI and CodeQL must pass before Commit 6 begins.
