# v0.83.0 Rejected Abstractions

Status: implementation stop; pentest required.

## Generic Address Strings

Rejected because route, owner, and destination identities are operationally
sensitive and must reject noncanonical spelling before path/form preparation.
The existing protected `RobotIpAddress` remains the shared boundary.

## CIDR Text As Provider Identity

Rejected because Robot returns separate `ip` and `netmask` fields. The decoder
validates family, mask continuity, and host bits directly, then exposes a
prefix without inventing a provider wire representation.

## Treat DELETE As No Content

Rejected because the official example returns a full JSON failover object with
`active_server_ip: null`. Requiring that acknowledgement proves both route
identity and deletion outcome; a generic `204` policy would discard evidence.

## Retry DELETE As Idempotent

Rejected because Robot documents locked, failed, and incomplete transitions.
After uncertain delivery, a second request can race an in-progress provider
operation. Both reroute and deletion therefore deny automatic retry.

## One Mutation Permit Type

Rejected because moving a route and removing it have different impact.
Reroute requires mutation authority; deletion requires destructive authority.
The common plan machinery is reused while provider wrappers retain exact type
association.

## Automatic Destination Health Checks

Rejected because Robot exposes route management, not an authenticated health
oracle. Selecting and validating an operational destination remains caller
policy and cannot be inferred safely from this response family.
