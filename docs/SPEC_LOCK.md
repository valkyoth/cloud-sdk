# Hetzner API Source Lock

Status: not locked yet.

The initial repository foundation references the public Hetzner API reference:

- <https://docs.hetzner.cloud/reference/cloud>

`v0.2.0` must discover and pin the current authoritative machine-readable
source when available, including retrieval date, URL, revision or digest, and
any docs changelog entry needed to explain differences from `docs/API_MATRIX.md`.

Robot Webservice is explicitly deferred until after the Cloud/DNS SDK reaches
1.0. Its source reference is:

- <https://robot.hetzner.com/doc/webservice/en.html>

`v1.1.0` must pin Robot separately before any Robot operation is implemented.
