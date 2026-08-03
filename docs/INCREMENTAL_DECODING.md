# Incremental Provider Decoding

`cloud-sdk-hetzner` exposes a bounded incremental JSON visitor under its
optional `serde` feature. It processes large provider bodies across arbitrary
input chunks without constructing one complete JSON tree.

The repository also contains a compile-checked
[`incremental_json` example](../crates/cloud-sdk-hetzner/examples/incremental_json.rs).

## Install

```toml
[dependencies]
cloud-sdk-hetzner = { version = "0.37.0", features = ["serde"] }
```

## Visitor

```rust
use core::convert::Infallible;
use cloud_sdk_hetzner::serde::{
    IncrementalJsonDecoder, IncrementalJsonEvent, IncrementalJsonProgress,
    IncrementalJsonVisitor, VisitControl,
};

struct Counter {
    values: usize,
}

impl IncrementalJsonVisitor for Counter {
    type Error = Infallible;

    fn visit(
        &mut self,
        event: IncrementalJsonEvent<'_>,
    ) -> Result<VisitControl, Self::Error> {
        if matches!(
            event,
            IncrementalJsonEvent::StringStart
                | IncrementalJsonEvent::Number(_)
                | IncrementalJsonEvent::Bool(_)
                | IncrementalJsonEvent::Null
        ) {
            self.values = self.values.saturating_add(1);
        }
        Ok(VisitControl::Continue)
    }
}

let mut decoder = IncrementalJsonDecoder::new();
let mut visitor = Counter { values: 0 };
assert_eq!(
    decoder.push(br#"{"items":[1,"two",true]}"#, &mut visitor)?,
    IncrementalJsonProgress::Pending,
);
assert_eq!(
    decoder.finish(&mut visitor)?,
    IncrementalJsonProgress::Complete,
);
assert_eq!(visitor.values, 3);
# Ok::<(), cloud_sdk_hetzner::serde::IncrementalJsonError<Infallible>>(())
```

Text supplied by `Key`, `StringFragment`, and `Number` is borrowed only for
the visitor call. A string can produce multiple fragments, including one
decoded character per callback at hostile chunk boundaries. Visitors that
retain sensitive text must move it directly into protected storage rather
than create an ordinary `String` copy.

## Limits

`IncrementalJsonLimits::DEFAULT` applies these reviewed hard ceilings:

| Resource | Default and hard maximum |
| --- | --- |
| Input bytes | `8,388,608` |
| Open arrays and objects | `64` |
| Value and object-key tokens | `65,536` |
| Object fields across the document | `65,536` |
| Fields in one object | `4,096` |
| Decoded bytes in one string or key | `1,048,576` |
| Bytes in one number token | `128` |
| Digits in one exponent | `6` |

Builder methods can lower, but never raise, these limits. Values and object
keys each charge one token. The input-byte total includes whitespace and is
charged for a complete supplied chunk before any event in that chunk is
visited.

## Validation Boundary

The decoder validates exactly one JSON document, duplicate decoded keys,
grammar, nesting, numeric shape, Unicode escapes, surrogate pairs, and UTF-8
continued across chunks. Numeric values must also be finite under the same
admission rule as the buffered checked decoder. `push()` always reports `Pending` while validation
continues. `finish()` is mandatory and is the only operation that can report
`Complete`.

`VisitControl::Stop` reports the distinct terminal result `Stopped`. Bytes
after that event are deliberately not parsed, so `Stopped` must never be used
as proof that a complete response was valid.

The incremental parser does not admit HTTP status, content type, operation
identity, or response-envelope shape. Apply the prepared response policy
before feeding a provider body, or use the existing checked buffered decoder
when a complete typed response is required.

## Secret And Cleanup Boundary

Duplicate-detection keys and number tokens use the admitted sanitization
crate's growth-aware protected storage. Partial UTF-8 and escaped-character
scratch is volatile-cleared after use and on drop. Decoder errors clear all
owned lexical and structural staging before returning.

Input chunks remain caller-owned. The decoder cannot erase them, transport
buffers, visitor-owned copies, operating-system buffers, logs, crash dumps,
or remote service data. Keep sensitive response storage under the existing
`ResponseBuffer` cleanup owner and avoid formatting payload-bearing events;
their SDK `Debug` implementations are redacted.
