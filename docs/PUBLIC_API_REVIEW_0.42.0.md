# v0.42.0 Public API Review

Date: 2026-07-30

Scope: Basic authentication, canonical signing inputs, and the narrow Robot
wire source lock.

## Core API

`cloud-sdk::authentication` adds bounded `SigningKeyId`, `SigningAlgorithm`,
`SigningNonce`, and `SigningFreshness` values; complete `SigningContext`;
ordered `SigningHeaders`; cleanup-owning `CanonicalSigningInput`; validated
cleanup-owning `SignedRequest`; and caller-implemented `RequestBodyHasher` and
`RequestSigner` traits.

The v2 canonical bytes bind provider, service, normalized endpoint scheme,
host, effective port, base path, optional audience/account/tenant presence,
key ID, algorithm, exact method and target, selected headers, internally
produced exact-body digest, nonce, and time. There is no public detached digest
constructor or unchecked signer-output helper.

Construction verifies every selected header, hashes `request.body()` inside
the transaction, retains the exact request borrow, and clears digest scratch.
Signing consumes the canonical object, rejects zero or out-of-bounds lengths,
retains the same request, and clears output on all failure paths or drop. Core
remains allocation-free and `no_std`; it acquires no key, clock, randomness,
filesystem, signer, hasher, or replay state.

## Adapter API

`cloud-sdk-reqwest` adds `BasicUsername`, `BasicPassword`,
`BasicCredentialScope`, `BasicCredential`, payload-free construction errors,
and blocking and async Basic client/builders. Basic and bearer credentials
cannot be interchanged through safe APIs.

Both Basic clients implement only the authenticated transport contract. They
apply the same complete scope validator as bearer clients before constructing
an authorization header or starting network work. The existing hardened
request executor, TLS policy, origin check, response limits, metadata parsing,
and mandatory response cleanup are shared rather than duplicated.

## Compatibility

Existing bearer APIs and raw clients are source compatible. The crate versions
change because `cloud-sdk` and `cloud-sdk-reqwest` add public APIs.

Basic credentials use a deliberately conservative printable-ASCII
interoperability profile. Expanding that profile later requires an explicit
provider source lock and ambiguity review.

The Robot JSON fixture is repository test evidence outside publishable crates.
It does not add a Robot service marker, operation, model, decoder, or client
and does not alter the current Hetzner coverage claim.
