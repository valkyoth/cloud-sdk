<p align="center">
  <b>provider-neutral mandatory cleanup and optional secret storage for cloud-sdk.</b><br>
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

Provider-neutral cleanup and secret-handling boundary for the main
[`cloud-sdk`](https://github.com/valkyoth/cloud-sdk) workspace and
[`cloud-sdk`](https://crates.io/crates/cloud-sdk) crate.

This README tracks the unreleased workspace `1.1.0` candidate. The published
stable cleanup crate remains `1.0.0` until the complete candidate train is
qualified.

Version 1.0 provides the mandatory volatile cleanup primitive used by the
stable default `no_std` SDK plus reusable caller-owned guards. It delegates
clearing to the independently reviewed
[`sanitization`](https://crates.io/crates/sanitization) crate with default
features disabled.

## Install

```toml
[dependencies]
cloud-sdk = "=1.1.0"
cloud-sdk-sanitization = "=1.1.0"
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

With the optional `alloc` feature, the reviewed
`sanitization::SecretString` and `sanitization::SecretBoxBytes` types are
re-exported. `SecretString` consumes an owned `String` without copying its
plaintext bytes, restricts access to checked closures, and volatile-clears the
full allocation capacity on drop:

```rust
# #[cfg(feature = "alloc")]
# fn main() {
extern crate alloc;

use alloc::string::String;
use cloud_sdk_sanitization::SecretString;

let secret = SecretString::from_string(String::from("temporary secret"));
assert_eq!(
    secret.try_with_secret(|value| value == "temporary secret"),
    Ok(true)
);
assert!(!alloc::format!("{secret:?}").contains("temporary secret"));
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```

`SecretBoxBytes` provides fixed-length, fallibly allocated protected bytes.
Moving the owner transfers only allocation metadata, so the classified bytes
remain at one stable address until the allocation is cleared on drop:

```rust
# #[cfg(feature = "alloc")]
# fn main() {
use cloud_sdk_sanitization::SecretBoxBytes;

let protected = SecretBoxBytes::try_from_slice(b"topology", 8)
    .unwrap_or_else(|_| unreachable!("fixed protected allocation failed"));
let before = protected.with_secret(<[u8]>::as_ptr);
let moved = protected;
assert_eq!(before, moved.with_secret(<[u8]>::as_ptr));
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```

`try_append_secret_string` grows protected text with fallible allocation and
a caller-supplied public byte ceiling. Growth prepares replacement storage,
then clears the old allocation before replacing it:

```rust
# #[cfg(feature = "alloc")]
# fn main() -> Result<(), cloud_sdk_sanitization::SecretStringAppendError> {
use cloud_sdk_sanitization::{SecretString, try_append_secret_string};

let mut secret = SecretString::empty();
try_append_secret_string(&mut secret, "bounded", 32)?;
assert_eq!(secret.try_with_secret(|text| text == "bounded"), Ok(true));
# Ok(())
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```

## Features

| Feature | Default | Effect |
| --- | --- | --- |
| `default` | yes | Empty; keeps the boundary `no_std`. |
| `alloc` | no | Adds stable owned volatile-clearing UTF-8 and fixed-byte secret storage. |
| `std` | no | Enables `alloc` and standard-library integration in `cloud-sdk`; clearing behavior is unchanged. |

Docs.rs builds with all features. The underlying `sanitization` dependency
keeps its default features disabled in every configuration.

## Security Notes

`SecretBuffer` volatile-clears its entire borrowed slice on drop, including
after early returns and unwind where unwind exists. `SecretString` and
`SecretBoxBytes` clear their full owned allocation capacities on drop. Moving
a `SecretBoxBytes` owner does not move its classified allocation.
`try_append_secret_string` reports
bounded growth failure and clears old storage before replacement.
`sanitize_bytes` provides the reviewed byte primitive used by core;
`sanitize_value` applies the same boundary to scalar lifecycle state.

These helpers do not clear immutable source strings or copies made by
transports, operating systems, crash handlers, swap, remote services, or other
processes. They also do not replace review of token ownership, logging,
environment variables, paging, compiler behavior, or process boundaries.
