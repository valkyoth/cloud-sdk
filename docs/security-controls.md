# Security Controls

| Control | Status |
| --- | --- |
| no_std default graph | active |
| unsafe forbidden | active |
| default transport dependency | none |
| default token storage | none |
| dependency policy | active |
| SBOM generation | scripted |
| cargo audit | required before tags |
| cargo deny | required before tags |
| pentest before tags | required |
| pentest content binding | final release commit changes only `security/pentest/vX.Y.Z.md` from its direct reviewed parent |
| pentest provenance | committed report with required PASS, reviewed commit, tester, scope, and date fields |
| release publishing | clean `HEAD` with a verifiable signed annotated tag; no normal-path bypass flags |
| OpenAPI integrity | full pinned SHA-256 before parsing, with size/time ceilings and no-follow descriptor reads for local inputs |
| public IPv6 targets | conservative IANA allocation allowlist pinned in `docs/IANA_IPV6_SOURCE_LOCK.md`; live registry drift is release-gated |
| secret buffer failure | JSON writes preflight capacity and leave undersized buffers unchanged |
| secret buffer cleanup | `cloud-sdk-sanitization::SecretBuffer` volatile-clears the full caller-owned destination on drop |
| private-key output | escaped atomic writer only; no raw accessor or ordinary equality; guarded cleanup tested |
| DNS TSIG policy | HMAC-SHA256 only; canonical Base64; minimum 32 decoded bytes; no ordinary equality on secret-bearing types |
| DNS RRSet mutations | source-locked RR types; bounded unique redacted records; mandatory change-TTL intent; atomic JSON-string writers |
| optional Serde boundary | default graph exclusion; no Serde `std`; 1 MiB request and 8 MiB response policies; bounded validated response envelopes |
| testkit boundary | no_std ordered mock; atomic bounded response writes; payload-free mismatch errors; redacted fixture/request debug |
| transport contract | origin-form targets only; caller-owned response buffers; no authentication, headers, TLS, retry, or network implementation |
| CodeQL default setup | repository setting |
| API source lock | active for `v0.2.0` |
| Storage Boxes drift check | active for `v0.2.0` |
