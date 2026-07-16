<p align="center">
  <b>optional provider-neutral sanitization boundary for cloud-sdk.</b><br>
  Provider crates, explicit API domains, security-first release gates, and transport-free core types.
</p>

<div align="center">
  <a href="https://crates.io/crates/cloud-sdk">cloud-sdk crate</a>
  |
  <a href="https://docs.rs/cloud-sdk-sanitization">Docs.rs</a>
  |
  <a href="https://github.com/valkyoth/cloud-sdk/blob/main/docs/RELEASE_PLAN.md">Release Plan</a>
  |
  <a href="https://github.com/valkyoth/cloud-sdk/blob/main/docs/threat-model.md">Threat Model</a>
  |
  <a href="https://github.com/valkyoth/cloud-sdk/blob/main/SECURITY.md">Security</a>
</div>

<br>

<p align="center">
  <a href="https://github.com/valkyoth/cloud-sdk">
    <img src="https://raw.githubusercontent.com/valkyoth/cloud-sdk/main/.github/images/cloud-sdk.webp" alt="cloud-sdk Rust crate overview">
  </a>
</p>

# cloud-sdk-sanitization

Optional provider-neutral secret-handling boundary for the main
[`cloud-sdk`](https://github.com/valkyoth/cloud-sdk) workspace and
[`cloud-sdk`](https://crates.io/crates/cloud-sdk) crate.

This crate provides reusable caller-owned buffer sanitization outside the
default no_std SDK and provider crates. It delegates volatile clearing to the
independently reviewed [`sanitization`](https://crates.io/crates/sanitization)
crate with default features disabled.

## Install

```toml
[dependencies]
cloud-sdk = "0.31.0"
cloud-sdk-sanitization = "0.14.0"
```

## Example

```rust
use cloud_sdk_sanitization::SecretBuffer;

let mut output = [0_u8; 128];
{
    let mut guarded = SecretBuffer::new(&mut output);
    guarded.as_mut_slice()[..6].copy_from_slice(b"secret");
    assert_eq!(&guarded.as_slice()[..6], b"secret");
}
assert_eq!(output, [0_u8; 128]);
```

With the optional `alloc` feature, `SecretText` consumes an owned `String`
without copying its plaintext bytes and volatile-clears the initialized UTF-8
storage on drop:

```rust
# #[cfg(feature = "alloc")]
# fn main() {
extern crate alloc;

use alloc::string::String;
use cloud_sdk_sanitization::SecretText;

let secret = SecretText::new(String::from("temporary secret"));
assert_eq!(secret.expose_secret(), "temporary secret");
assert_eq!(alloc::format!("{secret:?}"), "SecretText([redacted])");
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```

## Features

| Feature | Default | Effect |
| --- | --- | --- |
| `default` | yes | Empty; keeps the boundary `no_std`. |
| `alloc` | no | Adds owned volatile-clearing UTF-8 secret storage. |
| `std` | no | Enables `alloc` and standard-library integration in `cloud-sdk`; clearing behavior is unchanged. |

Docs.rs builds with all features. The underlying `sanitization` dependency
keeps its default features disabled in every configuration.

## Security Notes

`SecretBuffer` volatile-clears its entire borrowed slice on drop, including
after early returns and unwind where unwind exists. `SecretText` clears its
initialized owned UTF-8 bytes on drop. `sanitize_bytes` provides the same
reviewed primitive for explicit cleanup.

These helpers do not clear immutable source strings or copies made by
transports, operating systems, crash handlers, swap, remote services, or other
processes. They also do not replace review of token ownership, logging,
environment variables, paging, compiler behavior, or process boundaries.
