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
| pentest provenance | detached OpenSSH signature from an approved key distinct from the release signer |
| release publishing | clean `HEAD` with a verifiable signed annotated tag; no normal-path bypass flags |
| OpenAPI integrity | full pinned SHA-256 before parsing, with size and time ceilings |
| secret buffer failure | JSON writes preflight capacity and leave undersized buffers unchanged |
| CodeQL default setup | repository setting |
| API source lock | active for `v0.2.0` |
| Storage Boxes drift check | active for `v0.2.0` |
