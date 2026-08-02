use super::{IdempotencyIntent, MaxAttempts};
use crate::retry::RetryPermit;

#[test]
fn zero_attempt_policy_is_unrepresentable() {
    assert!(MaxAttempts::new(0).is_err());
    assert_eq!(MaxAttempts::new(1).map(MaxAttempts::get), Ok(1));
    assert!(core::mem::size_of::<RetryPermit<'static, 'static, 'static>>() <= 128);
}

#[test]
fn intent_shape_rejects_and_clears_invalid_values() {
    let mut empty = [];
    let mut short = [1_u8; 15];
    let mut oversized = [1_u8; 65];
    let mut zero = [0_u8; 16];
    assert!(IdempotencyIntent::new(&mut empty).is_err());
    assert!(IdempotencyIntent::new(&mut short).is_err());
    assert!(IdempotencyIntent::new(&mut oversized).is_err());
    assert!(IdempotencyIntent::new(&mut zero).is_err());
    assert!(short.iter().all(|byte| *byte == 0));
    assert!(oversized.iter().all(|byte| *byte == 0));
    assert!(zero.iter().all(|byte| *byte == 0));
}
