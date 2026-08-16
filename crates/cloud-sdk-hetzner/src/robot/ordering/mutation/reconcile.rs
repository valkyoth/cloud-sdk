use super::request::{
    RobotAddonOrderCreateRequest, RobotMarketOrderCreateRequest, RobotStandardOrderCreateRequest,
};
use crate::robot::ordering::{
    RobotAddonTransaction, RobotAddonTransactionList, RobotMarketTransaction,
    RobotMarketTransactionList, RobotStandardTransaction, RobotStandardTransactionList,
};

/// Reconciliation stopped because transaction history contains the intended order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotOrderReconciliationError {
    /// A matching transaction means the order may have been applied.
    MatchingTransaction,
    /// Reconciliation proof belongs to another request instance.
    RequestMismatch,
}

impl_static_error!(RobotOrderReconciliationError,
    Self::MatchingTransaction => "Robot order transaction history contains the intended order",
    Self::RequestMismatch => "Robot order reconciliation belongs to another request",
);

/// Opaque proof that a complete bounded transaction snapshot had no matching order.
pub struct RobotOrderNotApplied<'request, R> {
    pub(super) request: &'request R,
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
        transactions: &RobotStandardTransactionList,
    ) -> Result<RobotOrderNotApplied<'request, Self>, RobotOrderReconciliationError> {
        if transactions
            .transactions()
            .iter()
            .any(|value| self.matches_transaction(value))
        {
            Err(RobotOrderReconciliationError::MatchingTransaction)
        } else {
            Ok(RobotOrderNotApplied { request: self })
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
        let expected = plan
            .addons()
            .iter()
            .map(|selection| (selection.addon().id(), selection.quantity()));
        let mut index = 0_usize;
        for (id, quantity) in expected {
            for _ in 0..quantity {
                if value.addons().get(index) != Some(id) {
                    return false;
                }
                index = index.saturating_add(1);
            }
        }
        index == value.addons().len()
    }
}

impl RobotMarketOrderCreateRequest<'_> {
    /// Fails closed when the 30-day snapshot contains this auction intent.
    pub fn reconcile_not_applied<'request>(
        &'request self,
        transactions: &RobotMarketTransactionList,
    ) -> Result<RobotOrderNotApplied<'request, Self>, RobotOrderReconciliationError> {
        if transactions
            .transactions()
            .iter()
            .any(|value| self.matches_transaction(value))
        {
            Err(RobotOrderReconciliationError::MatchingTransaction)
        } else {
            Ok(RobotOrderNotApplied { request: self })
        }
    }

    pub(super) fn matches_transaction(&self, value: &RobotMarketTransaction) -> bool {
        value.product().id() == self.plan.product().id()
            && value.product().distribution() == self.plan.distribution()
            && value.product().language() == self.plan.language()
    }
}

impl RobotAddonOrderCreateRequest<'_, '_> {
    /// Fails closed when the 30-day snapshot contains this server-addon intent.
    pub fn reconcile_not_applied<'request>(
        &'request self,
        transactions: &RobotAddonTransactionList,
    ) -> Result<RobotOrderNotApplied<'request, Self>, RobotOrderReconciliationError> {
        if transactions
            .transactions()
            .iter()
            .any(|value| self.matches_transaction(value))
        {
            Err(RobotOrderReconciliationError::MatchingTransaction)
        } else {
            Ok(RobotOrderNotApplied { request: self })
        }
    }

    pub(super) fn matches_transaction(&self, value: &RobotAddonTransaction) -> bool {
        value.server_number() == self.plan.server()
            && value.product().id() == self.plan.product().id()
    }
}
