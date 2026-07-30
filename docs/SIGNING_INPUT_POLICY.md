# Canonical Signing Input Policy

Status: v0.42.0 implementation stop; pentest required.

## Boundary

`cloud-sdk` defines only a bounded canonical input. Providers remain
responsible for selecting the hash, signature algorithm, signed headers,
nonce semantics, timestamp tolerance, key identity, and verification rules.
Callers remain responsible for keys, clocks, cryptographically secure
randomness, replay storage, and rotation.

The core layer does not perform I/O, allocate, read a clock, generate a nonce,
open a key store, or choose cryptography.

## Format

The domain separator is `cloud-sdk-signing-v1` followed by a zero byte.
Fields are encoded in this order:

1. method with an unsigned 8-bit byte length;
2. exact request target with an unsigned 16-bit big-endian byte length;
3. selected-header count as an unsigned 8-bit value;
4. each lower-case header name with an 8-bit length and exact value with a
   16-bit big-endian length;
5. caller-produced body digest with an 8-bit length;
6. caller-produced nonce with a 16-bit big-endian length;
7. caller-observed Unix seconds as unsigned 64-bit big-endian.

Selected headers must be strictly ordered and unique by validated canonical
name. Every selected name, value, and sensitivity classification must exactly
match the request. The exact final target bytes are reused; no second path or
query normalization occurs.

## Bounds And Cleanup

Digest bytes are capped at 128, nonce bytes at 256, selected headers at 32,
and the complete canonical input at 12,288 bytes. Construction measures and
replays the immutable snapshot transactionally. An undersized output remains
unchanged.

`CanonicalSigningInput` borrows caller storage and clears the complete buffer
on drop. Its `Debug` output exposes only length. Signer output remains
caller-owned and must be cleared according to the selected algorithm and
provider policy.

## Verification

`scripts/check_basic_and_signing.sh` covers exact vectors, ordering,
mismatches, replay-distinguishing nonce/time changes, caller-supplied hashing
and signing, every undersized output, redaction, and cleanup.
