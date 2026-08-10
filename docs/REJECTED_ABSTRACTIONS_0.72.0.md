# v0.72.0 Rejected Abstractions

Status: implementation stop reached; pentest required before tagging.

## Hand-Written Security Method Classification

Maintaining 14 methods separately from the operation manifest would permit
method, service, pagination, and permit drift. v0.72 generates and checks the
complete surface.

## One-Step SSH-Key Rotation

The API exposes create and delete as separate operations and cannot prove that
a replacement key has propagated. The SDK does not synthesize an atomic
rotation or automatically delete the old key. Applications verify deployment
before separately authorizing destructive deletion.

## Owned Private-Key Request Objects

Copying caller private-key text into hidden client storage would enlarge the
secret lifetime and cleanup surface. Request domains borrow validated material,
and only cleanup-owning preparation storage receives the escaped wire value.

## Direct Security Mutation Calls

Certificate retry, create/update, SSH-key create/update, and deletions can
change access or availability. Named execution therefore requires an exact
plan-confirm permit attempt.

## Executable Custom Endpoints

Official operation credentials are not sent to configurable destinations.
Custom clients remain constructible only with explicit operator trust and have
no generated execution methods.
