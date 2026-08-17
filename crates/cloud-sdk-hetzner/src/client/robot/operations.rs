use cloud_sdk::authentication::CredentialBinding;
use cloud_sdk::operation::CheckedResponseGuard;
use cloud_sdk::transport::{ResponseDecodeWorkspace, TransportResponse};

use super::operation::{RobotClientOperation, RobotDirectClientOperation, private};
use crate::robot::*;

macro_rules! checked_operation {
    ($request:ty, $output:ty, $error:ty, $checked:ident, universal) => {
        impl private::Sealed for $request {}
        impl RobotClientOperation for $request {
            type Output<'request>
                = $output
            where
                Self: 'request;
            type SuccessError = $error;

            fn decode_success<'request>(
                &'request self,
                response: CheckedResponseGuard<'_>,
                _credential: CredentialBinding,
            ) -> Result<Self::Output<'request>, Self::SuccessError> {
                $checked::from_executed(self, response).decode_response()
            }

            fn decode_failure(
                &self,
                response: TransportResponse<'_, '_>,
                workspace: &mut ResponseDecodeWorkspace,
            ) -> Result<RobotFailure, RobotDecodeError> {
                decode_robot_failure(response, workspace)
            }
        }
    };
    ($request:ty, $output:ty, $error:ty, $checked:ident, request) => {
        impl private::Sealed for $request {}
        impl RobotClientOperation for $request {
            type Output<'request>
                = $output
            where
                Self: 'request;
            type SuccessError = $error;

            fn decode_success<'request>(
                &'request self,
                response: CheckedResponseGuard<'_>,
                _credential: CredentialBinding,
            ) -> Result<Self::Output<'request>, Self::SuccessError> {
                $checked::from_executed(self, response).decode_response()
            }

            fn decode_failure(
                &self,
                response: TransportResponse<'_, '_>,
                workspace: &mut ResponseDecodeWorkspace,
            ) -> Result<RobotFailure, RobotDecodeError> {
                self.decode_failure(response, workspace)
            }
        }
    };
}

macro_rules! direct {
    ($($request:ty),+ $(,)?) => {$ (
        impl private::DirectSealed for $request {}
        impl RobotDirectClientOperation for $request {
            type PreparationError = <Self as cloud_sdk::operation::PrepareOperation>::Error;

            fn prepare_client<'guard>(
                &self,
                storage: &'guard mut cloud_sdk::operation::PreparationStorageGuard<'_>,
            ) -> Result<cloud_sdk::operation::PreparedRequest<'guard>, Self::PreparationError> {
                storage.prepare(self)
            }
        }
    )+ };
}

macro_rules! guarded_direct {
    ($($request:ty),+ $(,)?) => {$ (
        impl private::DirectSealed for $request {}
        impl RobotDirectClientOperation for $request {
            type PreparationError = RobotOrderRequestError;

            fn prepare_client<'guard>(
                &self,
                storage: &'guard mut cloud_sdk::operation::PreparationStorageGuard<'_>,
            ) -> Result<cloud_sdk::operation::PreparedRequest<'guard>, Self::PreparationError> {
                self.prepare_guarded(storage)
            }
        }
    )+ };
}

macro_rules! server_operation {
    ($request:ty, $output:ty) => {
        impl private::Sealed for $request {}
        impl RobotClientOperation for $request {
            type Output<'request>
                = $output
            where
                Self: 'request;
            type SuccessError = RobotServerDecodeError;

            fn decode_success<'request>(
                &'request self,
                response: CheckedResponseGuard<'_>,
                _credential: CredentialBinding,
            ) -> Result<Self::Output<'request>, Self::SuccessError> {
                self.decode_response(response)
            }

            fn decode_failure(
                &self,
                response: TransportResponse<'_, '_>,
                workspace: &mut ResponseDecodeWorkspace,
            ) -> Result<RobotFailure, RobotDecodeError> {
                decode_robot_failure(response, workspace)
            }
        }
    };
}

#[rustfmt::skip]
server_operation!(RobotServerListRequest, RobotServerList);
#[rustfmt::skip]
server_operation!(RobotServerGetRequest, RobotServer);
#[rustfmt::skip]
server_operation!(RobotServerUpdateRequest<'_>, RobotServer);

#[rustfmt::skip]
checked_operation!(RobotServerCancellationGetRequest, RobotServerCancellation, RobotCancellationDecodeError, CheckedCancellation, universal);
#[rustfmt::skip]
checked_operation!(RobotServerCancellationCreateRequest<'_>, RobotServerCancellation, RobotCancellationDecodeError, CheckedCancellation, universal);
#[rustfmt::skip]
checked_operation!(RobotServerCancellationDeleteRequest, (), RobotCancellationDecodeError, CheckedCancellation, universal);
#[rustfmt::skip]
checked_operation!(RobotIpCancellationGetRequest, RobotIpCancellation, RobotCancellationDecodeError, CheckedCancellation, universal);
#[rustfmt::skip]
checked_operation!(RobotIpCancellationCreateRequest, RobotIpCancellation, RobotCancellationDecodeError, CheckedCancellation, universal);
#[rustfmt::skip]
checked_operation!(RobotIpCancellationDeleteRequest, RobotIpCancellation, RobotCancellationDecodeError, CheckedCancellation, universal);
#[rustfmt::skip]
checked_operation!(RobotSubnetCancellationGetRequest, RobotSubnetCancellation, RobotCancellationDecodeError, CheckedCancellation, universal);
#[rustfmt::skip]
checked_operation!(RobotSubnetCancellationCreateRequest, RobotSubnetCancellation, RobotCancellationDecodeError, CheckedCancellation, universal);
#[rustfmt::skip]
checked_operation!(RobotSubnetCancellationDeleteRequest, RobotSubnetCancellation, RobotCancellationDecodeError, CheckedCancellation, universal);

#[rustfmt::skip]
checked_operation!(RobotIpListRequest, RobotIpList, RobotIpDecodeError, CheckedRobotIp, universal);
#[rustfmt::skip]
checked_operation!(RobotIpGetRequest, RobotIp, RobotIpDecodeError, CheckedRobotIp, universal);
#[rustfmt::skip]
checked_operation!(RobotIpUpdateRequest, RobotIp, RobotIpDecodeError, CheckedRobotIp, universal);
#[rustfmt::skip]
checked_operation!(RobotIpMacGetRequest, RobotIpMac, RobotIpDecodeError, CheckedRobotIp, universal);
#[rustfmt::skip]
checked_operation!(RobotIpMacSetRequest, RobotIpMac, RobotIpDecodeError, CheckedRobotIp, universal);
#[rustfmt::skip]
checked_operation!(RobotIpMacDeleteRequest, RobotIpMac, RobotIpDecodeError, CheckedRobotIp, universal);

#[rustfmt::skip]
checked_operation!(RobotSubnetListRequest, RobotSubnetList, RobotSubnetDecodeError, CheckedRobotSubnet, request);
#[rustfmt::skip]
checked_operation!(RobotSubnetGetRequest, RobotSubnet, RobotSubnetDecodeError, CheckedRobotSubnet, request);
#[rustfmt::skip]
checked_operation!(RobotSubnetUpdateRequest, RobotSubnet, RobotSubnetDecodeError, CheckedRobotSubnet, request);
#[rustfmt::skip]
checked_operation!(RobotSubnetMacGetRequest, RobotSubnetMac, RobotSubnetDecodeError, CheckedRobotSubnet, request);
#[rustfmt::skip]
checked_operation!(RobotSubnetMacSetRequest, RobotSubnetMac, RobotSubnetDecodeError, CheckedRobotSubnet, request);
#[rustfmt::skip]
checked_operation!(RobotSubnetMacDeleteRequest, RobotSubnetMac, RobotSubnetDecodeError, CheckedRobotSubnet, request);

#[rustfmt::skip]
checked_operation!(RobotResetListRequest, RobotResetList, RobotResetDecodeError, CheckedRobotReset, request);
#[rustfmt::skip]
checked_operation!(RobotResetGetRequest, RobotReset, RobotResetDecodeError, CheckedRobotReset, request);
#[rustfmt::skip]
checked_operation!(RobotResetExecuteRequest<'_>, RobotResetAction, RobotResetDecodeError, CheckedRobotReset, request);

#[rustfmt::skip]
checked_operation!(RobotFailoverListRequest, RobotFailoverList, RobotFailoverDecodeError, CheckedRobotFailover, request);
#[rustfmt::skip]
checked_operation!(RobotFailoverGetRequest, RobotFailover, RobotFailoverDecodeError, CheckedRobotFailover, request);
#[rustfmt::skip]
checked_operation!(RobotFailoverRerouteRequest, RobotFailover, RobotFailoverDecodeError, CheckedRobotFailover, request);
#[rustfmt::skip]
checked_operation!(RobotFailoverDeleteRouteRequest, RobotFailover, RobotFailoverDecodeError, CheckedRobotFailover, request);

#[rustfmt::skip]
checked_operation!(RobotWolGetRequest, RobotWol, RobotWolDecodeError, CheckedRobotWol, request);
#[rustfmt::skip]
checked_operation!(RobotWolSendRequest<'_>, RobotWol, RobotWolDecodeError, CheckedRobotWol, request);

#[rustfmt::skip]
checked_operation!(RobotBootGetRequest, RobotBoot, RobotBootDecodeError, CheckedRobotBoot, request);
#[rustfmt::skip]
checked_operation!(RobotRescueGetRequest, RobotBootEntry, RobotBootDecodeError, CheckedRobotBoot, request);
#[rustfmt::skip]
checked_operation!(RobotRescueActivateRequest<'_>, RobotBootEntry, RobotBootDecodeError, CheckedRobotBoot, request);
#[rustfmt::skip]
checked_operation!(RobotRescueDeactivateRequest, RobotBootEntry, RobotBootDecodeError, CheckedRobotBoot, request);
#[rustfmt::skip]
checked_operation!(RobotRescueLastRequest, RobotBootEntry, RobotBootDecodeError, CheckedRobotBoot, request);
#[rustfmt::skip]
checked_operation!(RobotLinuxGetRequest, RobotBootEntry, RobotBootDecodeError, CheckedRobotBoot, request);
#[rustfmt::skip]
checked_operation!(RobotLinuxActivateRequest<'_>, RobotBootEntry, RobotBootDecodeError, CheckedRobotBoot, request);
#[rustfmt::skip]
checked_operation!(RobotLinuxDeactivateRequest, RobotBootEntry, RobotBootDecodeError, CheckedRobotBoot, request);
#[rustfmt::skip]
checked_operation!(RobotLinuxLastRequest, RobotBootEntry, RobotBootDecodeError, CheckedRobotBoot, request);
#[rustfmt::skip]
checked_operation!(RobotVncGetRequest, RobotBootEntry, RobotBootDecodeError, CheckedRobotBoot, request);
#[rustfmt::skip]
checked_operation!(RobotVncActivateRequest<'_>, RobotBootEntry, RobotBootDecodeError, CheckedRobotBoot, request);
#[rustfmt::skip]
checked_operation!(RobotVncDeactivateRequest, RobotBootEntry, RobotBootDecodeError, CheckedRobotBoot, request);
#[rustfmt::skip]
checked_operation!(RobotWindowsGetRequest, RobotBootEntry, RobotBootDecodeError, CheckedRobotBoot, request);
#[rustfmt::skip]
checked_operation!(RobotWindowsActivateRequest<'_>, RobotBootEntry, RobotBootDecodeError, CheckedRobotBoot, request);
#[rustfmt::skip]
checked_operation!(RobotWindowsDeactivateRequest, RobotBootEntry, RobotBootDecodeError, CheckedRobotBoot, request);

#[rustfmt::skip]
checked_operation!(RobotRdnsListRequest, RobotRdnsList, RobotRdnsDecodeError, CheckedRobotRdns, request);
#[rustfmt::skip]
checked_operation!(RobotRdnsGetRequest, RobotRdns, RobotRdnsDecodeError, CheckedRobotRdns, request);
#[rustfmt::skip]
checked_operation!(RobotRdnsSetRequest, RobotRdns, RobotRdnsDecodeError, CheckedRobotRdns, request);
#[rustfmt::skip]
checked_operation!(RobotRdnsUpdateRequest, RobotRdns, RobotRdnsDecodeError, CheckedRobotRdns, request);
#[rustfmt::skip]
checked_operation!(RobotRdnsDeleteRequest, (), RobotRdnsDecodeError, CheckedRobotRdns, request);

#[rustfmt::skip]
checked_operation!(RobotTrafficRequest, RobotTrafficReport, RobotTrafficDecodeError, CheckedRobotTraffic, request);

#[rustfmt::skip]
checked_operation!(RobotSshKeyListRequest, RobotSshKeyList, RobotSshKeyDecodeError, CheckedRobotSshKey, request);
#[rustfmt::skip]
checked_operation!(RobotSshKeyCreateRequest<'_>, RobotSshKey, RobotSshKeyDecodeError, CheckedRobotSshKey, request);
#[rustfmt::skip]
checked_operation!(RobotSshKeyGetRequest, RobotSshKey, RobotSshKeyDecodeError, CheckedRobotSshKey, request);
#[rustfmt::skip]
checked_operation!(RobotSshKeyUpdateRequest, RobotSshKey, RobotSshKeyDecodeError, CheckedRobotSshKey, request);
#[rustfmt::skip]
checked_operation!(RobotSshKeyDeleteRequest, (), RobotSshKeyDecodeError, CheckedRobotSshKey, request);

#[rustfmt::skip]
checked_operation!(RobotFirewallGetRequest, RobotFirewall, RobotFirewallDecodeError, CheckedRobotFirewall, request);
#[rustfmt::skip]
checked_operation!(RobotFirewallReplaceRequest<'_>, RobotFirewall, RobotFirewallDecodeError, CheckedRobotFirewall, request);
#[rustfmt::skip]
checked_operation!(RobotFirewallDeleteRequest, RobotFirewall, RobotFirewallDecodeError, CheckedRobotFirewall, request);
#[rustfmt::skip]
checked_operation!(RobotFirewallTemplateListRequest, RobotFirewallTemplateList, RobotFirewallDecodeError, CheckedRobotFirewall, request);
#[rustfmt::skip]
checked_operation!(RobotFirewallTemplateGetRequest, RobotFirewallTemplate, RobotFirewallDecodeError, CheckedRobotFirewall, request);
#[rustfmt::skip]
checked_operation!(RobotFirewallTemplateCreateRequest<'_>, RobotFirewallTemplateMutationOutcome<'request>, RobotFirewallDecodeError, CheckedRobotFirewall, request);
#[rustfmt::skip]
checked_operation!(RobotFirewallTemplateUpdateRequest<'_>, RobotFirewallTemplateMutationOutcome<'request>, RobotFirewallDecodeError, CheckedRobotFirewall, request);
#[rustfmt::skip]
checked_operation!(RobotFirewallTemplateDeleteRequest, (), RobotFirewallDecodeError, CheckedRobotFirewall, request);

#[rustfmt::skip]
checked_operation!(RobotVSwitchListRequest, RobotVSwitchList, RobotVSwitchDecodeError, CheckedRobotVSwitch, request);
#[rustfmt::skip]
checked_operation!(RobotVSwitchGetRequest, RobotVSwitch, RobotVSwitchDecodeError, CheckedRobotVSwitch, request);
#[rustfmt::skip]
checked_operation!(RobotVSwitchCreateRequest, RobotVSwitch, RobotVSwitchDecodeError, CheckedRobotVSwitch, request);
#[rustfmt::skip]
checked_operation!(RobotVSwitchUpdateRequest, (), RobotVSwitchDecodeError, CheckedRobotVSwitch, request);
#[rustfmt::skip]
checked_operation!(RobotVSwitchCancelRequest, (), RobotVSwitchDecodeError, CheckedRobotVSwitch, request);
#[rustfmt::skip]
checked_operation!(RobotVSwitchAddServersRequest<'_>, (), RobotVSwitchDecodeError, CheckedRobotVSwitch, request);
#[rustfmt::skip]
checked_operation!(RobotVSwitchRemoveServersRequest<'_>, (), RobotVSwitchDecodeError, CheckedRobotVSwitch, request);

macro_rules! observed_operation {
    ($request:ty, $output:ty, $error:ty, $checked:ident) => {
        impl private::Sealed for $request {}
        impl RobotClientOperation for $request {
            type Output<'request>
                = CredentialObserved<$output>
            where
                Self: 'request;
            type SuccessError = $error;

            fn decode_success<'request>(
                &'request self,
                response: CheckedResponseGuard<'_>,
                credential: CredentialBinding,
            ) -> Result<Self::Output<'request>, Self::SuccessError> {
                let value = $checked::from_executed(self, response).decode_response()?;
                Ok(CredentialObserved::from_parts(value, credential))
            }

            fn decode_failure(
                &self,
                response: TransportResponse<'_, '_>,
                workspace: &mut ResponseDecodeWorkspace,
            ) -> Result<RobotFailure, RobotDecodeError> {
                self.decode_failure(response, workspace)
            }
        }
    };
}

#[rustfmt::skip]
observed_operation!(RobotStandardProductListRequest, RobotStandardProductList, RobotOrderCatalogDecodeError, CheckedRobotOrderCatalog);
#[rustfmt::skip]
observed_operation!(RobotStandardProductGetRequest, RobotStandardProduct, RobotOrderCatalogDecodeError, CheckedRobotOrderCatalog);
#[rustfmt::skip]
observed_operation!(RobotMarketProductListRequest, RobotMarketProductList, RobotOrderCatalogDecodeError, CheckedRobotOrderCatalog);
#[rustfmt::skip]
observed_operation!(RobotMarketProductGetRequest, RobotMarketProduct, RobotOrderCatalogDecodeError, CheckedRobotOrderCatalog);
#[rustfmt::skip]
observed_operation!(RobotAddonProductListRequest, RobotAddonCatalog<'request>, RobotOrderCatalogDecodeError, CheckedRobotOrderCatalog);
#[rustfmt::skip]
observed_operation!(RobotOrderCurrencyRequest, RobotOrderCurrency, RobotOrderCatalogDecodeError, CheckedRobotOrderCatalog);
#[rustfmt::skip]
observed_operation!(RobotStandardTransactionListRequest, RobotStandardTransactionList, RobotOrderTransactionDecodeError, CheckedRobotOrderTransaction);
#[rustfmt::skip]
observed_operation!(RobotStandardTransactionGetRequest, RobotStandardTransaction, RobotOrderTransactionDecodeError, CheckedRobotOrderTransaction);
#[rustfmt::skip]
observed_operation!(RobotMarketTransactionListRequest, RobotMarketTransactionList, RobotOrderTransactionDecodeError, CheckedRobotOrderTransaction);
#[rustfmt::skip]
observed_operation!(RobotMarketTransactionGetRequest, RobotMarketTransaction, RobotOrderTransactionDecodeError, CheckedRobotOrderTransaction);
#[rustfmt::skip]
observed_operation!(RobotAddonTransactionListRequest, RobotAddonTransactionList, RobotOrderTransactionDecodeError, CheckedRobotOrderTransaction);
#[rustfmt::skip]
observed_operation!(RobotAddonTransactionGetRequest, RobotAddonTransaction, RobotOrderTransactionDecodeError, CheckedRobotOrderTransaction);

#[rustfmt::skip]
checked_operation!(RobotStandardOrderCreateRequest<'_>, RobotStandardTransaction, RobotOrderMutationDecodeError, CheckedRobotOrderMutation, request);
#[rustfmt::skip]
checked_operation!(RobotMarketOrderCreateRequest<'_>, RobotMarketCreatedTransaction, RobotOrderMutationDecodeError, CheckedRobotOrderMutation, request);
#[rustfmt::skip]
checked_operation!(RobotAddonOrderCreateRequest<'_, '_>, RobotAddonTransaction, RobotOrderMutationDecodeError, CheckedRobotOrderMutation, request);

#[rustfmt::skip]
direct!(
    RobotServerListRequest,
    RobotServerGetRequest,
    RobotServerCancellationGetRequest,
    RobotIpCancellationGetRequest,
    RobotSubnetCancellationGetRequest,
    RobotIpListRequest,
    RobotIpGetRequest,
    RobotIpMacGetRequest,
    RobotSubnetListRequest,
    RobotSubnetGetRequest,
    RobotSubnetMacGetRequest,
    RobotResetListRequest,
    RobotResetGetRequest,
    RobotFailoverListRequest,
    RobotFailoverGetRequest,
    RobotWolGetRequest,
    RobotBootGetRequest,
    RobotRescueGetRequest,
    RobotRescueLastRequest,
    RobotLinuxGetRequest,
    RobotLinuxLastRequest,
    RobotVncGetRequest,
    RobotWindowsGetRequest,
    RobotRdnsListRequest,
    RobotRdnsGetRequest,
    RobotTrafficRequest,
    RobotSshKeyListRequest,
    RobotSshKeyGetRequest,
    RobotFirewallGetRequest,
    RobotFirewallTemplateListRequest,
    RobotFirewallTemplateGetRequest,
    RobotVSwitchListRequest,
    RobotVSwitchGetRequest,
    RobotStandardProductListRequest,
    RobotStandardProductGetRequest,
    RobotMarketProductListRequest,
    RobotMarketProductGetRequest,
    RobotAddonProductListRequest,
    RobotOrderCurrencyRequest,
);

#[rustfmt::skip]
guarded_direct!(
    RobotStandardTransactionListRequest,
    RobotStandardTransactionGetRequest,
    RobotMarketTransactionListRequest,
    RobotMarketTransactionGetRequest,
    RobotAddonTransactionListRequest,
    RobotAddonTransactionGetRequest,
);
