use alloc::vec::Vec;

use cloud_sdk_sanitization::sanitize_bytes;

use super::decode::RobotServerDecodeError;

pub(super) fn reject_duplicates<T, const N: usize>(
    values: &[T],
    mut identity: impl FnMut(&T) -> [u8; N],
) -> Result<(), RobotServerDecodeError> {
    let mut scratch = IdentityScratch::new(values.len())?;
    for value in values {
        scratch.push(identity(value));
    }
    scratch.reject_duplicates()
}

struct IdentityScratch<const N: usize>(Vec<[u8; N]>);

impl<const N: usize> IdentityScratch<N> {
    fn new(capacity: usize) -> Result<Self, RobotServerDecodeError> {
        let mut values = Vec::new();
        values
            .try_reserve_exact(capacity)
            .map_err(|_| RobotServerDecodeError::Allocation)?;
        Ok(Self(values))
    }

    fn push(&mut self, mut identity: [u8; N]) {
        self.0.push(identity);
        sanitize_bytes(&mut identity);
    }

    fn reject_duplicates(&mut self) -> Result<(), RobotServerDecodeError> {
        self.0.sort_unstable();
        if self
            .0
            .windows(2)
            .any(|pair| matches!(pair, [left, right] if left == right))
        {
            Err(RobotServerDecodeError::DuplicateIdentity)
        } else {
            Ok(())
        }
    }
}

impl<const N: usize> Drop for IdentityScratch<N> {
    fn drop(&mut self) {
        for identity in &mut self.0 {
            sanitize_bytes(identity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{IdentityScratch, reject_duplicates};
    use crate::robot::server::RobotServerDecodeError;

    #[test]
    fn sorted_scratch_detects_only_exact_duplicates() {
        let distinct = [[2_u8], [1_u8], [3_u8]];
        assert_eq!(reject_duplicates(&distinct, |value| *value), Ok(()));

        let duplicate = [[2_u8], [1_u8], [2_u8]];
        assert_eq!(
            reject_duplicates(&duplicate, |value| *value),
            Err(RobotServerDecodeError::DuplicateIdentity)
        );
    }

    #[test]
    fn impossible_scratch_capacity_maps_to_allocation_failure() {
        assert!(matches!(
            IdentityScratch::<1>::new(usize::MAX),
            Err(RobotServerDecodeError::Allocation)
        ));
    }
}
