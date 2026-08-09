# v0.71.0 Dependency Review

Status: implementation in progress.

This change retires the experimental AWS-LC FIPS transport from the active
workspace and removes its optional native dependency graph. The ordinary
AWS-LC provider remains admitted for the non-default blocking, deterministic-
root, and async rustls transports. No package, feature activation, build
script, native component, runtime, network stack, unsafe code, or normal
provider dependency is added.

## Root Lockfile Changes

| Package | Previous | v0.71 | Review |
| --- | --- | --- | --- |
| `aho-corasick` | `1.1.5` | `-` | Removed with the FIPS bindgen graph. |
| `aws-lc-fips-sys` | `0.13.16` | `-` | Retired optional FIPS native module. |
| `bindgen` | `0.72.1` | `-` | Removed with `aws-lc-fips-sys`. |
| `cexpr` | `0.6.0` | `-` | Removed with bindgen. |
| `clang-sys` | `1.9.1` | `-` | Removed with bindgen. |
| `either` | `1.17.0` | `-` | Removed with the FIPS build graph. |
| `glob` | `0.3.4` | `-` | Removed with clang-sys. |
| `itertools` | `0.13.0` | `-` | Removed with bindgen. |
| `libloading` | `0.8.9` | `-` | Removed with clang-sys. |
| `minimal-lexical` | `0.2.1` | `-` | Removed with nom. |
| `nom` | `7.1.3` | `-` | Removed with cexpr. |
| `prettyplease` | `0.2.37` | `-` | Removed with bindgen. |
| `regex` | `1.13.1` | `-` | Removed with bindgen. |
| `regex-automata` | `0.4.18` | `-` | Removed with regex. |
| `regex-syntax` | `0.8.11` | `-` | Removed with regex-automata. |
| `shlex` | `1.3.0` | `-` | Removed bindgen-only duplicate; ordinary AWS-LC retains current `shlex 2.0.1`. |

## Boundary Decision

- Active manifests and lockfiles contain no `aws-lc-fips-sys`.
- `cloud-sdk-reqwest` exposes no FIPS feature, policy type, builder method, or
  FIPS-specific construction error.
- The downstream feature-unification fixture covers only supported blocking,
  deterministic-root, and async transports.
- The package gate no longer compiles or ships retired FIPS fixtures.
- `scripts/check_fips_deferred.py` rejects reintroduction before the Brynja
  admission conditions in [`FIPS_DEFERMENT.md`](FIPS_DEFERMENT.md) are met.

Historical v0.23-v0.70 release, dependency, and security evidence remains
unchanged and does not describe the active v0.71 graph.
