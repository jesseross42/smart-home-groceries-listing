//! "Pick any N" offers that mix different products.
//!
//! A [`Deal`] with a `group_id` covers every product the store put in that
//! offer, so units of different products count towards the same group of
//! `min_quantity`. [`pooled_totals`] works out what each part of an order
//! costs once its units are pooled with the rest of its offer.
//!
//! Which units get the deal price: the dearest ones first. With a fixed
//! deal price per unit that is the cheapest way to fill the groups, and a
//! product whose shelf price is already at or below the deal price is never
//! put in a group. Pure.

use std::collections::BTreeMap;

use rust_decimal::Decimal;

use super::deal::Deal;

/// Units of one product bought under a mix-and-match offer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupedUnits {
    /// The store's id for the offer.
    pub group_id: String,
    /// Units per group the deal price applies to.
    pub min_quantity: u32,
    /// Price per unit inside a complete group.
    pub deal_price: Decimal,
    /// Shelf price for one.
    pub shelf_price: Decimal,
    pub quantity: u32,
    /// What these units cost priced on their own, deals applied.
    pub alone: Decimal,
}

impl GroupedUnits {
    /// The product's mix-and-match offer, if it has one. `alone` is what
    /// `quantity` units cost priced on their own.
    pub fn new(
        shelf_price: Decimal,
        deals: &[Deal],
        quantity: u32,
        alone: Decimal,
    ) -> Option<Self> {
        let deal = deals
            .iter()
            .find(|deal| deal.group_id.is_some() && deal.min_quantity > 0)?;
        Some(Self {
            group_id: deal.group_id.clone()?,
            min_quantity: deal.min_quantity,
            deal_price: deal.unit_price,
            shelf_price,
            quantity,
            alone,
        })
    }
}

/// What each part costs with its offer's units pooled, in `parts`' order.
///
/// An offer's parts are pooled only when that makes the offer as a whole
/// cheaper than pricing each part alone; otherwise they keep `alone`. The
/// first part of an offer sets its group size and deal price.
pub fn pooled_totals(parts: &[GroupedUnits]) -> Vec<Decimal> {
    let mut totals: Vec<Decimal> = parts.iter().map(|part| part.alone).collect();
    let mut offers: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (index, part) in parts.iter().enumerate() {
        offers.entry(&part.group_id).or_default().push(index);
    }
    for members in offers.values() {
        let pooled = pool(parts, members);
        let before: Decimal = members.iter().map(|&i| parts[i].alone).sum();
        let after: Decimal = pooled.iter().map(|(_, total)| *total).sum();
        if after < before {
            for (index, total) in pooled {
                totals[index] = total;
            }
        }
    }
    totals
}

/// One offer's parts priced with their units pooled, dearest units first.
fn pool(parts: &[GroupedUnits], members: &[usize]) -> Vec<(usize, Decimal)> {
    let first = &parts[members[0]];
    let (size, deal_price) = (first.min_quantity, first.deal_price);
    let mut eligible: Vec<usize> = members
        .iter()
        .copied()
        .filter(|&i| parts[i].shelf_price > deal_price)
        .collect();
    eligible.sort_by(|&a, &b| parts[b].shelf_price.cmp(&parts[a].shelf_price));
    let units: u32 = eligible.iter().map(|&i| parts[i].quantity).sum();
    let mut left = units / size * size;
    let mut pooled: Vec<(usize, Decimal)> = members
        .iter()
        .filter(|i| !eligible.contains(i))
        .map(|&i| (i, parts[i].alone))
        .collect();
    for i in eligible {
        let part = &parts[i];
        let grouped = part.quantity.min(left);
        left -= grouped;
        let total = Decimal::from(grouped) * deal_price
            + Decimal::from(part.quantity - grouped) * part.shelf_price;
        pooled.push((i, total));
    }
    pooled
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    /// A part of offer `group` ("2 for $5.80", so $2.90 each).
    fn part(group: &str, shelf: Decimal, quantity: u32) -> GroupedUnits {
        let deal = Deal {
            description: "Pick any 2 for $5.80".into(),
            min_quantity: 2,
            unit_price: dec!(2.90),
            group_id: Some(group.into()),
            member_only: false,
        };
        let alone = crate::services::stores::deal::total_price(
            shelf,
            std::slice::from_ref(&deal),
            quantity,
        );
        GroupedUnits::new(shelf, &[deal], quantity, alone).unwrap()
    }

    #[test]
    fn two_different_products_in_one_offer_make_a_group() {
        let totals = pooled_totals(&[part("a", dec!(4.50), 1), part("a", dec!(4.00), 1)]);
        assert_eq!(totals, vec![dec!(2.90), dec!(2.90)]);
    }

    #[test]
    fn different_offers_do_not_mix() {
        let totals = pooled_totals(&[part("a", dec!(4.50), 1), part("b", dec!(4.00), 1)]);
        assert_eq!(totals, vec![dec!(4.50), dec!(4.00)]);
    }

    #[test]
    fn the_dearest_units_get_the_deal_and_the_rest_pay_shelf() {
        let totals = pooled_totals(&[part("a", dec!(4.00), 1), part("a", dec!(4.50), 2)]);
        assert_eq!(totals, vec![dec!(4.00), dec!(5.80)]);
    }

    #[test]
    fn a_product_already_cheaper_than_the_deal_is_left_out() {
        let totals = pooled_totals(&[part("a", dec!(2.00), 1), part("a", dec!(4.50), 1)]);
        assert_eq!(totals, vec![dec!(2.00), dec!(4.50)]);
    }

    #[test]
    fn a_part_that_fills_its_own_group_keeps_it() {
        let totals = pooled_totals(&[part("a", dec!(4.50), 2)]);
        assert_eq!(totals, vec![dec!(5.80)]);
    }

    #[test]
    fn a_product_without_a_mix_and_match_offer_has_no_grouped_units() {
        let deal = Deal {
            description: "2 for $6".into(),
            min_quantity: 2,
            unit_price: dec!(3),
            group_id: None,
            member_only: false,
        };
        assert_eq!(GroupedUnits::new(dec!(4), &[deal], 2, dec!(6)), None);
    }
}
