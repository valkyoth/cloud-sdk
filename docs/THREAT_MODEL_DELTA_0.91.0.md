# Threat Model Delta 0.91.0

Status: implementation stop; pentest required.

## New Assets

- current standard-server, Server Auction, and addon catalog observations;
- exact current net, gross, hourly, and setup prices;
- account currency, locations, distributions, languages, and quantity limits;
- non-executable plan selections bound to one decoded catalog snapshot.

## New Untrusted Inputs

Robot JSON can contain oversized arrays or text, malformed or overly precise
prices, duplicate identities, unknown fields, inconsistent location prices,
incoherent hourly net/gross values, and a product ID different from the one
requested. Query filters can contain attacker-influenced decimal and location
values.

## Controls

- response bodies are capped at 4 MiB for lists and 1 MiB for detail/currency;
- product and nested arrays are capped at 4,096 entries;
- exact decimals admit at most 18 digits and four fractional digits, reject
  signs/exponents/noncanonical leading zeroes, and never use floating point;
- product identifiers, locations, choices, and returned text are bounded and
  protected with redacted diagnostics;
- strict decoders reject unknown fields, duplicate product identities,
  mismatched detail identities, invalid locations, and partial hourly prices;
- request preparation is transactional and clears caller storage on failure;
- all six requests are safe read-only `GET` operations with no known direct
  cost and no implicit retry;
- plan types have no transport implementation and carry an unavoidable current-
  price revalidation warning.

## Residual Boundaries

Catalog data is an observation, not a quote. Robot can change availability,
price, tax, location, addon bounds, or currency after the response. v0.91 does
not authorize or execute purchases. A future billable operation must fetch the
current catalog again and bind explicit cost authority to the newly observed
values. Provider text remains untrusted when rendered and requires context-
appropriate escaping after closure-scoped inspection.
