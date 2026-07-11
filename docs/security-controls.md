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
| pentest content binding | release-sensitive paths unchanged after the reviewed commit from `v0.11.0` |
| pentest provenance | transactionally published OpenSSH-signed commit/path/SHA-256 bundle for an immutable report Git blob, from an approved key distinct from the release signer |
| release publishing | clean `HEAD` with a verifiable signed annotated tag; no normal-path bypass flags |
| OpenAPI integrity | full pinned SHA-256 before parsing, with size/time ceilings and no-follow descriptor reads for local inputs |
| public IPv6 targets | conservative IANA allocation allowlist pinned in `docs/IANA_IPV6_SOURCE_LOCK.md`; live registry drift is release-gated |
| secret buffer failure | JSON writes preflight capacity and leave undersized buffers unchanged |
| DNS TSIG policy | HMAC-SHA256 only; canonical Base64; minimum 32 decoded bytes; no ordinary equality on secret-bearing types |
| CodeQL default setup | repository setting |
| API source lock | active for `v0.2.0` |
| Storage Boxes drift check | active for `v0.2.0` |
