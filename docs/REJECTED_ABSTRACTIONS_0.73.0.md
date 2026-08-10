# v0.73.0 Rejected Abstractions

Status: implementation stop reached; pentest required before tagging.

## Hand-Written Storage Method Classification

Maintaining 31 methods separately from the operation manifest would permit
method, service, pagination, sensitivity, and permit drift. v0.73 generates
and checks the complete surface.

## Automatic Product Selection Or Cost Approval

The SDK cannot infer workload requirements, price acceptance, location, or
budget. Box creation and type changes retain explicit cost permits and exact
caller-reviewed plans.

## Automatic Password Generation Or Rotation

The SDK owns no entropy source or secret store. It accepts validated borrowed
password markers, confines serialized values to cleanup-owning storage, and
does not retain, generate, rotate, or retry credentials.

## Automatic Snapshot Rollback

Rollback can replace current data and is not treated as recovery merely
because the endpoint is action-shaped. It remains a separately reviewed
destructive operation with no automatic retry.

## Executable Custom Endpoints

Official operation credentials are not sent to configurable destinations.
Custom clients remain constructible only with explicit operator trust and have
no generated execution methods.
