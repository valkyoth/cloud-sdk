# OVHcloud API v2 Probe Threat Note

## Security Objective

Use a materially different provider to find weaknesses in neutral contracts
without shipping an OVHcloud client or placing real credentials, mutations, or
cost-bearing behavior in the repository or CI.

## Trust Boundaries

- Official unauthenticated documents and schemas are hostile until exact URL,
  DNS, TLS, byte, time, and SHA-256 checks pass.
- The reviewed adapter parses all fetched input inside the killable provider
  drift worker and emits only bounded payload-free evidence.
- OAuth client IDs, client secrets, access tokens, tenant data, account names,
  URNs, resource identifiers, cursor values, task messages, and event messages
  are never source-lock or test-fixture inputs.
- API and token authorities are region-bound. A token must never be sent to a
  different region, alias, redirect target, configurable endpoint, or
  tenant-controlled host.

## Principal Risks

### Source and schema substitution

Mutable official sources can change without notice. Exact endpoints are
hard-coded, the validated DNS address set is used directly for the TLS socket,
redirects and proxies are disabled, and every response is authenticated
against its reviewed digest. The two GitHub guides are pinned to an immutable
official commit. The IAM schema authenticator normalizes only its unstable
unique top-level path order plus insignificant object representation before
hashing the complete strict UTF-8 JSON value; duplicate keys, duplicate paths,
and non-finite constants fail closed.
Digest rotation requires review, a new observation, tests, and pentesting.

### Credential exfiltration and overbroad scopes

The guide demonstrates `scope=all`, but that is evidence of protocol shape,
not SDK guidance. Probe execution must use the minimum read-only IAM actions
for selected operations. Credentials remain region, provider, service, and
endpoint bound, and token acquisition must not follow redirects.

### Expiry and rotation races

`expires_in` is a duration supplied by the provider, not a trustworthy local
clock. The v0.58 conformance layer converts it through explicit caller-owned
monotonic time, rejects zero/overflow/incoherent refresh windows, treats expiry
as exclusive, and issues refresh handoffs only inside the configured window.
Token and replacement lifetime rotate atomically under one generation-safe
compare-and-swap; in-flight snapshots retain their prior generation and
retired storage clears after its final owner drops.

### Schema override misuse

`X-Schemas-Version` is source-locked as validation-only. Production requests
must normally use the account-selected major version. A caller-provided or
tenant-controlled override cannot silently select an incompatible schema.

### Cursor and asynchronous-data disclosure

Cursors, task errors, event links, and messages may contain opaque provider or
tenant data. They are sensitive by default, bounded, never formatted in public
errors, and cleared with their owning response or workflow state.

### Accidental support or publication

The probe is not a Cargo package, workspace member, release-plan crate, or
publisher entry. CI checks this invariant. No probe result expands the Hetzner
support claim or pre-approves a future OVHcloud provider crate.

## Exclusions

Mutation, ordering, billing, account creation, service-account management,
OpenStack APIs, API v1 compatibility, live authenticated calls, and complete
OVHcloud product coverage are outside this probe. No claim of provider,
regulatory, military, or organizational accreditation is made.
