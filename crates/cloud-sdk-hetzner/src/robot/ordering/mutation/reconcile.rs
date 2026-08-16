use super::request::{
    RobotAddonOrderCreateRequest, RobotMarketOrderCreateRequest, RobotStandardOrderCreateRequest,
};
use crate::robot::ordering::{
    CredentialObserved, RobotAddonTransaction, RobotAddonTransactionList,
    RobotMarketCreatedTransaction, RobotMarketTransaction, RobotMarketTransactionList,
    RobotOrderPrice, RobotOrderPricePair, RobotStandardTransaction, RobotStandardTransactionList,
};
use cloud_sdk::authentication::CredentialBinding;

/// Reconciliation stopped because transaction history contains the intended order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotOrderReconciliationError {
    /// A matching transaction means the order may have been applied.
    MatchingTransaction,
    /// Reconciliation proof belongs to another request instance.
    RequestMismatch,
    /// Transaction history came from another credential lifecycle.
    CredentialMismatch,
}

impl_static_error!(RobotOrderReconciliationError,
    Self::MatchingTransaction => "Robot order transaction history contains the intended order",
    Self::RequestMismatch => "Robot order reconciliation belongs to another request",
    Self::CredentialMismatch => "Robot order reconciliation uses another credential",
);

/// Opaque proof that a complete bounded transaction snapshot had no matching order.
pub struct RobotOrderNotApplied<'request, R> {
    pub(super) request: &'request R,
    pub(super) credential: CredentialBinding,
}

impl<R> core::fmt::Debug for RobotOrderNotApplied<'_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotOrderNotApplied([redacted])")
    }
}

impl RobotStandardOrderCreateRequest<'_> {
    /// Fails closed when the 30-day snapshot contains this exact observable intent.
    pub fn reconcile_not_applied<'request>(
        &'request self,
        transactions: &CredentialObserved<RobotStandardTransactionList>,
    ) -> Result<RobotOrderNotApplied<'request, Self>, RobotOrderReconciliationError> {
        if !self.plan.credential().matches(transactions.credential()) {
            return Err(RobotOrderReconciliationError::CredentialMismatch);
        }
        if transactions
            .value()
            .transactions()
            .iter()
            .any(|value| self.matches_transaction(value))
        {
            Err(RobotOrderReconciliationError::MatchingTransaction)
        } else {
            Ok(RobotOrderNotApplied {
                request: self,
                credential: transactions.credential(),
            })
        }
    }

    pub(super) fn matches_transaction(&self, value: &RobotStandardTransaction) -> bool {
        let plan = self.plan;
        if value.product().id() != plan.product().id()
            || value.product().distribution() != plan.distribution()
            || value.product().language() != plan.language()
            || value.product().location() != Some(plan.price().location())
        {
            return false;
        }
        let expected_len = plan.addons().iter().try_fold(0_usize, |total, selection| {
            usize::try_from(selection.quantity())
                .ok()
                .and_then(|quantity| total.checked_add(quantity))
        });
        expected_len == Some(value.addons().len())
            && plan.addons().iter().all(|selection| {
                let count = value
                    .addons()
                    .iter()
                    .filter(|candidate| *candidate == selection.addon().id())
                    .count();
                u64::try_from(count).ok() == Some(selection.quantity())
            })
    }
}

impl RobotMarketOrderCreateRequest<'_> {
    /// Fails closed when the 30-day snapshot contains this auction intent.
    pub fn reconcile_not_applied<'request>(
        &'request self,
        transactions: &CredentialObserved<RobotMarketTransactionList>,
    ) -> Result<RobotOrderNotApplied<'request, Self>, RobotOrderReconciliationError> {
        if !self.plan.credential().matches(transactions.credential()) {
            return Err(RobotOrderReconciliationError::CredentialMismatch);
        }
        if transactions
            .value()
            .transactions()
            .iter()
            .any(|value| self.matches_transaction(value))
        {
            Err(RobotOrderReconciliationError::MatchingTransaction)
        } else {
            Ok(RobotOrderNotApplied {
                request: self,
                credential: transactions.credential(),
            })
        }
    }

    pub(super) fn matches_transaction(&self, value: &RobotMarketTransaction) -> bool {
        value.product().id() == self.plan.product().id()
            && value.product().distribution() == self.plan.distribution()
            && value.product().language() == self.plan.language()
    }

    pub(super) fn matches_created_transaction(
        &self,
        value: &RobotMarketCreatedTransaction,
    ) -> bool {
        value.product().id() == self.plan.product().id()
            && value.product().distribution() == self.plan.distribution()
            && value.product().language() == self.plan.language()
            && value.addons().is_empty()
    }
}

impl RobotAddonOrderCreateRequest<'_, '_> {
    /// Fails closed when the 30-day snapshot contains this server-addon intent.
    pub fn reconcile_not_applied<'request>(
        &'request self,
        transactions: &CredentialObserved<RobotAddonTransactionList>,
    ) -> Result<RobotOrderNotApplied<'request, Self>, RobotOrderReconciliationError> {
        if !self.plan.credential().matches(transactions.credential()) {
            return Err(RobotOrderReconciliationError::CredentialMismatch);
        }
        if transactions
            .value()
            .transactions()
            .iter()
            .any(|value| self.matches_reconciliation_transaction(value))
        {
            Err(RobotOrderReconciliationError::MatchingTransaction)
        } else {
            Ok(RobotOrderNotApplied {
                request: self,
                credential: transactions.credential(),
            })
        }
    }

    fn matches_reconciliation_transaction(&self, value: &RobotAddonTransaction) -> bool {
        value.server_number() == self.plan.server()
            && value.product().id() == self.plan.product().id()
    }

    pub(super) fn matches_created_transaction(&self, value: &RobotAddonTransaction) -> bool {
        self.matches_reconciliation_transaction(value)
            && value
                .product()
                .kind()
                .is_some_and(|kind| kind.compare(self.plan.product().kind()).is_eq())
            && price_matches(self.plan.product().price(), value.product().price())
    }
}

fn price_matches(expected: &RobotOrderPrice, actual: &RobotOrderPrice) -> bool {
    expected.location() == actual.location()
        && pair_matches(expected.recurring(), actual.recurring())
        && pair_matches(expected.setup(), actual.setup())
        && match (expected.hourly(), actual.hourly()) {
            (Some(expected), Some(actual)) => pair_matches(expected, actual),
            (None, None) => true,
            _ => false,
        }
}

fn pair_matches(expected: &RobotOrderPricePair, actual: &RobotOrderPricePair) -> bool {
    expected.net() == actual.net() && expected.gross() == actual.gross()
}
