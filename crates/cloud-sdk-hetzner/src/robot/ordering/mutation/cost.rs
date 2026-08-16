use cloud_sdk::operation::{CurrencyCode, PermitContext, PlanCost};

use super::super::{
    RobotAddonOrderPlan, RobotMarketOrderPlan, RobotOrderCurrency, RobotOrderDecimal,
    RobotOrderPrice, RobotStandardOrderPlan,
};

const COST_SCALE: u8 = 4;

/// Failure while deriving exact billable-order authority from catalog evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotOrderCostError {
    /// Account context was empty or exceeded the provider-neutral bound.
    InvalidAccount,
    /// Catalog currency was not accepted by the provider-neutral cost model.
    InvalidCurrency,
    /// Price normalization, quantity multiplication, or aggregation overflowed.
    Overflow,
    /// The caller's first-invoice ceiling was below the observed amount.
    SpendingCeilingExceeded,
}

impl_static_error!(RobotOrderCostError,
    Self::InvalidAccount => "Robot order account context is invalid",
    Self::InvalidCurrency => "Robot order currency is invalid",
    Self::Overflow => "Robot order cost calculation overflowed",
    Self::SpendingCeilingExceeded => "Robot order cost exceeds the spending ceiling",
);

/// Non-empty caller-owned Robot account identity used in plan fingerprints.
#[derive(Clone, Copy)]
pub struct RobotOrderAccount<'a>(&'a [u8]);

impl<'a> RobotOrderAccount<'a> {
    /// Validates a stable account identity without retaining or exposing it.
    pub fn new(value: &'a [u8]) -> Result<Self, RobotOrderCostError> {
        PermitContext::new(value)
            .map(|_| Self(value))
            .map_err(|_| RobotOrderCostError::InvalidAccount)
    }

    pub(super) const fn bytes(self) -> &'a [u8] {
        self.0
    }
}

impl core::fmt::Debug for RobotOrderAccount<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotOrderAccount([redacted])")
    }
}

pub(super) fn standard_cost(
    plan: &RobotStandardOrderPlan<'_>,
    ceiling_units: u128,
) -> Result<PlanCost, RobotOrderCostError> {
    let mut observed = price_total(plan.price())?;
    for addon in plan.addons() {
        observed = observed
            .checked_add(
                price_total(addon.price())?
                    .checked_mul(u128::from(addon.quantity()))
                    .ok_or(RobotOrderCostError::Overflow)?,
            )
            .ok_or(RobotOrderCostError::Overflow)?;
    }
    plan_cost(plan.currency(), observed, ceiling_units)
}

pub(super) fn market_cost(
    plan: &RobotMarketOrderPlan<'_>,
    ceiling_units: u128,
) -> Result<PlanCost, RobotOrderCostError> {
    let observed = units(plan.product().monthly_gross())?
        .checked_add(units(plan.product().setup_gross())?)
        .ok_or(RobotOrderCostError::Overflow)?;
    plan_cost(plan.currency(), observed, ceiling_units)
}

pub(super) fn addon_cost(
    plan: &RobotAddonOrderPlan<'_, '_>,
    ceiling_units: u128,
) -> Result<PlanCost, RobotOrderCostError> {
    plan_cost(
        plan.currency(),
        price_total(plan.product().price())?,
        ceiling_units,
    )
}

fn price_total(price: &RobotOrderPrice) -> Result<u128, RobotOrderCostError> {
    units(price.recurring().gross())?
        .checked_add(units(price.setup().gross())?)
        .ok_or(RobotOrderCostError::Overflow)
}

fn units(value: &RobotOrderDecimal) -> Result<u128, RobotOrderCostError> {
    value
        .checked_units(COST_SCALE)
        .ok_or(RobotOrderCostError::Overflow)
}

fn plan_cost(
    currency: &RobotOrderCurrency,
    observed: u128,
    ceiling: u128,
) -> Result<PlanCost, RobotOrderCostError> {
    let code = currency.with_code(CurrencyCode::new);
    let code = code.map_err(|_| RobotOrderCostError::InvalidCurrency)?;
    PlanCost::new(code, COST_SCALE, observed, ceiling).map_err(|error| match error {
        cloud_sdk::operation::PlanCostError::SpendingCeilingExceeded => {
            RobotOrderCostError::SpendingCeilingExceeded
        }
        cloud_sdk::operation::PlanCostError::ZeroObservedPrice => RobotOrderCostError::Overflow,
    })
}
