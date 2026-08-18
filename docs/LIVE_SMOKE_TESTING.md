# Hetzner Live Smoke Testing

The live smoke harness validates the published SDK request, transport, and
response boundaries against the public Hetzner Cloud and Robot APIs. It is
opt-in, read-only by construction, and disabled in normal CI and workspace
tests.

## Scope

The harness sends only `GET` requests. Its original `v0.19.0` catalog probes
cover:

- locations;
- server types;
- load balancer types;
- ISOs;
- public system images; and
- pricing.

Later releases add read-only typed probes for DNS zones, certificates, SSH
keys, Storage Boxes, and Storage Box types. No live probe creates, updates,
resets, or deletes provider state.

v0.95 adds a separately launched Robot probe for the bodyless
`GET /server` operation. It uses `RobotClient::official`, a scoped Basic
transport, an 8 MiB response limit, strict checked server-list decoding, and
one attempt. It sends no invalid credential, form body, mutation, order,
transaction query, reset, Wake-on-LAN, retry, or custom-endpoint request.

List probes request one entry, require strict Hetzner pagination metadata, and
validate the expected top-level collection. Pricing must return its expected
object. The endpoint is fixed to `https://api.hetzner.cloud/v1`; callers cannot
redirect authenticated traffic to another origin.

Since v0.43, each probe prepares the same provider-owned operation exposed to
applications. The prepared request supplies exact Cloud service identity,
official endpoint, required authentication scope, bounded raw response policy,
and checked response policy to the authenticated raw adapter. v0.70 also runs
`list_locations_blocking` through the official service-typed Cloud client,
caller-owned workspace pool, complete response policy, and typed decoder.
v0.71 runs the paginated `list_zones_blocking` path through the equivalent
official DNS client and checked DNS resource model. The harness has no separate
legacy request assembly path for either client probe. v0.72 similarly runs
`list_certificates_blocking` and `list_ssh_keys_blocking` through the official
Security client, bounded workspace leases, and dedicated checked resource
models. v0.73 runs `list_storage_boxes_blocking` and
`list_storage_box_types_blocking` through the official Console Storage client
with one-entry pages, bounded workspace leases, and source-complete checked
Storage response models.

Ordinary checks run all offline harness tests but leave the authenticated test
ignored:

```sh
scripts/smoke_hetzner_live.sh --check
```

Both `--check` and the build phase reject Cloud token and Robot username or
password file environment variables so Cargo, build scripts, procedural
macros, compiler wrappers, linkers, and other build tooling cannot discover
their paths through the inherited environment.

## Least-Privilege Project

Create the token in a dedicated Hetzner Cloud test project with no production
resources. Select the provider's **Read** permission, not **Read & Write**.
Do not reuse a production token, owner credential, CI release credential, or
token shared with another application.

The SDK cannot prove the provider-side scope of a bearer token. The harness
limits its own behavior to typed read-only requests, but token scope, project
membership, creation, rotation, revocation, and billing controls remain caller
responsibilities.

Robot uses separate HTTP Basic Webservice credentials and warns that three
failed logins block the caller's source IP for ten minutes. The harness never
intentionally submits invalid credentials and performs no automatic retry.
Use a separate Robot Webservice user with the narrowest account and server
access available. The SDK cannot prove that a Robot credential is read-only;
the root-owned launcher limits only this executable's behavior. Do not use an
owner login, production automation credential, or credential shared with
another process. Revoke or rotate both values after the probe.

## Credential-Free Build Phase

Build the live-smoke executable from a clean reviewed commit **before** any
Cloud or Robot credential file exists or is mounted:

```sh
unset CLOUD_SDK_HETZNER_TOKEN_FILE
unset CLOUD_SDK_HETZNER_ROBOT_USERNAME_FILE
unset CLOUD_SDK_HETZNER_ROBOT_PASSWORD_FILE
unset CLOUD_SDK_HETZNER_ALLOW_DESTRUCTIVE
scripts/smoke_hetzner_live.sh --prepare
```

`--prepare` invokes Cargo without credential variables, selects exactly one
`live_smoke` test executable from Cargo's structured JSON output, and creates an
ignored staging bundle containing the executable, runtime, launchers, manifest,
SHA-256 digests, and reviewed Git commit. The wrapper rejects a dirty worktree
and anchors all repository paths to its own physical location, not the caller's
working directory.

The staging directory is user-owned and **not trusted**. Read-only mode bits and
adjacent hashes do not make it authentic. Credential removal or mount isolation
during this phase remains an operational requirement.

## Privileged Sealing Phase

After the build process and any build container have exited, review the staged
bundle and install it with trusted absolute utilities. Do not run a repository
script as root:

```sh
stage="$PWD/target/cloud-sdk-live-smoke/staging"

sudo /usr/bin/install -d -o root -g root -m 0755 \
    /usr/local/libexec/cloud-sdk-live-smoke
sudo /usr/bin/install -o root -g root -m 0555 \
    "$stage/live_smoke" /usr/local/libexec/cloud-sdk-live-smoke/live_smoke
sudo /usr/bin/install -o root -g root -m 0444 \
    "$stage/runner.py" /usr/local/libexec/cloud-sdk-live-smoke/runner.py
sudo /usr/bin/install -o root -g root -m 0444 \
    "$stage/manifest" /usr/local/libexec/cloud-sdk-live-smoke/manifest
sudo /usr/bin/install -o root -g root -m 0555 \
    "$stage/cloud-sdk-hetzner-smoke" /usr/local/bin/cloud-sdk-hetzner-smoke
sudo /usr/bin/install -o root -g root -m 0555 \
    "$stage/cloud-sdk-hetzner-robot-smoke" \
    /usr/local/bin/cloud-sdk-hetzner-robot-smoke
```

Install both launchers last so an incomplete update fails closed. Confirm that
`/usr/local`, `/usr/local/libexec`, the bundle directory, `/usr/local/bin`, and
all installed files are owned by root and are not group- or world-writable.
Terminate the credential-free build environment before creating or mounting
credentials. The repository wrapper cannot perform this privileged trust
transition.

Do not rebuild or reseal after provisioning credentials. If code changes,
revoke or remove every credential first, commit and review the changes, then
repeat both credential-free phases.

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
    /usr/local/bin/cloud-sdk-hetzner-smoke
```

Do not invoke the mutable repository wrapper with a credential. The root-owned
launcher starts the system Python interpreter in isolated, no-site mode. Its
root-owned runner clears the inherited environment, rejects arguments and
destructive opt-in, validates UID/GID 0 ownership, exact file modes, regular
single-link files, non-writable root-owned parent directories, and the bounded
manifest. It hashes an already-open executable descriptor and executes that
same descriptor, eliminating path substitution between verification and
execution. Only the fixed read-only marker, minimal `PATH`, and token-file path
reach the test process.

Root ownership is the authenticity trust anchor for this local operational
workflow. The project does not claim offline-signature provenance for the
staging bundle; review and privileged installation remain administrator duties.

Delete or revoke the token after the run. Before reading, the harness reserves
the complete bounded token capacity in one allocation so buffer growth cannot
leave plaintext fragments in retired allocations. It clears that token source
buffer, the response buffer, adapter-owned authorization bytes, and
adapter-owned request storage. It cannot clear copies retained by the shell,
filesystem, OS cache, reqwest, rustls, crash tooling, swap, or the remote
service.

## Private Robot Credential Files

Create two different private files after the credential-free bundle has been
sealed. Do not place either value in command arguments, raw environment
variables, shell history, the repository, or one combined file:

```sh
credential_dir="${XDG_CONFIG_HOME:-$HOME/.config}/cloud-sdk"
username_file="$credential_dir/hetzner-robot.username"
password_file="$credential_dir/hetzner-robot.password"
install -d -m 700 -- "$credential_dir"
install -m 600 /dev/null "$username_file"
install -m 600 /dev/null "$password_file"
IFS= read -r robot_username
IFS= read -r -s robot_password
printf '\n'
printf '%s\n' "$robot_username" >"$username_file"
printf '%s\n' "$robot_password" >"$password_file"
unset robot_username robot_password
```

The Robot path is supported only on Unix and fails closed elsewhere. It rejects
missing or identical paths, untrusted parent directories, symlinks,
non-regular files, wrong ownership, multiple hard links, group/world permission
bits, and values beyond the Basic-auth bounds. Each file is opened once with
descriptor-level no-follow and close-on-exec semantics, then validated and read
only through that descriptor. It permits one terminal LF or CRLF and clears
both complete source allocations on success or rejection. Filesystem caches,
shell input, transport copies, crash tooling, swap, and remote-service handling
remain operational cleanup boundaries.

Run only the root-owned Robot launcher:

```sh
CLOUD_SDK_HETZNER_ROBOT_USERNAME_FILE="$username_file" \
CLOUD_SDK_HETZNER_ROBOT_PASSWORD_FILE="$password_file" \
    /usr/local/bin/cloud-sdk-hetzner-robot-smoke
```

The runner rejects a Cloud bearer-token path, destructive opt-in, missing
files, mixed modes, additional arguments, and same textual file paths. It
clears the inherited environment and selects exactly
`read_only_robot_server_smoke`. Inspect the Robot login/security view after
the run, then revoke or rotate the Webservice credential and securely remove
both files.

## Output Policy

Successful output contains only static probe names. Failure diagnostics contain
only static error categories, the static probe name, and possibly an HTTP
status. Token, username, password, credential-file paths, endpoints, response
bodies, and provider resource IDs are never written by the harness.

Do not add `--debug`, shell tracing, packet capture, or response-body logging to
an authenticated run. Treat terminal capture and CI logs as potentially
persistent records.

## Destructive Test Plan

Mutation execution is deliberately not implemented. A future destructive
harness must remain a separate command and satisfy all of these
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
