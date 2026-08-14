//! Hetzner Robot Webservice request primitives.
//!
//! Robot uses HTTP Basic authentication and form bodies rather than the
//! bearer-token JSON protocol used by Hetzner Cloud APIs. The module provides
//! bounded forms plus an allocation-gated protected credential and lockout
//! policy. Server operations are source-locked in `v0.78.0`, and server, IP,
//! and subnet cancellation operations are source-locked in `v0.79.0`. Active
//! single-IP and separate-MAC operations are source-locked in `v0.80.0`, and
//! active subnet and subnet-MAC operations are source-locked in `v0.81.0`.
//! Reset discovery and disruptive execution are source-locked in `v0.82.0`.
//! Failover discovery, rerouting, and route deletion are source-locked in
//! `v0.83.0`. Capability-checked Wake-on-LAN discovery and execution are
//! source-locked in `v0.84.0`.
//! Rescue, Linux, VNC, and Windows boot configuration operations are
//! source-locked in `v0.85.0`.
//! Reverse-DNS discovery and mutations are source-locked in `v0.86.0`.
//! Bounded traffic queries and incremental traffic decoding are source-locked
//! in `v0.87.0`.
//! SSH-key inventory and lifecycle operations are source-locked in `v0.88.0`.
//! Ordered firewall and firewall-template operations are source-locked in
//! `v0.89.0`.
//! vSwitch inventory, configuration, cancellation, and repeated server
//! membership operations are source-locked in `v0.90.0`.
//! Read-only server, Server Auction, addon, and account-currency catalogs are
//! source-locked in `v0.91.0`; ordering remains deliberately non-executable.

#[cfg(feature = "alloc")]
mod boot;
#[cfg(feature = "alloc")]
mod cancellation;
#[cfg(feature = "alloc")]
mod canonical;
#[cfg(feature = "alloc")]
mod credentials;
#[cfg(feature = "serde")]
mod duplicates;
#[cfg(feature = "alloc")]
mod failover;
#[cfg(feature = "alloc")]
mod firewall;
mod form;
#[cfg(feature = "alloc")]
mod ip;
#[cfg(feature = "alloc")]
mod ordering;
#[cfg(feature = "serde")]
mod protocol;
#[cfg(feature = "alloc")]
mod rdns;
#[cfg(feature = "alloc")]
mod reset;
#[cfg(feature = "alloc")]
mod server;
#[cfg(feature = "alloc")]
mod ssh_keys;
#[cfg(feature = "alloc")]
mod subnet;
#[cfg(feature = "serde")]
mod traffic;
#[cfg(feature = "alloc")]
mod vswitch;
#[cfg(feature = "alloc")]
mod wol;

/// Maximum Robot error-body bytes admitted by request and response policies.
pub const MAX_ROBOT_ERROR_BODY_BYTES: usize = 65_536;

#[cfg(feature = "alloc")]
pub use cancellation::{
    MAX_ROBOT_CANCELLATION_REASON_INPUT_BYTES, RobotCancellationDate, RobotCancellationReason,
    RobotCancellationRequestError, RobotCancellationSchedule, RobotCancellationValueError,
    RobotIpAddress, RobotIpCancellationCreateRequest, RobotIpCancellationDeleteRequest,
    RobotIpCancellationGetRequest, RobotLocationReservationIntent,
    RobotServerCancellationCreateRequest, RobotServerCancellationDeleteRequest,
    RobotServerCancellationGetRequest, RobotSubnetAddress, RobotSubnetCancellationCreateRequest,
    RobotSubnetCancellationDeleteRequest, RobotSubnetCancellationGetRequest,
};

#[cfg(feature = "serde")]
pub use cancellation::{
    CancellationCanonicalPlanFingerprint, CancellationDestructivePermit, CancellationPermitAttempt,
    CancellationPlanConfirmation, CancellationPlanFingerprintDigest, CancellationPlanSubject,
    CancellationSharedDestructivePermit, CheckedCancellation, MAX_ROBOT_CANCELLATION_REASON_BYTES,
    MAX_ROBOT_CANCELLATION_REASONS, PreparedCancellation, RobotCancellationDecodeError,
    RobotIpCancellation, RobotServerCancellation, RobotServerCancellationReason,
    RobotSubnetCancellation, build_cancellation_canonical_plan, build_cancellation_plan_digest,
    decode_robot_ip_cancellation, decode_robot_server_cancellation,
    decode_robot_subnet_cancellation,
};

#[cfg(feature = "alloc")]
pub use credentials::{
    MAX_ROBOT_PASSWORD_BYTES, MAX_ROBOT_USERNAME_BYTES, RobotCredentialAttempt,
    RobotCredentialError, RobotCredentialRotationError, RobotCredentialScope,
    RobotCredentialStateError, RobotCredentials,
};

pub use form::{
    EncodedRobotForm, MAX_ROBOT_FORM_BODY_BYTES, MAX_ROBOT_FORM_FIELDS, MAX_ROBOT_FORM_NAME_BYTES,
    MAX_ROBOT_FORM_VALUE_BYTES, RobotForm, RobotFormError, RobotFormField, RobotFormSensitivity,
};

#[cfg(feature = "alloc")]
pub use boot::{
    MAX_ROBOT_BOOT_AUTHORIZED_KEYS, MAX_ROBOT_BOOT_KEY_BYTES, MAX_ROBOT_BOOT_RESPONSE_BYTES,
    MAX_ROBOT_BOOT_VALUE_BYTES, ROBOT_BOOT_QUOTA, RobotBootGetRequest, RobotBootKey,
    RobotBootQuota, RobotBootRequestError, RobotBootValue, RobotKeyboardLayout,
    RobotLinuxActivateRequest, RobotLinuxDeactivateRequest, RobotLinuxGetRequest,
    RobotLinuxLastRequest, RobotRescueActivateRequest, RobotRescueDeactivateRequest,
    RobotRescueGetRequest, RobotRescueLastRequest, RobotVncActivateRequest,
    RobotVncDeactivateRequest, RobotVncGetRequest, RobotWindowsActivateRequest,
    RobotWindowsDeactivateRequest, RobotWindowsGetRequest,
};

#[cfg(feature = "serde")]
pub use boot::{
    CheckedRobotBoot, PreparedRobotBoot, RobotBoot, RobotBootChoice, RobotBootDecodeError,
    RobotBootEntry, RobotBootFailureCode, RobotBootFamily, RobotBootSecret,
};

#[cfg(feature = "alloc")]
pub use failover::{
    MAX_ROBOT_FAILOVER_ITEM_RESPONSE_BYTES, MAX_ROBOT_FAILOVER_LIST_RESPONSE_BYTES,
    RobotFailoverDeleteRouteRequest, RobotFailoverGetRequest, RobotFailoverListRequest,
    RobotFailoverRequestError, RobotFailoverRerouteRequest,
};

#[cfg(feature = "alloc")]
pub use firewall::{
    MAX_ROBOT_FIREWALL_ITEM_RESPONSE_BYTES, MAX_ROBOT_FIREWALL_RULE_NAME_BYTES,
    MAX_ROBOT_FIREWALL_RULES_PER_DIRECTION, MAX_ROBOT_FIREWALL_TEMPLATE_LIST_RESPONSE_BYTES,
    MAX_ROBOT_FIREWALL_TEMPLATE_NAME_BYTES, RobotFirewallAction, RobotFirewallCidr,
    RobotFirewallDeleteRequest, RobotFirewallGetRequest, RobotFirewallIpVersion,
    RobotFirewallPortRange, RobotFirewallProtocol, RobotFirewallReplaceIntent,
    RobotFirewallReplaceRequest, RobotFirewallRequestError, RobotFirewallRule,
    RobotFirewallRuleError, RobotFirewallRules, RobotFirewallStatus, RobotFirewallTcpFlags,
    RobotFirewallTemplateConfig, RobotFirewallTemplateCreateRequest,
    RobotFirewallTemplateDeleteRequest, RobotFirewallTemplateGetRequest, RobotFirewallTemplateId,
    RobotFirewallTemplateListRequest, RobotFirewallTemplateName,
    RobotFirewallTemplateUpdateRequest,
};

#[cfg(feature = "serde")]
pub use firewall::{
    CheckedRobotFirewall, MAX_ROBOT_FIREWALL_TEMPLATE_LIST_ITEMS, PendingRobotFirewallTemplate,
    PreparedRobotFirewall, RobotFirewall, RobotFirewallCanonicalPlanFingerprint,
    RobotFirewallDecodeError, RobotFirewallDestructivePermit, RobotFirewallFailureCode,
    RobotFirewallMutationPermit, RobotFirewallPermitAttempt, RobotFirewallPermitRequest,
    RobotFirewallPlanConfirmation, RobotFirewallPlanFingerprintDigest, RobotFirewallPlanSubject,
    RobotFirewallPort, RobotFirewallRuleModel, RobotFirewallRuleSet, RobotFirewallRuntimeStatus,
    RobotFirewallSharedDestructivePermit, RobotFirewallSharedMutationPermit, RobotFirewallTemplate,
    RobotFirewallTemplateList, RobotFirewallTemplateMutationOutcome,
    RobotFirewallTemplateReconciliation, RobotFirewallTemplateSummary,
    build_robot_firewall_canonical_plan, build_robot_firewall_plan_digest,
};

#[cfg(feature = "serde")]
pub use failover::{
    CheckedRobotFailover, MAX_ROBOT_FAILOVER_LIST_ITEMS, PreparedRobotFailover, RobotFailover,
    RobotFailoverCanonicalPlanFingerprint, RobotFailoverDecodeError,
    RobotFailoverDestructivePermit, RobotFailoverFailureCode, RobotFailoverList,
    RobotFailoverMutationPermit, RobotFailoverPermitAttempt, RobotFailoverPermitRequest,
    RobotFailoverPlanConfirmation, RobotFailoverPlanFingerprintDigest, RobotFailoverPlanSubject,
    RobotFailoverSharedDestructivePermit, RobotFailoverSharedMutationPermit,
    build_robot_failover_canonical_plan, build_robot_failover_plan_digest, decode_robot_failover,
    decode_robot_failover_list,
};

#[cfg(feature = "alloc")]
pub use ip::{
    RobotIpGetRequest, RobotIpListRequest, RobotIpMacDeleteRequest, RobotIpMacGetRequest,
    RobotIpMacSetRequest, RobotIpRequestError, RobotIpTrafficUpdate, RobotIpUpdateRequest,
    RobotMacAddress, RobotMacAddressError,
};

#[cfg(feature = "alloc")]
pub use ordering::{
    MAX_ROBOT_ORDER_CHOICE_BYTES, MAX_ROBOT_ORDER_ITEM_RESPONSE_BYTES,
    MAX_ROBOT_ORDER_LIST_RESPONSE_BYTES, MAX_ROBOT_ORDER_LOCATION_BYTES,
    MAX_ROBOT_ORDER_PRODUCT_ID_BYTES, RobotAddonProductListRequest, RobotMarketProductGetRequest,
    RobotMarketProductId, RobotMarketProductListRequest, RobotOrderChoice, RobotOrderCurrency,
    RobotOrderCurrencyRequest, RobotOrderDecimal, RobotOrderLocation, RobotOrderProductId,
    RobotOrderRequestError, RobotOrderValueError, RobotStandardProductFilters,
    RobotStandardProductGetRequest, RobotStandardProductListRequest,
};

#[cfg(feature = "serde")]
pub use ordering::{
    CheckedRobotOrderCatalog, MAX_ROBOT_ADDON_PRODUCTS, MAX_ROBOT_MARKET_PRODUCTS,
    MAX_ROBOT_STANDARD_PRODUCTS, PreparedRobotOrderCatalog, RobotAddonCatalog, RobotAddonOrderPlan,
    RobotAddonProduct, RobotAddonProductList, RobotCatalogPlanError, RobotCatalogPriceWarning,
    RobotMarketOrderPlan, RobotMarketProduct, RobotMarketProductList, RobotOrderCatalogDecodeError,
    RobotOrderFailureCode, RobotOrderPrice, RobotOrderPricePair, RobotOrderText,
    RobotOrderableAddon, RobotStandardAddonSelection, RobotStandardOrderPlan, RobotStandardProduct,
    RobotStandardProductList,
};

#[cfg(feature = "alloc")]
pub use rdns::{
    MAX_ROBOT_RDNS_ITEM_RESPONSE_BYTES, MAX_ROBOT_RDNS_LIST_RESPONSE_BYTES,
    MAX_ROBOT_RDNS_NAME_BYTES, RobotRdnsDeleteRequest, RobotRdnsGetRequest, RobotRdnsListRequest,
    RobotRdnsName, RobotRdnsNameError, RobotRdnsRequestError, RobotRdnsSetRequest,
    RobotRdnsUpdateRequest,
};

#[cfg(feature = "alloc")]
pub use ssh_keys::{
    MAX_ROBOT_SSH_KEY_DATA_BYTES, MAX_ROBOT_SSH_KEY_ITEM_RESPONSE_BYTES,
    MAX_ROBOT_SSH_KEY_LIST_RESPONSE_BYTES, MAX_ROBOT_SSH_KEY_NAME_BYTES, RobotSshKeyCreateRequest,
    RobotSshKeyData, RobotSshKeyDeleteRequest, RobotSshKeyFingerprint, RobotSshKeyGetRequest,
    RobotSshKeyListRequest, RobotSshKeyName, RobotSshKeyRequestError, RobotSshKeyUpdateRequest,
    RobotSshKeyValueError,
};

#[cfg(feature = "serde")]
pub use ssh_keys::{
    CheckedRobotSshKey, MAX_ROBOT_SSH_KEY_LIST_ITEMS, PreparedRobotSshKey, RobotSshKey,
    RobotSshKeyAlgorithm, RobotSshKeyCanonicalPlanFingerprint, RobotSshKeyCreatedAt,
    RobotSshKeyDecodeError, RobotSshKeyDestructivePermit, RobotSshKeyFailureCode, RobotSshKeyList,
    RobotSshKeyMutationPermit, RobotSshKeyPermitAttempt, RobotSshKeyPermitRequest,
    RobotSshKeyPlanConfirmation, RobotSshKeyPlanFingerprintDigest, RobotSshKeyPlanSubject,
    RobotSshKeySharedDestructivePermit, RobotSshKeySharedMutationPermit,
    build_robot_ssh_key_canonical_plan, build_robot_ssh_key_plan_digest,
};

#[cfg(feature = "serde")]
pub use rdns::{
    CheckedRobotRdns, MAX_ROBOT_RDNS_LIST_ITEMS, PreparedRobotRdns, RobotRdns,
    RobotRdnsCanonicalPlanFingerprint, RobotRdnsDecodeError, RobotRdnsDestructivePermit,
    RobotRdnsFailureCode, RobotRdnsFilteredMembership, RobotRdnsList, RobotRdnsMutationPermit,
    RobotRdnsPermitAttempt, RobotRdnsPermitRequest, RobotRdnsPlanConfirmation,
    RobotRdnsPlanFingerprintDigest, RobotRdnsPlanSubject, RobotRdnsSharedDestructivePermit,
    RobotRdnsSharedMutationPermit, build_robot_rdns_canonical_plan, build_robot_rdns_plan_digest,
};

#[cfg(feature = "serde")]
pub use ip::{
    CheckedRobotIp, MAX_ROBOT_IP_LIST_ITEMS, PreparedRobotIp, RobotIp,
    RobotIpCanonicalPlanFingerprint, RobotIpDecodeError, RobotIpDestructivePermit, RobotIpList,
    RobotIpMac, RobotIpMutationPermit, RobotIpPermitAttempt, RobotIpPermitRequest,
    RobotIpPlanConfirmation, RobotIpPlanFingerprintDigest, RobotIpPlanSubject,
    RobotIpSharedDestructivePermit, RobotIpSharedMutationPermit, RobotIpSummary,
    RobotIpTrafficPolicy, build_robot_ip_canonical_plan, build_robot_ip_plan_digest,
    decode_robot_ip, decode_robot_ip_list, decode_robot_ip_mac,
};

#[cfg(feature = "serde")]
pub use protocol::{
    MAX_ROBOT_INPUT_FIELDS, RobotDecodeError, RobotFailure, RobotFailureCategory,
    RobotInvalidInput, RobotProviderError, RobotProviderErrorCode, RobotQuota,
    RobotRetryDisposition, RobotTransientTransport, decode_robot_failure,
};

#[cfg(feature = "alloc")]
pub use reset::{
    MAX_ROBOT_RESET_ACTION_RESPONSE_BYTES, MAX_ROBOT_RESET_DETAIL_RESPONSE_BYTES,
    MAX_ROBOT_RESET_LIST_RESPONSE_BYTES, RobotResetGetRequest, RobotResetIntent,
    RobotResetListRequest, RobotResetRequestError, RobotResetType,
};

#[cfg(feature = "serde")]
pub use reset::{
    AuthorizedRobotReset, CheckedRobotReset, MAX_ROBOT_RESET_EVIDENCE_AGE_SECONDS,
    MAX_ROBOT_RESET_LIST_ITEMS, PreparedRobotReset, RobotReset, RobotResetAction,
    RobotResetCanonicalPlanFingerprint, RobotResetDecodeError, RobotResetDestructivePermit,
    RobotResetEvidenceError, RobotResetExecuteRequest, RobotResetFailureCode, RobotResetList,
    RobotResetOperatingStatus, RobotResetPermitAttempt, RobotResetPermitRequest,
    RobotResetPlanConfirmation, RobotResetPlanFingerprintDigest, RobotResetPlanSubject,
    RobotResetPreflightError, RobotResetSharedDestructivePermit, RobotResetSummary,
    build_robot_reset_canonical_plan, build_robot_reset_plan_digest, decode_robot_reset,
    decode_robot_reset_action, decode_robot_reset_list,
};

#[cfg(feature = "alloc")]
pub use server::{
    MAX_ROBOT_SERVER_NAME_BYTES, RobotServerGetRequest, RobotServerListRequest, RobotServerName,
    RobotServerNumber, RobotServerNumberError, RobotServerRequestError, RobotServerUpdateIntent,
    RobotServerUpdateRequest,
};

#[cfg(feature = "alloc")]
pub use subnet::{
    RobotSubnetGetRequest, RobotSubnetListRequest, RobotSubnetMacDeleteRequest,
    RobotSubnetMacGetRequest, RobotSubnetMacSetRequest, RobotSubnetRequestError,
    RobotSubnetTrafficUpdate, RobotSubnetUpdateRequest,
};

#[cfg(feature = "serde")]
pub use traffic::{
    CheckedRobotTraffic, MAX_ROBOT_TRAFFIC_RESPONSE_BYTES, MAX_ROBOT_TRAFFIC_SINGLE_VALUE_TARGETS,
    MAX_ROBOT_TRAFFIC_TARGETS, PreparedRobotTraffic, ROBOT_TRAFFIC_QUOTA, RobotTrafficAmount,
    RobotTrafficData, RobotTrafficDecodeError, RobotTrafficFailureCode, RobotTrafficGranularity,
    RobotTrafficInterval, RobotTrafficIntervalError, RobotTrafficPoint, RobotTrafficQuota,
    RobotTrafficReport, RobotTrafficRequest, RobotTrafficRequestError, RobotTrafficResult,
    RobotTrafficResultTarget, RobotTrafficTarget,
};

#[cfg(feature = "alloc")]
pub use wol::{
    MAX_ROBOT_WOL_RESPONSE_BYTES, ROBOT_WOL_DISCOVERY_QUOTA, ROBOT_WOL_SEND_QUOTA,
    RobotWolGetRequest, RobotWolIntent, RobotWolQuota, RobotWolRequestError,
};

#[cfg(feature = "alloc")]
pub use vswitch::{
    MAX_ROBOT_VSWITCH_ITEM_RESPONSE_BYTES, MAX_ROBOT_VSWITCH_LIST_RESPONSE_BYTES,
    MAX_ROBOT_VSWITCH_NAME_BYTES, MAX_ROBOT_VSWITCH_SERVERS_PER_REQUEST,
    RobotVSwitchAddServersRequest, RobotVSwitchCancelRequest, RobotVSwitchCreateRequest,
    RobotVSwitchGetRequest, RobotVSwitchId, RobotVSwitchListRequest, RobotVSwitchName,
    RobotVSwitchRemoveServersRequest, RobotVSwitchRequestError, RobotVSwitchServerIdentifier,
    RobotVSwitchServers, RobotVSwitchUpdateIntent, RobotVSwitchUpdateRequest,
    RobotVSwitchValueError, RobotVlanId,
};

#[cfg(feature = "serde")]
pub use vswitch::{
    CheckedRobotVSwitch, MAX_ROBOT_VSWITCH_CLOUD_NETWORKS, MAX_ROBOT_VSWITCH_LIST_ITEMS,
    MAX_ROBOT_VSWITCH_MEMBER_SERVERS, MAX_ROBOT_VSWITCH_SUBNETS, PreparedRobotVSwitch,
    RobotVSwitch, RobotVSwitchCanonicalPlanFingerprint, RobotVSwitchCloudNetwork,
    RobotVSwitchDecodeError, RobotVSwitchDestructivePermit, RobotVSwitchFailureCode,
    RobotVSwitchList, RobotVSwitchMutationPermit, RobotVSwitchObservedName,
    RobotVSwitchPermitAttempt, RobotVSwitchPermitRequest, RobotVSwitchPlanConfirmation,
    RobotVSwitchPlanFingerprintDigest, RobotVSwitchPlanSubject, RobotVSwitchServer,
    RobotVSwitchServerStatus, RobotVSwitchSharedDestructivePermit,
    RobotVSwitchSharedMutationPermit, RobotVSwitchSubnet, RobotVSwitchSummary,
    build_robot_vswitch_canonical_plan, build_robot_vswitch_plan_digest,
};

#[cfg(feature = "serde")]
pub use wol::{
    AuthorizedRobotWol, CheckedRobotWol, MAX_ROBOT_WOL_EVIDENCE_AGE_SECONDS, PreparedRobotWol,
    RobotWol, RobotWolCanonicalPlanFingerprint, RobotWolDecodeError, RobotWolEvidenceError,
    RobotWolFailureCode, RobotWolMutationPermit, RobotWolPermitAttempt, RobotWolPermitRequest,
    RobotWolPlanConfirmation, RobotWolPlanFingerprintDigest, RobotWolPlanSubject,
    RobotWolPreflightError, RobotWolSendRequest, RobotWolSharedMutationPermit,
    build_robot_wol_canonical_plan, build_robot_wol_plan_digest, decode_robot_wol,
};

#[cfg(feature = "serde")]
pub use subnet::{
    CheckedRobotSubnet, MAX_ROBOT_SUBNET_EVIDENCE_AGE_SECONDS, MAX_ROBOT_SUBNET_LIST_ITEMS,
    MAX_ROBOT_SUBNET_LOCK_ID_BYTES, MAX_ROBOT_SUBNET_MAC_OPTIONS, PreparedRobotSubnet, RobotSubnet,
    RobotSubnetCanonicalPlanFingerprint, RobotSubnetDecodeError, RobotSubnetDestructivePermit,
    RobotSubnetEvidenceError, RobotSubnetFailureCode, RobotSubnetList, RobotSubnetMac,
    RobotSubnetMacOption, RobotSubnetMutationLease, RobotSubnetMutationPermit,
    RobotSubnetObservationWindow, RobotSubnetPermitAttempt, RobotSubnetPermitRequest,
    RobotSubnetPlanConfirmation, RobotSubnetPlanFingerprintDigest, RobotSubnetPlanSubject,
    RobotSubnetSharedDestructivePermit, RobotSubnetSharedMutationPermit, RobotSubnetTrafficPolicy,
    build_robot_subnet_canonical_plan, build_robot_subnet_plan_digest, decode_robot_subnet,
    decode_robot_subnet_list, decode_robot_subnet_mac,
};

#[cfg(feature = "serde")]
pub use server::{
    MAX_ROBOT_SERVER_ADDRESSES, MAX_ROBOT_SERVER_LIST_ITEMS, ProtectedIpAddr, RobotServer,
    RobotServerCapabilities, RobotServerDate, RobotServerDecodeError, RobotServerList,
    RobotServerStatus, RobotServerSubnet, RobotServerSummary, RobotStorageBoxNumber,
    decode_robot_server, decode_robot_server_list,
};
