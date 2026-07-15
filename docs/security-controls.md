# Security Controls

| Control | Status |
| --- | --- |
| no_std default graph | active |
| unsafe forbidden | active |
| default transport dependency | none |
| default token storage | none |
| dependency policy | active |
| SBOM generation | scripted for the production workspace, reqwest feature-unification fixture, and fuzz tooling graph; cargo-sbom output is completed from locked Cargo metadata; independent package completeness and canonical freshness checks are required in CI and release gates |
| cargo audit | required before tags |
| cargo deny | required before tags |
| fuzzing | six isolated non-published libFuzzer targets; pinned nightly and cargo-fuzz; synthetic tracked seeds; temporary writable smoke corpora; deterministic regressions remain authoritative |
| pentest before tags | required |
| pentest content binding | report records an exact reviewed implementation commit that must be an ancestor of the final GitHub-validated release commit |
| pentest provenance | committed report with required PASS, reviewed commit, tester, scope, and date fields |
| release publishing | readiness and the complete release gate require one clean unchanged `HEAD` at entry and exit; clean checks explicitly include all untracked files regardless of Git status display configuration; signed annotated tag must verify; publisher captures and revalidates the clean commit, tag target, and signature before every locked crate publication; no normal-path bypass flags |
| OpenAPI integrity | exact non-redirecting HTTPS source URL with default certificate and hostname verification; full pinned SHA-256 before parsing; size/time ceilings and no-follow descriptor reads for local inputs |
| public IPv6 targets | conservative IANA allocation allowlist pinned in `docs/IANA_IPV6_SOURCE_LOCK.md`; live registry drift uses exact non-redirecting HTTPS URLs, bounded downloads, and digest-before-parse authentication |
| secret buffer failure | JSON writes preflight capacity and leave undersized buffers unchanged |
| secret buffer cleanup | `cloud-sdk-sanitization::SecretBuffer` volatile-clears the full caller-owned destination on drop |
| private-key output | escaped atomic writer only; no raw accessor or ordinary equality; guarded cleanup tested |
| DNS TSIG policy | HMAC-SHA256 only; canonical Base64; minimum 32 decoded bytes; no ordinary equality on secret-bearing types |
| DNS RRSet mutations | source-locked RR types; bounded unique redacted records; mandatory change-TTL intent; atomic JSON-string writers |
| optional Serde boundary | default graph exclusion; no Serde `std`; 1 MiB request and 8 MiB response policies; bounded validated response envelopes |
| testkit boundary | no_std ordered mock; atomic bounded response writes; payload-free mismatch errors; redacted fixture/request debug |
| transport contract | shared-reference blocking plus executor-neutral async traits; normalized credential-free endpoint identity; origin-form targets reject leading `//`, backslash, fragments, controls, spaces, and non-ASCII; responses borrow their initialized caller-buffer slice; no independent untrusted body length; no authentication, headers, TLS, retry, runtime, or network implementation |
| optional blocking transport | cloneable shared non-default reqwest/rustls client; HTTPS only; TLS 1.2 minimum; HTTP/1 and system DNS forced under feature unification; explicit bounded timeouts and user agent; no redirect, retry, proxy, referer, decompression, queue, or background worker; exact response bounds; payload-free failures |
| optional blocking FIPS mode | dedicated non-default feature; exact published FIPS/TLS constraints; explicit rustls FIPS provider; provider and complete client configuration checked at runtime; mandatory deployment roots and complete CRLs; chain-wide unknown-status denial and CRL-expiration enforcement; FIPS path wins under additive blocking features; a fresh isolated generated crate archive must contain and compile its public verifier fixtures; bundled Cargo-authenticated native source in repository checks; no current module-certificate, application, deployment, or organizational compliance claim |
| optional async transport | cloneable shared non-default reqwest/rustls client with caller-provided Tokio execution; blocking feature excluded from async-only graph; no credential lock across `.await`; complete-success response copy from bounded sanitized temporary storage; cancellation, timeout, read failure, and overflow leave caller output cleared |
| rate-limit metadata | transports admit only a complete coherent decimal header set with each field occurring exactly once; duplicate, malformed, partial, zero-limit, overflow, and remaining-above-limit values fail closed |
| pagination state | no_std caller-driven cursor with a hard page limit; caller-bound page size and first-response total/last snapshot; exact page transitions and known-last terminal state; decoded entries bounded by `per_page` and reconciled with supplied totals; traversal drift, contradictory, repeated, and empty non-terminal pages rejected; rate-limit metadata preserved per boundary |
| action polling state | no_std caller-driven poller; terminal provider failures take precedence over progress telemetry and are preserved; running progress regression and zero delay rejected; caller policy owns backoff, cancellation, timeout, sleep, and request execution |
| live smoke harness | ignored by default; repository-anchored clean-commit staging without credential variables; privileged installation into root-owned non-writable paths; isolated root-owned runtime validates ownership, modes, link count, bounded manifest, and open-descriptor SHA-256 before descriptor execution; authenticated phase invokes no Cargo/build tooling; exact read-only opt-in; fixed official origin; private regular token-file input; single-allocation bounded token read and guarded cleanup; Unix symlink, permission, and opened-file identity checks; typed GET-only catalog probes; static diagnostics without token paths, bodies, or IDs; destructive mode rejected |
| destructive live tests | not implemented; future plan requires a separate command, disposable project, short-lived read-write token, unique prefix, explicit pricing review, cleanup on every path, and empty post-run inventory |
| content-type diagnostics | validated values remain available to the adapter but all `Debug` output is structurally redacted |
| adapter secret ownership | mutable and guarded bearer-token sources clear on success/rejection; atomic rotation preserves in-flight snapshots and clears retired adapter-owned storage after last use; bearer and request-body copies are redacted and cleared through `cloud-sdk-sanitization`; immutable caller and reqwest/TLS/OS copies remain caller/operational boundaries |
| CodeQL default setup | repository setting |
| API source lock | active for `v0.2.0` |
| Storage Boxes drift check | active for `v0.2.0` |
