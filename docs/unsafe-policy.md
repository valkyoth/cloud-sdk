# Unsafe Policy

First-party crates forbid unsafe code through workspace lints.

No unsafe code may be added without a versioned design document, a reviewed
threat model update, tests, and explicit approval. The expected SDK path is to
remain safe Rust in first-party crates.
