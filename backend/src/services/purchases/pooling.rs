//! "Pick any N" offers across the products of one trolley fill.
//!
//! Each bought line is first priced on its own ([`super::price`]). Units of
//! different products in the same mix-and-match offer count together, so the
//! saving is shared out here before the order is saved, the same way the
//! order review does it. Pure.

use rust_decimal::{Decimal, RoundingStrategy};

use super::price::BoughtLine;
use crate::services::stores::deal_group::{pooled_totals, GroupedUnits};

/// Re-prices `bought` (one store's fill) with their offers' units pooled.
pub fn pool_offers(bought: &mut [BoughtLine]) {
    let (owners, parts): (Vec<usize>, Vec<GroupedUnits>) = bought
        .iter()
        .enumerate()
        .filter_map(|(index, line)| line.grouped.clone().map(|part| (index, part)))
        .unzip();
    for ((owner, part), total) in owners.into_iter().zip(&parts).zip(pooled_totals(&parts)) {
        if total == part.alone {
            continue;
        }
        let line = &mut bought[owner];
        line.total_price = cents(line.total_price + total - part.alone);
        line.unit_price = cents(line.total_price / Decimal::from(part.quantity));
    }
}

/// Rounds to whole cents, as every saved price is.
fn cents(amount: Decimal) -> Decimal {
    amount.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    use super::*;
    use crate::models::db::GroceryItemStatus;
    use crate::services::stores::Store;

    /// One unit bought at `shelf`, in offer `group` ("2 for $5.80").
    fn bought(shelf: Decimal, group: Option<&str>) -> BoughtLine {
        BoughtLine {
            grocery_item_id: Uuid::new_v4(),
            item_name: "drink".into(),
            item_status_before: GroceryItemStatus::Committed,
            store: Store::Coles,
            product_id: shelf.to_string(),
            product_name: "Drink".into(),
            brand: None,
            package_size: None,
            quantity: 1,
            unit_price: shelf,
            shelf_price: Some(shelf),
            total_price: shelf,
            store_category: None,
            grouped: group.map(|group| GroupedUnits {
                group_id: group.into(),
                min_quantity: 2,
                deal_price: dec!(2.90),
                shelf_price: shelf,
                quantity: 1,
                alone: shelf,
            }),
        }
    }

    #[test]
    fn two_products_in_one_offer_are_saved_at_the_deal_price() {
        let mut lines = vec![bought(dec!(4.50), Some("a")), bought(dec!(4.00), Some("a"))];
        pool_offers(&mut lines);
        assert_eq!(lines[0].total_price, dec!(2.90));
        assert_eq!(lines[1].unit_price, dec!(2.90));
    }

    #[test]
    fn a_product_without_an_offer_keeps_its_price() {
        let mut lines = vec![bought(dec!(4.50), Some("a")), bought(dec!(4.00), None)];
        pool_offers(&mut lines);
        assert_eq!(lines[0].total_price, dec!(4.50));
        assert_eq!(lines[1].total_price, dec!(4.00));
    }

    #[test]
    fn a_pooled_total_is_saved_in_whole_cents() {
        let mut lines: Vec<BoughtLine> = (0..3).map(|_| bought(dec!(4), Some("a"))).collect();
        for line in &mut lines {
            let part = line.grouped.as_mut().unwrap();
            part.min_quantity = 3;
            part.deal_price = dec!(3.3333);
        }
        pool_offers(&mut lines);
        assert!(lines.iter().all(|line| line.total_price.scale() <= 2));
        assert_eq!(lines[0].total_price, dec!(3.33));
    }
}
