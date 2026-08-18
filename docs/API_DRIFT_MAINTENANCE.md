# Hetzner API Drift Maintenance

This document covers the full Hetzner-specific API-source workflow. New provider
probes and crates also use the provider-neutral manifest and canonical diff
layer in [`PROVIDER_DRIFT.md`](PROVIDER_DRIFT.md). The neutral Hetzner bridge
must remain green, but it does not replace the deeper checks below.

This runbook governs changes detected between the reviewed Hetzner OpenAPI
source lock and the current official Cloud/DNS and Console API specifications,
and between the reviewed Robot HTML lock and the official Robot Webservice
documentation. The detectors never modify source, lock evidence, or release
metadata unless a maintainer uses an explicitly documented OpenAPI lock-refresh
procedure.

## Monitoring

The read-only `Hetzner API Drift` GitHub workflow runs every Monday and can be
started manually. The same complete check is one local command:

```bash
scripts/check_hetzner_api_surface.sh --fetch
```

Release gates run the same live comparison. The fetch accepts only the two
exact official HTTPS URLs, rejects redirects, bounds connection and total
time, limits each response to 32 MiB, and requires valid UTF-8 JSON with an
object root. A current source digest may differ from the reviewed digest so the
tool can classify the change, but any digest or semantic difference makes the
command fail. Fetched content is maintenance input only and is never compiled,
packaged, or accepted automatically.

The Robot checker independently rejects redirects, limits the document to
8 MiB, enforces a 90-second POSIX wall-clock deadline, verifies the exact
reviewed source SHA-256 and operation-policy SHA-256, compares all 105 HTTP
operation headings in source order, and requires upstream deprecation markers
for the 16 excluded legacy Storage Box operations. Robot lock refreshes are
manual reviewed changes; the checker has no write mode.

The changelog checker separately authenticates and bounds Hetzner's official
RSS feed. This closes the OpenAPI blind spot for advance deprecations and
operational behavior changes that may not alter the specification. Any feed
semantic-digest or latest-entry change is a release stop even when both OpenAPI
documents remain unchanged. Server Metadata prose is raw-digest-bound inside
the Cloud specification and receives a dedicated operation lock in `v0.97.0`.

## Triage

Treat every nonzero drift result as a release stop until it is explicitly
accepted, rejected, or deferred.

| Category | Required review |
| --- | --- |
| Added operation | Confirm it is official and non-deprecated, assign an owner module and release, then add it to the API matrix. |
| Removed operation | Confirm the upstream removal and compatibility impact. Do not silently remove a public SDK API. |
| Deprecated operation | Record the replacement and removal date. Keep it excluded or provide a documented migration policy. |
| Changed operation | Review method, path, parameters, request body, responses, authentication, pagination, actions, and cost impact. |
| Added or removed parameter | Assign the exact request model and encoder change, or record an official exclusion; an operation-level association is not implementation evidence. |
| Changed parameter | Review requiredness, scalar/array cardinality, repeated-value encoding, enum/format/bounds, request ownership, target-size accounting, and negative tests. |
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
3. Implement required SDK, validation, test, API matrix, request-fidelity
   inventory, and documentation changes before refreshing the lock. Every
   active parameter row must map to an executable typed request field and wire
   test; merely retaining the operation association is insufficient.
4. Update the pinned SHA-256 values in
   `scripts/check_hetzner_api_drift.py`, `scripts/check_hetzner_upstream.sh`,
   and `docs/SPEC_LOCK.md` in the same reviewed change.
5. Refresh fingerprints only after review:

   ```bash
   scripts/check_hetzner_api_drift.py \
       --fetch --write-lock --accept-lock-refresh
   ```

   The refresh intentionally makes the executable request inventory stale.
   After implementing and testing every accepted parameter/body change, review
   and regenerate it explicitly:

   ```bash
   python3 scripts/generate_request_contract_inventory.py --write
   python3 scripts/generate_request_contract_inventory.py
   python3 scripts/test-request-contract-inventory.py
   ```

   Do not regenerate merely to make the gate green. The inventory contains a
   row for every path/query parameter and request-body operation; the embedded
   provider query contract is compiled from the same reviewed rows.

6. Complete the upstream-drift release-note template, run the full release
   gate, and include the change in pentest scope.

The explicit refresh flags authorize operation, parameter, schema, and model
lock replacement only when the fetched bytes match the newly reviewed pins;
they do not approve SDK behavior or a new source digest by themselves.

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
scripts/test-robot-api-lock.py
scripts/check_robot_api_lock.py
scripts/check_robot_api_lock.py --fetch
scripts/test-hetzner-changelog.py
scripts/check_hetzner_changelog.py --fetch
scripts/check_hetzner_api_surface.sh --fetch
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
probe harness must remain exactly `publish = false`, covered by workspace
checks, and absent from publish ordering and supported-provider documentation.
A future full `cloud-sdk-ovhcloud` implementation requires a separate post-1.0 plan and
review; the probe cannot be renamed or promoted into it.
