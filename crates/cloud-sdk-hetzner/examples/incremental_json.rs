//! Counts resource identifiers while validating a chunked provider response.

use core::convert::Infallible;

use cloud_sdk_hetzner::serde::{
    IncrementalJsonDecoder, IncrementalJsonEvent, IncrementalJsonProgress, IncrementalJsonVisitor,
    VisitControl,
};

struct ResourceCounter {
    ids: usize,
}

impl IncrementalJsonVisitor for ResourceCounter {
    type Error = Infallible;

    fn visit(&mut self, event: IncrementalJsonEvent<'_>) -> Result<VisitControl, Self::Error> {
        if matches!(event, IncrementalJsonEvent::Key("id")) {
            self.ids = self.ids.saturating_add(1);
        }
        Ok(VisitControl::Continue)
    }
}

fn main() -> Result<(), cloud_sdk_hetzner::serde::IncrementalJsonError<Infallible>> {
    let chunks: &[&[u8]] = &[br#"{"servers":[{"id":1},"#, br#"{"id":2}]}"#];
    let mut decoder = IncrementalJsonDecoder::new();
    let mut visitor = ResourceCounter { ids: 0 };

    for chunk in chunks {
        assert_eq!(
            decoder.push(chunk, &mut visitor)?,
            IncrementalJsonProgress::Pending
        );
    }
    assert_eq!(
        decoder.finish(&mut visitor)?,
        IncrementalJsonProgress::Complete
    );
    assert_eq!(visitor.ids, 2);
    Ok(())
}
