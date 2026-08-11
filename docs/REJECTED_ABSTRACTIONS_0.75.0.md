# v0.75.0 Rejected Abstractions

Status: release candidate; pentest and final retest passed.

## A Third-Party URL Or Form Dependency

The required grammar is small, no_std, allocation-free, and already fits the
reviewed transactional encoder. A URL crate would enlarge the default supply
chain and could apply query or WHATWG URL semantics not source-locked for
Robot.

## A Map-Based Form Model

Maps erase repeated names and can reorder fields. Robot explicitly uses
ordered duplicates and indexed bracket names, so the public input is a borrowed
ordered slice.

## Returning A Bare Body Slice

A bare slice would release cleanup ownership and allow stale secret tails in a
reused backing buffer. The encoded guard retains the mutable borrow and clears
the complete destination on drop.

## Automatically Marking Named Fields Secret

Future Robot operations know which values are secrets, but a generic name
heuristic is incomplete and bypassable. Constructors require an explicit
public or sensitive choice; value-bearing diagnostics are redacted in both
cases.

## Combining Credentials Or Network Execution

Form encoding is independently testable and does not need a username,
password, endpoint, TLS stack, or client. Credential generation and lockout
policy receive their own v0.76 security review.

## Implementing Deprecated Robot Storage Boxes

The 16 legacy operations remain excluded. Their supported Console API
replacement is already implemented and published in `cloud-sdk-hetzner`.
