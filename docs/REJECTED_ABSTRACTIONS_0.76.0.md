# v0.76.0 Rejected Abstractions

Status: implementation complete; pentest required.

## Reusing Bare Basic Credentials As Robot Credentials

Provider-neutral Basic credentials do not encode Robot's three-failure
lockout policy. A Robot-owned type fixes provider, service, and endpoint scope
and requires a generation attempt before secret use.

## Automatic Retry After Authentication Rejection

A 401 is evidence that the current credential generation may be unsafe to try
again. No backoff duration makes the credential valid. The generation closes
until replacement material or explicit caller reconfirmation.

## Reconfirming An Open Generation

Advancing unchanged credentials before a concurrent rejection arrives would
turn that rejection stale and leave the same material open. Reconfirmation is
therefore valid only after the current generation is rejected.

## A Mutex Held Across Network Execution

Credentials and generation state are checked before execution. Atomic state
permits caller-bounded concurrent attempts without a lock across `.await` or
network I/O. Existing in-flight requests remain an explicit caller
concurrency boundary.

## Public Username Or Password Accessors

Returning `&str`, `String`, or cloned bytes would make protected storage
advisory. Credential text is closure-scoped to a validated attempt, and a
compile-fail contract prevents borrowed secrets from escaping.

## Building A Robot Client Early

v0.76 lacks the source-locked typed 401/error decoder assigned to v0.77 and
resource operations assigned from v0.78 onward. A partial client could not
close generations from provider responses reliably, so network execution
remains unavailable.
