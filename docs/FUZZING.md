# Fuzzing

The repository has a non-published `cargo-fuzz` package under `fuzz/`. It is
excluded from the production workspace so nightly Rust, libFuzzer, sanitizers,
and fuzz-only dependencies cannot enter a published crate or its `no_std`
dependency graph.

## Pinned Tooling

| Component | Version |
| --- | --- |
| Rust nightly | `nightly-2026-07-20` |
| `cargo-fuzz` | `0.13.2` |
| `libfuzzer-sys` | `0.4.13` |

Install the exact tools:

```sh
rustup toolchain install nightly-2026-07-20 --profile minimal
cargo install --locked cargo-fuzz --version 0.13.2
```

The normal stable gate validates target layout, formatting, the locked
dependency graph, and named seeds:

```sh
scripts/check_fuzz_harness.sh --metadata
```

The dedicated CI job and release gate build every target and replay 64 bounded
runs from temporary copies of the committed seeds:

```sh
scripts/check_fuzz_harness.sh --build
scripts/check_fuzz_harness.sh --smoke
```

## Targets

| Target | Security boundary |
| --- | --- |
| `buffer_writers` | decimal, percent, and atomic JSON fixed-buffer writers |
| `request_targets` | origin-form paths, query validation, ordering, and encoding |
| `action_requests` | global and certificate action path/query buffer boundaries |
| `labels_dns` | labels, selectors, DNS names, endpoint paths, and record JSON |
| `pagination` | metadata coherence, entry bounds, traversal locks, and non-mutation |
| `action_polling` | progress, policy, terminal state, and non-mutation |
| `response_envelopes` | bounded action, error, and pagination JSON envelopes |
| `response_content_type` | media-type essence, parameters, quoted strings, escapes, and bounded owned response metadata |
| `checked_response` | prepared-policy binding, source-locked operation decoding, typed success/error envelopes, invalid UTF-8, oversized integers, deep nesting, and malformed payload rejection |

Named seeds under `fuzz/seeds/` are synthetic valid and invalid cases derived
from source-locked API examples and SDK policy boundaries. Generated corpora
belong under ignored `fuzz/corpus/`; crashes belong under ignored
`fuzz/artifacts/`. Never seed from production responses, credentials, private
DNS data, request bodies, or logs.

## Longer Runs

Use a temporary writable corpus so libFuzzer cannot add generated entries to
the reviewed seed directories:

```sh
target=response_envelopes
corpus="$(mktemp -d)"
trap 'rm -rf "$corpus"' EXIT
cp -R "fuzz/seeds/${target}/." "$corpus"
cargo +nightly-2026-07-20 fuzz run "$target" "$corpus" -- \
    -max_total_time=3600 -max_len=16384 -timeout=10
```

Targets perform no network, filesystem, environment, credential, or provider
operations. The 16 KiB libFuzzer input ceiling is complemented by deterministic
tests for the 8 MiB response boundary and exact oversized model fields.

## Crash Reproduction

`cargo-fuzz` writes a crashing input under `fuzz/artifacts/TARGET/`. Preserve
the original file privately while investigating and replay it exactly:

```sh
cargo +nightly-2026-07-20 fuzz run response_envelopes \
    fuzz/artifacts/response_envelopes/crash-HASH
```

Minimize only after exact replay succeeds:

```sh
cargo +nightly-2026-07-20 fuzz tmin response_envelopes \
    fuzz/artifacts/response_envelopes/crash-HASH
```

Turn every confirmed SDK defect into a deterministic regression test in the
owning published crate. A sanitized minimal input may become a named seed only
after checking that it contains no secret, customer, production, or billable
resource data. Do not commit generated hash-named corpus files or artifacts.

For a release finding, record the target, exact command, sanitizer result,
root cause, remediation commit, and deterministic regression in temporary
`PENTEST.md`. Remove that scratch file after remediation and retest.
