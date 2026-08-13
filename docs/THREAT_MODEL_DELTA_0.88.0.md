# Threat Model Delta 0.88.0

Status: implementation stop; pentest required.

## New Inputs

Robot SSH-key operations introduce account key names, public-key material,
legacy MD5 fingerprints, algorithm/size metadata, creation timestamps, and
key-management mutations. Public keys are not private cryptographic material,
but the account inventory and deployment associations are sensitive metadata.

## Controls

- Names and path fingerprints use protected non-copyable storage and redacted
  diagnostics. Response key text uses cleanup-owning secret storage and only
  closure-scoped access.
- OpenSSH responses are Base64-decoded, parsed as exact RFC 4253 wire values,
  and checked for algorithm-specific structure. Source `type` and `size` must
  agree with the parsed key.
- The provider MD5 fingerprint must equal the decoded key-wire MD5. The SDK
  separately computes SHA-256 over those bytes; MD5 is never treated as the
  strong SDK identity.
- Create accepts only bounded conservative OpenSSH or RFC 4716 SSH2 shapes.
  Checked create decoding normalizes either form to key wire and requires the
  provider response to match both the requested name and SHA-256 identity.
- List responses are bounded and reject duplicate fingerprints. Get and
  update responses must match the exact protected path fingerprint; rename
  responses must also match the requested name.
- Create and rename bodies are sensitive atomic forms. They require exact
  request-bound strong-digest mutation authority. Delete requires separate
  destructive authority. Automatic mutation retry remains forbidden.

## Residual Boundaries

Caller-owned source strings and transport copies remain caller-owned and need
appropriate lifecycle handling. Robot documents an MD5 path identifier; an
MD5 collision at the provider API layer cannot be removed by this SDK, so
callers should use the SDK SHA-256 fingerprint for independent identity checks.
The provider-local `created_at` value has no documented timezone and is exposed
as validated components rather than converted to an instant.
