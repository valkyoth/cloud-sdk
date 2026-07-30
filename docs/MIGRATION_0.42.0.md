# Migrating To v0.42

v0.42 adds Basic authentication transports and provider-neutral canonical
signing inputs. Existing bearer clients keep their v0.41 behavior.

## Dependency Versions

```toml
[dependencies]
cloud-sdk = "0.42.0"
cloud-sdk-hetzner = "0.32.3"
cloud-sdk-reqwest = { version = "0.29.0", features = ["blocking-rustls"] }
cloud-sdk-sanitization = "0.16.0"
cloud-sdk-testkit = "0.24.2"
```

`cloud-sdk-sanitization` is unchanged and is not published for this release.
The Hetzner and testkit releases only update their `cloud-sdk` dependency.

## Basic Authentication

`BasicUsername`, `BasicPassword`, `BasicCredentialScope`, and
`BasicCredential` are type-separated from bearer credentials. Construct a
`BlockingBasicClientBuilder` or `AsyncBasicClientBuilder`; the bearer builders
do not accept Basic credentials.

The username profile is nonempty visible ASCII without spaces or colons. The
password profile is nonempty printable ASCII and may contain spaces and
colons. These conservative rules avoid the ambiguous default character
encoding in RFC 7617. Both fields and the complete encoded authorization value
are bounded.

Prefer `from_mut_bytes` or `from_secret_buffer` constructors when caller-owned
input can be cleared. The immutable `new(&str)` constructors cannot clear
their source. Adapter-owned input, intermediate `user:password` bytes, encoded
authorization storage, and header copies clear through
`cloud-sdk-sanitization`.

Basic clients require the same complete provider, service, endpoint, audience,
account, and tenant policy as bearer clients. Provider, service, and endpoint
must be exact `Required` values on every send.

## Canonical Signing Inputs

Core adds `SigningContext`, `SigningKeyId`, `SigningDigestAlgorithm`,
`SigningAlgorithm`, `SigningNonce`, `SigningFreshness`, `UnixTime`,
`SigningHeaders`, `CanonicalSigningInput`, `RequestBodyHasher`,
`RequestSigner`, and `SignedRequest`.

`CanonicalSigningInput::new_hashed` uses the v2 length-framed format. It binds
provider, service, normalized endpoint identity with tagged canonical host
bytes, optional audience/account/tenant scope, key ID, distinct digest and
signature algorithms, exact method and target, selected ordered headers, an
internally produced digest of the retained request body, nonce, and time.
Equivalent IPv6 spellings produce the same bytes. No public arbitrary-digest
constructor exists.

`RequestBodyHasher::digest_algorithm` must return the algorithm implemented by
the hasher. Construction rejects a mismatch with the
`SigningDigestAlgorithm` bound into `SigningContext` before hashing.

`sign_into` consumes the canonical object, validates signer output, and returns
a cleanup-owning `SignedRequest` retaining the exact signed
`TransportRequest`. Replace any code written against the unpublished v1
candidate API; v1 bytes are intentionally not accepted or produced.

The SDK intentionally supplies no hash algorithm, signature algorithm, key,
clock, random source, replay cache, filesystem, or key store. Provider crates
must define those choices before exposing a signed operation.

## Robot Boundary

v0.42 also source-locks a narrow credential-free Robot wire fixture. It proves
Basic/form/error/quota/maintenance/lockout/empty-body distinctions but adds no
Robot operation model or live request. Complete Robot implementation remains
scheduled for later pre-1.0 milestones.

See [`AUTHENTICATION_POLICY.md`](AUTHENTICATION_POLICY.md),
[`SIGNING_INPUT_POLICY.md`](SIGNING_INPUT_POLICY.md), and
[`ROBOT_WIRE_SOURCE_LOCK.md`](ROBOT_WIRE_SOURCE_LOCK.md).
