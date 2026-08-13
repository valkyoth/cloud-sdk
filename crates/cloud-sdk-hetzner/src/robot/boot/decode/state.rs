use super::RobotBootDecodeError;
use crate::robot::boot::{RobotBootChoice, RobotBootEntry};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::robot::boot) enum RobotBootEntryShape {
    Overview,
    Current,
    Last,
    Activation,
    Deactivation,
}

pub(super) fn validate_state(
    entry: &RobotBootEntry,
    shape: RobotBootEntryShape,
) -> Result<(), RobotBootDecodeError> {
    let active = entry.is_active();
    let has_password = entry.password().is_some();
    let primary_selected = entry.primary_choice().is_selected();
    let language_selected = entry.languages().map(RobotBootChoice::is_selected);
    let valid = match shape {
        RobotBootEntryShape::Overview | RobotBootEntryShape::Current => {
            active == has_password
                && active == primary_selected
                && language_selected.is_none_or(|selected| selected == active)
        }
        RobotBootEntryShape::Last => {
            active == has_password
                && primary_selected
                && language_selected.is_none_or(|selected| selected)
        }
        RobotBootEntryShape::Activation => {
            active
                && has_password
                && primary_selected
                && language_selected.is_none_or(|selected| selected)
        }
        RobotBootEntryShape::Deactivation => {
            !active
                && !has_password
                && !primary_selected
                && language_selected.is_none_or(|selected| !selected)
        }
    };
    if valid {
        Ok(())
    } else {
        Err(RobotBootDecodeError::MutationOutcomeMismatch)
    }
}
