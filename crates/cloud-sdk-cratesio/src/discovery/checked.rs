use super::DiscoveryError;
use crate::{
    endpoint::{ApiRequestTarget, OfficialCratesIoEndpoint},
    query::QueryError,
    wire::JsonSuccess,
};

/// Crate-private contract: only reviewed provider GETs enter the shared runner.
pub(crate) trait CheckedGet: Copy {
    type Response;
    fn target(self, output: &mut [u8]) -> Result<ApiRequestTarget<'_>, QueryError>;
    fn anonymous(self) -> Result<(), DiscoveryError> {
        Ok(())
    }
    fn decode(
        self,
        endpoint: OfficialCratesIoEndpoint,
        success: JsonSuccess<'_>,
    ) -> Result<Self::Response, DiscoveryError>;
}

impl CheckedGet for super::DiscoveryRequest<'_> {
    type Response = super::DiscoveryResponse;
    fn target(self, output: &mut [u8]) -> Result<ApiRequestTarget<'_>, QueryError> {
        self.write_target(output)
    }
    fn decode(
        self,
        endpoint: OfficialCratesIoEndpoint,
        success: JsonSuccess<'_>,
    ) -> Result<Self::Response, DiscoveryError> {
        self.decode(endpoint, success)
    }
}
