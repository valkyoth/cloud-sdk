# Hetzner Changelog Source Lock

Status: current operational source reviewed after stable `1.0.0`.

Retrieved: 2026-09-10

Official source:
<https://docs.hetzner.cloud/changelog/feed.rss>

Normalized semantic SHA-256:
`170fc4dbef43b82cb80562cc793aa8d38146c5641291609c07e8353906dff0c5`

Latest reviewed entry:
<https://docs.hetzner.cloud/changelog#2026-09-09-object-storage-new-delete-rule>

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
