# Upstream API Drift Release Note Template

Use this section in the release notes whenever a Hetzner source digest,
operation fingerprint, or component schema fingerprint changes. Remove
instructions and empty rows before release.

## Upstream API Drift

Decision: `accepted | rejected | deferred`

| API | Previous SHA-256 | Current SHA-256 | Official source |
| --- | --- | --- | --- |
| Cloud/DNS | `<digest>` | `<digest>` | <https://docs.hetzner.cloud/cloud.spec.json> |
| Console | `<digest>` | `<digest>` | <https://docs.hetzner.cloud/hetzner.spec.json> |

Detected changes:

| Category | Operations or schemas | SDK disposition |
| --- | --- | --- |
| Added | `<items or none>` | `<implemented, assigned, or excluded reason>` |
| Removed | `<items or none>` | `<compatibility decision>` |
| Deprecated | `<items or none>` | `<migration or exclusion policy>` |
| Changed | `<items or none>` | `<model and validation changes>` |
| Schema-only | `<items or none>` | `<affected models and tests>` |

Review evidence:

- Official changelog/reference entries: `<links and dates>`
- Complete source diff reviewed by: `<reviewer>`
- API matrix changes: `<summary or none>`
- Security and cost impact: `<summary>`
- Regression and adversarial tests: `<summary>`
- Known deferred work: `<release assignment or none>`
