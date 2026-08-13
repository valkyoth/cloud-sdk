# Rejected Abstractions 0.88.0

Status: implementation stop; pentest required.

## MD5 As The SDK Identity

Robot paths require the provider's colon-separated MD5 fingerprint. Reusing
that value as the SDK's cryptographic identity was rejected. The decoder
verifies it for wire compatibility and computes SHA-256 independently.

## Unchecked Provider Metadata

Treating `type`, `size`, `fingerprint`, and `data` as unrelated display fields
was rejected. All four are reconciled against one parsed RFC 4253 key wire.

## One Generic Key Mutation

A generic mutation that could rename, create, or delete based on optional
fields was rejected. Separate request and checked-response types preserve exact
status, body, retry, and execution-authority rules.

## Public Raw Decoding

Raw SSH-key decoders and key-wire helpers remain internal. Public decoding
requires a checked response retaining the exact request type and identity.
