# Migrating To v0.64.0

v0.64.0 is an internal source milestone after signed v0.63.0. No crate is
published; applications remain on the versions from the v0.60.0 checkpoint.

Metric timestamps and steps are now exact values rather than binary floats:

```rust,ignore
let exact_step = metrics.step().as_str();
let exact_timestamp = metrics.series()[0].points()[0].timestamp().as_str();
```

Use `try_clone()` when a complete metric result must be copied. Its diagnostic
output deliberately does not include timestamps, values, names, or range text.

Composite actions are no longer flattened. Use `action()` for the singular
source field, `actions()` for the source collection, and `next_actions()` for
follow-up actions. Nullable secret outputs can be inspected without exposing
their value:

```rust,ignore
match composite.secret("root_password") {
    None => { /* field absent */ }
    Some(None) => { /* provider returned null */ }
    Some(Some(secret)) => secret.try_with_secret(|value| use_password(value))?,
}
```

`actions()` therefore has a narrower meaning than in v0.63. Unknown provider
error codes remain classified as `ApiErrorCode::Unknown`, but callers can now
read their exact validated text through `code_text()`.

Checked Cloud action and metric timestamps now reject lowercase separators and
numeric UTC offsets. The source contract for these models is canonical UTC
text ending in uppercase `Z`.

The next cumulative crates.io checkpoint is v0.65.0.
