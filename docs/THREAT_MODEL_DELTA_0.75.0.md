# v0.75.0 Threat Model Delta

Status: implementation stop reached; pentest required.

## New Boundary

v0.75 accepts caller-provided UTF-8 field values and emits a Robot form body
into caller-owned storage. Form values may contain passwords, SSH public-key
material, customer identifiers, firewall rules, and billable-order inputs.

## Threats And Controls

### Ambiguous Form Encoding

- Standard form safe bytes are fixed to ASCII alphanumerics and `*`, `-`, `.`,
  `_`; spaces become `+`; every other UTF-8 byte uses uppercase percent
  encoding.
- `&`, `=`, `+`, brackets, controls, and non-ASCII bytes cannot create
  unplanned separators or fields.
- Duplicate fields retain exact caller order instead of entering a map.
- Field names admit only the reviewed Robot identifier and bracket grammar.

### Partial Or Oversized Output

- Field count, name bytes, value bytes, checked encoded length, aggregate body
  bytes, and destination capacity are bounded before any output mutation.
- Validation, arithmetic, aggregate-cap, and capacity rejection leave the
  destination unchanged.
- An immutable snapshot is measured, written, and compared through the shared
  transactional encoder; write or replay disagreement clears admitted bytes.

### Secret Tail Retention

- After capacity admission, the complete caller output is volatile-cleared
  before encoding so a shorter body cannot retain an older secret tail.
- `EncodedRobotForm` keeps the mutable borrow for the full body lifetime and
  volatile-clears the complete output on drop.
- Field, form, body, and error Debug output omit values. Display messages are
  static and payload-free.
- Borrowed source values, transport copies, kernel buffers, crash dumps, and
  provider storage remain caller and operational cleanup boundaries.

### Capability Overstatement

- The provider README identifies the form codec as the only current Robot
  runtime primitive.
- The codec performs no authentication, endpoint selection, network request,
  retry, response parse, mutation, or billing action.

## Unchanged Boundaries

Default crate graphs remain no_std, allocation-free, transport-free,
runtime-free, filesystem-free, clock-free, and unsafe-free. Robot Basic
credentials and lockout state remain assigned to v0.76.
