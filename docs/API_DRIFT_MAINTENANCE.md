# Hetzner API Drift Maintenance

This document covers the full Hetzner-specific OpenAPI workflow. New provider
probes and crates also use the provider-neutral manifest and canonical diff
layer in [`PROVIDER_DRIFT.md`](PROVIDER_DRIFT.md). The neutral Hetzner bridge
must remain green, but it does not replace the deeper checks below.

This runbook governs changes detected between the reviewed Hetzner OpenAPI
source lock and the current official Cloud/DNS and Console API specifications.
The detector never modifies source, lock evidence, or release metadata unless a
maintainer supplies both lock-refresh flags.

## Monitoring

The read-only `Hetzner API Drift` GitHub workflow runs every Monday and can be
started manually. It invokes:

```bash
scripts/check_hetzner_api_drift.py --fetch
```

Release gates run the same live comparison. The fetch accepts only the two
exact official HTTPS URLs, rejects redirects, bounds connection and total
time, limits each response to 32 MiB, and requires valid UTF-8 JSON with an
object root. A current source digest may differ from the reviewed digest so the
tool can classify the change, but any digest or semantic difference makes the
command fail. Fetched content is maintenance input only and is never compiled,
packaged, or accepted automatically.

## Triage

Treat every nonzero drift result as a release stop until it is explicitly
accepted, rejected, or deferred.

| Category | Required review |
| --- | --- |
| Added operation | Confirm it is official and non-deprecated, assign an owner module and release, then add it to the API matrix. |
| Removed operation | Confirm the upstream removal and compatibility impact. Do not silently remove a public SDK API. |
| Deprecated operation | Record the replacement and removal date. Keep it excluded or provide a documented migration policy. |
| Changed operation | Review method, path, parameters, request body, responses, authentication, pagination, actions, and cost impact. |
| Schema-only change | Identify every request/response model using the schema and add positive, negative, and adversarial tests as needed. |
| Changed source digest | Compare the complete old and new documents. A prose-only change may rotate evidence without changing semantic fingerprints. |

Check the official Hetzner changelog and reference documentation during triage.
Do not infer safety only from the category or fingerprint value.

## Decisions

### Accept

1. Preserve both the reviewed old document and newly fetched document outside
   the repository long enough to inspect their complete diff.
2. Confirm the source URL, digest, OpenAPI version, API title, path and operation
   counts, response headers, and relevant changelog entries.
3. Implement required SDK, validation, test, API matrix, and documentation
   changes before refreshing the lock.
4. Update the pinned SHA-256 values in
   `scripts/check_hetzner_api_drift.py`, `scripts/check_hetzner_upstream.sh`,
   and `docs/SPEC_LOCK.md` in the same reviewed change.
5. Refresh fingerprints only after review:

   ```bash
   scripts/check_hetzner_api_drift.py \
       --fetch --write-lock --accept-lock-refresh
   ```

6. Complete the upstream-drift release-note template, run the full release
   gate, and include the change in pentest scope.

The explicit refresh flags authorize fingerprint file replacement only when
the fetched bytes match the newly reviewed pins; they do not approve SDK
behavior or a new source digest by themselves.

### Reject

Reject a result when the source is malformed, inconsistent with authoritative
documentation, unexpectedly redirected, too large, unavailable, or otherwise
not reviewable. Do not change pins or fingerprints. Record the reason in the
maintenance issue and rerun the detector after Hetzner resolves the source.

### Defer

Deferral is allowed only when the changed surface is not claimed by the SDK and
the API matrix records the exact status and rationale. Keep the detector red
until a reviewed source-lock update records that decision; do not suppress an
operation or schema merely to restore CI.

## Verification

Run at minimum:

```bash
scripts/test-hetzner-api-drift.py
scripts/check_hetzner_api_drift.py --local-only
scripts/check_hetzner_api_drift.py --fetch
scripts/checks.sh
```

The live command must report the reviewed source digests and `no drift` after
an accepted refresh. Commit source-lock files, implementation, tests,
documentation, release notes, and security evidence together.

## Unpublished OVHcloud Probe

The excluded v0.57-v0.61 OVHcloud API v2 probe uses the same neutral drift
engine but is not a provider support claim. Its source inventory, selected
read-only operations, and threat boundaries are documented in
[`provider-probes/ovhcloud-v2/README.md`](../provider-probes/ovhcloud-v2/README.md).

Run the tracked observation check locally with:

```bash
scripts/check_ovhcloud_probe.py
scripts/check_provider_drift.py \
    --plugin provider-drift/plugins/normalized-json-v1.json \
    --lock provider-drift/providers/ovhcloud-v2-probe.lock.json \
    --observation provider-drift/providers/ovhcloud-v2-probe.observed.json \
    --fetch-sources
```

Any official source-byte or normalized-contract change is a release stop. The
probe must remain outside Cargo workspace membership, package discovery,
publish ordering, and supported-provider documentation. A future full
`cloud-sdk-ovhcloud` implementation requires a separate post-1.0 plan and
review; the probe cannot be renamed or promoted into it.
