//! The delivery fees the planner charges: the cheapest fee each store showed
//! on the delivery day, in place of the typed one (issue #70).
//!
//! Pure: the household's settings and the reported fees in, the settings
//! with those fees swapped in and which ones were swapped out.

use crate::models::delivery_rows::DeliveryDay;
use crate::services::delivery_day_fees::{cheapest, DayFee};
use crate::services::delivery_settings::DeliverySettings;

/// Swaps each store's typed fee for the cheapest one it showed on `day` (or
/// on any day when `day` is `None`). A store with nothing reported keeps its
/// typed fee. Returns the fees used, one per store at most.
pub fn use_store_fees(
    settings: &mut DeliverySettings,
    fees: &[DayFee],
    day: Option<DeliveryDay>,
) -> Vec<DayFee> {
    let mut used = Vec::new();
    for rules in &mut settings.stores {
        if let Some(found) = cheapest(fees, rules.store, day) {
            rules.delivery_fee = Some(found.fee);
            used.push(found);
        }
    }
    used
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;
    use crate::models::delivery_rows::OrderMode;
    use crate::services::delivery_settings::FeeRules;
    use crate::services::stores::Store;

    fn settings() -> DeliverySettings {
        let mut woolworths = FeeRules::unset(Store::Woolworths);
        woolworths.delivery_fee = Some(dec!(15));
        woolworths.free_delivery_over = Some(dec!(300));
        let mut coles = FeeRules::unset(Store::Coles);
        coles.delivery_fee = Some(dec!(10));
        DeliverySettings {
            stores: vec![woolworths, coles],
            mode: OrderMode::MinimiseTotal,
            max_delivery_spend: None,
            default_delivery_day: None,
            allowed_delivery_days: Vec::new(),
            need_by: None,
            perishable_hours: None,
        }
    }

    #[test]
    fn a_reported_day_replaces_the_typed_fee_and_keeps_the_other_rules() {
        let mut settings = settings();
        let fees = [DayFee {
            store: Store::Woolworths,
            date: "2026-11-17".parse().unwrap(),
            fee: dec!(2),
        }];
        let used = use_store_fees(&mut settings, &fees, Some(DeliveryDay::Tuesday));
        let woolworths = settings.rules_for(Store::Woolworths);
        assert_eq!(woolworths.delivery_fee, Some(dec!(2)));
        assert_eq!(woolworths.free_delivery_over, Some(dec!(300)));
        assert_eq!(
            settings.rules_for(Store::Coles).delivery_fee,
            Some(dec!(10))
        );
        assert_eq!(used, fees.to_vec());
    }

    #[test]
    fn another_day_keeps_the_typed_fee() {
        let mut settings = settings();
        let fees = [DayFee {
            store: Store::Woolworths,
            date: "2026-11-17".parse().unwrap(),
            fee: dec!(2),
        }];
        let used = use_store_fees(&mut settings, &fees, Some(DeliveryDay::Friday));
        assert_eq!(
            settings.rules_for(Store::Woolworths).delivery_fee,
            Some(dec!(15))
        );
        assert!(used.is_empty());
    }
}
