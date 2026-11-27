//! "Pick any N" offers across the lines bought at one store.
//!
//! Each line is first priced on its own. A mix-and-match offer also counts
//! the units of other products in the same offer, so this module pools every
//! priced line's [`GroupedUnits`] at one store and lowers each line's total
//! by its share of the saving. Used by the order review and by the order
//! planner for each store's part of an option. Pure.

use rust_decimal::Decimal;

use super::line::{LineStatus, OrderLine};
use crate::services::stores::deal_group::{pooled_totals, GroupedUnits};

/// Re-prices `lines` (all at one store) with their offers' units pooled.
/// A line whose total drops is marked `deal_applied`.
pub fn pool_offers(lines: &mut [OrderLine]) {
    let mut owners: Vec<usize> = Vec::new();
    let mut parts: Vec<GroupedUnits> = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if line.status == LineStatus::Priced && line.total_price.is_some() {
            for part in &line.grouped {
                owners.push(index);
                parts.push(part.clone());
            }
        }
    }
    let pooled = pooled_totals(&parts);
    for ((owner, part), total) in owners.into_iter().zip(&parts).zip(pooled) {
        let change = total - part.alone;
        if change == Decimal::ZERO {
            continue;
        }
        let line = &mut lines[owner];
        line.total_price = line.total_price.map(|price| price + change);
        line.deal_applied |= change < Decimal::ZERO;
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::super::line::fixtures::{choice, item};
    use super::*;
    use crate::services::stores::Store;

    /// One unit at `shelf`, in Coles offer `group` ("2 for $5.80").
    fn line(name: &str, shelf: Decimal, group: &str, status: LineStatus) -> OrderLine {
        let item = item(name, 1);
        OrderLine {
            choice: choice(&item, Store::Coles, name, shelf),
            item,
            status,
            price: Some(shelf),
            total_price: Some(shelf),
            deal_applied: false,
            price_change: None,
            problem: None,
            grouped: vec![GroupedUnits {
                group_id: group.into(),
                min_quantity: 2,
                deal_price: dec!(2.90),
                shelf_price: shelf,
                quantity: 1,
                alone: shelf,
            }],
        }
    }

    #[test]
    fn two_items_in_one_offer_share_the_deal() {
        let mut lines = vec![
            line("cola", dec!(4.50), "a", LineStatus::Priced),
            line("lemonade", dec!(4.00), "a", LineStatus::Priced),
        ];
        pool_offers(&mut lines);
        assert_eq!(lines[0].total_price, Some(dec!(2.90)));
        assert_eq!(lines[1].total_price, Some(dec!(2.90)));
        assert!(lines.iter().all(|line| line.deal_applied));
    }

    #[test]
    fn a_line_that_cannot_be_bought_does_not_count() {
        let mut lines = vec![
            line("cola", dec!(4.50), "a", LineStatus::Priced),
            line("lemonade", dec!(4.00), "a", LineStatus::Unavailable),
        ];
        pool_offers(&mut lines);
        assert_eq!(lines[0].total_price, Some(dec!(4.50)));
        assert!(!lines[0].deal_applied);
    }

    #[test]
    fn lines_in_different_offers_keep_their_prices() {
        let mut lines = vec![
            line("cola", dec!(4.50), "a", LineStatus::Priced),
            line("chips", dec!(4.00), "b", LineStatus::Priced),
        ];
        pool_offers(&mut lines);
        assert_eq!(lines[0].total_price, Some(dec!(4.50)));
        assert_eq!(lines[1].total_price, Some(dec!(4.00)));
    }
}
