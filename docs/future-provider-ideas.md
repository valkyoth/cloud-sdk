# Future Provider Ideas

Research snapshot: 2026-09-26.

Status: exploratory ideas, not implementation commitments or release promises.
This document does not change the provider roadmap or authorize work on a new
provider. Recheck API scope, existing Rust clients, licensing, testing access,
and demand before turning an idea into a commit plan.

## Selection Goal

Add services that help Rust applications accomplish useful tasks, including SaaS
products rather than only infrastructure providers. A service that complements
Hetzner may offer more practical value than another overlapping compute API.

These rankings are engineering judgments, not measured adoption forecasts.
Missing SDK functionality does not by itself establish demand, and finding only
a few clients does not prove that no other implementation exists.

## Shortlist

| Investigation priority | Service | Example Rust workflows | Main qualification question |
| --- | --- | --- | --- |
| 1 | Tailscale | Device provisioning, access management, private networks, CI environments | Does a comprehensive management client fill a demonstrated gap? |
| 2 | Better Stack | Monitors, heartbeats, incidents, on-call schedules, status pages | How much management coverage exists beyond log-ingestion clients? |
| 3 | bunny.net | Asset uploads, CDN and DNS configuration, cache purging, video delivery | What would improve on existing broad Rust clients? |
| 4 | Linear | Issue synchronization, release automation, developer CLIs | What coverage or OAuth workflows are missing from existing clients? |
| 5 | Doppler | Secret retrieval and configuration management | Can protected secret handling provide a useful, ergonomic distinction? |
| 6 | Paddle | Subscription billing and product/customer administration | Can we justify the maintenance and correctness burden over existing clients? |

These priorities order further investigation, not implementation. Each service
would use one provider crate, with internal modules separating product APIs.
Transport, sanitization, and testkit functionality remain provider-neutral.

## Tailscale

A possible `cloud-sdk-tailscale` provider could connect existing Hetzner support
to private-network automation: provision a server, issue an enrollment key,
configure access, and revoke access during teardown.

The [management API](https://tailscale.com/docs/reference/tailscale-api) is
available across plans. An identified
[community Rust client](https://github.com/agentsea/tailscale.rs) describes itself
as minimal; compare actual endpoint coverage before concluding there is a gap.

Tailscale's own
[Rust networking library](https://tailscale.com/blog/tailscale-rs-rust-tsnet-library-preview)
embeds networking into applications. That is distinct from managing hosted
resources through the management API. A provider should complement that library,
not implement a VPN or duplicate its networking runtime.

Qualification must cover credential scopes, policy mutations, enrollment-key
handling, plan-dependent features, and disposable test environments.

## Better Stack

The [Uptime API](https://betterstack.com/docs/uptime/api/getting-started-with-uptime-api/)
covers monitors, heartbeats, incidents, on-call functionality, status pages, and
related administration. A deployment workflow could create a monitor and status
page entry alongside an application and manage maintenance periods explicitly.

The Rust integrations found during this investigation, such as
[tracing-better-stack](https://docs.rs/tracing-better-stack), focus on log
ingestion. That does not establish the absence of a management SDK, but suggests
a useful area for a deeper comparison.

Uptime is not the entire Better Stack platform. A full-provider plan must
inventory other product APIs separately and distinguish management operations
from telemetry ingestion. Live tests must avoid notifying real on-call staff or
publishing misleading incidents on production status pages.

## bunny.net

CDN, storage, DNS, and streaming could complement Hetzner-hosted applications.
The [official documentation repository](https://github.com/bunnyway/documentation)
includes OpenAPI-based references that could support source locking and drift
detection.

However, [bunny-net-api](https://docs.rs/crate/bunny-net-api/0.5.0) already
advertises broad Rust coverage, including CDN, storage, DNS, streaming, edge, and
containers. Do not describe this as an API without a Rust SDK. Compare coverage,
maintenance, security boundaries, and usability before committing to another.

Qualification includes region-specific endpoints, credential separation,
streaming transfers, cleanup, and potentially billable storage and delivery.
Full-platform coverage is a larger undertaking than a storage upload client.

## Linear

The [GraphQL API](https://linear.app/developers) could support Rust developer
tools, issue synchronization, and release automation. An existing
[typed Rust client](https://github.com/bipa-app/linear-api) already offers useful
functionality, so investigate concrete gaps rather than assuming an empty niche.

Potential comparison points include OAuth, schema coverage, pagination, and
webhook workflows. A plan would need GraphQL schema-drift checks, bounded query
and response handling, and an isolated workspace for mutation tests.

## Doppler

The [REST API](https://docs.doppler.com/reference/api) supports secret access and
administration with several credential types and scopes. Protected values and
explicit credential handling fit the project's existing security boundaries.

A comprehensive Rust-client coverage gap has not been established. Investigate
whether native API workflows improve on existing SDKs and CLI-based integration.
Testing must use disposable configurations and synthetic secrets, with no
production secret ingestion or accidental exposure through examples and logs.

## Paddle

Paddle could reach developers building paid Rust applications. Its
[official tooling](https://developer.paddle.com/sdks/) includes a sandbox and
OpenAPI specification; the reviewed official SDK list did not include Rust.
Nevertheless, a [community Rust client](https://docs.rs/paddle-rust-sdk/latest/paddle_rust_sdk/struct.Paddle.html)
already exists.

Compare API coverage, webhook verification, replay handling, and ergonomics.
Billing mutations require particularly careful idempotency, delivery ambiguity,
and retry behavior. Use the sandbox for testing; do not treat successful API
calls as sufficient qualification of a complete billing workflow.

## Lower-Priority Duplication

Email and authentication are useful, but
[Resend](https://resend.com/changelog/announcing-the-rust-sdk) and
[WorkOS](https://workos.com/changelog/rust-sdk) already offer official Rust SDKs.
A new provider would need a clear additional benefit rather than availability
alone. Contributing to an existing SDK can be better for the Rust ecosystem than
creating another competing implementation.

Existing larger candidates remain documented separately:

- [Scaleway commit plan](scaleway-commit-plan.md).
- [DigitalOcean commit plan](digitalocean-commit-plan.md).
- [Tenable commit plan](tenable-commit-plan.md).

Access to a Tenable demo tenant would help verification, but does not establish
Rust user demand. No candidate should be chosen solely because it has many
customers or because API credentials are available.

## Validation Before Planning

1. Identify concrete Rust users and workflows, ideally with external feedback.
2. Compare maintained official and community clients operation by operation.
3. Inventory public APIs, authentication, webhooks, data planes, and exclusions.
4. Confirm repeatable test access, cleanup, cost controls, and product entitlements.
5. Check source-lock and drift-detection feasibility, including schema changes.
6. Estimate full-support maintenance, not only initial endpoint implementation.
7. Write a commit plan with deliverables, verification, and pentest stop gates.

## Recommendation

Investigate Tailscale first and Better Stack second. Keep bunny.net as an
alternative after comparing its existing Rust SDKs. Do not start all three.

A coherent example combining Hetzner provisioning, Tailscale networking, and
Better Stack monitoring could demonstrate why the shared SDK is useful.
Independently usable provider crates and straightforward application examples
should remain priorities. Security reviews, bounded execution, and `no_std`
support are important foundations, but users still need an easy end-to-end path
to their task. Adoption remains a hypothesis to validate, not a promised result.
