# Threat Model

## Assets

- Hetzner API tokens supplied by callers.
- Cloud infrastructure state.
- DNS zone and RRSet state.
- Certificate and SSH key metadata.
- Local CI and release credentials.

## Primary Risks

- token exposure through logs, debug output, examples, test fixtures, or panic
  messages;
- accidental hidden network or secret-storage dependency in default no_std
  crates;
- incorrect pagination causing missing or repeated infrastructure operations;
- incorrect action polling causing premature success reports;
- rate-limit mishandling that triggers denial of service or retry storms;
- DNS record mutation mistakes;
- certificate or SSH key redaction failures;
- API drift from Hetzner documentation;
- malicious or compromised third-party dependency.

## Controls

- no_std default SDK crate with no transport or token storage;
- internal endpoint module boundaries plus optional adapter crates;
- dependency review before admission;
- cargo-deny and cargo-audit;
- explicit API source lock before endpoint implementation;
- mock and adversarial testkit before transport helpers are stabilized;
- pentest report before every tag.
