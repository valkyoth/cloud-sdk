# Hetzner Live Smoke Testing

The live smoke harness validates the published SDK request, transport, and
response boundaries against the public Hetzner Cloud API. It is opt-in,
read-only, and disabled in normal CI and workspace tests.

## Scope

The `v0.19.0` harness sends only `GET` requests for:

- locations;
- server types;
- load balancer types;
- ISOs;
- public system images; and
- pricing.

List probes request one entry, require strict Hetzner pagination metadata, and
validate the expected top-level collection. Pricing must return its expected
object. The endpoint is fixed to `https://api.hetzner.cloud/v1`; callers cannot
redirect authenticated traffic to another origin.

Ordinary checks run all offline harness tests but leave the authenticated test
ignored:

```sh
scripts/smoke_hetzner_live.sh --check
```

Both `--check` and the build phase reject a token-file environment variable so
Cargo, build scripts, procedural macros, compiler wrappers, linkers, and other
build tooling cannot discover its path through their inherited environment.

## Least-Privilege Project

Create the token in a dedicated Hetzner Cloud test project with no production
resources. Select the provider's **Read** permission, not **Read & Write**.
Do not reuse a production token, owner credential, CI release credential, or
token shared with another application.

The SDK cannot prove the provider-side scope of a bearer token. The harness
limits its own behavior to typed read-only requests, but token scope, project
membership, creation, rotation, revocation, and billing controls remain caller
responsibilities.

## Sealed Build Phase

Build the live-smoke executable from a clean reviewed commit **before** the
token file exists or is mounted:

```sh
unset CLOUD_SDK_HETZNER_TOKEN_FILE
unset CLOUD_SDK_HETZNER_ALLOW_DESTRUCTIVE
scripts/smoke_hetzner_live.sh --prepare
```

`--prepare` invokes Cargo without credential variables, selects exactly one
`live_smoke` test executable from Cargo's structured JSON output, installs it
read-only under ignored `target/` storage, and records its SHA-256 digest and
the reviewed Git commit. The wrapper rejects a dirty worktree. It cannot prove
that an unreferenced credential elsewhere on the build host is unavailable, so
credential removal or mount isolation during this phase is an operational
requirement.

Do not rebuild after provisioning the token. If code changes, revoke or remove
the token first, commit and review the changes, then run `--prepare` again.

## Private Token File

The harness does not accept a token as a command-line argument or raw token
environment variable. It accepts only the path in
`CLOUD_SDK_HETZNER_TOKEN_FILE`.

For Bash or Zsh, this creates a private file without placing the token value in
shell history:

```sh
token_dir="${XDG_CONFIG_HOME:-$HOME/.config}/cloud-sdk"
token_file="$token_dir/hetzner-read-only.token"
install -d -m 700 -- "$token_dir"
install -m 600 /dev/null "$token_file"
IFS= read -r -s token
printf '\n'
printf '%s\n' "$token" >"$token_file"
unset token
```

On Unix, the harness rejects symlinks, non-regular files, files with any group
or world permission bit, files that change device or inode during open, and
files above the bounded token size. On Windows, place the file in a private
user directory and restrict its ACL to the test account before running; Unix
mode and inode checks do not apply there.

Only after `--prepare` succeeds, create or mount the token file. Run the
authenticated smoke test with only the path in the environment:

```sh
CLOUD_SDK_HETZNER_TOKEN_FILE="$token_file" \
    scripts/smoke_hetzner_live.sh --read-only
```

The wrapper immediately removes the token path from its own exported
environment, verifies that the read-only sealed executable matches both the
recorded commit and SHA-256 digest, then replaces itself with that executable
under a minimal environment. It does not invoke Cargo or other build tooling in
the authenticated phase. The wrapper supplies
`CLOUD_SDK_HETZNER_LIVE_MODE=read-only` only to the test process and rejects
`CLOUD_SDK_HETZNER_ALLOW_DESTRUCTIVE` in read-only mode.

Delete or revoke the token after the run. Before reading, the harness reserves
the complete bounded token capacity in one allocation so buffer growth cannot
leave plaintext fragments in retired allocations. It clears that token source
buffer, the response buffer, adapter-owned authorization bytes, and
adapter-owned request storage. It cannot clear copies retained by the shell,
filesystem, OS cache, reqwest, rustls, crash tooling, swap, or the remote
service.

## Output Policy

Successful output contains only static probe names. Failure diagnostics contain
only static error categories, the static probe name, and possibly an HTTP
status. Token values, token-file paths, endpoints, response bodies, and provider
resource IDs are never written by the harness.

Do not add `--debug`, shell tracing, packet capture, or response-body logging to
an authenticated run. Treat terminal capture and CI logs as potentially
persistent records.

## Destructive Test Plan

Mutation execution is deliberately not implemented in `v0.19.0`. A future
destructive harness must remain a separate command and satisfy all of these
gates before its first network request:

1. Use a dedicated disposable project containing no production resources.
2. Create a short-lived **Read & Write** token only for that run.
3. Require an exact destructive acknowledgement distinct from `read-only`.
4. Require a unique resource prefix beginning with `cloud-sdk-live-`.
5. Review every operation, region, quota, and current provider price manually.
6. Record a resource inventory before mutation without logging provider IDs.
7. Create the minimum-sized resource set and never retry a mutation implicitly.
8. Run cleanup on success, failure, timeout, and interruption paths.
9. List resources after cleanup and fail until no prefixed resource remains.
10. Revoke the token and inspect the provider project and billing view manually.

No destructive command may infer consent from the token's permission, reuse
the read-only wrapper, accept an empty or generic prefix, or run in default CI.
