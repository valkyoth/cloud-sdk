//! Source-locked Primary IP selection during Load Balancer creation.

use crate::cloud::load_balancers::LoadBalancerCreateRequest;
use crate::prepared::{HetznerPreparationError, JsonWriter};

pub(super) fn write(
    request: LoadBalancerCreateRequest<'_>,
    writer: &mut JsonWriter<'_, '_>,
    first: &mut bool,
) -> Result<(), HetznerPreparationError> {
    let Some((ipv4, ipv6)) = request.public_net() else {
        return Ok(());
    };
    if request.public_interface() == Some(false) {
        return Err(HetznerPreparationError::Body);
    }
    writer.field(first, "public_net")?;
    writer.begin_object()?;
    let mut field = true;
    for (name, id) in [("ipv4", ipv4), ("ipv6", ipv6)] {
        match id {
            Some(id) => writer.field_u64(&mut field, name, id.get())?,
            None => writer.field_null(&mut field, name)?,
        }
    }
    writer.end_object()
}
