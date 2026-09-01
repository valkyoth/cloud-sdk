# Hetzner Changelog Source Lock

Status: current operational source reviewed after stable `1.0.0`.

Retrieved: 2026-09-01

Official source:
<https://docs.hetzner.cloud/changelog/feed.rss>

Normalized semantic SHA-256:
`29a1ea0bacae078b213916a41f6af27c53a1e74ead7bb4f90c9cda89470748aa`

Latest reviewed entry:
<https://docs.hetzner.cloud/changelog#2026-08-31-debian-11-image-is-deprecated>

The normalization excludes only RSS `lastBuildDate`, which Hetzner regenerates
without publishing a new entry. Element names, attributes, channel identity,
entry order, GUIDs, dates, categories, titles, links, and complete entry content
remain digest-bound.

The OpenAPI and Robot source locks detect machine-readable contract and Robot
documentation changes. This separate RSS lock detects operational,
deprecation, rollout, and behavior notices that may precede or never alter an
OpenAPI document. Any semantic digest or latest-entry change is a review stop;
fetched RSS is never compiled, packaged, or accepted automatically.

The latest review identified four post-spec-lock notices relevant to the SDK:

The 2026-09-01 refresh contains 142 entries and retains the same newest entry
identity as the preceding review. The feed's normalized historical content
changed, but no newer operational notice or machine-readable API drift was
present.

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
