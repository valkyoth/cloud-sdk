# Native Build Review

Status: reviewed for the v0.98.0 platform qualification milestone.

## Boundary

The default workspace graph remains transport-free and has no native crypto
build. Native build tools enter only when an optional `cloud-sdk-reqwest`
transport feature is selected on Linux, Windows, macOS, or FreeBSD.

The all-feature, all-target Cargo resolution contains build scripts for data
generation, target discovery, procedural macros, and platform bindings. A
Cargo `custom-build` target does not by itself mean C or assembly is compiled.
The exact package/version inventory is enforced by
`scripts/check_native_build_boundary.py`; any addition, removal, or version
change requires a new review. The same gate independently resolves each of the
three production transport features for Linux, Windows, both macOS
architectures, and FreeBSD. It follows effective normal Cargo edges from the
reqwest crate and requires the exact `cloud-sdk-reqwest -> aws-lc-rs ->
aws-lc-sys` chain while rejecting active `ring` or FIPS backends.

## Cryptographic Native Code

- `aws-lc-rs 1.18.0` selects bundled `aws-lc-sys 0.44.0` for admitted native
  transport builds. The repository forces `AWS_LC_SYS_USE_SYSTEM=0`, rejects
  target-specific override variables, and tests every native Cargo entry point
  for this policy.
- `ring 0.17.14` is retained in Cargo's all-target resolution through
  target-specific upstream verifier branches. It is not selected by the
  Linux, Windows, macOS, or FreeBSD transport graph, but remains pinned and
  reviewed because Cargo metadata and SBOM evidence must include conditional
  edges.
- `aws-lc-fips-sys` is forbidden from features, manifests, lockfiles, package
  contents, and claims. FIPS remains deferred until Brynja is ready.

Bundled AWS-LC requires the platform C/C++ compiler, assembler, CMake, linker,
and upstream build tooling required by that exact crate release. These tools
are part of the deployment build trust boundary. GitHub-hosted runner labels
are native compatibility evidence, not immutable or reproducible toolchain
images and not compliance accreditation. The workflow-policy checker admits
only the reviewed GitHub-hosted labels, closes the complete `matrix.os` axis,
and rejects arrays, self-hosted labels, and matrix include overrides.

## Qualification

Native CI executes every individual reqwest feature and the combined graph on
Linux, Windows, macOS ARM64, and macOS x86-64. FreeBSD receives compile
evidence because no GitHub-hosted FreeBSD runner is available. Android, iOS,
WASM, and bare-metal reject transport features with a crate-owned diagnostic
before target-incompatible networking dependencies compile. The negative gate
executes blocking platform roots, blocking deterministic roots, and async
transport separately for every unsupported target class.

The portable crates independently compile default, alloc, and Serde graphs on
all documented representative targets. Package verification, docs.rs
all-feature metadata, MSRV checks, dependency policy, RustSec, and complete
SBOM freshness remain separate release gates.
