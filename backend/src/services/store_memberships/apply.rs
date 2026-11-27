//! A store's product, priced for the household's membership. Pure.

use crate::services::stores::Product;

/// `product` as the household can buy it.
///
/// A member pays the member price when it is lower: it becomes `price`, the
/// shelf price becomes `was_price`, and the unit price is scaled to match;
/// `member_price` stays set so the web app can label it. A non-member never
/// sees a member price or a member-only multibuy.
pub fn apply(mut product: Product, member: bool) -> Product {
    if !member {
        product.member_price = None;
        product.deals.retain(|deal| !deal.member_only);
        return product;
    }
    match (product.member_price, product.price) {
        (Some(member_price), Some(shelf)) if member_price < shelf => {
            product.was_price = product.was_price.max(Some(shelf));
            product.price = Some(member_price);
            product.on_special = true;
            if let Some(unit) = product.unit_price.as_mut() {
                unit.amount = (unit.amount * member_price / shelf).round_dp(4);
            }
        }
        _ => product.member_price = None,
    }
    product
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;
    use crate::services::stores::product::fixtures::product;
    use crate::services::stores::{Basis, Deal, Store, UnitPrice};

    fn coffee() -> Product {
        let mut coffee = product(Store::Woolworths, "Coffee", dec!(10));
        coffee.member_price = Some(dec!(8));
        coffee.unit_price = Some(UnitPrice {
            amount: dec!(5),
            per: Basis::Per100Grams,
            converted_from: None,
        });
        coffee.deals.push(Deal {
            description: "2 for $15".into(),
            min_quantity: 2,
            unit_price: dec!(7.5),
            group_id: None,
            member_only: true,
        });
        coffee
    }

    #[test]
    fn a_member_pays_the_member_price() {
        let priced = apply(coffee(), true);
        assert_eq!(priced.price, Some(dec!(8)));
        assert_eq!(priced.was_price, Some(dec!(10)));
        assert_eq!(priced.member_price, Some(dec!(8)));
        assert!(priced.on_special);
        assert_eq!(priced.unit_price.unwrap().amount, dec!(4));
        assert_eq!(priced.deals.len(), 1);
    }

    #[test]
    fn a_non_member_sees_no_member_offers() {
        let priced = apply(coffee(), false);
        assert_eq!(priced.price, Some(dec!(10)));
        assert_eq!(priced.member_price, None);
        assert!(priced.deals.is_empty());
    }

    #[test]
    fn an_earlier_higher_price_is_kept_as_the_was_price() {
        let mut reduced = coffee();
        reduced.was_price = Some(dec!(12));
        assert_eq!(apply(reduced, true).was_price, Some(dec!(12)));
    }

    #[test]
    fn a_member_price_no_lower_than_the_shelf_price_is_dropped() {
        let mut same = coffee();
        same.member_price = Some(dec!(10));
        let priced = apply(same, true);
        assert_eq!(priced.price, Some(dec!(10)));
        assert_eq!(priced.member_price, None);
    }
}
