use cloud_sdk::incremental_json::{IncrementalJsonEvent, IncrementalJsonVisitor, VisitControl};

use super::CratesIoWireError;

/// Only structural error metadata is retained; all provider text is discarded.
#[derive(Default)]
pub(super) struct EnvelopeVisitor {
    depth: usize,
    root: bool,
    root_errors_key: bool,
    detail_key: bool,
    detail_seen: bool,
    in_errors: bool,
    pub(super) errors_present: bool,
    pub(super) error_count: usize,
}

impl IncrementalJsonVisitor for EnvelopeVisitor {
    type Error = CratesIoWireError;

    fn visit(&mut self, event: IncrementalJsonEvent<'_>) -> Result<VisitControl, Self::Error> {
        use IncrementalJsonEvent as Event;
        if let Event::Key(key) = event {
            self.root_errors_key = self.depth == 1 && key == "errors";
            self.detail_key = self.in_errors && self.depth == 3 && key == "detail";
            return Ok(VisitControl::Continue);
        }
        let starts_value = matches!(
            event,
            Event::StartObject
                | Event::StartArray
                | Event::StringStart
                | Event::Number(_)
                | Event::Bool(_)
                | Event::Null
        );
        if starts_value {
            if !self.root {
                if event != Event::StartObject {
                    return Err(CratesIoWireError::Envelope);
                }
                self.root = true;
            }
            if self.depth == 1 && self.root_errors_key {
                if event != Event::StartArray {
                    return Err(CratesIoWireError::Envelope);
                }
                self.errors_present = true;
                self.in_errors = true;
            } else if self.in_errors && self.depth == 2 {
                if event != Event::StartObject {
                    return Err(CratesIoWireError::Envelope);
                }
                self.error_count = self
                    .error_count
                    .checked_add(1)
                    .filter(|count| *count <= 256)
                    .ok_or(CratesIoWireError::Envelope)?;
                self.detail_seen = false;
            } else if self.in_errors && self.depth == 3 && self.detail_key {
                if event != Event::StringStart {
                    return Err(CratesIoWireError::Envelope);
                }
                self.detail_seen = true;
            }
            self.root_errors_key = false;
            self.detail_key = false;
        }
        match event {
            Event::StartObject | Event::StartArray => self.depth = self.depth.saturating_add(1),
            Event::EndObject | Event::EndArray => {
                if self.in_errors
                    && self.depth == 3
                    && event == Event::EndObject
                    && !self.detail_seen
                {
                    return Err(CratesIoWireError::Envelope);
                }
                if self.in_errors && self.depth == 2 && event == Event::EndArray {
                    if self.error_count == 0 {
                        return Err(CratesIoWireError::Envelope);
                    }
                    self.in_errors = false;
                }
                self.depth = self
                    .depth
                    .checked_sub(1)
                    .ok_or(CratesIoWireError::Envelope)?;
            }
            Event::StringStart
            | Event::StringFragment(_)
            | Event::StringEnd
            | Event::Number(_)
            | Event::Bool(_)
            | Event::Null => {}
            _ => return Err(CratesIoWireError::Envelope),
        }
        Ok(VisitControl::Continue)
    }
}
