# Public API Review 0.88.0

Status: implementation stop; pentest required.

## Added Surface

`cloud_sdk_hetzner::robot` now exposes all five active Robot SSH-key request
types, protected names and MD5 path fingerprints, bounded OpenSSH/SSH2 create
input, typed prepared and checked associations, strict owned key/list models,
algorithm and timestamp values, operation-specific failures, and request-bound
mutation/destructive permits.

The public response model exposes key material only through closure-scoped
access. Provider MD5 text remains available as a compatibility path identity,
while `RobotSshKey::sha256_fingerprint` exposes the SDK-computed strong
identity. Raw response decoders and wire parsers remain crate-private.

## Compatibility

This is additive pre-1.0 API. `cloud-sdk` advances to `0.88.0` for source and
tag identity. `cloud-sdk-hetzner` remains `0.44.0` until the v0.90 cumulative
publication checkpoint. No neutral transport, authentication, client, retry,
or permit contract changes.

## Review Result

The exact operation type remains part of preparation, response validation,
decoding, and execution authority. Create/update/delete cannot execute through
read-only client paths, and mutation and destructive permits are not
interchangeable.
