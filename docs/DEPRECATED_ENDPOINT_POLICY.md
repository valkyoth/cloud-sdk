# Deprecated Upstream Endpoint Policy

## Default Rule

New SDK releases do not add endpoints that upstream Hetzner documentation marks
deprecated. Existing deprecated operations are omitted when a supported
replacement exists. The source-locked API matrix records each deliberate
exclusion so omission cannot be confused with an unimplemented active
operation.

## Upstream Changes

When an implemented endpoint becomes deprecated:

1. The drift process records the upstream notice, replacement, and announced
   removal date when available.
2. The SDK documents the migration before removing the public operation.
3. A pre-1.0 minor may remove it after the replacement is implemented and
   tested. After `1.0.0`, removal normally waits for the next major release.
4. Earlier removal is allowed when the endpoint no longer functions or keeping
   it would create a material security or data-loss risk.

Deprecated operations are never silently redirected to a replacement with
different cost, mutation, retry, idempotency, or deletion semantics.

## Current Hetzner Scope

The Cloud/DNS source lock contains 13 deliberately excluded deprecated
operations. Robot's deprecated legacy Storage Box operations will remain
excluded because the Console Storage Box API is the supported replacement.
Active-operation completeness is checked independently from deprecated rows.
