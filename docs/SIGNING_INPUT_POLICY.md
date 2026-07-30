# Canonical Signing Input Policy

Status: v0.42.0 implementation stop; pentest required.

## Boundary

`cloud-sdk` defines a bounded canonical input and a cleanup-owning signed
request. Providers remain responsible for implementing the declared hash and
signature algorithm, selecting signed headers, nonce semantics, timestamp
tolerance, key identity, and verification rules.
Callers remain responsible for keys, clocks, cryptographically secure
randomness, replay storage, and rotation.

The core layer does not perform I/O, allocate, read a clock, generate a nonce,
open a key store, or choose cryptography.

## Format

The domain separator is `cloud-sdk-signing-v2` followed by a zero byte. v1 is
not exposed. Fields are encoded in this order:

1. provider and service IDs, each with an unsigned 8-bit byte length;
2. endpoint scheme with an unsigned 8-bit byte length;
3. normalized host with an unsigned 16-bit big-endian byte length;
4. effective port as unsigned 16-bit big-endian;
5. normalized endpoint base path with an unsigned 16-bit byte length;
6. audience, account, and tenant, each as an explicit absent byte or a present
   byte followed by an unsigned 16-bit length and exact value;
7. key ID and algorithm, each with an unsigned 16-bit byte length;
8. method with an unsigned 8-bit byte length;
9. exact request target with an unsigned 16-bit big-endian byte length;
10. selected-header count as an unsigned 8-bit value;
11. each lower-case header name with an 8-bit length and exact value with a
    16-bit big-endian length;
12. internally produced body digest with an 8-bit length;
13. caller-produced nonce with a 16-bit big-endian length;
14. caller-observed Unix seconds as unsigned 64-bit big-endian.

The context uses `ProviderId`, `ServiceId`, and `EndpointIdentity`; scheme,
canonical host, effective port, and normalized base path therefore have the
same meaning as credential binding. `SigningKeyId` and `SigningAlgorithm` are
required bounded visible-ASCII values. Every context field is length-framed
or fixed-width, and optional scope presence is unambiguous.

`CanonicalSigningInput::new_hashed` invokes the caller-selected
`RequestBodyHasher` over `request.body()` inside construction. There is no
public arbitrary-digest constructor. The canonical object retains that exact
`TransportRequest`, preventing safe mutation of its borrowed body while the
object exists.

Selected headers must be strictly ordered and unique by validated canonical
name. Every selected name, value, and sensitivity classification must exactly
match the retained request. The exact final target bytes are reused; no second
path or query normalization occurs.

## Signing Output

`CanonicalSigningInput::sign_into` consumes the canonical object and invokes a
caller-provided `RequestSigner`. It rejects zero output and lengths beyond the
supplied buffer. Signer errors, invalid lengths, and panic unwind clear the
complete output. Success returns `SignedRequest`, which retains the same exact
request, exposes only the validated signature prefix, redacts diagnostics, and
clears the complete signature buffer on drop.

The core authenticated transport traits do not yet consume signed requests.
Provider integrations must accept `SignedRequest` rather than reconstructing a
request beside a detached signature.

## Bounds And Cleanup

Digest bytes are capped at 128, nonce bytes at 256, key IDs at 256,
algorithms at 128, selected headers at 32, and the complete canonical input at
12,288 bytes. Construction measures and replays the immutable snapshot
transactionally. An undersized canonical output remains unchanged.

`CanonicalSigningInput` borrows caller storage and clears the complete buffer
on drop. Digest scratch clears on success, error, and panic unwind.
`SignedRequest` owns signature-buffer cleanup. Debug output exposes no digest,
nonce, scope value, key ID, algorithm, canonical bytes, request body, or
signature bytes.

## Verification

`scripts/check_basic_and_signing.sh` covers an exact v2 vector, independent
changes to every security-domain field, ordering, mismatches, exact-body
hashing, retained request identity, invalid hasher/signer lengths, signer and
hasher failures, panic unwind, every undersized output, redaction, and
complete cleanup.
