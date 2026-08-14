use alloc::vec::Vec;

use cloud_sdk::buffer::write_u64;

use super::prepare::Kind;
use super::{RobotVSwitchRequestError, RobotVSwitchUpdateIntent};
use crate::robot::{RobotCancellationSchedule, RobotForm, RobotFormField};

pub(super) fn write_form(
    kind: Kind<'_>,
    output: &mut [u8],
) -> Result<usize, RobotVSwitchRequestError> {
    match kind {
        Kind::List | Kind::Get(_) => Ok(0),
        Kind::Create(name, vlan) => name.with_text(|name| {
            with_decimal(u64::from(vlan.get()), |vlan| {
                encode(&[field("name", name)?, field("vlan", vlan)?], output)
            })
        }),
        Kind::Update(_, intent) => write_update(intent, output),
        Kind::Cancel(_, schedule) => write_cancellation(schedule, output),
        Kind::AddServers(_, servers) | Kind::RemoveServers(_, servers) => {
            let selectors = servers.as_slice();
            let mut fields = Vec::new();
            fields
                .try_reserve_exact(selectors.len())
                .map_err(|_| RobotVSwitchRequestError::Allocation)?;
            for selector in selectors {
                fields.push(field("server[]", selector.as_str())?);
            }
            encode(&fields, output)
        }
    }
}

fn write_update(
    intent: &RobotVSwitchUpdateIntent,
    output: &mut [u8],
) -> Result<usize, RobotVSwitchRequestError> {
    match intent {
        RobotVSwitchUpdateIntent::Rename(name) => {
            name.with_text(|name| encode(&[field("name", name)?], output))
        }
        RobotVSwitchUpdateIntent::ChangeVlan(vlan) => with_decimal(u64::from(vlan.get()), |vlan| {
            encode(&[field("vlan", vlan)?], output)
        }),
        RobotVSwitchUpdateIntent::RenameAndChangeVlan { name, vlan } => name.with_text(|name| {
            with_decimal(u64::from(vlan.get()), |vlan| {
                encode(&[field("name", name)?, field("vlan", vlan)?], output)
            })
        }),
    }
}

fn write_cancellation(
    schedule: &RobotCancellationSchedule,
    output: &mut [u8],
) -> Result<usize, RobotVSwitchRequestError> {
    match schedule {
        RobotCancellationSchedule::Immediate => {
            encode(&[field("cancellation_date", "now")?], output)
        }
        RobotCancellationSchedule::On(date) => {
            date.with_text(|date| encode(&[field("cancellation_date", date)?], output))
        }
    }
}

fn field<'a>(
    name: &'a str,
    value: &'a str,
) -> Result<RobotFormField<'a>, RobotVSwitchRequestError> {
    RobotFormField::sensitive(name, value).map_err(RobotVSwitchRequestError::Form)
}

fn encode(
    fields: &[RobotFormField<'_>],
    output: &mut [u8],
) -> Result<usize, RobotVSwitchRequestError> {
    RobotForm::new(fields)
        .and_then(|form| form.write_prepared(output))
        .map_err(RobotVSwitchRequestError::Form)
}

fn with_decimal<R>(
    value: u64,
    inspect: impl FnOnce(&str) -> Result<R, RobotVSwitchRequestError>,
) -> Result<R, RobotVSwitchRequestError> {
    let mut storage = [0_u8; 20];
    let mut len = 0;
    write_u64(
        &mut storage,
        &mut len,
        value,
        RobotVSwitchRequestError::Path,
    )?;
    let text = core::str::from_utf8(storage.get(..len).unwrap_or_default())
        .map_err(|_| RobotVSwitchRequestError::Path)?;
    inspect(text)
}
