use alloc::vec::Vec;
use core::{net::IpAddr, str::FromStr};

use cloud_sdk::operation::CheckedResponse;
use cloud_sdk::transport::StatusCode;
use cloud_sdk_sanitization::{SecretString, try_append_secret_string};

use super::model::*;
use super::{MAX_ROBOT_TRAFFIC_RESPONSE_BYTES, RobotTrafficGranularity, RobotTrafficRequest};
use crate::robot::{RobotIpAddress, RobotSubnetAddress};
use crate::serde::{
    IncrementalJsonDecoder, IncrementalJsonError, IncrementalJsonEvent, IncrementalJsonLimits,
    IncrementalJsonProgress, IncrementalJsonVisitor, VisitControl,
};

/// Failure while incrementally decoding a source-locked traffic response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotTrafficDecodeError {
    /// The checked status was not `200 OK`.
    UnexpectedStatus,
    /// The body exceeded the independent traffic response limit.
    ResponseTooLarge,
    /// JSON syntax, UTF-8, duplicates, or parser bounds were invalid.
    MalformedPayload,
    /// The exact source envelope or value shape was violated.
    InvalidEnvelope,
    /// A target key was malformed, non-canonical, duplicated, or unrequested.
    InvalidTarget,
    /// A response interval or aggregation type contradicted the request.
    IntervalMismatch,
    /// A traffic number was negative or outside its lexical bound.
    InvalidAmount,
    /// An individual interval ordinal was invalid or duplicated.
    InvalidPoint,
    /// Bounded protected result storage could not be allocated.
    Allocation,
}

impl_static_error!(RobotTrafficDecodeError,
    Self::UnexpectedStatus => "Robot traffic response status is unexpected",
    Self::ResponseTooLarge => "Robot traffic response exceeds its operation limit",
    Self::MalformedPayload => "Robot traffic response JSON is malformed",
    Self::InvalidEnvelope => "Robot traffic response envelope is invalid",
    Self::InvalidTarget => "Robot traffic response target is invalid",
    Self::IntervalMismatch => "Robot traffic response interval contradicts the request",
    Self::InvalidAmount => "Robot traffic response amount is invalid",
    Self::InvalidPoint => "Robot traffic response interval point is invalid",
    Self::Allocation => "Robot traffic response allocation failed",
);

pub(super) fn decode_robot_traffic(
    checked: CheckedResponse<'_>,
    request: &RobotTrafficRequest,
) -> Result<RobotTrafficReport, RobotTrafficDecodeError> {
    if checked.status() != StatusCode::OK {
        return Err(RobotTrafficDecodeError::UnexpectedStatus);
    }
    if checked.body().len() > MAX_ROBOT_TRAFFIC_RESPONSE_BYTES {
        return Err(RobotTrafficDecodeError::ResponseTooLarge);
    }
    let limits = IncrementalJsonLimits::DEFAULT
        .with_input_bytes(MAX_ROBOT_TRAFFIC_RESPONSE_BYTES)
        .map_err(|_| RobotTrafficDecodeError::MalformedPayload)?
        .with_object_fields(4_096)
        .map_err(|_| RobotTrafficDecodeError::MalformedPayload)?;
    let mut decoder = IncrementalJsonDecoder::with_limits(limits);
    let mut visitor = TrafficVisitor::new(request)?;
    for chunk in checked.body().chunks(4_096) {
        if decoder.push(chunk, &mut visitor).map_err(map_parser)?
            != IncrementalJsonProgress::Pending
        {
            return Err(RobotTrafficDecodeError::MalformedPayload);
        }
    }
    if decoder.finish(&mut visitor).map_err(map_parser)? != IncrementalJsonProgress::Complete {
        return Err(RobotTrafficDecodeError::MalformedPayload);
    }
    visitor.finish()
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Pending {
    None,
    Root,
    Kind,
    From,
    To,
    Data,
    Target,
    Incoming,
    Outgoing,
    Total,
    Point,
}

struct AmountBuilder {
    incoming: Option<RobotTrafficAmount>,
    outgoing: Option<RobotTrafficAmount>,
    total: Option<RobotTrafficAmount>,
}

impl AmountBuilder {
    const fn new() -> Self {
        Self {
            incoming: None,
            outgoing: None,
            total: None,
        }
    }

    fn set(&mut self, field: Pending, value: &str) -> Result<(), RobotTrafficDecodeError> {
        let value =
            RobotTrafficAmount::new(value).map_err(|()| RobotTrafficDecodeError::InvalidAmount)?;
        let slot = match field {
            Pending::Incoming => &mut self.incoming,
            Pending::Outgoing => &mut self.outgoing,
            Pending::Total => &mut self.total,
            _ => return Err(RobotTrafficDecodeError::InvalidEnvelope),
        };
        if slot.replace(value).is_some() {
            return Err(RobotTrafficDecodeError::InvalidEnvelope);
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<RobotTrafficData, RobotTrafficDecodeError> {
        Ok(RobotTrafficData {
            incoming: self
                .incoming
                .take()
                .ok_or(RobotTrafficDecodeError::InvalidEnvelope)?,
            outgoing: self
                .outgoing
                .take()
                .ok_or(RobotTrafficDecodeError::InvalidEnvelope)?,
            total: self
                .total
                .take()
                .ok_or(RobotTrafficDecodeError::InvalidEnvelope)?,
        })
    }
}

struct TrafficVisitor<'request> {
    request: &'request RobotTrafficRequest,
    depth: u8,
    pending: Pending,
    traffic_fields: u8,
    root_seen: bool,
    string: Option<(Pending, SecretString)>,
    target: Option<RobotTrafficResultTarget>,
    point: Option<u8>,
    amount: AmountBuilder,
    points: Vec<RobotTrafficPoint>,
    results: Vec<RobotTrafficResult>,
    seen_targets: Vec<u8>,
}

impl<'request> TrafficVisitor<'request> {
    fn new(request: &'request RobotTrafficRequest) -> Result<Self, RobotTrafficDecodeError> {
        let mut seen_targets = Vec::new();
        seen_targets
            .try_reserve_exact(request.targets.len())
            .map_err(|_| RobotTrafficDecodeError::Allocation)?;
        seen_targets.resize(request.targets.len(), 0);
        Ok(Self {
            request,
            depth: 0,
            pending: Pending::None,
            traffic_fields: 0,
            root_seen: false,
            string: None,
            target: None,
            point: None,
            amount: AmountBuilder::new(),
            points: Vec::new(),
            results: Vec::new(),
            seen_targets,
        })
    }

    fn finish(self) -> Result<RobotTrafficReport, RobotTrafficDecodeError> {
        if self.depth != 0 || !self.root_seen || self.traffic_fields != 0b1111 {
            return Err(RobotTrafficDecodeError::InvalidEnvelope);
        }
        Ok(RobotTrafficReport {
            granularity: self.request.interval.granularity(),
            results: self.results,
        })
    }

    fn key(&mut self, key: &str) -> Result<(), RobotTrafficDecodeError> {
        if self.pending != Pending::None || self.string.is_some() {
            return Err(RobotTrafficDecodeError::InvalidEnvelope);
        }
        self.pending = match self.depth {
            1 if key == "traffic" => {
                self.root_seen = true;
                Pending::Root
            }
            2 => match key {
                "type" => self.traffic_field(0b0001, Pending::Kind)?,
                "from" => self.traffic_field(0b0010, Pending::From)?,
                "to" => self.traffic_field(0b0100, Pending::To)?,
                "data" => self.traffic_field(0b1000, Pending::Data)?,
                _ => return Err(RobotTrafficDecodeError::InvalidEnvelope),
            },
            3 => {
                let (target, index) = parse_target(key, self.request)?;
                let seen = self
                    .seen_targets
                    .get_mut(index)
                    .ok_or(RobotTrafficDecodeError::InvalidTarget)?;
                if core::mem::replace(seen, 1) != 0 {
                    return Err(RobotTrafficDecodeError::InvalidTarget);
                }
                self.target = Some(target);
                Pending::Target
            }
            4 if self.request.single_values => {
                self.point = Some(parse_point(key, self.request.interval.granularity())?);
                Pending::Point
            }
            4 | 5 => amount_field(key)?,
            _ => return Err(RobotTrafficDecodeError::InvalidEnvelope),
        };
        Ok(())
    }

    fn traffic_field(
        &mut self,
        bit: u8,
        pending: Pending,
    ) -> Result<Pending, RobotTrafficDecodeError> {
        if self.traffic_fields & bit != 0 {
            return Err(RobotTrafficDecodeError::InvalidEnvelope);
        }
        self.traffic_fields |= bit;
        Ok(pending)
    }

    fn start_object(&mut self) -> Result<(), RobotTrafficDecodeError> {
        let valid = matches!(
            (self.depth, self.pending),
            (0, Pending::None)
                | (1, Pending::Root)
                | (2, Pending::Data)
                | (3, Pending::Target)
                | (4, Pending::Point)
        );
        if !valid {
            return Err(RobotTrafficDecodeError::InvalidEnvelope);
        }
        self.pending = Pending::None;
        self.depth = self
            .depth
            .checked_add(1)
            .ok_or(RobotTrafficDecodeError::InvalidEnvelope)?;
        Ok(())
    }

    fn end_object(&mut self) -> Result<(), RobotTrafficDecodeError> {
        if self.pending != Pending::None || self.string.is_some() {
            return Err(RobotTrafficDecodeError::InvalidEnvelope);
        }
        match self.depth {
            5 => {
                let ordinal = self
                    .point
                    .take()
                    .ok_or(RobotTrafficDecodeError::InvalidPoint)?;
                let data = self.amount.finish()?;
                self.points
                    .try_reserve(1)
                    .map_err(|_| RobotTrafficDecodeError::Allocation)?;
                self.points.push(RobotTrafficPoint { ordinal, data });
            }
            4 => self.finish_target()?,
            3 => {}
            2 if self.traffic_fields == 0b1111 => {}
            1 if self.root_seen => {}
            _ => return Err(RobotTrafficDecodeError::InvalidEnvelope),
        }
        self.depth = self
            .depth
            .checked_sub(1)
            .ok_or(RobotTrafficDecodeError::InvalidEnvelope)?;
        Ok(())
    }

    fn finish_target(&mut self) -> Result<(), RobotTrafficDecodeError> {
        let target = self
            .target
            .take()
            .ok_or(RobotTrafficDecodeError::InvalidTarget)?;
        let result = if self.request.single_values {
            if self.points.is_empty() {
                return Err(RobotTrafficDecodeError::InvalidPoint);
            }
            self.points.sort_unstable_by_key(RobotTrafficPoint::ordinal);
            if self.points.windows(2).any(|pair| {
                pair.first().map(RobotTrafficPoint::ordinal)
                    == pair.get(1).map(RobotTrafficPoint::ordinal)
            }) {
                return Err(RobotTrafficDecodeError::InvalidPoint);
            }
            RobotTrafficResult::SingleValues {
                target,
                points: core::mem::take(&mut self.points),
            }
        } else {
            RobotTrafficResult::Aggregate {
                target,
                data: self.amount.finish()?,
            }
        };
        self.results
            .try_reserve(1)
            .map_err(|_| RobotTrafficDecodeError::Allocation)?;
        self.results.push(result);
        Ok(())
    }

    fn start_string(&mut self) -> Result<(), RobotTrafficDecodeError> {
        if !matches!(self.pending, Pending::Kind | Pending::From | Pending::To) {
            return Err(RobotTrafficDecodeError::InvalidEnvelope);
        }
        let field = core::mem::replace(&mut self.pending, Pending::None);
        let value =
            SecretString::try_with_capacity(13).map_err(|_| RobotTrafficDecodeError::Allocation)?;
        self.string = Some((field, value));
        Ok(())
    }

    fn string_fragment(&mut self, fragment: &str) -> Result<(), RobotTrafficDecodeError> {
        let (_, value) = self
            .string
            .as_mut()
            .ok_or(RobotTrafficDecodeError::InvalidEnvelope)?;
        try_append_secret_string(value, fragment, 13)
            .map_err(|_| RobotTrafficDecodeError::InvalidEnvelope)
    }

    fn end_string(&mut self) -> Result<(), RobotTrafficDecodeError> {
        let (field, value) = self
            .string
            .take()
            .ok_or(RobotTrafficDecodeError::InvalidEnvelope)?;
        let matches = value
            .try_with_secret(|text| match field {
                Pending::Kind => text == self.request.interval.granularity().wire_name(),
                Pending::From => self.request.interval.matches_from(text),
                Pending::To => self.request.interval.matches_to(text),
                _ => false,
            })
            .map_err(|_| RobotTrafficDecodeError::InvalidEnvelope)?;
        if matches {
            Ok(())
        } else {
            Err(RobotTrafficDecodeError::IntervalMismatch)
        }
    }
}

impl IncrementalJsonVisitor for TrafficVisitor<'_> {
    type Error = RobotTrafficDecodeError;

    fn visit(&mut self, event: IncrementalJsonEvent<'_>) -> Result<VisitControl, Self::Error> {
        match event {
            IncrementalJsonEvent::StartObject => self.start_object()?,
            IncrementalJsonEvent::EndObject => self.end_object()?,
            IncrementalJsonEvent::Key(key) => self.key(key)?,
            IncrementalJsonEvent::StringStart => self.start_string()?,
            IncrementalJsonEvent::StringFragment(value) => self.string_fragment(value)?,
            IncrementalJsonEvent::StringEnd => self.end_string()?,
            IncrementalJsonEvent::Number(value) => {
                let field = core::mem::replace(&mut self.pending, Pending::None);
                self.amount.set(field, value)?;
            }
            IncrementalJsonEvent::StartArray
            | IncrementalJsonEvent::EndArray
            | IncrementalJsonEvent::Bool(_)
            | IncrementalJsonEvent::Null => return Err(RobotTrafficDecodeError::InvalidEnvelope),
        }
        Ok(VisitControl::Continue)
    }
}

fn amount_field(key: &str) -> Result<Pending, RobotTrafficDecodeError> {
    match key {
        "in" => Ok(Pending::Incoming),
        "out" => Ok(Pending::Outgoing),
        "sum" => Ok(Pending::Total),
        _ => Err(RobotTrafficDecodeError::InvalidEnvelope),
    }
}

fn parse_point(
    value: &str,
    granularity: RobotTrafficGranularity,
) -> Result<u8, RobotTrafficDecodeError> {
    let bytes = value.as_bytes();
    if bytes.len() != 2 || !bytes.iter().all(u8::is_ascii_digit) {
        return Err(RobotTrafficDecodeError::InvalidPoint);
    }
    let ordinal = bytes
        .first()
        .and_then(|tens| tens.checked_sub(b'0'))
        .and_then(|tens| {
            bytes
                .get(1)
                .and_then(|ones| ones.checked_sub(b'0'))
                .map(|ones| tens.saturating_mul(10).saturating_add(ones))
        })
        .ok_or(RobotTrafficDecodeError::InvalidPoint)?;
    let valid = match granularity {
        RobotTrafficGranularity::Day => ordinal <= 23,
        RobotTrafficGranularity::Month => (1..=31).contains(&ordinal),
        RobotTrafficGranularity::Year => (1..=12).contains(&ordinal),
    };
    valid
        .then_some(ordinal)
        .ok_or(RobotTrafficDecodeError::InvalidPoint)
}

fn parse_target(
    value: &str,
    request: &RobotTrafficRequest,
) -> Result<(RobotTrafficResultTarget, usize), RobotTrafficDecodeError> {
    let (address_text, prefix) = match value.split_once('/') {
        Some((address, prefix)) => {
            let prefix = parse_canonical_prefix(prefix)?;
            (address, Some(prefix))
        }
        None => (value, None),
    };
    let address =
        IpAddr::from_str(address_text).map_err(|_| RobotTrafficDecodeError::InvalidTarget)?;
    if !canonical_network(address, prefix) {
        return Err(RobotTrafficDecodeError::InvalidTarget);
    }
    let index = request
        .target_index(address)
        .ok_or(RobotTrafficDecodeError::InvalidTarget)?;
    let requested = request
        .targets
        .get(index)
        .ok_or(RobotTrafficDecodeError::InvalidTarget)?;
    let requested_kind = requested.with_address(|_, subnet| subnet);
    if requested_kind != prefix.is_some() {
        return Err(RobotTrafficDecodeError::InvalidTarget);
    }
    let target = match prefix {
        None => RobotIpAddress::new(address_text)
            .map(RobotTrafficResultTarget::Ip)
            .map_err(|_| RobotTrafficDecodeError::InvalidTarget),
        Some(prefix) => RobotSubnetAddress::new(address_text)
            .map(|address| RobotTrafficResultTarget::Subnet { address, prefix })
            .map_err(|_| RobotTrafficDecodeError::InvalidTarget),
    }?;
    Ok((target, index))
}

fn parse_canonical_prefix(value: &str) -> Result<u8, RobotTrafficDecodeError> {
    let bytes = value.as_bytes();
    if bytes.is_empty()
        || !bytes.iter().all(u8::is_ascii_digit)
        || (bytes.len() > 1 && bytes.first() == Some(&b'0'))
    {
        return Err(RobotTrafficDecodeError::InvalidTarget);
    }
    value
        .parse::<u8>()
        .map_err(|_| RobotTrafficDecodeError::InvalidTarget)
}

fn canonical_network(address: IpAddr, prefix: Option<u8>) -> bool {
    match (address, prefix) {
        (_, None) => true,
        (IpAddr::V4(address), Some(prefix)) if prefix <= 32 => {
            let shift = 32_u32.saturating_sub(u32::from(prefix));
            let mask = if prefix == 0 { 0 } else { u32::MAX << shift };
            u32::from(address) & !mask == 0
        }
        (IpAddr::V6(address), Some(prefix)) if prefix <= 128 => {
            let shift = 128_u32.saturating_sub(u32::from(prefix));
            let mask = if prefix == 0 { 0 } else { u128::MAX << shift };
            u128::from(address) & !mask == 0
        }
        _ => false,
    }
}

fn map_parser(error: IncrementalJsonError<RobotTrafficDecodeError>) -> RobotTrafficDecodeError {
    match error {
        IncrementalJsonError::Visitor(error) => error,
        IncrementalJsonError::Allocation => RobotTrafficDecodeError::Allocation,
        _ => RobotTrafficDecodeError::MalformedPayload,
    }
}
