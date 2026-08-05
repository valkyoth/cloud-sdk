# Provider-Generic Drift Evidence

The provider drift engine records reviewable, deterministic source-lock
evidence without putting provider-specific assumptions into `cloud-sdk`.
Hetzner remains checked by its full OpenAPI-aware drift tool; the v0.56 bridge
also proves that its existing locks can be represented by the neutral model.
Future provider probes and crates must use the neutral model from their first
source lock.

## Trust Boundary

Plugin declarations under `provider-drift/plugins/` are data, not executable
Python or dynamically loaded libraries. A declaration fixes one identifier,
version, and the complete canonical category list. Provider-specific adapters
are repository-reviewed scripts selected explicitly by a release gate; a
provider manifest cannot select a command, import a module, or execute code.

Adapters receive remote source bytes only inside a killable verification
worker. The worker and `check_provider_drift.py` together:

- permits only provider/source endpoints present in a hard-coded reviewed
  registry and rejects DNS results containing any non-global address;
- ignores ambient proxy variables and requests exact credential-free HTTPS
  URLs with the validating platform TLS context;
- refuses every redirect before constructing a follow-up request;
- checks the final URL, per-source byte bound, per-read time, whole-plan hard
  deadline, and SHA-256 in a killable worker process;
- invokes a hard-coded reviewed adapter only after every source authenticates,
  while the whole download, parse, normalization, comparison, and report path
  remains under one 180-second deadline;
- derives the live observation from fetched bytes and requires it to equal the
  separately tracked observation before comparison with the lock; and
- transfers only a bounded 2 MiB payload-free report across process IPC, never
  raw source bytes; and
- never puts source content into errors or canonical drift output, including
  malformed Unicode, excessive nesting, parser failures, and worker exits.

This authenticates a reviewed source snapshot. It does not establish that a
provider account, TLS environment, workstation, or upstream publisher is
trustworthy.

DNS is checked before opening the TLS connection, but application-level DNS
validation cannot by itself eliminate rebinding between resolution and
connection. Release hosts must enforce egress policy so approved provider
names cannot route to loopback, link-local, private, metadata, or internal
destinations after validation.

## Documents

A provider lock contains:

- exact source URLs, SHA-256 values, per-source byte bounds, and a 128 MiB
  aggregate admission bound;
- one fixed normalized plugin identity;
- provider, security, and release ownership roles;
- explicit add/change/remove severity for every category; and
- normalized rows for authentication, endpoints, operations, schemas,
  pagination, headers, retry, idempotency, and cost policy.

Reviewed adapters construct every category explicitly. Authentication,
servers, provider response headers, paginated operation sets, operation
fingerprints, response bindings, and schema fingerprints are derived from the
authenticated provider documents. Repository-owned response, rate-limit,
retry, idempotency, and cost policies are reconstructed from fixed semantics
and no-follow SHA-256 evidence for their authoritative local files. No live
category is initialized from lock contract values.

An observation contains only the plugin identity, observed sources, and the
same normalized categories. Both formats reject unknown fields, duplicate JSON
keys, duplicate identifiers, noncanonical identifiers, floats, excessive
nesting, oversized strings, oversized collections, symlinks, non-regular
files, and documents larger than 2 MiB.

Normalized row values are evidence rather than executable configuration. They
may contain bounded JSON integers, booleans, strings, nulls, arrays, and
objects. Float rejection avoids nonportable canonical-number encodings. Source
URLs also reject query strings, user information, fragments, controls,
backslashes, uppercase authorities, and an explicitly redundant port 443.

## Canonical Diff

`scripts/check_provider_drift.py` sorts categories and identifiers and emits
one compact JSON line. Changes contain only:

- category, row identifier, and add/change/remove kind;
- changed normalized field paths as unambiguous RFC 6901 JSON pointers;
- old and new canonical SHA-256 values;
- explicit owner and compatibility severity.

Raw source text and normalized values are omitted from the report. This keeps
malicious or sensitive input out of CI diagnostics while retaining exact
review identity. Equivalent document and row ordering produces identical
bytes. Source URL or digest rotation is always security-owned and blocking.

## Review And Rotation

Run the local model and fixture gate with:

```sh
scripts/check_provider_drift.sh
```

Run authenticated source verification with:

```sh
scripts/check_provider_drift.py \
  --plugin provider-drift/plugins/normalized-json-v1.json \
  --lock provider-drift/providers/hetzner.lock.json \
  --observation provider-drift/providers/hetzner.observed.json \
  --fetch-sources
```

Any non-clean report blocks release. Source digest rotation requires an
independent review of the fetched document and adapter output, an intentional
lock and observation update, updated provider evidence, tests, and the normal
pentest. The engine has no automatic lock-writing or acceptance switch.
