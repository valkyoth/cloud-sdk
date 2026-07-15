macro_rules! impl_endpoint_prepare {
    ($($type:ty),+ $(,)?) => {$ (
        impl PrepareOperation for $type {
            type Error = HetznerPreparationError;

            fn prepare<'storage>(
                &self,
                storage: PreparationStorage<'storage>,
            ) -> Result<PreparedRequest<'storage>, Self::Error> {
                HetznerPreparedOperation::endpoint(*self).prepare(storage)
            }
        }
    )+ };
}

macro_rules! endpoint_wire {
    (
        $type:ty,
        $value:ident => $shape:expr,
        $response:expr,
        match $key_value:ident {
            $($key_pattern:pat => $key:literal),+ $(,)?
        },
        $destructive:expr,
        $cost:expr
    ) => {
        impl crate::prepared::EndpointWire for $type {
            fn method(self) -> cloud_sdk::Method {
                <$type>::method(self)
            }

            fn api_base_url(self) -> crate::request::ApiBaseUrl {
                <$type>::api_base_url(self)
            }

            fn write_path(
                self,
                output: &mut [u8],
            ) -> Result<usize, super::HetznerPreparationError> {
                <$type>::write_path(self, output).map_err(|_| super::HetznerPreparationError::Path)
            }

            fn request_shape(self) -> super::RequestShape {
                let $value = self;
                let _ = $value;
                $shape
            }

            fn response_profile(self) -> super::ResponseProfile {
                let $value = self;
                let _ = $value;
                $response
            }

            fn metadata(
                self,
            ) -> Result<cloud_sdk::operation::OperationMetadata, super::HetznerPreparationError>
            {
                let $value = self;
                let _ = $value;
                crate::prepared::operation::method_metadata(self.method(), $destructive, $cost)
            }

            fn operation_key(self) -> &'static str {
                let $key_value = self;
                match $key_value {
                    $($key_pattern => $key),+
                }
            }
        }

        impl cloud_sdk::operation::PrepareOperation for $type {
            type Error = super::HetznerPreparationError;

            fn prepare<'storage>(
                &self,
                storage: cloud_sdk::operation::PreparationStorage<'storage>,
            ) -> Result<cloud_sdk::operation::PreparedRequest<'storage>, Self::Error> {
                super::HetznerPreparedOperation::endpoint(*self).prepare(storage)
            }
        }
    };
}

macro_rules! query_wire {
    ($type:ty, $value:ident => $endpoint:expr) => {
        impl super::QueryWire for $type {
            fn write_query(
                self,
                output: &mut [u8],
            ) -> Result<usize, super::HetznerPreparationError> {
                <$type>::write_query(self, output)
                    .map_err(|_| super::HetznerPreparationError::Query)
            }

            fn operation_key(self) -> &'static str {
                let $value = self;
                crate::prepared::EndpointWire::operation_key($endpoint)
            }
        }

        impl cloud_sdk::operation::PrepareOperation for $type {
            type Error = super::HetznerPreparationError;

            fn prepare<'storage>(
                &self,
                storage: cloud_sdk::operation::PreparationStorage<'storage>,
            ) -> Result<cloud_sdk::operation::PreparedRequest<'storage>, Self::Error> {
                let $value = *self;
                super::HetznerPreparedOperation::query($endpoint, *self).prepare(storage)
            }
        }
    };
}
