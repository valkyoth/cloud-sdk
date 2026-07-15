macro_rules! body_wire {
    ($type:ty, $value:ident => $endpoint:expr, $key:literal, $write:path) => {
        impl crate::prepared::BodyWire for $type {
            fn write_body(
                self,
                output: &mut [u8],
            ) -> Result<usize, crate::prepared::HetznerPreparationError> {
                $write(self, output)
            }

            fn operation_key(self) -> &'static str {
                $key
            }
        }

        impl cloud_sdk::operation::PrepareOperation for $type {
            type Error = crate::prepared::HetznerPreparationError;

            fn prepare<'storage>(
                &self,
                storage: cloud_sdk::operation::PreparationStorage<'storage>,
            ) -> Result<cloud_sdk::operation::PreparedRequest<'storage>, Self::Error> {
                let $value = *self;
                let _ = $value;
                let operation = crate::prepared::HetznerPreparedOperation::json($endpoint, *self);
                cloud_sdk::operation::PrepareOperation::prepare(&operation, storage)
            }
        }
    };
}

macro_rules! body_component {
    ($type:ty, $key:literal, $write:path) => {
        impl crate::prepared::BodyWire for $type {
            fn write_body(
                self,
                output: &mut [u8],
            ) -> Result<usize, crate::prepared::HetznerPreparationError> {
                $write(self, output)
            }

            fn operation_key(self) -> &'static str {
                $key
            }
        }
    };
}
