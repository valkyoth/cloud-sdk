# v0.62.0 Threat Model Delta

The neutral freeze adds no new network or credential boundary. It increases
provider response exposure in three areas:

- certificate PEM and managed-certificate errors are attacker-controlled
  provider text and must remain protected and redacted;
- Storage Box lists can be large and structurally amplifying, so they pass
  bounded incremental admission before duplicate-rejecting model decoding;
- DNS and security operations must retain exact service identity so a response
  cannot be decoded under a sibling Cloud API scope.

Controls include source-required fields, bounded arrays/maps/text, finite
coordinates, exact booleans and enums, coherent pagination, multiline secret
validation limited to tab/CR/LF, protected parser strings, and checked response
policy before model use. Remaining resource completion is explicitly deferred
to v0.63.0-v0.67.0 and is not claimed by these slices.
