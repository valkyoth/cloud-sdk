use core::fmt;

use cloud_sdk::Method;
use cloud_sdk::buffer::sanitize_bytes;
use cloud_sdk::transport::{
    AsyncExecutionError, AsyncRawHttpExecutor, BlockingRawHttpExecutor, BoundTransport,
    LocalAsyncRawHttpExecutor, MAX_REQUEST_TARGET_BYTES, RawResponsePolicy, ResponseWriter,
    ResponseWriterError, TransportRequest, TransportResponse, drive_async_raw, drive_local_raw,
};

use super::{
    ApiRequestTarget, CRATES_IO_STATIC_DOWNLOAD_BASE_URL, CratesIoEndpointError,
    CratesIoTargetError, OfficialCratesIoEndpoint, StaticDownloadTarget,
};

/// Maximum absolute static-download redirect accepted before target parsing.
pub const MAX_DOWNLOAD_REDIRECT_LOCATION_BYTES: usize =
    CRATES_IO_STATIC_DOWNLOAD_BASE_URL.len() + MAX_REQUEST_TARGET_BYTES;

/// Checked production redirect response detached into caller-owned storage.
///
/// Safe callers can obtain this proof only through its atomic source-execution
/// methods. Response body, content type, and retained-header shape are checked
/// before the static target is copied.
///
/// ```compile_fail
/// use cloud_sdk::transport::TransportResponse;
/// use cloud_sdk_cratesio::endpoint::{ApiRequestTarget, ProductionDownloadResponse};
/// fn cannot_assert_provenance<'a>(
///     source: ApiRequestTarget<'_>,
///     response: TransportResponse<'_, '_>,
///     storage: &'a mut [u8],
/// ) -> ProductionDownloadResponse<'a> {
///     ProductionDownloadResponse::from_executed_response(source, response, storage).unwrap()
/// }
/// ```
pub struct ProductionDownloadResponse<'storage> {
    target_storage: &'storage mut [u8],
    target_len: usize,
}

impl<'storage> ProductionDownloadResponse<'storage> {
    /// Checks an atomically executed response and copies its bounded target.
    ///
    /// `target_storage` is cleared before validation and remains cleared on
    /// every error. This constructor is restricted to the sibling source
    /// execution module, which owns endpoint verification and dispatch.
    pub(super) fn from_executed_response(
        source: ApiRequestTarget<'_>,
        response: TransportResponse<'_, '_>,
        target_storage: &'storage mut [u8],
    ) -> Result<Self, DownloadRedirectError> {
        sanitize_bytes(target_storage);
        let (name, version) =
            source_parts(source).ok_or(DownloadRedirectError::InvalidSourcePath)?;
        if response.status().get() != 302 {
            return Err(DownloadRedirectError::InvalidSourceStatus);
        }
        if !response.body().is_empty()
            || response
                .content_type()
                .map_err(|_| DownloadRedirectError::InvalidSourceResponse)?
                .is_some()
            || response.headers().len() != 1
        {
            return Err(DownloadRedirectError::InvalidSourceResponse);
        }
        let location = response
            .headers()
            .get("location")
            .ok_or(DownloadRedirectError::MissingLocation)?;
        let location = core::str::from_utf8(location.value())
            .map_err(|_| DownloadRedirectError::InvalidLocationEncoding)?;
        let target = validated_target(location, name, version)?;
        let target = target.as_str().as_bytes();
        let output = target_storage
            .get_mut(..target.len())
            .ok_or(DownloadRedirectError::TargetStorageTooSmall)?;
        output.copy_from_slice(target);
        Ok(Self {
            target_storage,
            target_len: target.len(),
        })
    }
}

impl fmt::Debug for ProductionDownloadResponse<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProductionDownloadResponse")
            .field("target_len", &self.target_len)
            .field("target", &"[redacted]")
            .finish()
    }
}

/// Opaque source-correlated redirect to the anonymous download authority.
///
/// Endpoint and target components are deliberately not exposed. Following the
/// redirect is atomic through one of the credential-free raw transport
/// methods, which always constructs an empty request-header block.
///
/// ```compile_fail
/// use cloud_sdk_cratesio::endpoint::DownloadRedirect;
/// fn cannot_extract_target(redirect: &DownloadRedirect<'_>) {
///     let _ = redirect.target();
/// }
/// ```
pub struct DownloadRedirect<'storage> {
    target_storage: &'storage mut [u8],
    target_len: usize,
}

impl<'storage> DownloadRedirect<'storage> {
    /// Consumes checked response provenance into an executable redirect.
    #[must_use]
    pub fn from_verified(response: ProductionDownloadResponse<'storage>) -> Self {
        Self {
            target_storage: response.target_storage,
            target_len: response.target_len,
        }
    }

    /// Follows the redirect through a credential-free blocking executor.
    pub fn follow_blocking<T>(
        &self,
        transport: &T,
        policy: RawResponsePolicy<'_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), DownloadExecutionError<T::Error>>
    where
        T: BlockingRawHttpExecutor + BoundTransport,
    {
        let request = self.request(transport).map_err(map_request_error)?;
        transport
            .execute(request, policy, response)
            .map_err(DownloadExecutionError::Transport)
    }

    /// Follows the redirect through a credential-free Send executor.
    pub async fn follow_async<'transport, 'redirect, 'policy, 'writer, T>(
        &'redirect self,
        transport: &'transport T,
        policy: RawResponsePolicy<'policy>,
        response: &'writer mut ResponseWriter<'_>,
    ) -> Result<(), DownloadExecutionError<T::Error>>
    where
        T: AsyncRawHttpExecutor + BoundTransport,
        'transport: 'writer,
        'redirect: 'writer,
        'policy: 'writer,
    {
        let request = self.request(transport).map_err(map_request_error)?;
        drive_async_raw(transport, request, policy, response)
            .await
            .map_err(map_async_error)
    }

    /// Follows the redirect through a credential-free local executor.
    pub async fn follow_local_async<'transport, 'redirect, 'policy, 'writer, T>(
        &'redirect self,
        transport: &'transport T,
        policy: RawResponsePolicy<'policy>,
        response: &'writer mut ResponseWriter<'_>,
    ) -> Result<(), DownloadExecutionError<T::Error>>
    where
        T: LocalAsyncRawHttpExecutor + BoundTransport,
        'transport: 'writer,
        'redirect: 'writer,
        'policy: 'writer,
    {
        let request = self.request(transport).map_err(map_request_error)?;
        drive_local_raw(transport, request, policy, response)
            .await
            .map_err(map_async_error)
    }

    fn request<T: BoundTransport + ?Sized>(
        &self,
        transport: &T,
    ) -> Result<TransportRequest<'_>, DownloadRequestError> {
        OfficialCratesIoEndpoint::static_downloads()
            .verify_transport(transport)
            .map_err(DownloadRequestError::InvalidDestinationEndpoint)?;
        let bytes = self
            .target_storage
            .get(..self.target_len)
            .ok_or(DownloadRequestError::InvalidStoredTarget)?;
        let value =
            core::str::from_utf8(bytes).map_err(|_| DownloadRequestError::InvalidStoredTarget)?;
        let target = StaticDownloadTarget::new(value)
            .map_err(|_| DownloadRequestError::InvalidStoredTarget)?;
        Ok(TransportRequest::new(
            Method::Get,
            target.as_request_target(),
        ))
    }
}

enum DownloadRequestError {
    InvalidDestinationEndpoint(CratesIoEndpointError),
    InvalidStoredTarget,
}

fn map_request_error<E>(error: DownloadRequestError) -> DownloadExecutionError<E> {
    match error {
        DownloadRequestError::InvalidDestinationEndpoint(error) => {
            DownloadExecutionError::InvalidDestinationEndpoint(error)
        }
        DownloadRequestError::InvalidStoredTarget => DownloadExecutionError::InvalidStoredTarget,
    }
}

impl fmt::Debug for DownloadRedirect<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DownloadRedirect")
            .field("target_len", &self.target_len)
            .field("target", &"[redacted]")
            .finish()
    }
}

/// crates.io static-download redirect validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DownloadRedirectError {
    /// The source is not an exact version-download API target.
    InvalidSourcePath,
    /// The source response is not exactly `302 Found`.
    InvalidSourceStatus,
    /// The redirect response has a body, media type, or extra retained header.
    InvalidSourceResponse,
    /// The checked response omitted its one required `Location` header.
    MissingLocation,
    /// The retained `Location` is not UTF-8.
    InvalidLocationEncoding,
    /// The absolute redirect exceeds the bounded location size.
    LocationTooLong,
    /// The redirect does not use the exact official static authority.
    DestinationMismatch,
    /// The redirect target failed canonical target validation.
    InvalidTarget(CratesIoTargetError),
    /// The destination crate name or archive does not match the source route.
    ArchiveMismatch,
    /// Caller-owned target storage cannot retain the complete checked target.
    TargetStorageTooSmall,
}

impl fmt::Display for DownloadRedirectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidSourcePath => "download redirect source path is invalid",
            Self::InvalidSourceStatus => "download redirect source status is invalid",
            Self::InvalidSourceResponse => "download redirect source response is invalid",
            Self::MissingLocation => "download redirect location is missing",
            Self::InvalidLocationEncoding => "download redirect location encoding is invalid",
            Self::LocationTooLong => "download redirect location exceeds the length limit",
            Self::DestinationMismatch => "download redirect destination is not official",
            Self::InvalidTarget(_) => "download redirect target is invalid",
            Self::ArchiveMismatch => "download redirect archive does not match its source",
            Self::TargetStorageTooSmall => "download redirect target storage is too small",
        })
    }
}

impl core::error::Error for DownloadRedirectError {}

/// Credential-free redirect execution failure.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum DownloadExecutionError<E> {
    /// The executor is not bound to the official static-download origin.
    InvalidDestinationEndpoint(CratesIoEndpointError),
    /// SDK-owned checked target state became invalid.
    InvalidStoredTarget,
    /// The raw credential-free executor failed.
    Transport(E),
    /// Asynchronous response staging or commitment failed.
    ResponseWriter(ResponseWriterError),
}

impl<E> fmt::Debug for DownloadExecutionError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDestinationEndpoint(error) => formatter
                .debug_tuple("InvalidDestinationEndpoint")
                .field(error)
                .finish(),
            Self::InvalidStoredTarget => formatter.write_str("InvalidStoredTarget"),
            Self::Transport(_) => formatter.write_str("Transport([redacted])"),
            Self::ResponseWriter(error) => formatter
                .debug_tuple("ResponseWriter")
                .field(error)
                .finish(),
        }
    }
}

impl<E> fmt::Display for DownloadExecutionError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidDestinationEndpoint(_) => "download executor endpoint is invalid",
            Self::InvalidStoredTarget => "checked download target is invalid",
            Self::Transport(_) => "anonymous download transport failed",
            Self::ResponseWriter(_) => "anonymous download response transaction failed",
        })
    }
}

impl<E> core::error::Error for DownloadExecutionError<E> {}

fn map_async_error<E>(error: AsyncExecutionError<E>) -> DownloadExecutionError<E> {
    match error {
        AsyncExecutionError::Transport(error) => DownloadExecutionError::Transport(error),
        AsyncExecutionError::Response(error) => DownloadExecutionError::ResponseWriter(error),
    }
}

pub(super) fn validated_target<'a>(
    location: &'a str,
    name: &str,
    version: &str,
) -> Result<StaticDownloadTarget<'a>, DownloadRedirectError> {
    if location.len() > MAX_DOWNLOAD_REDIRECT_LOCATION_BYTES {
        return Err(DownloadRedirectError::LocationTooLong);
    }
    let target = location
        .strip_prefix(CRATES_IO_STATIC_DOWNLOAD_BASE_URL)
        .ok_or(DownloadRedirectError::DestinationMismatch)?;
    let destination =
        StaticDownloadTarget::new(target).map_err(DownloadRedirectError::InvalidTarget)?;
    let (directory, archive) = destination
        .parts()
        .ok_or(DownloadRedirectError::ArchiveMismatch)?;
    if directory != name || !archive_matches(archive, name, version) {
        return Err(DownloadRedirectError::ArchiveMismatch);
    }
    Ok(destination)
}

fn source_parts(source: ApiRequestTarget<'_>) -> Option<(&str, &str)> {
    if source.as_request_target().query().is_present() {
        return None;
    }
    let remainder = source
        .as_request_target()
        .path()
        .as_str()
        .strip_prefix("/api/v1/crates/")?;
    let mut segments = remainder.split('/');
    let name = segments.next()?;
    let version = segments.next()?;
    let operation = segments.next()?;
    if name.is_empty() || version.is_empty() || operation != "download" || segments.next().is_some()
    {
        return None;
    }
    Some((name, version))
}

fn archive_matches(archive: &str, name: &str, version: &str) -> bool {
    archive
        .strip_suffix(".crate")
        .and_then(|stem| stem.strip_prefix(name))
        .and_then(|suffix| suffix.strip_prefix('-'))
        == Some(version)
}
