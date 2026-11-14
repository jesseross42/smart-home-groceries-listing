//! Bodies for Settings › Delivery: `GET` and `PUT /api/delivery-settings`.
//!
//! Money is a decimal string, as everywhere else. Every amount is between
//! $0 and $1000 with at most two decimal places, matching the tables' CHECKs.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use crate::models::delivery_rows::{DeliveryDay, OrderMode};
use crate::services::delivery_settings::{DeliverySettings, FeeRules, Preferences};
use crate::services::stores::Store;

/// The largest amount accepted for a fee, a threshold or a cap.
pub const MAX_DELIVERY_AMOUNT: Decimal = Decimal::from_parts(1000, 0, 0, false, 0);

/// One store's fee rules, both ways. `store_name` is only sent, never read.
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct StoreFeeRulesBody {
    pub store: Store,
    #[serde(default, skip_deserializing)]
    pub store_name: &'static str,
    #[validate(custom(function = "an_amount"))]
    pub delivery_fee: Option<Decimal>,
    #[validate(custom(function = "an_amount"))]
    pub free_delivery_over: Option<Decimal>,
    #[validate(custom(function = "an_amount"))]
    pub minimum_order: Option<Decimal>,
}

/// Settings › Delivery, both ways. `PUT` replaces the preferences and the
/// rules of every store it lists; stores it leaves out keep theirs.
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct DeliverySettingsBody {
    #[validate(length(max = 2), nested)]
    pub stores: Vec<StoreFeeRulesBody>,
    pub mode: OrderMode,
    #[validate(custom(function = "an_amount"))]
    pub max_delivery_spend: Option<Decimal>,
    /// The day the delivery chooser starts on. Left out or `null` = tomorrow.
    #[serde(default)]
    pub default_delivery_day: Option<DeliveryDay>,
}

impl DeliverySettingsBody {
    /// The preferences, as the service takes them.
    pub fn preferences(&self) -> Preferences {
        Preferences {
            mode: self.mode,
            max_delivery_spend: self.max_delivery_spend,
            default_delivery_day: self.default_delivery_day,
        }
    }

    /// The stores' rules, as the service takes them.
    pub fn fee_rules(&self) -> Vec<FeeRules> {
        self.stores
            .iter()
            .map(|body| FeeRules {
                store: body.store,
                delivery_fee: body.delivery_fee,
                free_delivery_over: body.free_delivery_over,
                minimum_order: body.minimum_order,
            })
            .collect()
    }
}

impl From<DeliverySettings> for DeliverySettingsBody {
    fn from(settings: DeliverySettings) -> Self {
        Self {
            stores: settings
                .stores
                .into_iter()
                .map(|rules| StoreFeeRulesBody {
                    store: rules.store,
                    store_name: rules.store.display_name(),
                    delivery_fee: rules.delivery_fee,
                    free_delivery_over: rules.free_delivery_over,
                    minimum_order: rules.minimum_order,
                })
                .collect(),
            mode: settings.mode,
            max_delivery_spend: settings.max_delivery_spend,
            default_delivery_day: settings.default_delivery_day,
        }
    }
}

/// $0 to $1000, in whole cents.
fn an_amount(amount: &Decimal) -> Result<(), ValidationError> {
    if amount.is_sign_negative() || *amount > MAX_DELIVERY_AMOUNT || amount.normalize().scale() > 2
    {
        Err(ValidationError::new("amount_out_of_range"))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn amounts_are_whole_cents_from_nothing_to_a_thousand_dollars() {
        assert!(an_amount(&dec!(0)).is_ok());
        assert!(an_amount(&dec!(9.00)).is_ok());
        assert!(an_amount(&dec!(1000)).is_ok());
        assert!(an_amount(&dec!(-0.01)).is_err());
        assert!(an_amount(&dec!(1000.01)).is_err());
        assert!(an_amount(&dec!(9.999)).is_err());
    }
}
