# Fuzzing

The repository has a non-published `cargo-fuzz` package under `fuzz/`. It is
excluded from the production workspace so nightly Rust, libFuzzer, sanitizers,
and fuzz-only dependencies cannot enter a published crate or its `no_std`
dependency graph.

## Pinned Tooling

| Component | Version |
| --- | --- |
| Rust nightly | `nightly-2026-07-26` |
| `cargo-fuzz` | `0.13.2` |
| `libfuzzer-sys` | `0.4.13` |

Install the exact tools:

```sh
rustup toolchain install nightly-2026-07-26 --profile minimal
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
| `incremental_json` | chunk-invariant bounded decoding, early stop, duplicate keys, and independent JSON-validity admission |
| `robot_form` | Robot form ordering, repeated fields, HTML-form byte grammar, capacity atomicity, and complete output cleanup |
| `robot_ip_response` | checked Robot IP list/detail/MAC/delete envelopes, bounded assignment state, canonical addresses/MACs, network consistency, and mutation outcome binding |
| `robot_subnet_response` | checked Robot subnet list/detail/MAC/set/delete envelopes, nullable assignments, prefix/gateway consistency, bounded selectable MAC maps, and mutation outcome binding |

Named seeds under `fuzz/seeds/` are synthetic valid and invalid cases derived
from source-locked API examples and SDK policy boundaries. Generated corpora
belong under ignored `fuzz/corpus/`; crashes belong under ignored
`fuzz/artifacts/`. Never seed from production responses, credentials, private
DNS data, request bodies, or logs.

The `incremental_json` corpus uses `.seed` files with the wire format
`[chunk_seed, stop_control, JSON payload...]`. Its deterministic integration
test verifies that the valid seed reaches `Complete` and the duplicate seed
reaches `DuplicateKey` before release fuzz smoke begins.

## Longer Runs

Use a temporary writable corpus so libFuzzer cannot add generated entries to
the reviewed seed directories:

```sh
target=response_envelopes
corpus="$(mktemp -d)"
trap 'rm -rf "$corpus"' EXIT
cp -R "fuzz/seeds/${target}/." "$corpus"
cargo +nightly-2026-07-26 fuzz run "$target" "$corpus" -- \
    -max_total_time=3600 -max_len=16384 -timeout=10
```

Targets perform no network, filesystem, environment, credential, or provider
operations. Most targets use a 16 KiB input ceiling. `raw_http1_wire` uses
`-max_len=66560` so mutations can cross the 64 KiB encoded response-head
boundary. Deterministic tests cover its below/exact/plus-one cases, the 8 MiB
body boundaries, and exact oversized model fields. `robot_reset_response` uses
a 1 MiB ceiling so mutations can reach realistic near-limit reset lists;
deterministic tests separately cover the exact 4,095/4,096/4,097 item boundary.

## Crash Reproduction

`cargo-fuzz` writes a crashing input under `fuzz/artifacts/TARGET/`. Preserve
the original file privately while investigating and replay it exactly:

```sh
cargo +nightly-2026-07-26 fuzz run response_envelopes \
    fuzz/artifacts/response_envelopes/crash-HASH
```

Minimize only after exact replay succeeds:

```sh
cargo +nightly-2026-07-26 fuzz tmin response_envelopes \
    fuzz/artifacts/response_envelopes/crash-HASH
```

Turn every confirmed SDK defect into a deterministic regression test in the
owning published crate. A sanitized minimal input may become a named seed only
after checking that it contains no secret, customer, production, or billable
resource data. Do not commit generated hash-named corpus files or artifacts.

For a release finding, record the target, exact command, sanitizer result,
root cause, remediation commit, and deterministic regression in temporary
`PENTEST.md`. Remove that scratch file after remediation and retest.
