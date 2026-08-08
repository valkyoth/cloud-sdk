# v0.66.0 Threat Model Delta

Status: implementation stop reached; pentest required.

## New Surface

Checked decoding now retains complete certificate and SSH-key responses,
including certificate chains, public keys, fingerprints, domains, resource
usage, and provider-managed issuance or renewal failures.

## Controls

- Deterministic schema evidence requires every source-known field, type,
  nullability rule, numeric/text bound, and known state from the exact pinned
  specification.
- Certificate chains admit at most five complete PEM certificate blocks under
  a 1 MiB protected-text limit. Markers, alphabet, empty blocks, truncation,
  and trailing data are checked before public model construction.
- Managed issuance and renewal states are closed source-known enums. Failed
  states require protected provider error detail; nonfailed states reject
  contradictory errors. Uploaded certificates reject managed status.
- SSH keys pass one exact seven-algorithm allowlist shared by request and
  response models, strict Base64 decoding, and bounded RFC 4253 structural
  validation. Prefix-confusable and vendor-suffixed names fail closed.
  Hetzner's 16-octet legacy MD5
  fingerprint must match the decoded wire key, and a separate SHA-256
  fingerprint is derived for identity comparisons. MD5 is compatibility-only,
  not collision-resistant proof against a malicious provider.
- Certificate chains, SSH public keys, and provider failure messages remain in
  protected owned storage with closure-scoped inspection. Names, domains,
  fingerprints, timestamps, labels, and usage metadata are redacted from
  diagnostics; owned strings are sanitized on drop.
- Parser temporaries use cleanup guards across late validation failures. The
  RFC 4253 parser operates directly on one cleanup-owned decoded allocation
  and creates no second owned public-key or comment model.
  Dedicated tests, named fuzz seeds, all-operation fixtures, vertical
  execution, and the credential-gated read-only probe cover the new routes.
- Certificate-specific error-code text survives generic classification in
  sanitized owned storage, while diagnostics remain redacted.

## Unchanged Boundaries

The default graph remains transport-free and `no_std`. The optional Serde
graph adds reviewed `no_std` Base64 and digest dependencies. Public keys
and certificate chains are not private keys, but are protected because they can
still reveal account identity and deployment topology. Callers remain
responsible for clearing any copies created inside inspection closures.
