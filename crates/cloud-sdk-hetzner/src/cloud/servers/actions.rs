//! Server action request domains.

use cloud_sdk::Method;

use crate::EndpointGroup;
use crate::actions::ActionId;
use crate::request::ApiBaseUrl;
use crate::security::ssh_keys::SshKeyId;

use super::shared::{ResourceId, ServerRequestError, write_id_path};
use super::{ServerId, ServerReference, ServerResourceId, TextValue, UserData};

/// Server action kind from the source-locked API matrix.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServerActionKind {
    /// `add_to_placement_group`.
    AddToPlacementGroup,
    /// `attach_iso`.
    AttachIso,
    /// `attach_to_network`.
    AttachToNetwork,
    /// `change_alias_ips`.
    ChangeAliasIps,
    /// `change_dns_ptr`.
    ChangeDnsPtr,
    /// `change_protection`.
    ChangeProtection,
    /// `change_type`.
    ChangeType,
    /// `create_image`.
    CreateImage,
    /// `detach_from_network`.
    DetachFromNetwork,
    /// `detach_iso`.
    DetachIso,
    /// `disable_backup`.
    DisableBackup,
    /// `disable_rescue`.
    DisableRescue,
    /// `enable_backup`.
    EnableBackup,
    /// `enable_rescue`.
    EnableRescue,
    /// `poweroff`.
    Poweroff,
    /// `poweron`.
    Poweron,
    /// `reboot`.
    Reboot,
    /// `rebuild`.
    Rebuild,
    /// `remove_from_placement_group`.
    RemoveFromPlacementGroup,
    /// `request_console`.
    RequestConsole,
    /// `reset`.
    Reset,
    /// `reset_password`.
    ResetPassword,
    /// `shutdown`.
    Shutdown,
}

impl ServerActionKind {
    const fn suffix(self) -> &'static str {
        match self {
            Self::AddToPlacementGroup => "/actions/add_to_placement_group",
            Self::AttachIso => "/actions/attach_iso",
            Self::AttachToNetwork => "/actions/attach_to_network",
            Self::ChangeAliasIps => "/actions/change_alias_ips",
            Self::ChangeDnsPtr => "/actions/change_dns_ptr",
            Self::ChangeProtection => "/actions/change_protection",
            Self::ChangeType => "/actions/change_type",
            Self::CreateImage => "/actions/create_image",
            Self::DetachFromNetwork => "/actions/detach_from_network",
            Self::DetachIso => "/actions/detach_iso",
            Self::DisableBackup => "/actions/disable_backup",
            Self::DisableRescue => "/actions/disable_rescue",
            Self::EnableBackup => "/actions/enable_backup",
            Self::EnableRescue => "/actions/enable_rescue",
            Self::Poweroff => "/actions/poweroff",
            Self::Poweron => "/actions/poweron",
            Self::Reboot => "/actions/reboot",
            Self::Rebuild => "/actions/rebuild",
            Self::RemoveFromPlacementGroup => "/actions/remove_from_placement_group",
            Self::RequestConsole => "/actions/request_console",
            Self::Reset => "/actions/reset",
            Self::ResetPassword => "/actions/reset_password",
            Self::Shutdown => "/actions/shutdown",
        }
    }
}

/// Server action endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServerActionEndpoint {
    /// `GET /servers/actions`.
    ListAll,
    /// `GET /servers/actions/{id}`.
    Get(ActionId),
    /// `GET /servers/{id}/actions`.
    ListForServer(ServerId),
    /// `POST /servers/{id}/actions/{kind}`.
    Start(ServerId, ServerActionKind),
}

impl ServerActionEndpoint {
    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        match self {
            Self::ListAll | Self::Get(_) | Self::ListForServer(_) => Method::Get,
            Self::Start(_, _) => Method::Post,
        }
    }

    /// Returns the base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::CloudV1
    }

    /// Returns the endpoint group.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::ServerActions
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, ServerRequestError> {
        match self {
            Self::ListAll => super::shared::write_static_path(output, "/servers/actions"),
            Self::Get(id) => {
                let id = ResourceId::new(id.get()).ok_or(ServerRequestError::InvalidReference)?;
                write_id_path(output, "/servers/actions/", id, "")
            }
            Self::ListForServer(id) => write_id_path(output, "/servers/", id, "/actions"),
            Self::Start(id, kind) => write_id_path(output, "/servers/", id, kind.suffix()),
        }
    }
}

/// Explicit DNS pointer action intent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DnsPtrIntent<'a> {
    /// Set PTR to this domain name.
    Set(TextValue<'a>),
    /// Reset IPv4 to default or remove IPv6 PTR.
    Reset,
}

/// Server action request body marker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServerActionRequest<'a> {
    /// Placement group assignment.
    AddToPlacementGroup {
        /// Placement group ID.
        placement_group: ServerResourceId,
    },
    /// Attach ISO by ID or name.
    AttachIso {
        /// ISO ID or name.
        iso: ServerReference<'a>,
    },
    /// Attach to network.
    AttachToNetwork {
        /// Network ID.
        network: ServerResourceId,
        /// Optional requested server IP in the network.
        ip: Option<TextValue<'a>>,
    },
    /// Change alias IPs for an attached network.
    ChangeAliasIps {
        /// Attached network ID.
        network: ServerResourceId,
        /// Replacement alias IP list.
        alias_ips: &'a [TextValue<'a>],
    },
    /// Change reverse DNS pointer with explicit set/reset intent.
    ChangeDnsPtr {
        /// IP address whose PTR should change.
        ip: TextValue<'a>,
        /// Explicit set or reset intent.
        dns_ptr: DnsPtrIntent<'a>,
    },
    /// Change delete/rebuild protection.
    ChangeProtection {
        /// Delete protection value.
        delete: bool,
        /// Rebuild protection value.
        rebuild: bool,
    },
    /// Change server type.
    ChangeType {
        /// Target server type ID or name.
        server_type: ServerReference<'a>,
        /// Whether disk should be upgraded.
        upgrade_disk: bool,
    },
    /// Create image.
    CreateImage {
        /// Optional image description.
        description: Option<TextValue<'a>>,
        /// Image type.
        image_type: ServerImageType,
    },
    /// Detach from network.
    DetachFromNetwork {
        /// Network ID.
        network: ServerResourceId,
    },
    /// Enable rescue.
    EnableRescue {
        /// Rescue system type.
        rescue_type: RescueType,
        /// SSH keys injected into rescue.
        ssh_keys: &'a [SshKeyId],
    },
    /// Rebuild server.
    Rebuild {
        /// Image ID or name.
        image: ServerReference<'a>,
        /// Optional cloud-init user data.
        user_data: Option<UserData<'a>>,
    },
    /// Path-only action without a request body.
    Empty(ServerActionKind),
}

impl<'a> ServerActionRequest<'a> {
    /// Creates a change-alias-IPs request.
    pub fn change_alias_ips(
        network: ServerResourceId,
        alias_ips: &'a [TextValue<'a>],
    ) -> Result<Self, ServerRequestError> {
        if alias_ips.is_empty() {
            return Err(ServerRequestError::MissingRequiredField);
        }
        Ok(Self::ChangeAliasIps { network, alias_ips })
    }

    /// Creates a change-DNS-PTR request requiring explicit set or reset.
    pub fn change_dns_ptr(
        ip: TextValue<'a>,
        dns_ptr: Option<DnsPtrIntent<'a>>,
    ) -> Result<Self, ServerRequestError> {
        Ok(Self::ChangeDnsPtr {
            ip,
            dns_ptr: dns_ptr.ok_or(ServerRequestError::MissingDnsPtrIntent)?,
        })
    }

    /// Creates a path-only action request and rejects actions requiring bodies.
    pub fn empty(kind: ServerActionKind) -> Result<Self, ServerRequestError> {
        if kind_requires_body(kind) {
            return Err(ServerRequestError::MissingRequiredField);
        }
        Ok(Self::Empty(kind))
    }
}

/// Server image type for create-image action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServerImageType {
    /// Snapshot image.
    Snapshot,
    /// Backup image.
    Backup,
}

/// Rescue system type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RescueType {
    /// Linux 64-bit rescue system.
    Linux64,
}

const fn kind_requires_body(kind: ServerActionKind) -> bool {
    match kind {
        ServerActionKind::AddToPlacementGroup
        | ServerActionKind::AttachIso
        | ServerActionKind::AttachToNetwork
        | ServerActionKind::ChangeAliasIps
        | ServerActionKind::ChangeDnsPtr
        | ServerActionKind::ChangeProtection
        | ServerActionKind::ChangeType
        | ServerActionKind::CreateImage
        | ServerActionKind::DetachFromNetwork
        | ServerActionKind::EnableRescue
        | ServerActionKind::Rebuild => true,
        ServerActionKind::DetachIso
        | ServerActionKind::DisableBackup
        | ServerActionKind::DisableRescue
        | ServerActionKind::EnableBackup
        | ServerActionKind::Poweroff
        | ServerActionKind::Poweron
        | ServerActionKind::Reboot
        | ServerActionKind::RemoveFromPlacementGroup
        | ServerActionKind::RequestConsole
        | ServerActionKind::Reset
        | ServerActionKind::ResetPassword
        | ServerActionKind::Shutdown => false,
    }
}
