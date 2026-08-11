use alloc::vec::Vec;

use super::decode::RobotServerDecodeError;

pub(super) fn reject_duplicates<T: Ord>(values: &[T]) -> Result<(), RobotServerDecodeError> {
    reject_duplicates_by(values, |value| value)
}

pub(super) fn reject_duplicates_by<T, K: Ord + ?Sized>(
    values: &[T],
    identity: impl Fn(&T) -> &K,
) -> Result<(), RobotServerDecodeError> {
    let mut order = IndexScratch::new(values.len())?;
    order.sort_by(|left, right| {
        identity(indexed(values, left)).cmp(identity(indexed(values, right)))
    });
    if order.as_slice().windows(2).any(|pair| {
        let [left, right] = pair else {
            unreachable!("two-index duplicate window changed length")
        };
        identity(indexed(values, left)) == identity(indexed(values, right))
    }) {
        Err(RobotServerDecodeError::DuplicateIdentity)
    } else {
        Ok(())
    }
}

fn indexed<'a, T>(values: &'a [T], index: &usize) -> &'a T {
    values
        .get(*index)
        .unwrap_or_else(|| unreachable!("generated duplicate index exceeded source length"))
}

struct IndexScratch(Vec<usize>);

impl IndexScratch {
    fn new(length: usize) -> Result<Self, RobotServerDecodeError> {
        let mut values = Vec::new();
        values
            .try_reserve_exact(length)
            .map_err(|_| RobotServerDecodeError::Allocation)?;
        values.extend(0..length);
        Ok(Self(values))
    }

    fn sort_by(&mut self, compare: impl FnMut(&usize, &usize) -> core::cmp::Ordering) {
        self.0.sort_unstable_by(compare);
    }

    fn as_slice(&self) -> &[usize] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{IndexScratch, reject_duplicates, reject_duplicates_by};
    use crate::robot::server::RobotServerDecodeError;

    #[test]
    fn sorted_indices_detect_only_exact_duplicates() {
        assert_eq!(reject_duplicates(&[2_u8, 1, 3]), Ok(()));
        assert_eq!(
            reject_duplicates(&[2_u8, 1, 2]),
            Err(RobotServerDecodeError::DuplicateIdentity)
        );

        let distinct = [(2_u8, 9_u8), (1, 8), (3, 9)];
        assert_eq!(reject_duplicates_by(&distinct, |value| &value.0), Ok(()));
    }

    #[test]
    fn impossible_index_capacity_maps_to_allocation_failure() {
        assert!(matches!(
            IndexScratch::new(usize::MAX),
            Err(RobotServerDecodeError::Allocation)
        ));
    }
}
