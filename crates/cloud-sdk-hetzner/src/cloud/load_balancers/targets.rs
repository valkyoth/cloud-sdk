//! Load Balancer target request models.

use crate::labels::LabelSelector;

use super::{LoadBalancerIp, LoadBalancerPublicIp, LoadBalancerRequestError, LoadBalancerServerId};

/// Mutually exclusive Load Balancer target selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadBalancerTarget<'a> {
    /// A Cloud server, optionally using a specific public address.
    Server {
        /// Server identifier.
        id: LoadBalancerServerId,
        /// Optional public IPv4 or IPv6 address belonging to the server.
        public_ip: Option<LoadBalancerPublicIp<'a>>,
    },
    /// All servers matching a validated label selector.
    LabelSelector(LabelSelector<'a>),
    /// A same-customer public or vSwitch IP target.
    Ip(LoadBalancerIp<'a>),
}

/// Target accepted during Load Balancer creation or add-target actions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadBalancerAddTargetRequest<'a> {
    target: LoadBalancerTarget<'a>,
    use_private_ip: bool,
}

impl<'a> LoadBalancerAddTargetRequest<'a> {
    /// Creates a target and rejects private-network selection for direct IP targets.
    pub const fn try_new(
        target: LoadBalancerTarget<'a>,
        use_private_ip: bool,
    ) -> Result<Self, LoadBalancerRequestError> {
        if use_private_ip && matches!(target, LoadBalancerTarget::Ip(_)) {
            return Err(LoadBalancerRequestError::InvalidTargetConfiguration);
        }
        if use_private_ip
            && matches!(
                target,
                LoadBalancerTarget::Server {
                    public_ip: Some(_),
                    ..
                }
            )
        {
            return Err(LoadBalancerRequestError::InvalidTargetConfiguration);
        }
        Ok(Self {
            target,
            use_private_ip,
        })
    }

    /// Returns the target selection.
    #[must_use]
    pub const fn target(self) -> LoadBalancerTarget<'a> {
        self.target
    }

    /// Returns whether the target should use its private network address.
    #[must_use]
    pub const fn use_private_ip(self) -> bool {
        self.use_private_ip
    }
}

/// Target accepted by remove-target actions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadBalancerRemoveTargetRequest<'a>(LoadBalancerTarget<'a>);

impl<'a> LoadBalancerRemoveTargetRequest<'a> {
    /// Creates a remove-target request.
    #[must_use]
    pub const fn new(target: LoadBalancerTarget<'a>) -> Self {
        Self(target)
    }

    /// Returns the target selection.
    #[must_use]
    pub const fn target(self) -> LoadBalancerTarget<'a> {
        self.0
    }
}
