# Rejected Abstractions 0.90.0

Status: release candidate; pentest and final retest passed.

## Optional Update Fields

A structure with optional `name` and `vlan` was rejected because it represents
an empty update. `RobotVSwitchUpdateIntent` admits rename, VLAN change, or both.

## Permissive Member Strings

Arbitrary server text was rejected. Membership accepts only canonical positive
server numbers or canonical IP addresses and rejects exact duplicates.

## Unbounded Membership

An allocation-grown list was rejected. Membership mutations and decoded
provider collections have explicit ceilings and use fallible allocation.

## Generic Mutation Authority

One reusable Robot mutation token was rejected. Every vSwitch permit retains
the exact request association; cancellation and removal require destructive
authority distinct from create, update, and attachment.

## Inferred Reconciliation

Synthesizing vSwitch state from an empty mutation acknowledgement was rejected.
The SDK returns `()` and requires a later detail read when confirmed state is
needed.

## Public Raw Decoding

Raw vSwitch decoders remain internal. Public decoding requires a checked
response retaining the exact request and expected identity.
