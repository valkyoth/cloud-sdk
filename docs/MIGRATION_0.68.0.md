# Migrating To v0.68.0

v0.68.0 is an internal source milestone after signed v0.67.0. No crate is
published; applications remain on package versions from v0.65.0 until the
cumulative v0.70.0 checkpoint.

## Application Code

No caller migration is required. Public operation marker names, associated
types, checked decoding, permits, and transport behavior are unchanged.
`OperationDescriptor` now exposes path-template and response-identity metadata.
Request preparation additionally rejects any internal encoder output that does
not match its source-locked template.

Provider integrations can review the complete binding contract in
`docs/TYPED_OPERATION_BINDINGS.tsv`. When changing a source-locked operation,
regenerate both operation artifacts and run the complete verifier:

```sh
scripts/generate_operation_associations.py
scripts/generate_typed_operation_bindings.py
scripts/check_typed_operation_bindings.py
```

The final command also checks all endpoint and body adapters through the Rust
AST tooling, compares compiled descriptor evidence, and rejects deprecated
executable bindings.
