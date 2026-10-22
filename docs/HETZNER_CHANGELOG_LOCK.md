# Hetzner Changelog Source Lock

Status: current operational source reviewed after stable `1.0.0`.

Retrieved: 2026-10-22

Official source:
<https://docs.hetzner.cloud/changelog/feed.rss>

Normalized semantic SHA-256:
`6f2da2458e3cd4cb3bad55c086c9934243bc1f28dc5b3c929fc52234da8099e8`

Latest reviewed entry:
<https://docs.hetzner.cloud/changelog#2026-10-15-server-subnet-type-deprecation>

The October 22 maintenance review covers the October 5 load-balancer Primary IP
selection and deletion-action changes. Creation now accepts existing IP IDs;
deletion returns a 201 action rather than a 204 empty response. The corresponding
request preparation, response schema and operation association are updated.
The October 5 entry "Deleting a Load Balancer now returns an Action" names
`delete_load_balancer` as the returned action. This digest-bound command, not
OpenAPI's generic `start_resource` example, is required by typed deletion
response validation together with the requested resource type and ID.
The October 15 server subnet-type deprecation permits existing subnets until
April 15, 2027. The October 12 Link-header deprecation does not affect the SDK's
JSON metadata pagination. The October 1 datacenter retirement affects excluded
deprecated operations. September 30 explicit-null TTL and DNS pointer semantics
are already enforced. No credentialed or mutating API calls were made.

The September 26 candidate review adds Network members (September 15) through
the prepared request, bounded response model and service client layers. The
September 23 Primary IP rollout matches the existing cross-field checks:
`unassigned` requires a null assignee ID and `server` requires a positive ID.
Request bodies continue to use `server`; `unassigned` is a response value.
The schema lock now records both response enum values. No API calls with
credentials or mutations were needed for this source review.

The normalization excludes only RSS `lastBuildDate`, which Hetzner regenerates
without publishing a new entry. Element names, attributes, channel identity,
entry order, GUIDs, dates, categories, titles, links, and complete entry content
remain digest-bound.

The OpenAPI and Robot source locks detect machine-readable contract and Robot
documentation changes. This separate RSS lock detects operational,
deprecation, rollout, and behavior notices that may precede or never alter an
OpenAPI document. Any semantic digest or latest-entry change is a review stop;
fetched RSS is never compiled, packaged, or accepted automatically.

The September 10 review contains 144 entries, including two new notices:

- September 8: legacy `deprecated` fields will disappear on November 2 from
  Images, Server Types and Load Balancer Types. Exact root/nested decoder
  exceptions accept omission now and still validate values when present.
  Applications should inspect image/type `deprecation` or server-type
  `locations[].deprecation`. Tests cover all six paths without relaxing
  required replacement fields. The new image field is schema-locked.
- September 9: bucket deletion can tolerate up to 1,000 leftover multipart
  upload parts, but project deletion still requires completely empty buckets.
  This is the S3-compatible Object Storage API, outside the Cloud, DNS,
  Storage Box, Robot and Server Metadata SDK scope. No endpoint/model is
  silently added; no mutation or deletion test was performed.

Previously reviewed operational notices remain relevant:

- The Debian 11 server image is deprecated and Hetzner announces that it will
  no longer be available for new servers after 30 November 2026. Image
  identities are caller-provided checked strings rather than an SDK allowlist,
  so this operational catalog change requires no request or response model
  change. Applications must choose a currently available image.

- Load Balancer health targets can include additive `detail` and
  `http_status_code` fields. `v0.97.0` validates their exact unhealthy-only and
  HTTP-status cross-field semantics.
- Only canonical `/hetzner/v1/*` Server Metadata routes remain. That prose-only
  service was absent from the OpenAPI operation inventory and is implemented
  and separately source-locked in `v0.97.0`.
- An unassigned Primary IP returns `assignee_type: "unassigned"` with
  `assignee_id: null`. Existing response models retain open text values and
  nullable IDs; `v0.97.0` adds exact cross-field regression fixtures.

Run every tracked Hetzner source check with:

```bash
scripts/check_hetzner_api_surface.sh --fetch
```

After a change, inspect the complete new feed and corresponding official
reference/specification changes. Update this document and the checker pin only
after implementation, tests, release notes, and security review are complete.
