# Rejected Abstractions 0.89.0

Status: release candidate; pentest and final retest passed.

## Unordered Rule Collections

Sets and maps were rejected because Robot firewall rules are indexed and
evaluated in order. The SDK preserves direction and position while rejecting
only exact duplicates.

## One Optional-Field Replacement Structure

A structure containing optional `template_id`, `whitelist_hos`, and `rules`
was rejected. It permits a source-forbidden combination. Separate inline and
template intent variants make the conflict unrepresentable.

## Permissive Network Text

Accepting host-bit CIDRs, noncanonical IPv4 text, reversed port ranges, or
protocol-free port constraints was rejected. These forms can silently widen or
change policy interpretation.

## Generic Mutation Authority

One reusable firewall mutation token was rejected. Strong-digest permits bind
to the exact prepared replacement/create/update request; destructive permits
bind separately to server clear or template delete.

## Public Raw Decoding

Raw firewall and template decoders remain internal. Public decoding requires a
checked response retaining the exact request type and requested identity.
